extern crate gl;
extern crate glfw;

mod glw;

use gl::types::*;
use glfw::{Context, WindowHint};
use glw::shader;

// Runs the main application
pub fn run() {
    println!("Starting up application.");

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(WindowHint::ContextVersion(4, 6));
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

    
    let composition = {
        let mut v_shader = shader::Shader::new(gl::VERTEX_SHADER);
        let mut f_shader = shader::Shader::new(gl::FRAGMENT_SHADER);
        v_shader.load_from_file(String::from("Shaders/passthrough.vert")).unwrap();
        f_shader.load_from_file(String::from("Shaders/composition.frag")).unwrap();

        let mut program = shader::Program::new();
        program.attach_shader(&v_shader);
        program.attach_shader(&f_shader);
        program.link();

        program
    };

    let (program, vao, ibo) = unsafe {
        let mut v_shader = shader::Shader::new(gl::VERTEX_SHADER);
        let mut f_shader = shader::Shader::new(gl::FRAGMENT_SHADER);
        v_shader.load_from_file(String::from("Shaders/shader.vert")).unwrap();
        f_shader.load_from_file(String::from("Shaders/shader.frag")).unwrap();

        //  Create the program
        let mut program = shader::Program::new();
        program.attach_shader(&v_shader);
        program.attach_shader(&f_shader);
        program.link();

        // Create the vertex array object
        let vertices: [f32; 32] = [
            -1.0, -1.0, 0.0, 0.0,0.0, 1.0,0.0,0.0,
             1.0, -1.0, 0.0, 1.0,0.0, 0.0,1.0,0.0,
             1.0, 1.0, 0.0,  1.0,1.0, 0.0,0.0,1.0,
             -1.0, 1.0, 0.0, 0.0,1.0, 1.0,0.0,1.0 ];

        let indices : [i32; 6] = [
            0, 1, 2,
            0, 2, 3
        ];

        let mut buffers : Vec<GLuint> = Vec::new();
        buffers.resize(2,0);

        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(2, buffers.as_ptr() as *mut GLuint);


        gl::BindBuffer(gl::ARRAY_BUFFER, buffers[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const std::os::raw::c_void,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,buffers[1]);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<GLint>()) as GLsizeiptr,
            &indices[0] as  *const GLint as *const std::os::raw::c_void,
            gl::STATIC_DRAW,
        );


        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffers[0]);

        let stride = 8 * std::mem::size_of::<GLfloat>() as GLsizei;
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
        gl::EnableVertexAttribArray(2);

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            std::ptr::null(),
        );


        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * std::mem::size_of::<GLfloat>()) as *const std::os::raw::c_void
        );

        gl::VertexAttribPointer(
            2,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (5 * std::mem::size_of::<GLfloat>()) as *const std::os::raw::c_void
        );



        // unbind
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,0);
        gl::BindVertexArray(0);

        (program, vao,buffers[1])
    };

    // Generate 2 textures to keep the previous state and our render target
    let (framebuffer,tex) = unsafe {
        let mut framebuffer = 0;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        
        let mut tex : GLuint = 0;
        gl::GenTextures(1,&mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MAG_FILTER,gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MIN_FILTER,gl::NEAREST as i32);

        let (width,height) = window.get_size();

        gl::TexImage2D(gl::TEXTURE_2D,0,gl::RGB as i32, width, height, 0, gl::RGB, gl::UNSIGNED_BYTE, std::ptr::null());

        gl::FramebufferTexture(gl::FRAMEBUFFER,gl::COLOR_ATTACHMENT0, tex,0);

        gl::BindFramebuffer(gl::FRAMEBUFFER,0);

        (framebuffer,tex)
    };

    let mut prev_time = glfw.get_time();
    let mut time = glfw.get_time();
    while !window.should_close() {

        prev_time = time;
        time = glfw.get_time();

        let dt = time - prev_time;
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            handle_events(&mut window, event);
        }

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER,framebuffer);
            let (x,y) = window.get_size();
            gl::Viewport(0,0,x,y);

            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            {
                program.bind();
                let uniform = std::ffi::CString::new("u_time").unwrap();
                program.set_uniform(uniform, shader::Uniform::Float(glfw.get_time() as f32) );

                gl::BindVertexArray(vao);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,ibo);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER,0);
            gl::Viewport(0,0,x,y);

            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            {
                composition.bind();
                gl::ActiveTexture(gl::TEXTURE0 + tex);
                gl::BindTexture(gl::TEXTURE_2D, tex);

                let uniform = std::ffi::CString::new("u_inTexture").unwrap();
                composition.set_uniform(uniform,shader::Uniform::Sampler2D(tex));


                gl::BindVertexArray(vao);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,ibo);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            }

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
