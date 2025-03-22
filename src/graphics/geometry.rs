use crate::rsutils::IdMapKey;
use glow::HasContext;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct GeometryId(pub usize);

impl IdMapKey for GeometryId {
    fn from_usize(id: usize) -> Self {
        GeometryId(id)
    }
}

pub struct Geometry {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    vao: Option<glow::VertexArray>,
    positions_vbo: Option<glow::Buffer>,
    indices_vbo: Option<glow::Buffer>,
    positions_uploaded: bool,
    indices_uploaded: bool,
}

impl Geometry {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            indices: Vec::new(),
            vao: None,
            positions_vbo: None,
            indices_vbo: None,
            positions_uploaded: false,
            indices_uploaded: false,
        }
    }

    fn create(&mut self, gl: &glow::Context) {
        unsafe {
            // Create resources
            self.vao = Some(gl.create_vertex_array().expect("Failed to create VAO"));
            self.positions_vbo = Some(gl.create_buffer().expect("Failed to create positions VBO"));
            self.indices_vbo = Some(gl.create_buffer().expect("Failed to create indices VBO"));

            // Set up VAO
            gl.bind_vertex_array(self.vao);
            gl.bind_buffer(glow::ARRAY_BUFFER, self.positions_vbo);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, self.indices_vbo);

            // Cleanup
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
    }

    pub(crate) fn destroy(&mut self, gl: &glow::Context) {
        unsafe {
            if let Some(vao) = self.vao.take() {
                gl.delete_vertex_array(vao)
            }
            [&mut self.positions_vbo, &mut self.indices_vbo]
                .iter_mut()
                .filter_map(|vbo| vbo.take())
                .for_each(|vbo| gl.delete_buffer(vbo));
        }
    }

    pub(crate) fn upload_positions(&mut self, gl: &glow::Context) {
        if self.positions.is_empty() {
            log::warn!("Attempting to upload empty positions buffer");
            self.positions_uploaded = true;
            return;
        }

        if self.vao.is_none() {
            self.create(gl);
        }

        unsafe {
            gl.bind_vertex_array(self.vao);
            gl.bind_buffer(glow::ARRAY_BUFFER, self.positions_vbo);
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&self.positions),
                glow::STATIC_DRAW,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);
        }
        self.positions_uploaded = true;
    }

    pub(crate) fn upload_indices(&mut self, gl: &glow::Context) {
        if self.indices.is_empty() {
            log::warn!("Attempting to upload empty indices buffer");
            self.indices_uploaded = true;
            return;
        }

        if self.vao.is_none() {
            self.create(gl);
        }

        unsafe {
            gl.bind_vertex_array(self.vao);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, self.indices_vbo);
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&self.indices),
                glow::STATIC_DRAW,
            );
        }
        self.indices_uploaded = true;
    }

    pub(crate) fn bind(&mut self, gl: &glow::Context) {
        if !self.positions_uploaded {
            self.upload_positions(gl);
        }
        if !self.indices_uploaded {
            self.upload_indices(gl);
        }
        unsafe {
            gl.bind_vertex_array(self.vao);
        }
    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        if self.vao.is_some() {
            log::warn!("Geometry dropped without calling destroy()");
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self::new()
    }
}
