 
use std::io::Error;


use wgpu::util::DeviceExt;



pub enum UIMode {
    Gameplay,
    PauseMenu
}

pub struct UICore {
    crosshair_tex: crate::wctx::texture::Texture,
    crosshair_bind_group: wgpu::BindGroup,
    crosshair_pipeline: wgpu::RenderPipeline,
}

impl UICore {
    pub fn new(out_format: &wgpu::TextureFormat, device: &wgpu::Device, queue: &wgpu::Queue, ) -> UICore {

        let crosshair_bytes = include_bytes!("../../res/texture/core/crosshair.png");
        let crosshair_tex = crate::wctx::texture::Texture::from_bytes(&device, &queue, crosshair_bytes, &"Crosshair Texture").unwrap();

        let gen_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Generic UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../ui_generic_shader.wgsl").into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let ui_generic_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gen UI Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let crosshair_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Crosshairs Pipeline"),
            layout: Some(&ui_generic_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gen_shader,
                entry_point: "vs_main",
                buffers: &[
                    UIVertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &gen_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: *out_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let crosshair_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&crosshair_tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&crosshair_tex.sampler),
                    }
                ],
                label: Some("crosshair_bind_group"),
            }
        );


        Self{
            crosshair_tex,
            crosshair_bind_group,
            crosshair_pipeline,
        }
    }

    pub fn draw(&self, mode: UIMode, target_size: (u32, u32), target_view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<wgpu::CommandEncoder, Error> {
        match mode {
            UIMode::Gameplay => {
                // setup stuff
                let mut crosshair_size = std::cmp::min( target_size.0 / 200, target_size.1 / 200 );
                crosshair_size = std::cmp::max( crosshair_size, 1 );
                crosshair_size *= 8;

                // begin drawing!
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("UI Render Encoder"),
                });

                // draw the crosshair
                let crosshair_vertices = vec![
                    UIVertex{ position: [ -1.0 * crosshair_size as f32 / target_size.0 as f32, crosshair_size as f32 / target_size.1 as f32 ], uv: [0.0, 0.0] },
                    UIVertex{ position: [ crosshair_size as f32 / target_size.0 as f32, crosshair_size as f32 / target_size.1 as f32 ], uv: [1.0, 0.0] },
                    UIVertex{ position: [ -1.0 * crosshair_size as f32 / target_size.0 as f32, -1.0 * crosshair_size as f32 / target_size.1 as f32 ], uv: [0.0, 1.0] },
                    UIVertex{ position: [ crosshair_size as f32 / target_size.0 as f32, -1.0 * crosshair_size as f32 / target_size.1 as f32 ], uv: [1.0, 1.0] },
                ];
                let crosshair_indices: Vec<u16> = vec![
                    0, 2, 1,
                    1, 2, 3
                ];

                let vertex_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&crosshair_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }
                );
                let index_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&crosshair_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }
                );
                let num_indices = crosshair_indices.len() as u32;

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Crosshair Draw Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&self.crosshair_pipeline);
                render_pass.set_bind_group(0, &self.crosshair_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..num_indices, 0, 0..1);

                Ok(encoder)
            }
            UIMode::PauseMenu => {
                 let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("UI Render Encoder"),
                });

                Ok(encoder)
            }

        }

    }


}






#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UIVertex {
    position: [f32; 2],
    uv: [f32; 2],

}

impl UIVertex {
    pub fn new(position: [f32; 2], uv: [f32; 2]) -> UIVertex {
        Self{
            position,
            uv
        }
    }

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}
