

use std::io::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::{
    Write,
    Read
};

use fs_extra::dir::create_all;

use cgmath::Angle;


const DRAW_WORLD_SCALE: [f32; 3] = [1.0/128.0, 1.0/192.0, 1.0/256.0];

pub struct WorldSaver {
}

impl WorldSaver {


    pub fn create_world_dir(name: String, size: usize) -> PathBuf {
        let pdirs = directories::ProjectDirs::from( "", "PhantasmaCora Games", "SGR_Cubes" ).expect("Failed to get project directories");
        let mut pbuf = pdirs.data_dir().to_path_buf();
        pbuf.push( "worlds/" );
        pbuf.push( name.clone() );
        create_all( &pbuf, false );

        let mut ibuf = pbuf.clone();
        ibuf.push("info.toml");
        let datastring = format!( "name = \"{}\"\nsize = {}", name, size );
        let mut file = File::create(ibuf).expect("failed to create file");
        file.write_all( datastring.as_bytes() ).expect("failed to write to file");

        pbuf
    }

    pub fn save_world(&mut self, world_render: &crate::wctx::world::WorldRender, device: &wgpu::Device, queue: &wgpu::Queue ) {
        let pbuf = Self::create_world_dir( world_render.world_name.clone(), world_render.world.chunk_manager.size );

        {
            let mut pklbuf = pbuf.clone();
            pklbuf.push("world_savestate.pkl");
            let mut fi = std::fs::File::create(pklbuf).expect("failed to create world file");
            let serialized = serde_pickle::to_vec(&world_render.world, Default::default()).unwrap();
            fi.write_all( &serialized ).expect("failed to write world file");
        }

        {
            let tex = crate::wctx::texture::Texture::from_descriptor(
                device,
                queue,
                &wgpu::TextureDescriptor {
                    label: Some("world overview render texture"),
                    size: wgpu::Extent3d{ width: 512, height: 512, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                },
                wgpu::TextureViewDimension::D2
            ).expect("failed to create texture");

            world_render.draw_custom_view( ( cgmath::Matrix4::from_translation( cgmath::Vector3::new( 0.0, 0.3, 0.5 ) ) * cgmath::Matrix4::from_nonuniform_scale(1.0, 1.0, -0.1) * cgmath::Matrix4::from_angle_x( cgmath::Rad::atan( 2.0_f32.sqrt() / 2.0 ) ) * cgmath::Matrix4::from_angle_y( cgmath::Rad::full_turn() / -8.0 ) * cgmath::Matrix4::from_translation( cgmath::Vector3::new( -0.5, -1.0, -0.5 ) ) * cgmath::Matrix4::from_scale( DRAW_WORLD_SCALE[world_render.world.chunk_manager.size] ) ).into(), device, queue, &tex );

            let buffer = device.create_buffer(
                &wgpu::BufferDescriptor {
                    label: None,
                    size: 512 * 512 * 4,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false
                }
            );

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Copy Encoder"),
            });

            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &tex.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d{x: 0, y:0, z:0},
                    aspect: wgpu::TextureAspect::All
                },
                wgpu:: ImageCopyBuffer {
                    buffer: &buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * 512),
                        rows_per_image: Some(512),
                    }
                },
                wgpu::Extent3d{
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1
                }
            );

            queue.submit( std::iter::once(encoder.finish()) );

            let buffer = std::sync::Arc::new(buffer);
            let capturable = buffer.clone();
            let mut ibuf = pbuf.clone();
            buffer.slice(..).map_async(wgpu::MapMode::Read, move |res| {
                if res.is_ok() {
                    let data = capturable.slice(..).get_mapped_range();
                    let img = image::RgbaImage::from_raw(512, 512, (*data).to_vec() ).expect("failed to create image");
                    ibuf.push("preview.png");
                    img.save(ibuf);
                }
            } );
            queue.submit( std::iter::empty() );

        }
    }

}
