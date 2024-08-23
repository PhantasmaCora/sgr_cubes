
use std::time::{Instant, Duration};
use std::path::PathBuf;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
    window::Window,
    dpi::PhysicalPosition,
};

use wgpu::util::DeviceExt;

use cgmath::SquareMatrix;

use grid_ray::GridRayIter3;

mod camera;

mod chunk;

mod texture;
mod atlas_tex;

mod block;
mod rotation_group;

mod data_loader;

mod ui;

struct MouseOps {
    left: bool,
    right: bool,
    left_just_now: bool,
    right_just_now: bool
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window, // The window must be declared after the surface so it gets dropped after it as the surface contains unsafe references to the window's resources.
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    camera: camera::Camera,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    block_atlas: atlas_tex::AtlasTexture,
    mouse_pressed: MouseOps,
    block_registry: block::BlockRegistry,
    shape_registry: block::BlockShapeRegistry,
    chunk_manager: chunk::ChunkManager,
    colormap_bind_group: wgpu::BindGroup,
    block_select: u16,
    selector_pipeline: wgpu::RenderPipeline,
    selector_bind_group: wgpu::BindGroup,
    selected_block: Option<(usize, usize, usize)>,
    select_timer: u16,
    last_qsecond: Instant,
    ui_core: ui::UICore,
    ui_mode: ui::UIMode,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let mut feature_list = wgpu::Features::empty();
        feature_list.insert( wgpu::Features::CLEAR_TEXTURE );
        feature_list.insert( wgpu::Features::PUSH_CONSTANTS );

