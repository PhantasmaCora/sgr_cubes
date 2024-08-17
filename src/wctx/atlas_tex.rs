 
// use image::GenericImageView;

use super::texture::Texture;



pub struct AtlasTexture {
    pub tex: Texture,
    array_use: u32
}

impl AtlasTexture {
    pub fn new(device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        size: (u32, u32)
    ) -> AtlasTexture {
        let descriptor = &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d{ width: size.0, height: size.1, depth_or_array_layers: 128 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let dim = wgpu::TextureViewDimension::D2Array;
        let tex = Texture::from_descriptor( device, queue, descriptor, dim ).unwrap();

        let array_use = 0;

        Self { tex, array_use }
    }

    pub fn add_texture(&mut self, new_tex: &Texture, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<u32, std::io::Error> {

        if  new_tex.texture.width() > self.tex.texture.width() || new_tex.texture.height() > self.tex.texture.height() || new_tex.texture.depth_or_array_layers() > 1 {
            return Err(std::io::Error::other("Submitted texture is too large for atlas size"));
        }

        if  new_tex.texture.format() != self.tex.texture.format() {
            return Err(std::io::Error::other("Submitted texture doesn't match the atlas texture format"));
        }

        if self.array_use >= self.tex.texture.depth_or_array_layers() {
            // need to expand the atlas texture itself!
            let descriptor = &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: self.tex.texture.width(), height: self.tex.texture.height(), depth_or_array_layers: self.tex.texture.depth_or_array_layers() * 2 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.tex.texture.format(),
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            };
            let dim = wgpu::TextureViewDimension::D2Array;

            let new_atlas = Texture::from_descriptor( device, queue, descriptor, dim ).unwrap();

            {
                let old_tex_layers = self.tex.texture.depth_or_array_layers();

                let mut copy_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Copy Transfer Encoder"),
                });

                copy_encoder.copy_texture_to_texture(
                    self.tex.texture.as_image_copy(),
                    new_atlas.texture.as_image_copy(),
                    wgpu::Extent3d{ width: self.tex.texture.width(), height: self.tex.texture.height(), depth_or_array_layers: old_tex_layers }
                );

                queue.submit(std::iter::once(copy_encoder.finish()));
            }

            self.tex = new_atlas;
        }

        // setup render pass to draw the texture on the atlas (using appropriate shader that might palettize)
        let mut final_copy_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Atlas Copy Encoder"),
        });

        let out_index = self.array_use; // the spot to put the new texture in at

        final_copy_encoder.copy_texture_to_texture(
            new_tex.texture.as_image_copy(),
            wgpu::ImageCopyTexture{texture: &self.tex.texture, mip_level: 0, origin: wgpu::Origin3d{ x: 0, y: 0, z: out_index }, aspect: wgpu::TextureAspect::All},
            wgpu::Extent3d{ width: new_tex.texture.width(), height: new_tex.texture.height(), depth_or_array_layers: 1 }
        );

        // finish and queue the command encoder to run the maybe-copy and the render pass
        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(final_copy_encoder.finish()));

        self.array_use += 1;

        Ok(out_index)
    }
}
