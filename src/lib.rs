extern crate gl;
extern crate glfw;
extern crate rand;

mod glw;

use std::error::Error;

use gl::types::*;
use glfw::{Context, WindowHint};

use glw::Shader;
use glw::Program;
use glw::Uniform;
use glw::RenderTarget;
use glw::Color;
use glw::Vec2;

use std::sync::mpsc::Receiver;

use rand::prelude::*;

pub struct Application
{
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,

    // App specific
    field_size : Vec2<i32>,

    // Frame buffers
    fb_prev_state : RenderTarget,
    fb_curr_state : RenderTarget,

    render_quad_prog : glw::Program,
    composite_quad_prog : glw::Program,

    // Quad mesh
    quad_vao : GLuint,
    quad_ibo : GLuint,

    // Program state
    is_paused: bool,
    gl_ctx : glw::GLContext
}

impl Application{
    
    pub fn new() -> Result<Application, Box<dyn std::error::Error>>
    {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
        glfw.window_hint(WindowHint::ContextVersion(4, 6));
        glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::Decorated(false));


        let (mut window, events) = glfw.with_primary_monitor(|instance, mon|{
                let vid_mode = mon.unwrap().get_video_mode().unwrap();
               instance.create_window(vid_mode.width,vid_mode.height, "Conway's game of life", glfw::WindowMode::Windowed).unwrap() 
        });

        gl::load_with(|s| window.get_proc_address(s) as *const _);

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.show();

        unsafe{
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(gl_debug_message, std::ptr::null());
        }

