use crate::{
    app_controller::AppController,
    graphics::{Renderer, Scene},
    Project,
};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, Version},
    display::GetGlDisplay,
    prelude::*,
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 600;

pub fn spawn_window(project: Project) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window_builder = WindowBuilder::new()
        .with_title("Layout Viewer")
        .with_inner_size(winit::dpi::LogicalSize::new(
            INITIAL_WINDOW_WIDTH,
            INITIAL_WINDOW_HEIGHT,
        ));

    let (window, gl, surface, context) = {
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(true);

        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
        let (window, gl_config) = display_builder
            .build(&event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        let transparency_check = config.supports_transparency().unwrap_or(false)
                            & !accum.supports_transparency().unwrap_or(false);
                        if transparency_check || config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let window = window.unwrap();
        let raw_window_handle = window.raw_window_handle();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(raw_window_handle));

        let gl_display = gl_config.display();
        let context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("Failed to create context")
        };

        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(INITIAL_WINDOW_WIDTH).unwrap(),
            NonZeroU32::new(INITIAL_WINDOW_HEIGHT).unwrap(),
        );

        let surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .expect("Failed to create surface")
        };

        let context = context
            .make_current(&surface)
            .expect("Failed to make context current");

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                let s = std::ffi::CString::new(s).unwrap();
                gl_display.get_proc_address(&s) as *const _
            })
        };

        (window, gl, surface, context)
    };

    let window_size = window.inner_size();

    let renderer = Renderer::new(gl);
    let scene = Scene::new();
    let mut controller = AppController::new(renderer, scene, window_size.width, window_size.height);

    controller.set_project(project);
    controller.resize(window_size.width, window_size.height);

    let mut current_cursor_pos: Option<PhysicalPosition<f64>> = None;
    let mut next_tick = Instant::now();
    let tick_interval = Duration::from_millis(16);

    let _ = event_loop.run(move |event, window_target| {
        if let Some(next_tick_time) = next_tick.checked_add(tick_interval) {
            window_target.set_control_flow(ControlFlow::WaitUntil(next_tick_time));
        }

        match event {
            Event::AboutToWait => {
                let now = Instant::now();
                if now >= next_tick {
                    if controller.tick() {
                        surface.swap_buffers(&context).unwrap();
                    }
                    next_tick = now + tick_interval;
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    controller.destroy();
                    window_target.exit();
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    use winit::keyboard::{KeyCode, PhysicalKey};
                    if let PhysicalKey::Code(code) = event.physical_key {
                        if code == KeyCode::Escape || code == KeyCode::KeyQ {
                            controller.destroy();
                            window_target.exit();
                        }
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    use winit::event::MouseButton;
                    if button == MouseButton::Left {
                        match state {
                            winit::event::ElementState::Pressed => {
                                if let Some(pos) = current_cursor_pos {
                                    controller.handle_mouse_press(pos.x as u32, pos.y as u32);
                                    controller.render();
                                }
                            }
                            winit::event::ElementState::Released => {
                                controller.handle_mouse_release();
                                controller.render();
                            }
                        }
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let Some(pos) = current_cursor_pos {
                        let delta_y = match delta {
                            winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                            winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                        };
                        controller.handle_mouse_wheel(pos.x as u32, pos.y as u32, delta_y);
                        controller.render();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    current_cursor_pos = Some(position);
                    let x = position.x as u32;
                    let y = position.y as u32;

                    controller.handle_mouse_move(x, y);
                    controller.render();
                }
                WindowEvent::Resized(size) => {
                    surface.resize(
                        &context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );

                    controller.resize(size.width, size.height);
                    controller.render();
                }
                WindowEvent::RedrawRequested => {
                    controller.render();
                }
                _ => (),
            },
            _ => (),
        }
    });

    Ok(())
}
