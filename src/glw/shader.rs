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
        pub fn new(shader_type : GLenum) -> Shader {
            unsafe{
                Shader{ id: gl::CreateShader(shader_type) }
            }
        }

        // Loads a shader from memory
        pub fn load_from_memory(&mut self, data: CString)  -> Result<(), String> {
            unsafe {
                let shader_id = self.id;

                gl::ShaderSource(shader_id, 1, &data.as_ptr(), ::std::ptr::null());
                gl::CompileShader(shader_id);

                let mut success: GLint = 0;
                gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);

                match success {
                    0 => {
                        let mut log_size : GLint = 0;
                        gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut log_size);
                        let mut msg : Vec<u8> = Vec::new();
                        msg.resize(log_size as usize, 0);

                        let mut new_length = 0;
                        gl::GetShaderInfoLog(shader_id, log_size,&mut new_length, msg.as_ptr() as *mut GLchar);

                        let msg = format!("Failed to compile shader : {}", String::from_utf8(msg).unwrap());
                        Err( msg )
                    },
                    _ => Ok( () ), // Return empty OK
                }
            }
        }

        pub fn load_from_file(&mut self, path: String) -> Result<(), String> {
            let mut file = File::open(&path).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("Failed to read from file");
            let c_content: CString = CString::new(content.as_bytes()).unwrap();

            self.load_from_memory(c_content)
        }
    }

    pub struct Program
    {
        id : GLuint
    }

    pub enum Uniform
    {
        Float(f32),
        Int(i32),
        Sampler2D(GLuint)
    }

    impl Drop for Program
    {
        fn drop(&mut self)
        {
            unsafe{
                gl::DeleteProgram(self.id);
            }

        }
    }

    impl Program
    {
        pub fn new() -> Program{
            unsafe{
                Program{
                    id: gl::CreateProgram()
                }
            }
        }

        pub fn attach_shader(&self, shader : &Shader)
        {
            unsafe {
                    gl::AttachShader(self.id, shader.id);
            }
        }

        pub fn link(&self){
            unsafe{
                gl::LinkProgram(self.id);
            }
        }
        pub fn bind(&self){
            unsafe{
                gl::UseProgram(self.id);
            }
        }

        pub fn set_uniform(&self, uniform_name : CString, uni : Uniform){
            unsafe {
                let loc : GLint = gl::GetUniformLocation(self.id, uniform_name.as_ptr() as *const GLchar);
                match loc {
                    -1 => { 
                            let name = uniform_name.into_string().unwrap();
                            println!("Failed to bind uniform \"{}\"!", name);
                        },
                    _ => {
                        match uni {
                            Uniform::Float(v) => gl::Uniform1f(loc, v),
                            Uniform::Int(v) => gl::Uniform1i(loc, v),
                            Uniform::Sampler2D(v) => gl::Uniform1i(loc,v as i32),
                        }
                    }
                } 

            }
        }
    }