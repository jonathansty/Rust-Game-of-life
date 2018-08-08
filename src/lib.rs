extern crate gl;
extern crate glw;
extern crate glfw;
extern crate rand;


use std::error::Error;
use glfw::{Context, WindowHint};
use glw::{Shader, GraphicsPipeline, Uniform, RenderTarget, Color, Vec2};
use glw::shader::ShaderType;

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

        // Setting up the opengl context
        let mut ctx = glw::GLContext::new(&mut window);

        #[cfg(debug_assertions)]
        ctx.set_debug();

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.show();



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

        self.glfw.set_swap_interval(glfw::SwapInterval::None);
        // Change this to influence the tick rate for the simulation
        // let update_time = 1.0 / 1.0;
        // let update_time = 1.0 / 400.0;
        let update_time = 0.0;

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


            let (width, height) = self.window.get_size();


            self.gl_ctx.bind_rt(&RenderTarget::default());
            self.gl_ctx.clear(Some(Color::new(0,0,0,0)));


            if !self.is_paused && timer <= 0.0 {
                timer = update_time;

                self.gl_ctx.bind_pipeline(&self.render_quad_prog); 
                self.gl_ctx.bind_image(&self.fb_curr_state);
                self.render_quad_prog.set_uniform("u_texture",Uniform::Sampler2D(self.fb_prev_state.get_texture())); // Bind our previous state as a texture

                self.gl_ctx.dispatch_compute(self.field_size.x as u32,self.field_size.y as u32,1);

                unsafe{
                    gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
                }

                std::mem::swap(&mut self.fb_curr_state,&mut self.fb_prev_state);
            }


            // Copy to screen FB
            self.gl_ctx.set_viewport(0,0,width,height);
            self.gl_ctx.bind_rt(&RenderTarget::default());
            self.gl_ctx.clear(None);
            {
                self.gl_ctx.bind_pipeline(&self.composite_quad_prog);
                self.composite_quad_prog.set_uniform("u_texture",Uniform::Sampler2D(self.fb_prev_state.get_texture()));

                self.draw_quad();
            }

            self.window.swap_buffers();

        }

        Ok( () )
    }

    fn load_resources(&mut self) -> Result< () , Box<dyn Error + 'static > >
    {
        self.composite_quad_prog = {
            use glw::shader::ShaderType;
            let mut v_shader = Shader::new(ShaderType::Vertex);
            let mut f_shader = Shader::new(ShaderType::Fragment);
            v_shader.load_from_file("Shaders/passthrough.vert").unwrap();
            f_shader.load_from_file("Shaders/composition.frag").unwrap();

            glw::PipelineBuilder::new()
                .with_vertex_shader(v_shader)
                .with_fragment_shader(f_shader)
                .build()
        };

        self.render_quad_prog = {

            let mut v_shader = Shader::new(ShaderType::Vertex);
            let mut f_shader = Shader::new(ShaderType::Fragment);
            let mut c_shader = Shader::new(ShaderType::Compute);

            v_shader.load_from_file("Shaders/shader.vert").unwrap();
            f_shader.load_from_file("Shaders/shader.frag").unwrap();
            c_shader.load_from_file("Shaders/shader.compute").unwrap();

            //  Create the program
            glw::PipelineBuilder::new()
                .with_compute_shader(c_shader)
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
        self.field_size = Vec2::<i32>{ x: width * 2 , y: height*2 };

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
