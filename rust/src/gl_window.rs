#![cfg(not(target_arch = "wasm32"))]

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
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::{controller::Controller, gl_renderer::Renderer, Scene};

const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 600;

pub fn spawn_window(scene: Scene) -> anyhow::Result<()> {
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
    let mut controller = Controller::new(renderer, scene, window_size.width, window_size.height);

    controller.resize(window_size.width, window_size.height);

    let mut current_cursor_pos: Option<PhysicalPosition<f64>> = None;
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    use winit::keyboard::{KeyCode, PhysicalKey};
                    if let PhysicalKey::Code(code) = event.physical_key {
                        if code == KeyCode::Escape || code == KeyCode::KeyQ {
                            elwt.exit();
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
                                }
                            }
                            winit::event::ElementState::Released => {
                                controller.handle_mouse_release();
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
                        surface.swap_buffers(&context).unwrap();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    current_cursor_pos = Some(position);
                    controller.handle_mouse_move(position.x as u32, position.y as u32);
                    controller.render();
                    surface.swap_buffers(&context).unwrap();
                }
                WindowEvent::Resized(size) => {
                    surface.resize(
                        &context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );

                    controller.resize(size.width, size.height);
                    controller.render();
                    surface.swap_buffers(&context).unwrap();
                }
                WindowEvent::RedrawRequested => {
                    controller.render();
                    surface.swap_buffers(&context).unwrap();
                }
                _ => (),
            },
            Event::AboutToWait => {
                // This is called right before the event loop starts waiting for new events
                // It's a good place to do cleanup when exiting
                if elwt.exiting() {
                    controller.cleanup();
                }
            }
            _ => (),
        }
    })?;

    Ok(())
}
