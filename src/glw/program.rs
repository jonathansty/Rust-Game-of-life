extern crate gl;

use super::shader::{Shader,Uniform};

use gl::types::*;

use std::ffi::CString;

pub struct Program {
    id: GLuint,
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

impl Default for Program {
    fn default() -> Program
    {
        Program{
            id: 0
        }
    }
}

impl Program {
    pub fn get_id(&self) -> u32 {
        return self.id;
    }

    pub fn new() -> Program {
        unsafe {
            Program {
                id: gl::CreateProgram(),
            }
        }
    }

    pub fn attach_shader(&self, shader: &Shader) {
        unsafe {
            gl::AttachShader(self.id, shader.id);
        }
    }

    pub fn link(&self) {
        unsafe {
            gl::LinkProgram(self.id);
        }
    }

    pub fn set_uniform(&self, uniform_name: &str, uni: Uniform) {
        unsafe {
            let c_string : CString = CString::new(uniform_name).unwrap();
            let loc = gl::GetUniformLocation(self.id, c_string.as_ptr() as *const GLchar);
            match loc {
                -1 => {}
                _ => match uni {
                    Uniform::Float(v) => gl::Uniform1f(loc, v),
                    Uniform::Int(v) => gl::Uniform1i(loc, v),
                    Uniform::Sampler2D(v) => {
                        self.set_sampler(v);

                        gl::Uniform1i(loc, v as i32);
                    }
                }
            }
        }
    }

    fn set_sampler(&self, sampler : GLuint)
    {
        unsafe {
            // Bind our input texture
            gl::ActiveTexture(gl::TEXTURE0 + sampler);
            gl::BindTexture(gl::TEXTURE_2D, sampler);
        }
    }

}

#[derive(Default)]
pub struct PipelineBuilder{
    vshader: Option<GLuint>,
    fshader: Option<GLuint>,
}

impl PipelineBuilder {
    pub fn new() -> PipelineBuilder{
        PipelineBuilder::default()
    }

    pub fn with_vertex_shader(&mut self, _shader: Shader) -> &mut Self
    {
        self
    }

    pub fn with_fragment_shader(&mut self, _shader: Shader) -> &mut Self
    {
        self
    }

    pub fn build(&self) -> Program
    {
        Program{
            id: 0
        }
    }

}