extern crate gl;

use super::shader::{Shader,Uniform};

use gl::types::*;

use std::ffi::CString;
use std::rc::Rc;

#[derive(Default)]
pub struct GraphicsPipeline {
    // Open GL program ID
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
                id: gl::CreateProgram()
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

    fn attach(&mut self, shader : &Shader)
    {
        unsafe {
            gl::AttachShader(self.id, shader.get_id());
        }
    }

    fn link(&mut self){
        unsafe{
            gl::LinkProgram(self.id);
        }
    }

    fn set_sampler(&self, sampler : GLuint){
        unsafe {
            // Bind our input texture
            gl::ActiveTexture(gl::TEXTURE0 + sampler);
            gl::BindTexture(gl::TEXTURE_2D, sampler);
        }
    }

}

#[derive(Default)]
pub struct PipelineBuilder{
    vshader: Option<Rc<Shader>>,
    fshader: Option<Rc<Shader>>,
}

impl PipelineBuilder {
    pub fn new() -> PipelineBuilder{
        PipelineBuilder::default()
    }

    pub fn with_vertex_shader(&mut self, shader: Shader) -> &mut Self
    {
        self.vshader = Some(Rc::new(shader));

        self
    }

    pub fn with_fragment_shader(&mut self, shader: Shader) -> &mut Self
    {
        self.fshader = Some(Rc::new(shader));

        self
    }

    pub fn build(&self) -> GraphicsPipeline
    {
        let mut result = GraphicsPipeline::new();

        if let Some(ref shader) = self.vshader {
            result.attach(&shader);
        }

        if let Some(ref shader) = self.fshader {
            result.attach(&shader);
        }

        result.link();

        result
    }

}
