use std::{marker::PhantomData, mem};

use bytemuck::Pod;
use glyphon::Resolution as GlyphonResolution;
use wgpu::{Buffer, BufferSlice, Device, Queue, RenderPass};

use crate::Ui;
use crate::ui_math::Axis::*;

impl Ui {
    
    pub fn render<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let n = self.sys.rects.len() as u32;
        if n > 0 {
            render_pass.set_pipeline(&self.sys.render_pipeline);
            render_pass.set_bind_group(0, &self.sys.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sys.gpu_vertex_buffer.slice(n));
            render_pass.draw(0..6, 0..n);
        }

        self.sys.text
            .text_renderer
            .render(&self.sys.text.atlas, render_pass)
            .unwrap();
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {       
        
        // self.prune();
        // self.build_buffers();
        self.sys.gpu_vertex_buffer.queue_write(&self.sys.rects[..], queue);
        
        self.sys.texture_atlas.load_to_gpu(&queue);

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
                GlyphonResolution {
                    width: self.sys.part.unifs.size[X] as u32,
                    height: self.sys.part.unifs.size[Y] as u32,
                },
                &mut self.sys.text.text_areas,
                &mut self.sys.text.cache,
                self.sys.part.current_frame,
            )
            .unwrap();

        // do cleanup here????
        self.sys.hovered.clear();
        // self.sys.clicked.clear()
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