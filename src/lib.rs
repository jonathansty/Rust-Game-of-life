extern crate gl;
extern crate glfw;
extern crate rand;

mod glw;

use std::error::Error;
use glfw::{Context, WindowHint};
use glw::{Shader, GraphicsPipeline, Uniform, RenderTarget, Color, Vec2};

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

    render_quad_prog : glw::GraphicsPipeline,
    composite_quad_prog : glw::GraphicsPipeline,

    // Quad mesh
    quad : Option<glw::Mesh>,

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
        // glfw.window_hint(WindowHint::Decorated(false));


        let (mut window, events) = glfw.with_primary_monitor(|instance, mon|{
                let vid_mode = mon.unwrap().get_video_mode().unwrap();
               instance.create_window(vid_mode.width / 2,vid_mode.height / 2, "Conway's game of life", glfw::WindowMode::Windowed).unwrap() 
        });

        gl::load_with(|s| window.get_proc_address(s) as *const _);

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.show();

        let ctx = glw::GLContext{};

        #[cfg(debug_assertions)]
        ctx.set_debug();

        Ok( Application{
                glfw,
                window,
                events,
                field_size: Vec2::<i32>{x: 1280,y: 720},
                gl_ctx: ctx,
                is_paused: false,
                composite_quad_prog: GraphicsPipeline::default(),
                render_quad_prog: GraphicsPipeline::default(),
                fb_curr_state: RenderTarget::default(),
                fb_prev_state: RenderTarget::default(),
                quad: None
            })
    }


    pub fn run(&mut self) -> Result< (), Box<dyn Error> > {
        // Load necessary resources (Framebuffer, textures, fullscreen quad and shader programs)
        self.load_resources()?;

        // Change this to influence the tick rate for the simulation
        let update_time = 1.0 / 30.0;

        let mut timer = 0.0;
        let mut time = self.get_time();
        let _x = 217;
        while !self.window.should_close() {
            let prev_time = time;
            time = self.get_time();

            let dt = time - prev_time;
            timer -= dt;

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
                        self.fb_prev_state.map_data(&image_data);
                    },
                    _ => {}
                }
            }


            let ctx = &self.gl_ctx;
            let (width, height) = self.window.get_size();

            ctx.bind_rt(&RenderTarget::default());
            ctx.clear(Some(Color::new(0,0,0,0)));


            if !self.is_paused && timer <= 0.0 {
                timer = update_time;

                ctx.set_viewport(0,0,self.field_size.x,self.field_size.y);

                ctx.bind_rt(&self.fb_curr_state); // Bind the render target 
                ctx.bind_pipeline(&self.render_quad_prog); // Use the Game of life program
                self.render_quad_prog.set_uniform("u_texture",Uniform::Sampler2D(self.fb_prev_state.get_texture())); // Bind our previous state as a texture
                self.draw_quad();

                // Copy new to our "old" render target
                self.gl_ctx.bind_rt(&self.fb_prev_state); // Bind our previous state as a render target
                ctx.bind_pipeline(&self.composite_quad_prog); // This program just copies over the bound texture u_texture pixel per pixel
                self.composite_quad_prog.set_uniform("u_texture",Uniform::Sampler2D(self.fb_curr_state.get_texture()));
                self.draw_quad();
            }


            // Copy to screen FB
            ctx.set_viewport(0,0,width,height);
            ctx.bind_rt(&RenderTarget::default());
            ctx.clear(None);
            {
                ctx.bind_pipeline(&self.composite_quad_prog);
                self.composite_quad_prog.set_uniform("u_texture",Uniform::Sampler2D(self.fb_curr_state.get_texture()));

                self.draw_quad();
            }

            self.window.swap_buffers();
        }

        Ok( () )
    }

    fn load_resources(&mut self) -> Result< () , Box<dyn Error + 'static > >
    {
        self.composite_quad_prog = {
            let mut v_shader = Shader::new(gl::VERTEX_SHADER);
            let mut f_shader = Shader::new(gl::FRAGMENT_SHADER);
            v_shader.load_from_file("Shaders/passthrough.vert").unwrap();
            f_shader.load_from_file("Shaders/composition.frag").unwrap();

            glw::PipelineBuilder::new()
                .with_vertex_shader(v_shader)
                .with_fragment_shader(f_shader)
                .build()
        };

        self.render_quad_prog = {

            let mut v_shader = Shader::new(gl::VERTEX_SHADER);
            let mut f_shader = Shader::new(gl::FRAGMENT_SHADER);
            v_shader.load_from_file("Shaders/shader.vert").unwrap();
            f_shader.load_from_file("Shaders/shader.frag").unwrap();

            //  Create the program
            glw::PipelineBuilder::new()
                .with_vertex_shader(v_shader)
                .with_fragment_shader(f_shader)
                .build()
        };

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

        self.quad = Some(glw::MeshBuilder::new()
                    .with_vertex_data(&vertices)
                    .with_index_data(&indices)
                    .build());

        // Generate 2 textures to keep the previous state and our render target

        let (width, height) = self.window.get_size();
        self.field_size = Vec2::<i32>{ x: width / 4 , y: height / 4 };

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
        if let Some(ref q) = self.quad {
            q.draw();
        }
    }

    fn get_time(&self) -> f64
    {
        self.glfw.get_time()
    }
}




pub fn run() -> Result< (), Box<dyn std::error::Error + 'static> > {

    // Opens a new window and initializes glfw,opengl
    let mut app = Application::new()?;
    app.run()?;

    Ok( () )
}