        Ok( Application{
                glfw,
                window,
                events,
                field_size: Vec2::<i32>{x: 1280,y: 720},
                gl_ctx: glw::GLContext{},
                is_paused: false,
                composite_quad_prog: Program::default(),
                render_quad_prog: Program::default(),
                fb_curr_state: RenderTarget::default(),
                fb_prev_state: RenderTarget::default(),
                quad_vao: 0,
                quad_ibo: 0,
            })
    }


    pub fn run(&mut self) -> Result< (), &'static str>
    {
        let mut time = self.get_time();
        while !self.window.should_close() {
            let prev_time = time;
            time = self.get_time();

            let _dt = time - prev_time;
            self.glfw.poll_events();

            for (_, event) in glfw::flush_messages(&self.events) {
                match event {
                    glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                        self.window.set_should_close(true)
                    },
                    glfw::WindowEvent::Key(glfw::Key::P,_,glfw::Action::Press,_) =>{
                            self.is_paused = !self.is_paused;
                    },
                    glfw::WindowEvent::Key(glfw::Key::R,_,glfw::Action::Press,_) =>{
                        let image_data = Application::generate_field(&self.field_size);
                        self.fb_curr_state.map_data(&image_data);
                        self.fb_prev_state.map_data(&image_data);
                    },
                    _ => {}
                }
            }

            if self.is_paused {
                continue;
            } 


            let ctx = &self.gl_ctx;
            let (width, height) = self.window.get_size();

            ctx.clear(Some(Color::new(0,0,0,0)));

            ctx.set_viewport(0,0,self.field_size.x,self.field_size.y);
            ctx.bind_rt(&self.fb_curr_state);
            {
                ctx.bind_shader(&self.render_quad_prog);
                self.render_quad_prog.set_uniform("u_texture",Uniform::Sampler2D(self.fb_prev_state.get_texture()));

                self.draw_quad();
            }

            // Copy new to our "old" render target
            self.gl_ctx.bind_rt(&self.fb_prev_state);
            {
                ctx.bind_shader(&self.composite_quad_prog);
                self.composite_quad_prog.set_uniform("u_texture",Uniform::Sampler2D(self.fb_curr_state.get_texture()));

                self.draw_quad();
            }


            // Copy to screen FB
            ctx.set_viewport(0,0,width,height);
            ctx.bind_rt(&RenderTarget::default());
            ctx.clear(None);
            {
                ctx.bind_shader(&self.composite_quad_prog);
                self.composite_quad_prog.set_uniform("u_texture",Uniform::Sampler2D(self.fb_curr_state.get_texture()));

                self.draw_quad();
            }

            self.window.swap_buffers();
        }

        Ok( () )
    }

    pub fn load_resources(&mut self) -> Result< () , Box<dyn Error + 'static > >
    {
        self.composite_quad_prog = {
            let mut v_shader = Shader::new(gl::VERTEX_SHADER);
            let mut f_shader = Shader::new(gl::FRAGMENT_SHADER);
            v_shader.load_from_file("Shaders/passthrough.vert").unwrap();
            f_shader.load_from_file("Shaders/composition.frag").unwrap();

            let program = Program::new();
            program.attach_shader(&v_shader);
            program.attach_shader(&f_shader);
            program.link();

            program
        };

        self.render_quad_prog = {
            let mut v_shader = Shader::new(gl::VERTEX_SHADER);
            let mut f_shader = Shader::new(gl::FRAGMENT_SHADER);
            v_shader.load_from_file("Shaders/shader.vert").unwrap();
            f_shader.load_from_file("Shaders/shader.frag").unwrap();

            //  Create the program
            let program = Program::new();
            program.attach_shader(&v_shader);
            program.attach_shader(&f_shader);
            program.link();
            program
        };

         let (vao, ibo) = unsafe {

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

            let (mut ibo, mut vbo) = (0,0);

            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut ibo);
            gl::GenBuffers(1, &mut vbo);


            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                &vertices[0] as *const f32 as *const std::os::raw::c_void,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<GLint>()) as GLsizeiptr,
                &indices[0] as  *const GLint as *const std::os::raw::c_void,
                gl::STATIC_DRAW,
            );


            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

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

            (vao,ibo)
        };
        self.quad_ibo = ibo;
        self.quad_vao = vao;

        // Generate 2 textures to keep the previous state and our render target

        let (width, height) = self.window.get_size();

        self.field_size = Vec2::<i32>{ x: width , y: height };

        self.fb_curr_state = glw::RenderTarget::new(self.field_size.clone())?;
        self.fb_prev_state = glw::RenderTarget::new(self.field_size.clone())?;

        let image_data = Application::generate_field(&self.field_size);
        self.fb_prev_state.map_data(&image_data);
        self.fb_curr_state.map_data(&image_data);

        Ok( () )
    }

    fn generate_field(field_size: &Vec2<i32>) -> Vec<Color> {
        let mut rng = rand::thread_rng();
        let mut image =  Vec::new();
        for _ in 0..field_size.x*field_size.y
        {
            if rng.gen::<f32>() > 0.1 {
                image.push(Color::new(0,0,0,255));
            }
            else
            {
                image.push(Color::new(255,255,255,255));
            }
        }

        image
    }

    fn draw_quad(&self)
    {
        unsafe{
            gl::BindVertexArray(self.quad_vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,self.quad_ibo);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }
    }
    fn get_time(&self) -> f64
    {
        self.glfw.get_time()
    }
}

#[allow(unused_variables)]
extern "system" fn gl_debug_message(source : GLenum, msg_type : GLenum, id : GLuint, severity : GLenum, length : GLsizei, message : *const GLchar, param : *mut std::os::raw::c_void)
{
    unsafe {
        let msg = std::ffi::CStr::from_ptr(message);
        println!("GL: {}", msg.to_str().unwrap());
    }
}


pub fn run() -> Result< (), Box<dyn std::error::Error + 'static> > {
    let mut app = Application::new().unwrap();

    // Load the frame buffers and generate a texture
    app.load_resources()?;

    // Start the game loop
    app.run()?;

    Ok( () )
}


#[cfg(test)]
fn main(){

    #[test]
    fn gl_test(){
        
    }
    
}