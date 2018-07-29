extern crate gl;

extern crate glfw;

use glfw::{Context,WindowHint};
use std::ffi::CString;
use gl::types::*;


const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"#;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3,3));
    glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw.create_window(512,512,"fluid",glfw::WindowMode::Windowed).expect("Failed to create GLFW window");
    let opengl_profile = window.get_opengl_profile();

    println!("OpenGL: {0}",opengl_profile);
    println!("Linking opengl to GLFW context.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.show();

    let (program, vao) = unsafe {
            let v_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let f_shader = gl::CreateShader(gl::FRAGMENT_SHADER);

            let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
            gl::ShaderSource(v_shader, 1, &c_str_vert.as_ptr(), std::ptr::null());
            gl::CompileShader(v_shader);

            let c_str_vert = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
            gl::ShaderSource(f_shader, 1, &c_str_vert.as_ptr(), std::ptr::null());
            gl::CompileShader(f_shader);

            //  Create the program
            let program = gl::CreateProgram();
            gl::AttachShader(program,v_shader);
            gl::AttachShader(program,f_shader);
            gl::LinkProgram(program);

            gl::DeleteShader(v_shader);
            gl::DeleteShader(f_shader);

            // Create the vertex array object
            let vertices: [f32; 9] = [
                -0.5, -0.5, 0.0,
                 0.5, -0.5, 0.0,
                 0.0, 0.5, 0.0
            ];

            let (mut vbo, mut vao) = (0,0);
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER,vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,&vertices[0] as *const f32 as *const std::os::raw::c_void, gl::STATIC_DRAW);
            gl::VertexAttribPointer(0,3,gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<GLfloat>() as GLsizei, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            // unbind
            gl::BindBuffer(gl::ARRAY_BUFFER,0);
            gl::BindVertexArray(0);

            (program,vao)
    };

    while !window.should_close(){
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events){
            handle_events(&mut window,event);
        }

        unsafe{
            gl::ClearColor(0.3,0.3,0.5,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES,0,3);
        }
        window.swap_buffers();
    }
}

fn handle_events(window: &mut glfw::Window, event : glfw::WindowEvent)
{
    match event {
        glfw::WindowEvent::Key(glfw::Key::Escape,_,glfw::Action::Press,_) => window.set_should_close(true),
        glfw::WindowEvent::FramebufferSize(width,height)=>{
            unsafe{ gl::Viewport(0,0,width,height) } 
        },
        _ => {}
    }
}