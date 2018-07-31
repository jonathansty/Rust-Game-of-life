extern crate gl;

use gl::types::*;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;

pub struct Shader {
    pub id: u32,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            println!("Dropping shader {}", self.id);

            gl::DeleteShader(self.id);
        }
    }
}

impl Shader {
    // Creates a new empty shader object
    pub fn new(shader_type: GLenum) -> Shader {
        unsafe {
            Shader {
                id: gl::CreateShader(shader_type),
            }
        }
    }

    // Loads a shader from memory
    pub fn load_from_memory(&mut self, data: &str) -> Result<(), String> {
        unsafe {
            let shader_id = self.id;

            let c_string = CString::new(data).unwrap();
            gl::ShaderSource(shader_id, 1, &c_string.as_ptr(), ::std::ptr::null());
            gl::CompileShader(shader_id);

            let mut success: GLint = 0;
            gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);

            match success {
                0 => {
                    let mut log_size: GLint = 0;
                    gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut log_size);
                    let mut msg: Vec<u8> = Vec::new();
                    msg.resize(log_size as usize, 0);

                    let mut new_length = 0;
                    gl::GetShaderInfoLog(
                        shader_id,
                        log_size,
                        &mut new_length,
                        msg.as_ptr() as *mut GLchar,
                    );

                    let msg = format!(
                        "Failed to compile shader : {}",
                        String::from_utf8(msg).unwrap()
                    );
                    Err(msg)
                }
                _ => Ok(()), // Return empty OK
            }
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let mut file = File::open(&path).unwrap();
        let mut content = String::new();

        file.read_to_string(&mut content)
            .expect("Failed to read from file");

        let c_content: CString = CString::new(content.as_bytes()).unwrap();

        match c_content.to_str() {
            Ok(v) => { 
                self.load_from_memory(v)
            },
            Err(err) => {
                Err(format!("Failed to load shader for path \"{}\": {}", path, err)) 
            }
        }
    }
}

pub struct Program {
    id: GLuint,
}

#[allow(dead_code)]
pub enum Uniform {
    Float(f32),
    Int(i32),
    Sampler2D(GLuint),
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

impl Program {
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
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
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
