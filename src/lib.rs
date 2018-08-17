extern crate gl;
extern crate glfw;
extern crate glw;
extern crate rand;

use glfw::{Context};
use glw::shader::ShaderType;
use glw::{Color, GraphicsPipeline, RenderTarget, Shader, Uniform, Vec2};
use std::error::Error;

use std::sync::mpsc::Receiver;

use rand::prelude::*;

mod app;

pub struct Application {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,

    // App specific
    field_size: Vec2<i32>,

    // Frame buffers
    fb_prev_state: RenderTarget,
    fb_curr_state: RenderTarget,

    render_quad_prog: glw::GraphicsPipeline,
    composite_quad_prog: glw::GraphicsPipeline,

    // Quad mesh
    quad: Option<glw::Mesh>,

    // Program state
    is_paused: bool,
    gl_ctx: glw::GLContext,
    state: Box<dyn AppState>,
    data: Box<GameData>,
}

pub enum Trans {
    None,
    Quit,
    Transition(Box<dyn AppState>),
}

pub struct GameData {
    sample_int: u8,
}

pub trait AppState {
    /// Called when we transition to the state
    fn activate(&mut self, _: &mut GameData) {}

    /// Called when we transition away from the state.
    fn deactivate(&mut self, _: &mut GameData) {}

    /// Called every frame to update the game state
    fn update(&mut self, _: &mut GameData) -> Trans {
        Trans::None
    }

    /// Called as the second stage of the frame to render out the updated scene
    fn render(&self, _: &GameData, _: &mut glw::GLContext) {}

    /// Used to handle input events for the state
    fn handle_event(&mut self, _: &mut GameData, _: glfw::WindowEvent) -> Trans {
        Trans::None
    }
}

#[derive(Default)]
pub struct ApplicationBuilder{
    window_size : Vec2<u32>,
    window_title : String,

    state : Option<Box<dyn AppState>>,

    gl_profile : Option<glfw::OpenGlProfileHint>,
    gl_version : Option<glfw::WindowHint>,
    
}

impl ApplicationBuilder {
    pub fn window_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.window_size = glw::Vec2::<u32>{ x: width, y: height};

        self
    }

    pub fn set_gl_profile(&mut self, profile: glfw::OpenGlProfileHint) -> &mut Self {
        self.gl_profile = Some(profile);

        self
    }

    pub fn set_gl_version(&mut self, major: u8, minor: u8) -> &mut Self {
        self.gl_version = Some(glfw::WindowHint::ContextVersion(major as u32,minor as u32));

        self
    }

    pub fn set_title(&mut self, title : &str) -> &mut Self{
        self.window_title = String::from(title);
        self
    }
    pub fn set_default_state(&mut self, state : Box<dyn AppState>) -> &mut Self {
        self.state = Some(state);
        self
    }

    pub fn build(&self) -> Result<Application, Box<dyn Error>> {

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
        if let Some(p) = self.gl_profile {
            glfw.window_hint(glfw::WindowHint::OpenGlProfile(p));
        }
        if let Some(p) = self.gl_version {
            glfw.window_hint(p);
        }

        let (mut window, events) = glfw.with_primary_monitor(|instance,_| {
            instance
                .create_window(
                    self.window_size.x,
                    self.window_size.y,
                    &self.window_title,
                    glfw::WindowMode::Windowed,
                )
                .unwrap()
        });

        // Setting up the opengl context
        let mut ctx = glw::GLContext::new(&mut window);

        #[cfg(debug_assertions)]
        ctx.set_debug();

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.show();
    
        Ok(Application {
            glfw,
            window,
            events,
            field_size: Vec2::<i32> { x: 1280, y: 720 },
            gl_ctx: ctx,
            is_paused: false,
            composite_quad_prog: GraphicsPipeline::default(),
            render_quad_prog: GraphicsPipeline::default(),
            fb_curr_state: RenderTarget::default(),
            fb_prev_state: RenderTarget::default(),
            quad: None,
            data: Box::new(GameData { sample_int: 15 }),
        })
    }
}

