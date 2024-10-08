

use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::path::PathBuf;
use std::io::{
    Write,
    Read
};


use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
    window::Window,
    dpi::PhysicalPosition,
};

use wgpu::util::DeviceExt;

mod camera;

mod chunk;

mod texture;
mod atlas_tex;

mod block;
mod rotation_group;

mod data_loader;

mod ui;
mod world;
mod world_loader;
mod world_saver;

#[derive(Clone, Copy)]
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
    mouse_pressed: MouseOps,
    ui_core: ui::UICore,
    ui_mode: ui::UIMode,
    world_render: Option<world::WorldRender>,
    world_loader: world_loader::WorldLoader,
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

        let mouse_pressed = MouseOps{left: false, right: false, left_just_now: false, right_just_now: false };


        let mut ui_core = ui::UICore::new(&config.format, &config, &device, &queue,);
        let ui_mode = ui::UIMode::MainTitle;

        let mut wl = world_loader::WorldLoader {
            previews: Vec::<world_loader::WorldPreview>::new(),
            name_map: HashMap::<String, usize>::new(),
        };
        let pvs = wl.load_previews( &ui_core.get_gfx(&device, &queue) );

        ui_core.update_world_list( pvs, &config, &device, &queue );

        //let wss = Self::load_world().unwrap_or( world::WorldSavestate::new(0) );
        //let wr = world::WorldRender::new(&device, &queue, &config, wss);

        Self {
            surface,
            device,
            size,
            window,
            queue,
            config,
            mouse_pressed,
            ui_core,
            ui_mode,
            world_render: None,
            world_loader: wl,
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

            if let Some(ref mut wr) = &mut self.world_render {
                wr.resize_window(&self.device, &self.config);
            }

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
                    } => {
                        if let Some(ref mut wr) = &mut self.world_render {
                            wr.process_keyboard(key, state)
                        } else {
                            false
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        //self.camera_controller.process_scroll(delta);
                        let shift = match delta {
                            MouseScrollDelta::LineDelta(_, scroll) => *scroll,
                            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
                        };

                        match self.ui_mode {
                            ui::UIMode::Gameplay => {
                                if let Some(ref mut wr) = &mut self.world_render {
                                    wr.scroll_shift(shift);

                                    self.ui_core.update_wield_item(
                                        ui::WieldItem::Block(wr.world.block_select),
                                        &self.device,
                                        &self.queue,
                                        &wr.block_registry,
                                        &wr.shape_registry,
                                        Some( ( &wr.render_pipeline, &wr.camera_bind_group_layout, &wr.diffuse_bind_group, &wr.colormap_bind_group ) ),
                                    );
                                }
                            }
                            _ => {

                            }
                        }

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
                    WindowEvent::KeyboardInput{ event, .. } => {
                        self.ui_core.keyboard_input( self.ui_mode, event.clone() )
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
            ui::UIMode::WorldSelection => {
                let pvs = self.world_loader.load_previews( &self.ui_core.get_gfx(&self.device, &self.queue) );
                self.ui_core.update_world_list( pvs, &self.config, &self.device, &self.queue );
                self.ui_mode = ui::UIMode::WorldSelection;
            }
            _ => {
                let _ = self.window.set_cursor_grab(winit::window::CursorGrabMode::None);
                self.window.set_cursor_visible(true);
                self.ui_mode = new;
            }

        }

    }

    fn update(&mut self, dt: std::time::Duration) {
        if let Some(new) = self.ui_core.update(self.ui_mode) {
            self.update_ui_mode( new );
        }

        match self.ui_mode {
            ui::UIMode::Gameplay => {
                if let Some(ref mut wr) = &mut self.world_render {
                    wr.update( &self.queue, self.mouse_pressed, dt );
                }
            }
            ui::UIMode::LoadWorld => {
                if let Some(name) = self.ui_core.world_selected_name.clone() {
                    self.ui_core.world_selected_name = None;
                    let pv = &self.world_loader.previews[ *self.world_loader.name_map.get(&name).unwrap() ];
                    let world_load = pv.load_world();

                    if let Ok(wss) = world_load {
                        self.world_render = Some( world::WorldRender::new(&self.device, &self.queue, &self.config, wss, name) );
                        self.ui_mode = ui::UIMode::PauseMenu;
                    }
                }
            }
            ui::UIMode::QuitGameplay => {
                let mut worldsaver = world_saver::WorldSaver{};
                worldsaver.save_world( &self.world_render.as_ref().unwrap(), &self.device, &self.queue );
                self.world_render = None;
                self.ui_mode = ui::UIMode::MainTitle;
            }
            ui::UIMode::CreateWorld => {
                let name = self.ui_core.world_selected_name.clone().expect("missing world name!");
                let wss = world::WorldSavestate::new(0);
                let mut wr = world::WorldRender::new(&self.device, &self.queue, &self.config, wss, name.clone() );
                wr.update_chunks();
                self.world_render = Some(wr);
                self.ui_mode = ui::UIMode::PauseMenu;
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

        // create world draw encoder if applicable
        let world_encoder = if let Some(ref mut wr) = &mut self.world_render {
            wr.render(&self.device, &self.queue, &view).expect("Error rendering the world")
        } else {
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: Some("Empty Encoder"), })
        };

        // create ui draw encoder
        let ui_encoder = self.ui_core.draw( self.ui_mode, (self.config.width, self.config.height), &view, &self.device, &self.queue );

        // submit will accept anything that implements IntoIter
        let draw_vec = vec![ world_encoder.finish(), ui_encoder.expect("Error rendering the UI").finish() ];
        self.queue.submit( draw_vec.into_iter() );
        output.present();

        Ok(())
    }


}






pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;
    let mut last_render_time = Instant::now();

    //let mut last_update = Instant::now();

    let _ = event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => {
                if state.ui_mode == ui::UIMode::Gameplay {
                    if let Some(ref mut wr) = &mut state.world_render {
                        wr.camera_controller.process_mouse(delta.0, delta.1);
                    }
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
                        } else if state.ui_mode == ui::UIMode::WorldSelection {
                            state.update_ui_mode( ui::UIMode::MainTitle );
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
                if state.ui_mode == ui::UIMode::Quit {
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

