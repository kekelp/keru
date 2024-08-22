use etagere::{size2, Allocation, AllocId, BucketedAtlasAllocator};
use wgpu::*;

use crate::Xy;

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
pub struct TexCoords {
    pub coords: Xy<[f32; 2]>,
    pub id: AllocId,
}

impl TextureAtlas {
    pub fn tex_coords(&self, allocation: Allocation) -> TexCoords {
        let size = self.packer.size();
        return TexCoords {
            coords: Xy::new([
                allocation.rectangle.min.x as f32 / size.width as f32, 
                allocation.rectangle.max.x as f32 / size.width as f32,
            ],
            [
                allocation.rectangle.max.y as f32 / size.height as f32, 
                allocation.rectangle.min.y as f32 / size.height as f32, 
            ]),
            id: allocation.id,
        }
    } 

    pub fn new(device: &Device) -> Self {
        let atlas_size = 2048;

        let packer = BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32));

        let atlas_texture = device.create_texture(&TextureDescriptor {
            label: Some("Fulgur texture atlas"),
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

    pub fn allocate_image(&mut self, image_bytes: &[u8]) -> TexCoords {    
        let img = image::load_from_memory(image_bytes).unwrap();

        // convert to RGBA8 format
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();

        return self.allocate_raw(img.into_raw(), width, height)
    }

    pub fn allocate_raw(&mut self, image_data: Vec<u8>, width: u32, height: u32) -> TexCoords {

        let size = size2(width as i32, height as i32);

        let allocation = self.packer.allocate(size).expect(
            "No more room in texture atlas. Don't use this for anything serious btw"
        );

        self.data_to_load.push(DataToLoad { allocation, image_data, width, height });

        return self.tex_coords(allocation);
    }

    pub fn load_to_gpu(&mut self, queue: &Queue) {
        for data in &self.data_to_load {
            let atlas_min = data.allocation.rectangle.min;

            queue.write_texture(
                ImageCopyTexture {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: atlas_min.x as u32,
                        y: atlas_min.y as u32,
                        z: 0,
                    },
                    aspect: TextureAspect::All,
                },
                &data.image_data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(data.width as u32 * 4 as u32),
                    rows_per_image: None,
                },
                Extent3d {
                    width: data.width as u32,
                    height: data.height as u32,
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