 

use std::path::PathBuf;
use std::io::Error;

use cgmath::SquareMatrix;

use grid_ray::GridRayIter3;

use serde::{
    Serialize,
    Deserialize
};

use wgpu::util::DeviceExt;



use crate::wctx::camera;
use crate::wctx::texture;
use crate::wctx::chunk;
use crate::wctx::block;
use crate::wctx::atlas_tex;

// state stored when a game world is saved
#[derive(Serialize, Deserialize)]
pub struct WorldSavestate {
    pub chunk_manager: chunk::ChunkManager,
    pub block_select: u16,
    pub camera: camera::Camera,
}

impl WorldSavestate {
    pub fn new() -> WorldSavestate {
        let chunk_manager = crate::wctx::chunk::ChunkManager::new();
        let block_select = 1;
        let camera = camera::Camera::new((63.0, 35.0, 62.0), cgmath::Deg(90.0), cgmath::Deg(-20.0));

        Self {
            chunk_manager,
            block_select,
            camera,
        }
    }
}


// resources used to render the game world
pub struct WorldRender {
    pub world: WorldSavestate,
    pub render_pipeline: wgpu::RenderPipeline,
    projection: camera::Projection,
    pub camera_controller: camera::CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: crate::wctx::texture::Texture,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub colormap_bind_group: wgpu::BindGroup,
    pub block_atlas: atlas_tex::AtlasTexture,
    pub block_registry: block::BlockRegistry,
    pub shape_registry: block::BlockShapeRegistry,
    selector_pipeline: wgpu::RenderPipeline,
    selector_bind_group: wgpu::BindGroup,
    selected_block: Option<(usize, usize, usize)>,
    select_timer: u8,
    select_duration: std::time::Duration
}

impl WorldRender {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration, world: WorldSavestate) -> WorldRender {

        let mut dl = crate::wctx::data_loader::BlockLoader::create(&device, &queue);
        let _ = dl.submit_blockshape_direct( crate::wctx::block::make_cube_shape(), &"CubeStatic".into() );
        let _ = dl.submit_blockshape_direct( crate::wctx::block::make_slope_shape(), &"Slope".into() );

        let pal_bytes = include_bytes!("../../res/texture/core/palette.png");
        let pal_img = image::load_from_memory(pal_bytes).unwrap();

        dl.load_toml_from_file( PathBuf::from("res/data/block.toml") ).expect("failed to load blocks!");
        dl.do_extract().expect("failed to extract config!");
        dl.resolve_blocks( &device, &queue, &pal_img ).expect("failed to resolve blocks!");

