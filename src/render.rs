use std::{marker::PhantomData, mem};

use bytemuck::Pod;
use wgpu::{Buffer, BufferSlice, Device, Queue, RenderPass};
use winit::event::*;

use crate::text::render_iter;
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
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.sys.mouse_input.handle_event(&event);

        match event {
            WindowEvent::CursorMoved { .. } => {              
                self.resolve_hover();
                // cursormoved is never consumed
            }
            WindowEvent::MouseInput { button, state, .. } => {
                // We have to test against all clickable rectangles immediately to know if the input is consumed or not
                match state {
                    ElementState::Pressed => {
                        let consumed = self.resolve_click_press(*button);
                        return consumed;
                    },
                    ElementState::Released => {
                        self.resolve_click_release(*button);
                        // Consuming mouse releases can very easily mess things up for whoever is below us.
                        // Some unexpected mouse releases probably won't be too annoying.
                        return false
                    },
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.sys.key_mods = modifiers.state();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if !is_synthetic {
                    let consumed = self.handle_keyboard_event(&event);
                    return consumed;
                }
            }
            WindowEvent::Moved(..) => {
                self.resolve_hover();
            }
            WindowEvent::Resized(size) => self.resize(&size),
            
            _ => {}
        }

        return false;
    }

    /// Renders the GUI render data that were previously loaded on the GPU with [`Ui::prepare`].
    pub fn render(&mut self, render_pass: &mut RenderPass) {        
        let n = self.sys.rects.len() as u32;
        if n > 0 {
            render_pass.set_pipeline(&self.sys.render_pipeline);
            render_pass.set_bind_group(0, &self.sys.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sys.gpu_rect_buffer.slice(n));
            render_pass.draw(0..6, 0..n);
        }

        self.sys.text
            .text_renderer
            .render(&self.sys.text.atlas, &self.sys.text.glyphon_viewport, render_pass)
            .unwrap();
        
        self.sys.changes.need_rerender = false;
    }

    /// Load the GUI render data onto the GPU. To render it, start a render pass, then call [`Ui::render`].
    pub fn prepare(&mut self, device: &Device, queue: &Queue) {       
        // update time + resolution        
        // since we have to update the time anyway, we also update the screen resolution all the time
        self.sys.unifs.t = ui_time_f32();

        let warning = "todo: change this";
        queue.write_buffer(
            &self.sys.base_uniform_buffer,
            0,
            &bytemuck::bytes_of(&self.sys.unifs),
        );

        // update glyphon size info
        if self.sys.changes.resize {
            self.sys.text.glyphon_viewport.update(
                queue,
                glyphon::Resolution {
                    width: self.sys.unifs.size.x as u32,
                    height: self.sys.unifs.size.y as u32,
                },
            );
            self.sys.changes.resize = false;
        }

        // update rects
        // todo: don't do this all the time
        self.sys.gpu_rect_buffer.queue_write(&self.sys.rects[..], queue);
        
        // texture atlas
        // todo: don't do this all the time
        self.sys.texture_atlas.load_to_gpu(queue);

        self.sys.text
            .text_renderer
            .prepare(
                device,
                queue,
                &mut self.sys.text.font_system,
                &mut self.sys.text.atlas,
                &self.sys.text.glyphon_viewport,
                render_iter(&mut self.sys.text.text_areas, self.sys.current_frame),
                &mut self.sys.text.cache,
            )
            .unwrap();
    }

    /// Returns `true` if the `Ui` needs to be rerendered.
    /// 
    /// If this is true, you should call [`Ui::prepare`] and [`Ui::render`] as soon as possible to display the updated GUI state on the screen.
    pub fn needs_rerender(&mut self) -> bool {
        return self.sys.changes.need_rerender || self.sys.anim_render_timer.is_live()
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