        let mut lim = wgpu::Limits::default();
        lim.max_push_constant_size = 128;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: feature_list,
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
               required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    lim
                },
                label: None,
                memory_hints: wgpu::MemoryHints::Performance
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);





        let pal_bytes = include_bytes!("../res/texture/core/palette.png");
        let pal_img = image::load_from_memory(pal_bytes).unwrap();

        let mut dl = data_loader::BlockLoader::create(&device, &queue);
        let _ = dl.submit_blockshape_direct( block::make_cube_shape(), &"CubeStatic".into() );
        let _ = dl.submit_blockshape_direct( block::make_slope_shape(), &"Slope".into() );

        let res_0 = dl.load_toml_from_file( PathBuf::from("res/data/block.toml") );
        let res_a = dl.do_extract();
        let res_b = dl.resolve_blocks( &device, &queue, &pal_img );

        let block_registry = dl.block_registry;
        let block_atlas = dl.texture_atlas;
        let shape_registry = dl.shape_registry;

        let chunk_manager = chunk::ChunkManager::new();


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

        let colormap_bytes = include_bytes!("../res/texture/core/colormap.png");
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






        let camera = camera::Camera::new((63.0, 35.0, 62.0), cgmath::Deg(90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(7.0, 0.37);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

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
            source: wgpu::ShaderSource::Wgsl(include_str!("block_shader.wgsl").into()),
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
            source: wgpu::ShaderSource::Wgsl(include_str!("selected_block_shader.wgsl").into()),
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

        let selector_bytes = include_bytes!("../res/texture/core/selector.png");
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





        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;

        let mouse_pressed = MouseOps{left: false, right: false, left_just_now: false, right_just_now: false };
        let block_select = 1;


        let ui_core = ui::UICore::new(&config.format, &config, &device, &queue,);
        let ui_mode = ui::UIMode::PauseMenu;

        Self {
            surface,
            device,
            size,
            window,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            depth_texture,
            diffuse_bind_group,
            block_atlas,
            block_registry,
            shape_registry,
            chunk_manager,
            mouse_pressed,
            colormap_bind_group,
            block_select,
            selector_pipeline,
            selector_bind_group,
            selected_block: None,
            select_timer: 0,
            last_qsecond: Instant::now(),
            ui_core,
            ui_mode,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.projection.resize(new_size.width, new_size.height);

            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.ui_core.resize( new_size, &self.device, &self.queue );
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match self.ui_mode {
            ui::UIMode::Gameplay => {

                match event {
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(key),
                                state,
                                ..
                            },
                        ..
                    } => self.camera_controller.process_keyboard(*key, *state),
                    WindowEvent::MouseWheel { delta, .. } => {
                        //self.camera_controller.process_scroll(delta);
                        let shift = match delta {
                            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
                            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
                        };
                        let shift_i = ( shift * -2.0 ) as i32;
                        let t_select = self.block_select as i32 + shift_i;
                        self.block_select = if t_select <= 0 {
                            self.block_registry.get_num_blocks() - 1
                        } else if t_select >= self.block_registry.get_num_blocks() as i32 {
                            1 as u16
                        } else {
                            t_select as u16
                        };

                        self.ui_core.update_wield_item(
                            ui::WieldItem::Block(self.block_select),
                            &self.device,
                            &self.queue,
                            &self.block_registry,
                            &self.shape_registry,
                            Some( ( &self.render_pipeline, &self.camera_bind_group_layout, &self.diffuse_bind_group, &self.colormap_bind_group ) ),
                        );

                        true
                    }
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state,
                        ..
                    } => {
                        self.mouse_pressed.left = *state == ElementState::Pressed;
                        self.mouse_pressed.left_just_now = self.mouse_pressed.left;
                        true
                    }
                    WindowEvent::MouseInput {
                        button: MouseButton::Right,
                        state,
                        ..
                    } => {
                        self.mouse_pressed.right = *state == ElementState::Pressed;
                        self.mouse_pressed.right_just_now = self.mouse_pressed.right;
                        true
                    }
                    _ => false,
                }

            }
            _ => {
                match event {
                    WindowEvent::CursorMoved{ device_id, position } => {
                        self.ui_core.cursor_moved( self.ui_mode, position );
                        true
                    }
                    WindowEvent::MouseInput{ state, button, .. } => {
                        self.ui_core.mouse_input( self.ui_mode, state, button );
                        true
                    }
                    _ => {
                        false
                    }
                }
            }
        }
    }

    fn update_ui_mode(&mut self, new: ui::UIMode) {
        match new {
            ui::UIMode::Gameplay => {
                let _ = self.window
                .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                .or_else(|_e| self.window.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                self.window.set_cursor_visible(false);
                self.ui_mode = ui::UIMode::Gameplay;
            }
            ui::UIMode::PauseMenu => {
                let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::None);
                self.window.set_cursor_visible(true);
                self.ui_mode = ui::UIMode::PauseMenu;
            }
            ui::UIMode::QuitGameplay => {
                let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::None);
                self.window.set_cursor_visible(true);
                self.ui_mode = ui::UIMode::QuitGameplay;
            }
        }

    }

    fn update(&mut self, dt: std::time::Duration) {
        self.update_ui_mode( self.ui_core.update(self.ui_mode) );

        match self.ui_mode {
            ui::UIMode::Gameplay => {
                self.camera_controller.update_camera(&mut self.camera, dt);
                self.camera_uniform.update_view_proj(&self.camera, &self.projection);
                self.queue.write_buffer(
                    &self.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[self.camera_uniform]),
                );

                // do block breaking and placing
                let mut last = grid_ray::ilattice::glam::IVec3::NEG_ONE;
                let mut current = grid_ray::ilattice::glam::IVec3::NEG_ONE;
                let dir = self.camera.get_forward_vector();
                let mut ray_iter = GridRayIter3::new(
                    grid_ray::ilattice::glam::Vec3A::new( self.camera.position.x, self.camera.position.y, self.camera.position.z ), // start position
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
                        let bdef = self.chunk_manager.get_block( ( current.x as usize, current.y as usize, current.z as usize ) ).blockdef;
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

                if hit && self.mouse_pressed.left_just_now {
                    let mut broken = self.chunk_manager.get_mut_block( ( current.x as usize, current.y as usize, current.z as usize ) );
                    broken.blockdef = 0;
                    broken.exparam = 0;
                } else if hit && self.mouse_pressed.right_just_now && (last.x >= 0 && last.y >= 0 && last.z >= 0 &&
                    last.x < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32 && last.y < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32 && last.z < (chunk::CHUNK_SIZE * chunk::WORLD_CHUNKS) as i32) {
                    let mut placed = self.chunk_manager.get_mut_block( ( last.x as usize, last.y as usize, last.z as usize ) );
                    placed.blockdef = self.block_select;
                    placed.exparam = 0;
                }


                {
                    self.chunk_manager.update_dirty_chunks( &self.block_registry, &self.shape_registry );
                }

                let now = Instant::now();
                if now.duration_since(self.last_qsecond) >= Duration::new(0, 250_000_000) {
                    self.last_qsecond = now;
                    self.select_timer += 1;
                    self.select_timer %= 13;
                }
            }
            ui::UIMode::PauseMenu => {

            }
            _ => {

            }

        }

        self.mouse_pressed.left_just_now = false;
        self.mouse_pressed.right_just_now = false;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let draw_chunk_list = self.chunk_manager.get_render_chunks();

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut first = true;

        for idx in 0..draw_chunk_list.len() {
            let c = &draw_chunk_list[idx];

            let vertex_buffer = self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&c.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            let index_buffer = self.device.create_buffer_init(
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
                    view: &view,
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

            let vertex_buffer = self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&sel_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            let index_buffer = self.device.create_buffer_init(
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
                    view: &view,
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

        // create ui draw encoder
        let ui_encoder = self.ui_core.draw( self.ui_mode, (self.config.width, self.config.height), &view, &self.device, &self.queue );

        // submit will accept anything that implements IntoIter
        let draw_vec = vec![ encoder.finish(), ui_encoder.expect("Error rendering the UI").finish() ];
        self.queue.submit( draw_vec.into_iter() );
        output.present();

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
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

// starting empty buffer states
const VERTICES: &[Vertex] = &[
    Vertex { position: [-50., -1., -50.0], uv: [0.0, 0.0], array_index: 0, light: 1.0 },
    Vertex { position: [-50., -1., 50.0], uv: [0.0, 0.0], array_index: 0, light: 1.0 },
    Vertex { position: [50., -1., -50.0], uv: [0.0, 0.0], array_index: 0, light: 1.0 },
    Vertex { position: [50., -1., 50.0], uv: [0.0, 0.0], array_index: 0, light: 1.0 },
];

const INDICES: &[u16] = &[
    0, 1, 2,
    1, 3, 2,
];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;
    let mut last_render_time = Instant::now();

    {
        let bi_2 = state.chunk_manager.get_mut_block( ( 64, 32, 63 ) );
        bi_2.blockdef = 3;
    }

    {
        state.chunk_manager.update_dirty_chunks( &state.block_registry, &state.shape_registry );
    }

    //let mut last_update = Instant::now();

    let _ = event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => {
                if state.ui_mode == ui::UIMode::Gameplay {
                    state.camera_controller.process_mouse(delta.0, delta.1)
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        if state.ui_mode == ui::UIMode::Gameplay {
                            state.update_ui_mode( ui::UIMode::PauseMenu );
                        } else if state.ui_mode == ui::UIMode::PauseMenu {
                            state.update_ui_mode( ui::UIMode::Gameplay );
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::Focused(has) => {
                        if *has && state.ui_mode == ui::UIMode::Gameplay {
                            let _ = state.window
                            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                            .or_else(|_e| state.window.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                            state.window.set_cursor_visible(false);
                        } else {
                            let _ = state.window.set_cursor_grab(winit::window::CursorGrabMode::None);
                            state.window.set_cursor_visible(true);
                        }
                    }
                    WindowEvent::RedrawRequested if window_id == state.window().id() => {
                        let now = Instant::now();
                        let dt = now - last_render_time;
                        last_render_time = now;
                        state.update(dt);

                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                if state.ui_mode == ui::UIMode::QuitGameplay {
                    control_flow.exit();
                }


                // RedrawRequested will only trigger once unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }

    });


}

