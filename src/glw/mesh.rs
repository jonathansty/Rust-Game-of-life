extern crate gl;

use gl::types::*;
use std;

pub struct Mesh {
    vertex_count: i32,
    index_count: i32,
    ibo: GLuint,
    vao: GLuint,
}

impl Mesh {
    pub fn draw(&self) {
        unsafe{
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,self.ibo);
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, std::ptr::null());
        } 
    }
}

#[derive(Default)]
pub struct MeshBuilder {
    indices: Option<Vec<i32>>,
    vertices: Option<Vec<f32>>,
}

impl MeshBuilder {
    pub fn new() -> MeshBuilder {
        MeshBuilder::default()
    }

    pub fn with_vertex_data(&mut self, vertex_data: &[f32]) -> &mut Self {
        //#TODO: Abstract away OpenGL calls
        self.vertices = Some(vertex_data.to_vec());

        self
    }

    pub fn with_index_data(&mut self, index_data: &[i32]) -> &mut Self {
        self.indices = Some(index_data.to_vec());
        //#TODO: Implement in a type safe way
        self
    }

    // #TODO: Build the mesh using a Vertex Description struct
    pub fn build(&self) -> Mesh {
        // Create indices
        let (mut ibo, mut vao) = (0, 0);
        let (mut vert_count, mut ind_count) = (0,0);
        if let Some(ref data) = self.indices {
            vert_count = data.len();
            unsafe {
                gl::GenBuffers(1, &mut ibo);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (data.len() * std::mem::size_of::<GLint>()) as GLsizeiptr,
                    &data[0] as *const GLint as *const std::os::raw::c_void,
                    gl::STATIC_DRAW,
                );
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }
        }

        // Create VAO
        if let Some(ref data) = self.vertices {
            ind_count = data.len();

            vao = 0;
            let mut vbo = 0;
            unsafe {
                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);

                gl::GenBuffers(1, &mut vbo);


                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (data.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                    &data[0] as *const f32 as *const std::os::raw::c_void,
                    gl::STATIC_DRAW,
                );

                gl::BindVertexArray(vao);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

                let stride = 8 * std::mem::size_of::<GLfloat>() as GLsizei;
                gl::EnableVertexAttribArray(0);
                gl::EnableVertexAttribArray(1);
                gl::EnableVertexAttribArray(2);

                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());

                gl::VertexAttribPointer(
                    1,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    (3 * std::mem::size_of::<GLfloat>()) as *const std::os::raw::c_void,
                );

                gl::VertexAttribPointer(
                    2,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    (5 * std::mem::size_of::<GLfloat>()) as *const std::os::raw::c_void,
                );

                // unbind
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::BindVertexArray(0);
            }
        }

        Mesh{
            vertex_count: vert_count as i32,
            index_count: ind_count as i32,
            ibo,
            vao,
        }
    }
}
