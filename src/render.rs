use std::{marker::PhantomData, mem};

use bytemuck::Pod;
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

        let mut focus_already_grabbed = false;
        self.recursive_text_events(ROOT_I, event, window, &mut focus_already_grabbed);

        self.ui_input(&event, window);
        
        return false;
    }

    fn recursive_text_events(&mut self, i: NodeI, event: &WindowEvent, window: &Window, focus_already_grabbed: &mut bool) {
        
        if let Some(text_i) = self.nodes[i].text_i {
            let res = match text_i {
                TextI::TextBox(idx) => {
                    let text_box = &mut self.sys.text_boxes[idx];
                    text_box.handle_event(event, window, *focus_already_grabbed)
                }
                TextI::StaticTextBox(idx) => {
                    let text_box = &mut self.sys.static_text_boxes[idx];
                    text_box.handle_event(event, window, *focus_already_grabbed)
                }
                TextI::TextEdit(idx) => {
                    let text_edit = &mut self.sys.text_edits[idx];
                    text_edit.handle_event(event, window, *focus_already_grabbed)
                }
            };
            if res.focus_grabbed {
                *focus_already_grabbed = true;
            }
            if res.text_changed {
                self.push_text_change(i);
            }
            if res.decorations_changed {
                self.sys.changes.text_changed = true;
            }
        } 

        for_each_child!(self, self.nodes[i], child, {
            self.recursive_text_events(child, event, window, focus_already_grabbed);
        });
    }

    pub fn ui_input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        match event {
            WindowEvent::CursorMoved { .. } => {              
                self.resolve_hover();
                // cursormoved is never consumed
            }
            WindowEvent::MouseInput { button, state, .. } => {
                // We have to test against all clickable rectangles immediately to know if the input is consumed or not
                match state {
                    ElementState::Pressed => {
                        let consumed = self.resolve_click_press(*button, event, window);
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
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
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
            }
            _ => {}
        }
        return false;
    }

    /// Updates the GUI data on the GPU and renders it. 
    pub fn render(&mut self, render_pass: &mut RenderPass, device: &Device, queue: &Queue) {  
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
        if self.sys.changes.need_gpu_rect_update {
            self.sys.gpu_rect_buffer.queue_write(&self.sys.rects[..], queue);
            self.sys.changes.need_gpu_rect_update = false;
            log::trace!("Update GPU rectangles");
        }
        
        // texture atlas
        // todo: don't do this all the time
        self.sys.texture_atlas.load_to_gpu(queue);

        self.sys.text_renderer.gpu_load(device, queue);
    }

    /// Returns `true` if the `Ui` needs to be rerendered.
    /// 
    /// If this is true, you should call [`Ui::render`] as soon as possible to display the updated GUI state on the screen.
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