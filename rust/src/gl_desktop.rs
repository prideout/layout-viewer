#![cfg(not(target_arch = "wasm32"))]

use glow::*;
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
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::Scene;

pub struct GlCircle {
    gl: glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    mouse_in_circle: bool,
}

impl GlCircle {
    pub fn new(gl: glow::Context) -> Self {
        use glow::HasContext;

        let vertex_shader_source = r#"
            #version 330
            layout (location = 0) in vec2 position;
            uniform vec2 viewport;
            void main() {
                vec2 aspect = vec2(min(viewport.x/viewport.y, 1.0), min(viewport.y/viewport.x, 1.0));
                gl_Position = vec4(position * aspect, 0.0, 1.0);
            }
        "#;

        let fragment_shader_source = r#"
            #version 330
            uniform bool filled;
            out vec4 FragColor;
            void main() {
                float dist = length(gl_FragCoord.xy - vec2(gl_FragCoord.w) * 0.5);
                if (filled) {
                    FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                } else {
                    float thickness = 2.0;
                    float radius = 0.4;
                    float alpha = smoothstep(radius - thickness, radius, dist) - 
                                smoothstep(radius, radius + thickness, dist);
                    FragColor = vec4(1.0, 1.0, 1.0, alpha);
                }
            }
        "#;

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let mut shaders = Vec::with_capacity(shader_sources.len());

            for (shader_type, shader_source) in shader_sources.iter() {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");
                gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("{}", gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let vertices: [f32; 360 * 2] = {
                let mut v = [0.0; 360 * 2];
                for i in 0..360 {
                    let angle = i as f32 * std::f32::consts::PI / 180.0;
                    v[i * 2] = angle.cos() * 0.8;
                    v[i * 2 + 1] = angle.sin() * 0.8;
                }
                v
            };

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    vertices.len() * std::mem::size_of::<f32>(),
                ),
                glow::STATIC_DRAW,
            );

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);

            Self {
                gl,
                program,
                vao,
                vbo,
                mouse_in_circle: false,
            }
        }
    }

    pub fn render(&self, width: u32, height: u32) {
        unsafe {
            let gl = &self.gl;

            gl.viewport(0, 0, width as i32, height as i32);
            gl.clear_color(0.1, 0.1, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));

            let viewport_loc = gl.get_uniform_location(self.program, "viewport");
            gl.uniform_2_f32(viewport_loc.as_ref(), width as f32, height as f32);

            let filled_loc = gl.get_uniform_location(self.program, "filled");
            gl.uniform_1_i32(filled_loc.as_ref(), self.mouse_in_circle as i32);

            gl.draw_arrays(glow::LINE_STRIP, 0, 360);
        }
    }

    pub fn update_mouse(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // Convert to normalized device coordinates
        let nx = x / width * 2.0 - 1.0;
        let ny = -(y / height * 2.0 - 1.0);

        // Account for aspect ratio
        let aspect = width / height;
        let nx = if aspect > 1.0 { nx } else { nx * aspect };
        let ny = if aspect > 1.0 { ny / aspect } else { ny };

        // Check if mouse is inside circle
        self.mouse_in_circle = (nx * nx + ny * ny).sqrt() <= 0.8;
    }
}

impl Drop for GlCircle {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

pub fn run_gl_window(_scene: Scene) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window_builder = WindowBuilder::new()
        .with_title("Layout Viewer")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600));

    #[cfg(target_arch = "wasm32")]
    let (window, gl) = {
        use wasm_bindgen::JsCast;
        use web_sys::WebGl2RenderingContext;

        let window = window_builder.build(&event_loop)?;
        let canvas = window.canvas();
        let gl = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        let gl = glow::Context::from_webgl2_context(gl);
        (window, gl)
    };

    #[cfg(not(target_arch = "wasm32"))]
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
            NonZeroU32::new(800).unwrap(),
            NonZeroU32::new(600).unwrap(),
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

    let mut circle = GlCircle::new(gl);

    event_loop.run(move |event, elwt| {
        if let Event::WindowEvent { event, .. } = event {
            match event {
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
                WindowEvent::Resized(size) => {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        surface.resize(
                            &context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                    }
                    circle.render(size.width, size.height);
                    #[cfg(not(target_arch = "wasm32"))]
                    surface.swap_buffers(&context).unwrap();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let size = window.inner_size();
                    circle.update_mouse(
                        position.x as f32,
                        position.y as f32,
                        size.width as f32,
                        size.height as f32,
                    );
                    circle.render(size.width, size.height);
                    #[cfg(not(target_arch = "wasm32"))]
                    surface.swap_buffers(&context).unwrap();
                }
                WindowEvent::RedrawRequested => {
                    let size = window.inner_size();
                    circle.render(size.width, size.height);
                    #[cfg(not(target_arch = "wasm32"))]
                    surface.swap_buffers(&context).unwrap();
                }
                _ => (),
            }
        }
    })?;

    Ok(())
}