impl Application {
    pub fn new() -> ApplicationBuilder{
        ApplicationBuilder::default()
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Load necessary resources (Framebuffer, textures, fullscreen quad and shader programs)
        self.load_resources()?;

        self.glfw.set_swap_interval(glfw::SwapInterval::None);
        // Change this to influence the tick rate for the simulation
        let update_time = 1.0 / 60.0;
        // let update_time = 1.0 / 400.0;

        let mut timer = 0.0;
        let mut time = self.get_time();

        // Activate first state
        self.state.activate(&mut self.data);

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
                    }
                    glfw::WindowEvent::Key(glfw::Key::P, _, glfw::Action::Press, _) => {
                        self.is_paused = !self.is_paused;
                    }
                    glfw::WindowEvent::Key(glfw::Key::R, _, glfw::Action::Press, _) => {
                        let image_data = Application::generate_field(&self.field_size);
                        self.fb_prev_state.map_data(&image_data);
                    }
                    _ => {
                        self.state.handle_event(&mut self.data, event);
                    }
                }
            }

            if let Trans::Transition(t) = self.state.update(&mut self.data) {
                self.state.deactivate(&mut self.data);
                self.state = t;
                self.state.activate(&mut self.data);
            }

            let (width, height) = self.window.get_size();

            // Rendering
            {
                self.gl_ctx.bind_rt(&RenderTarget::default());
                self.gl_ctx.clear(Some(Color::new(0, 0, 0, 0)));

                // State rendering
                self.state.render(&self.data, &mut self.gl_ctx);

                if !self.is_paused && timer <= 0.0 {
                    timer = update_time;

                    self.gl_ctx.bind_pipeline(&self.render_quad_prog);
                    self.gl_ctx.bind_image(&self.fb_curr_state);
                    self.render_quad_prog.set_uniform(
                        "u_texture",
                        Uniform::Sampler2D(self.fb_prev_state.get_texture()),
                    ); // Bind our previous state as a texture

                    // self.draw_quad();
                    self.gl_ctx.dispatch_compute(
                        self.field_size.x as u32 / 4,
                        self.field_size.y as u32 / 4,
                        1,
                    );

                    unsafe {
                        gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
                    }

                    std::mem::swap(&mut self.fb_curr_state, &mut self.fb_prev_state);
                }

                // Copy to screen FB
                self.gl_ctx.set_viewport(0, 0, width, height);
                self.gl_ctx.bind_rt(&RenderTarget::default());
                self.gl_ctx.clear(None);
                {
                    self.gl_ctx.bind_pipeline(&self.composite_quad_prog);
                    self.composite_quad_prog.set_uniform(
                        "u_texture",
                        Uniform::Sampler2D(self.fb_prev_state.get_texture()),
                    );

                    self.draw_quad();
                }

                self.window.swap_buffers();
            }
        }
        self.state.deactivate(&mut self.data);

        Ok(())
    }

    fn load_resources(&mut self) -> Result<(), Box<dyn Error + 'static>> {
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
                // .with_fragment_shader(f_shader)
                // .with_vertex_shader(v_shader)
                .with_compute_shader(c_shader)
                .build()
        };

        // Create the vertex array object
        let vertices: [f32; 32] = [
            -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        ];

        let indices: [i32; 6] = [0, 1, 2, 0, 2, 3];

        self.quad = Some(
            glw::MeshBuilder::new()
                .with_vertex_data(&vertices)
                .with_index_data(&indices)
                .build(),
        );

        // Generate 2 textures to keep the previous state and our render target

        let (width, height) = self.window.get_size();
        self.field_size = Vec2::<i32> {
            x: width * 2,
            y: height * 2,
        };

        self.fb_curr_state = glw::RenderTarget::new(self.field_size.clone())?;
        self.fb_prev_state = glw::RenderTarget::new(self.field_size.clone())?;

        let image_data = Application::generate_field(&self.field_size);
        self.fb_prev_state.map_data(&image_data);
        self.fb_curr_state.map_data(&image_data);

        Ok(())
    }

    fn generate_field(field_size: &Vec2<i32>) -> Vec<Color> {
        let mut rng = rand::thread_rng();
        let mut image = Vec::new();
        for _ in 0..field_size.x * field_size.y {
            if rng.gen::<f32>() > 0.1 {
                image.push(Color::new(0, 0, 0, 255));
            } else {
                image.push(Color::new(255, 255, 255, 255));
            }
        }

        image
    }

    fn draw_quad(&self) {
        if let Some(ref q) = self.quad {
            q.draw();
        }
    }

    fn get_time(&self) -> f64 {
        self.glfw.get_time()
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error + 'static>> {
    // Opens a new window and initializes glfw,opengl
    let mut app = Application::new()
        .window_size(512,512)
        .set_title("Game of life")
        .set_gl_version(4,6)
        .set_gl_profile(glfw::OpenGlProfileHint::Core)
        .set_default_state(Box::new(app::Loading::default()))
        .build()?;

    app.run()?;

    Ok(())
}