        let block_registry = dl.block_registry;
        let block_atlas = dl.texture_atlas;
        let shape_registry = dl.shape_registry;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Uint,
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

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&block_atlas.tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&block_atlas.tex.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let loadonly_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
                label: Some("loadonly_texture_bind_group_layout"),
            });

        let colormap_bytes = include_bytes!("../../res/texture/core/colormap.png");
        let colormap_tex = texture::Texture::from_bytes(&device, &queue, colormap_bytes, &"Colormap Texture").unwrap();
        let colormap_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &loadonly_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&colormap_tex.view),
                    }
                ],
                label: Some("colormap_bind_group"),
            }
        );


        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");


        let camera_controller = camera::CameraController::new(7.0, 0.37);
        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&world.camera, &projection);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });



        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Block Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../block_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
                &loadonly_texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let sel_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Selected Block Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../selected_block_shader.wgsl").into()),
        });

        let sel_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Selected Block Marker Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &loadonly_texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let selector_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Selected Block Marker Pipeline"),
            layout: Some(&sel_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sel_shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sel_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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

        let selector_bytes = include_bytes!("../../res/texture/core/selector.png");
        let selector_tex = texture::Texture::from_bytes(&device, &queue, selector_bytes, &"Selector Texture").unwrap();
        let selector_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &loadonly_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&selector_tex.view),
                    }
                ],
                label: Some("selector_bind_group"),
            }
        );

        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);

        Self{
            world,
            block_registry,
            block_atlas,
            shape_registry,
            projection,
            depth_texture,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            render_pipeline,
            diffuse_bind_group,
            colormap_bind_group,
            selector_pipeline,
            selector_bind_group,
            selected_block: None,
            select_timer: 0,
            select_duration: std::time::Duration::ZERO,
        }
    }

    pub fn resize_window (&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.projection.resize(config.width, config.height);
        self.depth_texture = texture::Texture::create_depth_texture(device, config, "depth_texture");
    }

    pub fn process_keyboard(&mut self, key: &winit::keyboard::KeyCode, state: &winit::event::ElementState) -> bool {
        self.camera_controller.process_keyboard(*key, *state)
    }

    pub fn scroll_shift(&mut self, del: f32) {
        let delta = del as i32;
        let mut moved_i = self.world.block_select as i32;
        if delta > 0 {
            moved_i += delta;
            moved_i %= self.block_registry.get_num_blocks() as i32;
            if moved_i == 0 {
                moved_i = 1;
            }
        } else {
            moved_i += delta;
            moved_i %= self.block_registry.get_num_blocks() as i32;
            while moved_i <= 0 {
                moved_i += self.block_registry.get_num_blocks() as i32 - 1;
            }
        }

        self.world.block_select = moved_i as u16;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, mouse_pressed: crate::wctx::MouseOps, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.world.camera, dt);
        self.camera_uniform.update_view_proj(&self.world.camera, &self.projection);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // do block breaking and placing
        let mut last = grid_ray::ilattice::glam::IVec3::NEG_ONE;
        let mut current = grid_ray::ilattice::glam::IVec3::NEG_ONE;
        let dir = self.world.camera.get_forward_vector();
        let mut ray_iter = GridRayIter3::new(
            grid_ray::ilattice::glam::Vec3A::new( self.world.camera.position.x, self.world.camera.position.y, self.world.camera.position.z ), // start position
            grid_ray::ilattice::glam::Vec3A::new( dir.x, dir.y, dir.z )
        );

        let mut run = true;
        let mut hit = false;
        while run {
            let next = ray_iter.next().unwrap();
            if next.0 > 5.0 { run = false; }
            last = current;
            current = next.1;
            if current.x >= 0 && current.y >= 0 && current.z >= 0 &&
            current.x < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32 && current.y < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32 && current.z < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32 {
                let bdef = self.world.chunk_manager.get_block( ( current.x as usize, current.y as usize, current.z as usize ) ).blockdef;
                if bdef != 0 {
                    run = false;
                    hit = true;
                }
            }
        }
        if hit {
            self.selected_block = Some( ( current.x as usize, current.y as usize, current.z as usize ) );
        } else {
            self.selected_block = None;
        }

        if hit && mouse_pressed.left_just_now {
            let mut broken = self.world.chunk_manager.get_mut_block( ( current.x as usize, current.y as usize, current.z as usize ) );
            broken.blockdef = 0;
            broken.exparam = 0;
        } else if hit && mouse_pressed.right_just_now && (last.x >= 0 && last.y >= 0 && last.z >= 0 &&
            last.x < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32 && last.y < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32 && last.z < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32) {
            let mut placed = self.world.chunk_manager.get_mut_block( ( last.x as usize, last.y as usize, last.z as usize ) );
            placed.blockdef = self.world.block_select;
            placed.exparam = 0;
        }

        {
            self.world.chunk_manager.update_dirty_chunks( &self.block_registry, &self.shape_registry );
        }

        self.select_duration += dt;
        if self.select_duration > std::time::Duration::new(0, 250_000_000) {
            self.select_timer += 1;
            self.select_timer %= 13;
            self.select_duration -= std::time::Duration::new(0, 250_000_000);
        }
    }



    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, out_view: &wgpu::TextureView) -> Result<wgpu::CommandEncoder, Error> {
        let draw_chunk_list = self.world.chunk_manager.get_render_chunks();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut first = true;

        for idx in 0..draw_chunk_list.len() {
            let c = &draw_chunk_list[idx];

            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&c.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&c.indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );
            let num_indices = c.indices.len() as u32;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: if first {wgpu::LoadOp::Clear(1.0)} else {wgpu::LoadOp::Load},
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            first = false;

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(2, &self.colormap_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);

        }

        // draw the marker for the selected block!
        if let Some(pos) = self.selected_block {
            let sel_vertices = vec![
                Vertex { position: [ pos.0 as f32, pos.1 as f32, pos.2 as f32 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 }, // -Z
                Vertex { position: [ pos.0 as f32, pos.1 as f32 + 1.0, pos.2 as f32 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32, pos.2 as f32 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32 + 1.0, pos.2 as f32 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },

                Vertex { position: [ pos.0 as f32, pos.1 as f32, pos.2 as f32 + 1.0 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 }, // +Z
                Vertex { position: [ pos.0 as f32, pos.1 as f32 + 1.0, pos.2 as f32 + 1.0 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32, pos.2 as f32 + 1.0 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32 + 1.0, pos.2 as f32 + 1.0 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },

                Vertex { position: [ pos.0 as f32, pos.1 as f32, pos.2 as f32 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 }, // -X
                Vertex { position: [ pos.0 as f32, pos.1 as f32 + 1.0, pos.2 as f32 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32, pos.1 as f32, pos.2 as f32 + 1.0 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32, pos.1 as f32 + 1.0, pos.2 as f32 + 1.0 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },

                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32, pos.2 as f32 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 }, // +X
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32 + 1.0, pos.2 as f32 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32, pos.2 as f32 + 1.0 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32 + 1.0, pos.2 as f32 + 1.0 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },

                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32, pos.2 as f32 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 }, // -Y
                Vertex { position: [ pos.0 as f32, pos.1 as f32, pos.2 as f32 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32, pos.2 as f32 + 1.0 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32, pos.1 as f32, pos.2 as f32 + 1.0 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },

                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32 + 1.0, pos.2 as f32 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 }, // +Y
                Vertex { position: [ pos.0 as f32, pos.1 as f32 + 1.0, pos.2 as f32 ], uv: [( 1.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32 + 1.0, pos.1 as f32 + 1.0, pos.2 as f32 + 1.0 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 1.0], array_index: 0, light: 0.0 },
                Vertex { position: [ pos.0 as f32, pos.1 as f32 + 1.0, pos.2 as f32 + 1.0 ], uv: [( 0.0 + self.select_timer as f32 ) / 13.0, 0.0], array_index: 0, light: 0.0 },
            ];
            let sel_indices: Vec<u16> = vec![
                0, 1, 2,
                1, 3, 2,
                4, 6, 5,
                5, 6, 7,
                8, 10, 9,
                9, 10, 11,
                12, 13, 14,
                13, 15, 14,
                16, 18, 17,
                17, 18, 19,
                20, 21, 22,
                21, 23, 22
            ];

            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&sel_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&sel_indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );
            let num_indices = sel_indices.len() as u32;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: out_view,
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

            render_pass.set_pipeline(&self.selector_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.selector_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }


        Ok(encoder)
    }
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}



#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
    array_index: u32,
    light: f32,

}

impl Vertex {
    pub fn new(position: [f32; 3], uv: [f32; 2], array_index: u32, light: f32) -> Vertex {
        Self{
            position,
            uv,
            array_index,
            light
        }
    }

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                }
            ]
        }
    }
}
