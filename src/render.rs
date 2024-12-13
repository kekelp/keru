use std::{marker::PhantomData, mem};

use bytemuck::Pod;
use wgpu::{Buffer, BufferSlice, Device, Queue, RenderPass};

use crate::text::render_iter;
use crate::Ui;

impl Ui {
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
        
        if self.sys.changes.resize {
            self.sys.text.glyphon_viewport.update(
                queue,
                glyphon::Resolution {
                    width: self.sys.part.unifs.size.x as u32,
                    height: self.sys.part.unifs.size.y as u32,
                },
            );
            let warning = "todo: change this";
            queue.write_buffer(
                &self.sys.base_uniform_buffer,
                0,
                &bytemuck::bytes_of(&self.sys.part.unifs)[..16],
            );

            self.sys.changes.resize = false;
        }

        self.sys.gpu_rect_buffer.queue_write(&self.sys.rects[..], queue);
        
        self.sys.texture_atlas.load_to_gpu(queue);

        // update gpu time
        // magical offset...
        queue.write_buffer(&self.sys.base_uniform_buffer, 8, bytemuck::bytes_of(&self.sys.frame_t));

        self.sys.text
            .text_renderer
            .prepare(
                device,
                queue,
                &mut self.sys.text.font_system,
                &mut self.sys.text.atlas,
                &self.sys.text.glyphon_viewport,
                render_iter(&mut self.sys.text.text_areas, self.sys.part.current_frame),
                &mut self.sys.text.cache,
            )
            .unwrap();
    }

    /// Returns `true` if the `Ui` needs to be rerendered.
    /// 
    /// If this is true, you should call [`Ui::prepare`] and [`Ui::render`] as soon as possible to display the updated GUI state on the screen.
    pub fn needs_rerender(&self) -> bool {
        return self.sys.changes.need_rerender || self.sys.changes.animation_rerender_time.is_some();
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