extern crate gl;

use super::shader::{Shader,Uniform};

use gl::types::*;

use std::ffi::CString;

#[derive(Default)]
pub struct GraphicsPipeline {
    id: GLuint,
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

impl GraphicsPipeline {
    pub fn get_id(&self) -> GLuint {
        return self.id;
    }

    fn new() -> GraphicsPipeline {
        unsafe {
            GraphicsPipeline {
                id: gl::CreateProgram(),
            }
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
    vshader: Option<Shader>,
    fshader: Option<Shader>,
}

impl PipelineBuilder {
    pub fn new() -> PipelineBuilder{
        PipelineBuilder::default()
    }

    pub fn with_vertex_shader(&mut self, shader: Shader) -> &mut Self
    {
        self.vshader = Some(shader);

        self
    }

    pub fn with_fragment_shader(&mut self, shader: Shader) -> &mut Self
    {
        self.fshader = Some(shader);

        self
    }

    pub fn build(&self) -> GraphicsPipeline
    {
        let result = GraphicsPipeline::new();

        if let Some(ref shader) = self.vshader {
            unsafe {
                gl::AttachShader(result.id, shader.get_id());
            }
        }

        if let Some(ref shader) = self.fshader {
            unsafe {
                gl::AttachShader(result.id, shader.get_id());
            }
        }

        unsafe {
            gl::LinkProgram(result.get_id());
        }

        result
    }

}
