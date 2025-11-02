use std::{marker::PhantomData, mem};

use bytemuck::Pod;
use glam::dvec2;
use wgpu::{Buffer, BufferSlice, Device, Queue, RenderPass};
use winit::event::*;
use winit::window::Window;

use crate::*;

impl Ui {
    /// Handles window events and updates the `Ui`'s internal state accordingly.
    /// 
    /// You can then check for input on specific nodes with [`Ui::is_clicked`] and similar functions.
    ///
    /// You should pass all events from winit to this method, unless they are "consumed" by something "above" the GUI.
    ///
    /// Returns `true` if the event was "consumed" by the `Ui`, e.g. if a mouse click hit an opaque panel.
    /// 
    // todo: move or rename the file
    pub fn window_event(&mut self, event: &WindowEvent, window: &Window) -> bool {
        self.sys.mouse_input.window_event(event);
        self.sys.key_input.window_event(event);

        self.text_window_event(ROOT_I, event, window);

        self.ui_input(&event, window);
        
        return false;
    }

    fn text_window_event(&mut self, _i: NodeI, event: &WindowEvent, window: &Window){     
        self.sys.text.handle_event(event, window);
        
        let text_changed = self.sys.text.any_text_changed();
        if text_changed {
            if let Some(node_id) = self.sys.focused {
                self.sys.text_edit_changed_this_frame = Some(node_id);
            }
            self.sys.changes.text_changed = true;
        }
    }

    pub fn ui_input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        match event {
            WindowEvent::RedrawRequested => {
                self.new_redraw_requested_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {

                let last_cursor_pos = self.sys.mouse_input.prev_cursor_position();
                if dvec2(position.x, position.y) != last_cursor_pos {
                    self.set_new_ui_input(); // here
                }

                self.resolve_hover();
                // cursormoved is never consumed
            }
            WindowEvent::MouseInput { button, state, .. } => {

                let Some(clicked_id) = self.sys.mouse_input.current_tag() else { return false };
                let Some(clicked_i) = self.nodes.node_hashmap.get(&clicked_id) else { return false };
                let clicked_i = clicked_i.slab_i;

                match state {
                    ElementState::Pressed => {
                        let consumed = self.resolve_click_press(*button, event, window, clicked_i);
                        return consumed;
                    },
                    ElementState::Released => {
                        self.resolve_click_release(*button, clicked_i);
                        // Consuming mouse releases can very easily mess things up for whoever is below us.
                        // Some unexpected mouse releases probably won't be too annoying.
                        return false
                    },
                }
            }
            WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                // todo: set new_input only if a node is focused? hard to tell... users probably *shouldn't* listen for unconditional key inputs, but they definitely can
                // probably should have two different bools: one for focused input, one for generic new input. the event loop can decide to wake up and/or update depending on either 
                self.set_new_ui_input();
                if !is_synthetic {
                    let consumed = self.handle_keyboard_event(event);
                    return consumed;
                }
            }
            WindowEvent::Ime(_) => {
                self.set_new_ui_input();
            }
            WindowEvent::Moved(..) => {
                self.resolve_hover();
            }
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_scroll_event(delta);
                self.set_new_ui_input();
            }
            _ => {}
        }
        return false;
    }

    /// Updates the GUI data on the GPU and renders it. 
    pub fn render_in_render_pass(&mut self, render_pass: &mut RenderPass, device: &Device, queue: &Queue) {  
        if self.sys.changes.should_rebuild_render_data {
            self.rebuild_render_data();
        }
        
        log::trace!("Render");
        self.do_cosmetic_rect_updates();

        self.prepare(device, queue);
        let n = self.sys.rects.len() as u32;
        if n > 0 {
            render_pass.set_pipeline(&self.sys.render_pipeline);
            render_pass.set_bind_group(0, &self.sys.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sys.gpu_rect_buffer.slice(n));
            render_pass.draw(0..6, 0..n);
        }

        self.sys.text_renderer.render(render_pass);
        
        self.sys.changes.need_rerender = false;
    }
    
    /// Renders quads within the specified z range.
    pub fn render_z_range(&mut self, render_pass: &mut RenderPass, device: &Device, queue: &Queue, z_range: [f32; 2]) {
        if self.sys.changes.should_rebuild_render_data {
            self.rebuild_render_data();
        }
        
        debug_assert!(z_range[0] > z_range[1], "z_range[0] should be greater than z_range[1].");
        
        log::trace!("Render");
        self.do_cosmetic_rect_updates();

        self.prepare(device, queue);
        let n = self.sys.rects.len() as u32;
        if n > 0 {
            render_pass.set_pipeline(&self.sys.render_pipeline);
            render_pass.set_bind_group(0, &self.sys.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sys.gpu_rect_buffer.slice(n));
            render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::bytes_of(&z_range));
            render_pass.draw(0..6, 0..n);
        }

        self.sys.text_renderer.render_z_range(render_pass, z_range);
        
        self.sys.changes.need_rerender = false;
    }

    /// Load the GUI render data that has changed onto the GPU.
    fn prepare(&mut self, device: &Device, queue: &Queue) {
        // update time + resolution. since we have to update the time anyway, we also update the screen resolution all the time
        self.sys.unifs.t = ui_time_f32();
        queue.write_buffer(
            &self.sys.base_uniform_buffer,
            0,
            bytemuck::bytes_of(&self.sys.unifs),
        );

        // update rects
        if self.sys.changes.need_gpu_rect_update || self.sys.changes.should_rebuild_render_data {
            self.sys.gpu_rect_buffer.queue_write(&self.sys.rects[..], queue);
            self.sys.changes.need_gpu_rect_update = false;
            log::trace!("Update GPU rectangles");
        }
        
        // texture atlas
        // todo: don't do this all the time
        self.sys.texture_atlas.load_to_gpu(queue);

        self.sys.text.prepare_all(&mut self.sys.text_renderer);
        self.sys.text_renderer.load_to_gpu(device, queue);
    }

    /// Renders the UI to a surface with full render pass management.
    /// 
    /// This is a helper method that creates the render pass, calls [`Ui::render_in_render_pass()`], and presents to the screen.
    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        depth_texture: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let surface_texture = surface.get_current_texture().unwrap();
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                        store: wgpu::StoreOp::Store 
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(1.0), 
                        store: wgpu::StoreOp::Store 
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            self.render_in_render_pass(&mut render_pass, device, queue);
        }
        
        queue.submit([encoder.finish()]);
        surface_texture.present();
    }

    /// Returns `true` if the `Ui` needs to be rerendered.
    /// 
    /// If this is true, you should call [`Ui::render`] as soon as possible to display the updated GUI state on the screen.
    pub fn should_rerender(&mut self) -> bool {
        return self.sys.changes.need_rerender
            || self.sys.anim_render_timer.is_live()
            || self.sys.text.needs_rerender()
            || self.sys.changes.should_rebuild_render_data;
    }
}



#[derive(Debug)]
pub struct TypedGpuBuffer<T: Pod> {
    pub buffer: Buffer,
    pub marker: std::marker::PhantomData<T>,
}
impl<T: Pod> TypedGpuBuffer<T> {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            marker: PhantomData::<T>,
        }
    }

    #[allow(dead_code)]
    pub fn size() -> u64 {
        mem::size_of::<T>() as u64
    }

    pub fn slice<N: Into<u64>>(&self, n: N) -> BufferSlice {
        let bytes = n.into() * (mem::size_of::<T>()) as u64;
        return self.buffer.slice(..bytes);
    }

    pub fn queue_write(&mut self, data: &[T], queue: &Queue) {
        let data = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buffer, 0, data);
    }
}