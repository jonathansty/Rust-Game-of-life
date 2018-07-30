extern crate gl;

use gl::types::*;
use std::fs::File;
use std::io::Read;
use std::ffi::CString;

pub struct Shader 
{
    pub id : u32
}

impl Drop for Shader
{
    fn drop(&mut self)
    {
        unsafe{
            println!("Dropping shader {}", self.id);
            gl::DeleteShader(self.id);
        }
    }
}

impl Shader
{
    pub fn load_from_memory(data: CString, shader_type: GLenum) -> Result<Shader, String>
    {
        unsafe{
            let shader_id = gl::CreateShader(shader_type);  


            gl::ShaderSource(shader_id, 1, &data.as_ptr(), ::std::ptr::null());
            gl::CompileShader(shader_id);

            let mut success : GLint = 0;
            gl::GetShaderiv(shader_id, gl::COMPILE_STATUS,&mut success);
            
            match success {
                0 => Err(String::from("Failed to compile shader.")),
                _ => Ok( Shader{id: shader_id })
            }
        }
    }

    pub fn load_from_file(path: String, shader_type: GLenum) -> Result<Shader, String>
    {
            let mut file = File::open(&path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).expect("Failed to read from file");
            let c_content : CString = CString::new(content.as_bytes()).unwrap();

            Shader::load_from_memory(c_content, shader_type)
    }
}
