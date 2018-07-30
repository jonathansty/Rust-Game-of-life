extern crate gl;
extern crate glfw;

mod glw;

use gl::types::*;
use glfw::{Context, WindowHint};

// Runs the main application
pub fn run() {
    println!("Starting up application.");

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(WindowHint::ContextVersion(4, 2));
    glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw
        .create_window(512, 512, "fluid", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    let opengl_profile = window.get_opengl_profile();

    println!("OpenGL: {0}", opengl_profile);
    println!("Linking opengl to GLFW context.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.show();

    
    let (program, vao, ibo) = unsafe {
        let v_shader : glw::Shader = glw::Shader::load_from_file(String::from("Shaders/shader.vert"),gl::VERTEX_SHADER).unwrap();
        let f_shader : glw::Shader = glw::Shader::load_from_file(String::from("Shaders/shader.frag"),gl::FRAGMENT_SHADER).unwrap();

        //  Create the program
        let program = gl::CreateProgram();
        gl::AttachShader(program, v_shader.id);
        gl::AttachShader(program, f_shader.id);
        gl::LinkProgram(program);

        // Create the vertex array object
        let vertices: [f32; 12] = [
            -1.0, -1.0, 0.0, 
             1.0, -1.0, 0.0, 
             1.0, 1.0, 0.0,
             -1.0, 1.0, 0.0 ];

        let indices : [i32; 6] = [
            0, 1, 2,
            0, 2, 3
        ];

        let (mut vbo, mut vao) = (0, 0);
        gl::GenVertexArrays(1, &mut vao);

        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const std::os::raw::c_void,
            gl::STATIC_DRAW,
        );

        let mut ibo = 0;
        gl::GenBuffers(1, &mut ibo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,ibo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<GLint>()) as GLsizeiptr,
            &indices[0] as  *const GLint as *const std::os::raw::c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * std::mem::size_of::<GLfloat>() as GLsizei,
            std::ptr::null(),
        );

        gl::EnableVertexAttribArray(0);
        // unbind
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,0);
        gl::BindVertexArray(0);

        (program, vao,ibo)
    };

    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            handle_events(&mut window, event);
        }

        unsafe {
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,ibo);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const i32 as *const std::os::raw::c_void );
            // gl::DrawArrays(gl::TRIANGLES,0,6);
        }

        window.swap_buffers();
    }
}

fn handle_events(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
            window.set_should_close(true)
        }
        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height)
        },
        _ => {}
    }
}
