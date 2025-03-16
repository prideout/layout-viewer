#![allow(dead_code)]

use glow::*;

pub struct GlRenderer {
    gl: glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

impl GlRenderer {
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
            out vec4 FragColor;
            void main() {
                FragColor = vec4(1.0, 1.0, 1.0, 1.0);
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

            let vertices: [f32; 8] = [
                -0.8, -0.8,
                 0.8, -0.8,
                 0.8,  0.8,
                -0.8,  0.8,
            ];

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

            gl.draw_arrays(glow::LINE_LOOP, 0, 4);
        }
    }
}

impl Drop for GlRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
} 