use etagere::{size2, Allocation, AllocId, BucketedAtlasAllocator};
use image::{ImageBuffer, Rgba};
use wgpu::*;


pub struct TextureAtlas {
    pub sampler: Sampler,
    pub atlas_texture: Texture,
    pub atlas_texture_view: TextureView,
    pub packer: BucketedAtlasAllocator,
}

#[derive(Copy, Clone, Debug)]
pub struct TexCoords {
    pub coords: [f32; 4],
    pub id: AllocId,
}

impl TextureAtlas {
    pub fn tex_coords(&self, allocation: Allocation) -> TexCoords {
        let size = self.packer.size();
        return TexCoords {
            coords: [
                allocation.rectangle.min.x as f32 / size.width as f32, 
                allocation.rectangle.max.x as f32 / size.width as f32, 
                allocation.rectangle.min.y as f32 / size.height as f32, 
                allocation.rectangle.max.y as f32 / size.height as f32, 
            ],
            id: allocation.id,
        }
    } 

    pub fn new(device: &Device, queue: &Queue) -> Self {
        // let max_texture_dimension_2d = device.limits().max_texture_dimension_2d;
        // let atlas_size = max_texture_dimension_2d;
        let atlas_size = 2048;

        let packer = BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32));

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("glyphon sampler"),
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0f32,
            lod_max_clamp: 0f32,
            ..Default::default()
        });

        let atlas_texture = device.create_texture(&TextureDescriptor {
            label: Some("glyphon atlas"),
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
            packer,
            sampler,
            atlas_texture,
            atlas_texture_view,
        }
    }

    pub fn load_texture(&mut self, queue: &Queue, nwidth: u32, nheight: u32, hue: i32) -> TexCoords {

        let png_bytes = include_bytes!("texture_small.png");
    
        // Decode the PNG image
        let img = image::load_from_memory(png_bytes).unwrap();
        
        // let random_number: u64 = rand::thread_rng().gen();
        let img = img.huerotate(hue);
        let img = img.resize(nwidth, nheight, image::imageops::FilterType::CatmullRom);

        // Convert image to RGBA8 format
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();

        let size = size2(width as i32, height as i32);

        let allocation = self.packer.allocate(size).expect(
            "No more room in texture atlas. Don't use this for anything serious btw"
        );

        let atlas_min = allocation.rectangle.min;

        let image_data = img.into_raw();

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
            &image_data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width as u32 * 4 as u32),
                rows_per_image: None,
            },
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
        );

        println!("  {:?}", allocation);

        return self.tex_coords(allocation);
    } 

    pub fn texture_view(&self) -> &TextureView {
        return &self.atlas_texture_view
    }

    pub fn sampler(&self) -> &Sampler {
        return &self.sampler
    }
}