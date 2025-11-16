use etagere::{size2, Allocation, AllocId, BucketedAtlasAllocator};
use wgpu::*;

use crate::math::Xy;

pub struct TextureAtlas {
    pub atlas_texture: Texture,
    pub atlas_texture_view: TextureView,
    pub packer: BucketedAtlasAllocator,
    pub data_to_load: Vec<DataToLoad>,
}

pub struct DataToLoad {
    allocation: Allocation,
    image_data: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct ImageRef {
    pub image_id: vello_common::paint::ImageId,
    pub original_size: Xy<f32>,
}

impl TextureAtlas {
    pub fn tex_coords(&self, allocation: Allocation) -> Xy<[f32; 2]> {
        let size = self.packer.size();
        return Xy::new([
            allocation.rectangle.min.x as f32 / size.width as f32, 
            allocation.rectangle.max.x as f32 / size.width as f32,
        ],
        [
            allocation.rectangle.max.y as f32 / size.height as f32, 
            allocation.rectangle.min.y as f32 / size.height as f32, 
        ]);
    } 

    pub fn new(device: &Device) -> Self {
        let atlas_size = 2048;

        let packer = BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32));

        let atlas_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let atlas_texture_view = atlas_texture.create_view(&TextureViewDescriptor::default());

        // fill with a debug texture
        // let mut img = ImageBuffer::new(atlas_size, atlas_size);
        // const TILE: u32 = 256;
        // for (x, y, pixel) in img.enumerate_pixels_mut() {
        //     let is_black = ((x / TILE) % 2 == 0) ^ ((y / TILE) % 2 == 0);
        //     *pixel = if is_black {
        //         Rgba([0, 255, 0, 255])
        //     } else {
        //         Rgba([255, 0, 255, 255])
        //     };
        // }
        // let image_data = img.into_raw();

        // queue.write_texture(
        //     ImageCopyTexture {
        //         texture: &atlas_texture,
        //         mip_level: 0,
        //         origin: Origin3d {
        //             x: 0 as u32,
        //             y: 0 as u32,
        //             z: 0,
        //         },
        //         aspect: TextureAspect::All,
        //     },
        //     &image_data,
        //     ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: Some(atlas_size * 4 as u32),
        //         rows_per_image: None,
        //     },
        //     Extent3d {
        //         width: atlas_size,
        //         height: atlas_size,
        //         depth_or_array_layers: 1,
        //     },
        // );

        return Self {
            data_to_load: Vec::with_capacity(20),
            packer,
            atlas_texture,
            atlas_texture_view,
        }
    }

    pub fn allocate_image(&mut self, image_bytes: &[u8]) -> ImageRef {

        log::info!("Allocating an image"); 
        let img = image::load_from_memory(image_bytes).unwrap(); // todo: don't unwrap here

        // convert to RGBA8 format
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();

        return self.allocate_raw(img.into_raw(), width, height)
    }

    pub fn allocate_raw(&mut self, image_data: Vec<u8>, width: u32, height: u32) -> ImageRef {
        // For now, return a placeholder ImageRef
        // The actual image upload will happen during render
        return ImageRef {
            image_id: vello_common::paint::ImageId::new(0),
            original_size: Xy::new(width as f32, height as f32),
        }
    }

    pub fn load_to_gpu(&mut self, queue: &Queue) {
        for data in &self.data_to_load {
            let atlas_min = data.allocation.rectangle.min;

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: atlas_min.x as u32,
                        y: atlas_min.y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &data.image_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(data.width * 4_u32),
                    rows_per_image: Some(data.height),
                },
                wgpu::Extent3d {
                    width: data.width,
                    height: data.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.data_to_load.clear();
    }

    pub fn texture_view(&self) -> &TextureView {
        return &self.atlas_texture_view
    }
}