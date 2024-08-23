 
use std::io::Error;


use wgpu::util::DeviceExt;

use cgmath::Angle;

use figures::units::{
    Px,
    UPx
};

use cushy::{
    styles,
    widgets
};
use cushy::widget::MakeWidget;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UIMode {
    Gameplay,
    PauseMenu
}

pub struct UICore {
    crosshair_tex: crate::wctx::texture::Texture,
    crosshair_bind_group: wgpu::BindGroup,
    generic_pipeline: wgpu::RenderPipeline,
    wield_tex: crate::wctx::texture::Texture,
    wield_dt: crate::wctx::texture::Texture,
    wielditem_bind_group: wgpu::BindGroup,
    overlaid_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    pause_menu_tex: crate::wctx::texture::Texture,
    pause_bind_group: wgpu::BindGroup,
    pause_buttonmenu: cushy::window::VirtualWindow,
    pause_buttons_tex: cushy::kludgine::Texture,
    pause_buttons_srctex: crate::wctx::texture::Texture,
    pause_buttons_bind_group: wgpu::BindGroup,
    pub wants_to_quit: bool,
}

impl UICore {
    pub fn new(out_format: &wgpu::TextureFormat, config: &wgpu::SurfaceConfiguration, device: &wgpu::Device, queue: &wgpu::Queue, ) -> UICore {

        let crosshair_bytes = include_bytes!("../../res/texture/core/crosshair.png");
        let crosshair_tex = crate::wctx::texture::Texture::from_bytes(&device, &queue, crosshair_bytes, &"Crosshair Texture").unwrap();

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

        let gen_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Generic UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../ui_generic_shader.wgsl").into()),
        });
        let ui_generic_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gen UI Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let generic_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Generic UI Pipeline"),
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

        let layer_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Overlaid UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../ui_overlaid_shader.wgsl").into()),
        });
        let ui_overlay_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Overlaid UI Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let overlaid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Overlaid UI Pipeline"),
            layout: Some(&ui_overlay_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &layer_shader,
                entry_point: "vs_main",
                buffers: &[
                    BMVertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &layer_shader,
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

        let wield_tex = crate::wctx::texture::Texture::from_descriptor(
            device, queue,
            &wgpu::TextureDescriptor {
                label: Some("WieldItem texture"),
                size: wgpu::Extent3d {
                    width: 1024,
                    height: 1024,
                    depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[]
            },
            wgpu::TextureViewDimension::D2
        ).expect("failed to create wielditem texture");

        let wielditem_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&wield_tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&wield_tex.sampler),
                    }
                ],
                label: Some("wielditem_bind_group"),
            }
        );

        let wield_dt = crate::wctx::texture::Texture::create_sized_depth_texture( device, wield_tex.texture.size(), "WieldItem Depth Texture" );

        let pause_bytes = include_bytes!("../../res/texture/ui/pause_menu_bg.png");
        let pause_menu_tex = crate::wctx::texture::Texture::from_bytes(&device, &queue, pause_bytes, &"Pause Menu Texture").unwrap();

        let pause_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&pause_menu_tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&pause_menu_tex.sampler),
                    }
                ],
                label: Some("wielditem_bind_group"),
            }
        );


        let mut menub_list = cushy::widget::WidgetList::new();

        let mut return_button = widgets::Button::new( widgets::Label::<&str>::new("Continue") );
        return_button = return_button.kind( widgets::button::ButtonKind::Solid );
        return_button = return_button.on_click( |_click| {  } );
        menub_list.push(widgets::Expand::new(return_button));

        let mut quit_button = widgets::Button::new( widgets::Label::<&str>::new("Quit") );
        quit_button = quit_button.kind( widgets::button::ButtonKind::Solid );
        quit_button = quit_button.on_click( |_click| {  } );
        menub_list.push(widgets::Expand::new(quit_button));


        let menub_stack = widgets::Stack::rows( menub_list );

        let mut styles = styles::Styles::new();
        styles.insert( &styles::components::CornerRadius, styles::CornerRadii{ top_left: figures::units::Px::new(0), top_right: figures::units::Px::new(0), bottom_left: figures::units::Px::new(0), bottom_right: figures::units::Px::new(0) } );
        styles.insert( &styles::components::OutlineColor, styles::Color::new(0,0,0,0) );
        styles.insert( &styles::components::HighlightColor, styles::Color::new(0,0,0,0) );

        styles.insert( &widgets::button::ButtonOutline, styles::Color::new(0,0,0,0) );
        styles.insert( &widgets::button::ButtonHoverOutline, styles::Color::new(0,0,0,0) );
        styles.insert( &widgets::button::ButtonDisabledOutline, styles::Color::new(0,0,0,0) );
        styles.insert( &widgets::button::ButtonActiveOutline, styles::Color::new(0,0,0,0) );

        styles.insert( &widgets::button::ButtonBackground, styles::Color::new(0,0,0,230) );
        styles.insert( &widgets::button::ButtonHoverBackground, styles::Color::new(108,96,24,210) );
        styles.insert( &widgets::button::ButtonDisabledBackground, styles::Color::new(128,128,128,240) );
        styles.insert( &widgets::button::ButtonActiveBackground, styles::Color::new(128,0,0,200) );

        styles.insert( &styles::components::BaseTextSize, styles::Dimension::Px( Px::new(48) ) );

        let menub_style = widgets::Style::new( styles, menub_stack );

        let menub_align = widgets::Align::new( styles::Edges { left: styles::FlexibleDimension::Dimension( styles::Dimension::Px(0.into()) ), right: styles::FlexibleDimension::Dimension( styles::Dimension::Px( (config.width as f32 * 0.45).into()) ), top: styles::FlexibleDimension::Dimension( styles::Dimension::Px(50.into()) ), bottom: styles::FlexibleDimension::Dimension( styles::Dimension::Px(50.into()) ) }, menub_style );

        let mut builder = cushy::window::StandaloneWindowBuilder::new( menub_align ).transparent();
        builder = builder.size( figures::Size { width: config.width, height: config.height } );
        //builder = builder.multisample_count(1);
        let mut pause_buttonmenu = builder.finish_virtual(device, queue);

        let pause_buttons_tex = cushy::kludgine::Texture::multisampled(
            &pause_buttonmenu.graphics(device, queue),
            4,
            figures::Size{width: UPx::new(config.width), height: UPx::new(config.height)},
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest
        );

        let pause_buttons_srctex = crate::wctx::texture::Texture::from_descriptor(
            device,
            queue,
            &wgpu::TextureDescriptor {
                label: Some("pause buttons final source tex"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            wgpu::TextureViewDimension::D2
        ).expect("texture creation failed");

        let pause_buttons_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&pause_buttons_srctex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&pause_buttons_srctex.sampler),
                    }
                ],
                label: Some("pause_buttons_bind_group"),
            }
        );



        let wants_to_quit = false;

        Self{
            crosshair_tex,
            crosshair_bind_group,
            generic_pipeline,
            overlaid_pipeline,
            texture_bind_group_layout,
            wield_tex,
            wield_dt,
            wielditem_bind_group,
            pause_menu_tex,
            pause_bind_group,
            pause_buttonmenu,
            pause_buttons_tex,
            pause_buttons_srctex,
            pause_buttons_bind_group,
            wants_to_quit,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.pause_buttonmenu.resize( figures::Size { width: UPx::new(new_size.width), height: UPx::new(new_size.height) }, 1.0, queue );

        self.pause_buttons_tex = cushy::kludgine::Texture::multisampled(
            &self.pause_buttonmenu.graphics(device, queue),
            4,
            figures::Size{width: UPx::new(new_size.width), height: UPx::new(new_size.height)},
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest
        );

        self.pause_buttons_srctex = crate::wctx::texture::Texture::from_descriptor(
            device,
            queue,
            &wgpu::TextureDescriptor {
                label: Some("pause buttons final source tex"),
                size: wgpu::Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            wgpu::TextureViewDimension::D2
        ).expect("texture creation failed");

        self.pause_buttons_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.pause_buttons_srctex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.pause_buttons_srctex.sampler),
                    }
                ],
                label: Some("pause_buttons_bind_group"),
            }
        );
    }

    pub fn cursor_moved(&mut self, mode: UIMode, position: &winit::dpi::PhysicalPosition<f64> ) {
        match mode {
            UIMode::PauseMenu => { self.pause_buttonmenu.cursor_moved(cushy::window::DeviceId::Virtual(0), figures::Point::new( figures::units::Px::new(position.x as i32), figures::units::Px::new(position.y as i32) ) ); }
            _ => {}
        }
    }

    pub fn mouse_input(&mut self, mode: UIMode, state: &winit::event::ElementState, button: &winit::event::MouseButton ) {
        let kstate = match state {
            &winit::event::ElementState::Pressed => cushy::kludgine::app::winit::event::ElementState::Pressed,
            &winit::event::ElementState::Released => cushy::kludgine::app::winit::event::ElementState::Released
        };
        let kbutton = match button {
            &winit::event::MouseButton::Left => cushy::kludgine::app::winit::event::MouseButton::Left,
            &winit::event::MouseButton::Right => cushy::kludgine::app::winit::event::MouseButton::Right,
            &winit::event::MouseButton::Middle => cushy::kludgine::app::winit::event::MouseButton::Middle,
            &winit::event::MouseButton::Back => cushy::kludgine::app::winit::event::MouseButton::Back,
            &winit::event::MouseButton::Forward => cushy::kludgine::app::winit::event::MouseButton::Forward,
            &winit::event::MouseButton::Other(id) => cushy::kludgine::app::winit::event::MouseButton::Other(id)
        };

        match mode {
            UIMode::PauseMenu => { self.pause_buttonmenu.mouse_input(cushy::window::DeviceId::Virtual(0), kstate, kbutton); }
            _ => {}
        }
    }

    pub fn draw(&mut self, mode: UIMode, target_size: (u32, u32), target_view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<wgpu::CommandEncoder, Error> {
        match mode {
            UIMode::Gameplay => {
                // setup stuff
                let mut crosshair_size = std::cmp::min( target_size.0 / 200, target_size.1 / 200 );
                crosshair_size = std::cmp::max( crosshair_size, 1 );
                crosshair_size *= 8;

                let mut wielditem_size = std::cmp::min( target_size.0 / 120, target_size.1 / 120 );
                wielditem_size = std::cmp::max( crosshair_size, 1 );
                wielditem_size *= 16;

                // begin drawing!
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("UI Render Encoder"),
                });

                // draw the crosshair
                {
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

                    render_pass.set_pipeline(&self.generic_pipeline);
                    render_pass.set_bind_group(0, &self.crosshair_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..num_indices, 0, 0..1);
                }

                // draw the wield item
                {


                    let wielditem_vertices = vec![
                        UIVertex{ position: [ -0.9 - wielditem_size as f32 / target_size.0 as f32, -0.9 + wielditem_size as f32 / target_size.1 as f32 ], uv: [0.0, 0.0] },
                        UIVertex{ position: [ -0.9 + wielditem_size as f32 / target_size.0 as f32, -0.9 + wielditem_size as f32 / target_size.1 as f32 ], uv: [1.0, 0.0] },
                        UIVertex{ position: [ -0.9 - wielditem_size as f32 / target_size.0 as f32, -0.9 - wielditem_size as f32 / target_size.1 as f32 ], uv: [0.0, 1.0] },
                        UIVertex{ position: [ -0.9 + wielditem_size as f32 / target_size.0 as f32, -0.9 - wielditem_size as f32 / target_size.1 as f32 ], uv: [1.0, 1.0] },
                    ];
                    let wielditem_indices: Vec<u16> = vec![
                        0, 2, 1,
                        1, 2, 3
                    ];

                    let vertex_buffer = device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&wielditem_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        }
                    );
                    let index_buffer = device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&wielditem_indices),
                            usage: wgpu::BufferUsages::INDEX,
                        }
                    );
                    let num_indices = wielditem_indices.len() as u32;

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

                    render_pass.set_pipeline(&self.generic_pipeline);
                    render_pass.set_bind_group(0, &self.wielditem_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..num_indices, 0, 0..1);
                }

                Ok(encoder)
            }
            UIMode::PauseMenu => {
                // setup stuff
                let rw = self.pause_menu_tex.texture.width() as f32 / target_size.0 as f32;
                let rh = self.pause_menu_tex.texture.height() as f32 / target_size.1 as f32;
                let mut bgx = 0.5;
                let mut bgy = 0.5;
                if rw < rh {
                    bgy = rw / rh;
                } else {
                    bgx = rh / rw;
                }

                // redraw the button menu
                self.pause_buttonmenu.prepare(device, queue);
                self.pause_buttonmenu.render_into(
                    &self.pause_buttons_tex,
                    wgpu::LoadOp::Clear( cushy::styles::Color::new(0, 0, 0, 0) ),
                    device,
                    queue
                );

                 let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("UI Render Encoder"),
                });

                // copy the button menu over
                encoder.copy_texture_to_texture(
                    wgpu::ImageCopyTexture{
                        texture: self.pause_buttons_tex.wgpu(),
                        mip_level: 0,
                        origin: wgpu::Origin3d{x: 0, y: 0, z: 0},
                        aspect: wgpu::TextureAspect::All
                    },
                    wgpu::ImageCopyTexture{
                        texture: &self.pause_buttons_srctex.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d{x: 0, y: 0, z: 0},
                        aspect: wgpu::TextureAspect::All
                    },
                    self.pause_buttons_srctex.texture.size()
                );

                // draw the menu bg
                {
                    let bg_vertices = vec![
                        BMVertex{ position: [ -1.0, 1.0 ], uv: [0.0, 0.0], uv2: [0.0, 0.0] },
                        BMVertex{ position: [ 1.0, 1.0 ], uv: [1.0, 0.0], uv2: [1.0, 0.0] },
                        BMVertex{ position: [ -1.0, -1.0 ], uv: [0.0, 1.0], uv2: [0.0, 1.0] },
                        BMVertex{ position: [ 1.0, -1.0 ], uv: [1.0, 1.0], uv2: [1.0, 1.0] },
                    ];
                    let bg_indices: Vec<u16> = vec![
                        0, 2, 3,
                        0, 3, 1
                    ];

                    let vertex_buffer = device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&bg_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        }
                    );
                    let index_buffer = device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&bg_indices),
                            usage: wgpu::BufferUsages::INDEX,
                        }
                    );
                    let num_indices = bg_indices.len() as u32;

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

                    render_pass.set_pipeline(&self.overlaid_pipeline);
                    render_pass.set_bind_group(0, &self.pause_bind_group, &[]);
                    render_pass.set_bind_group(1, &self.pause_buttons_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..num_indices, 0, 0..1);
                }

                Ok(encoder)
            }

        }

    }

    pub fn update_wield_item( &mut self, wi: WieldItem, device: &wgpu::Device, queue: &wgpu::Queue, br: &crate::wctx::block::BlockRegistry, sr: &crate::wctx::block::BlockShapeRegistry, block_render_setup: Option< (&wgpu::RenderPipeline, &wgpu::BindGroupLayout, &wgpu::BindGroup, &wgpu::BindGroup) > ) {
        match wi {
            WieldItem::Block(block_id) => {
                let setup = block_render_setup.expect("Some(Block render pipeline) is REQUIRED for drawing block to wielditem texture, found None");
                let render_pipeline = setup.0;

                let mut tverts = Vec::<crate::wctx::Vertex>::new();
                let mut tinds = Vec::<u16>::new();

                // get block data
                let blockdef = br.get(block_id).expect("Failed to find block in registry");
                let shapedef = sr.get(blockdef.shape_id).expect("Failed to find shape in registry");
                shapedef.generate_draw_buffers(
                    &mut tverts,
                    &mut tinds,
                    blockdef,
                    0,
                    crate::wctx::chunk::BlockDrawContext::default(),
                    (0,0,0),
                    (0,0,0)
                );

                let vertex_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&tverts),
                        usage: wgpu::BufferUsages::VERTEX,
                    }
                );
                let index_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&tinds),
                        usage: wgpu::BufferUsages::INDEX,
                    }
                );
                let num_indices = tinds.len() as u32;

                let view = &self.wield_tex.view;
                let dt_view = &self.wield_dt.view;

                let mut camera_uniform = crate::wctx::CameraUniform{ view_proj:
                    ( cgmath::Matrix4::from_translation( cgmath::Vector3::new( 0.5, 0.3, 0.5 ) ) * cgmath::Matrix4::from_nonuniform_scale(0.5, 0.5, 0.1) * cgmath::Matrix4::from_angle_x( cgmath::Rad::atan( 2.0_f32.sqrt() / 2.0 ) ) * cgmath::Matrix4::from_angle_y( cgmath::Rad::full_turn() / -8.0 ) * cgmath::Matrix4::from_translation( cgmath::Vector3::new( -0.5, -0.5, -0.5 ) ) ).into()
                };

                let camera_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Camera Buffer"),
                        contents: bytemuck::cast_slice(&[camera_uniform]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    }
                );

                let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: setup.1,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: camera_buffer.as_entire_binding(),
                        }
                    ],
                    label: Some("camera_bind_group"),
                });

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("WieldItem Render Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("WieldItem Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear( wgpu::Color::TRANSPARENT ),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &dt_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Discard,
                            }),
                            stencil_ops: None,
                        }),
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    render_pass.set_pipeline(render_pipeline);
                    render_pass.set_bind_group(0, &camera_bind_group, &[]);
                    render_pass.set_bind_group(1, setup.2, &[]);
                    render_pass.set_bind_group(2, setup.3, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..num_indices, 0, 0..1);
                }

                queue.submit( std::iter::once( encoder.finish() ) );
            }
            WieldItem::Sprite => {

            }
        }
    }

}






pub enum WieldItem {
    Block(u16),
    Sprite
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UIVertex {
    position: [f32; 2],
    uv: [f32; 2],

}

impl UIVertex {

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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BMVertex {
    position: [f32; 2],
    uv: [f32; 2],
    uv2: [f32; 2],
}

impl BMVertex {

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BMVertex>() as wgpu::BufferAddress,
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
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}
