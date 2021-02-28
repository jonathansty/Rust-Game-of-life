use glw::glfw;

use glfw::{Context, WindowHint};
use glw::shader::ShaderType;
use glw::{Color, RenderTarget, Shader, Uniform, Vec2, MemoryBarrier};
use glw::buffers::StructuredBuffer;
use glw::program::CommandList;
use std::error::Error;

use std::sync::mpsc::Receiver;

use rand::*;

const WINDOW_WIDTH: u32 = 512;
const WINDOW_HEIGHT: u32 = 512;
const FIELD_WIDTH: i32 = 256;
const FIELD_HEIGHT: i32 = 256;

/// Structure used for our structured buffers
#[allow(dead_code)]
#[derive(Default, Copy, Clone)]
struct Data{
    alive : bool,
    lifetime : f32,
    t : f32,
}

pub struct Application {
    /// GLFW specific things
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,

    // App specific
    field_size: Vec2<i32>,

    // 2 Structured buffers needed to store the data for our compute shader
    curr_sb: StructuredBuffer<Data>,
    prev_sb: StructuredBuffer<Data>,

    compute_program: glw::GraphicsPipeline,
    render_program: glw::GraphicsPipeline,

    // Quad mesh
    quad: glw::Mesh,

    // Program state
    is_paused: bool,
    gl_ctx: glw::GLContext,
}

impl Application {
    pub fn new() -> Result<Application, Box<dyn std::error::Error>> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
        glfw.window_hint(WindowHint::ContextVersion(4, 5));
        glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        let (mut window, events) = glfw.with_primary_monitor(|instance, _mon| {
            instance
                .create_window(
                    WINDOW_WIDTH,
                    WINDOW_HEIGHT,
                    "Conway's game of life",
                    glfw::WindowMode::Windowed,
                ).unwrap()
        });

        // Setting up the opengl context
        let mut ctx = glw::GLContext::new(&mut window);

        #[cfg(debug_assertions)]
        ctx.set_debug();

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.show();

        let vertices: [f32; 32] = [
            -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        ];

        let indices: [i32; 6] = [0, 1, 2, 0, 2, 3];

        let quad = glw::MeshBuilder::new()
                .with_vertex_data(&vertices)
                .with_index_data(&indices)
                .build();

        let render_program = {
            let mut v_shader = Shader::new(ShaderType::Vertex);
            let mut f_shader = Shader::new(ShaderType::Fragment);
            v_shader.load_from_file("Shaders/passthrough.vert").unwrap();
            f_shader.load_from_file("Shaders/composition.frag").unwrap();

            glw::PipelineBuilder::new()
                .with_vertex_shader(v_shader)
                .with_fragment_shader(f_shader)
                .build()
        };

        let compute_program = {
            let mut c_shader = Shader::new(ShaderType::Compute);
            c_shader.load_from_file("Shaders/shader.compute").unwrap();

            glw::PipelineBuilder::new()
                .with_compute_shader(c_shader)
                .build()
        };

        let field_size = Vec2::<i32> {
            x: FIELD_WIDTH,
            y: FIELD_HEIGHT,
        };

        let image_data = Application::generate_field(&field_size);
        let prev_sb = StructuredBuffer::from(image_data);
        let curr_sb = StructuredBuffer::new((field_size.x * field_size.y) as usize);

        Ok(Application {
            glfw,
            window,
            events,
            field_size: field_size,
            gl_ctx: ctx,
            is_paused:false,
            compute_program,
            render_program,
            curr_sb,
            prev_sb,
            quad
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {

        // Setup the swap interval
        self.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

        let update_time = 1.0 / 20.0;

        let mut timer = 0.0;
        let mut time = self.get_time();

        while !self.window.should_close() {
            let (width,height) = self.window.get_size();
            let middle = width / 2;

            let width = std::cmp::min(width,height);

            let mut cmd_list = self.gl_ctx.create_command_list();
            cmd_list.set_viewport(middle - width / 2,0,width,width);

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
                        self.prev_sb.map_data(&image_data);
                    }
                    _ => {}
                }
            }

            cmd_list.bind_rt(&RenderTarget::default());
            cmd_list.clear(Some(Color::new(0, 0, 0, 0)));

            // Update the simulation
            if !self.is_paused && timer <= 0.0 {
                timer = update_time;

                cmd_list.bind_pipeline(&self.compute_program);

                cmd_list.set_uniform("u_field_size", Uniform::Vec2(self.field_size.x as f32,self.field_size.y as f32));
                cmd_list.set_uniform("u_dt", Uniform::Float(update_time as f32));
                cmd_list.set_uniform("u_time", Uniform::Float(self.get_time() as f32));

                cmd_list.bind_buffer(&self.curr_sb,0);
                cmd_list.bind_buffer(&self.prev_sb,1);

                cmd_list.dispatch(
                    self.field_size.x as u32 / 8,
                    self.field_size.y as u32 / 8,
                    1,
                );

                cmd_list.memory_barrier(MemoryBarrier::ShaderStorage);

                std::mem::swap(&mut self.curr_sb, &mut self.prev_sb);
            }

            // Display the compute simulation on screen
            {
                cmd_list.bind_pipeline(&self.render_program);
                cmd_list.set_uniform("u_field_size", Uniform::Vec2(self.field_size.x as f32, self.field_size.y as f32) );
                cmd_list.bind_buffer(&self.prev_sb, 0);

                // #TODO: Do mesh drawing using cmd list
                self.quad.draw();

                self.gl_ctx.execute_command_list(&cmd_list);
            }

            self.window.swap_buffers();
        }

        Ok(())
    }

    /// Randomly generates a starting field for the game of life
    fn generate_field(field_size: &Vec2<i32>) -> Vec<Data> {
        let mut rng = rand::thread_rng();
        let mut image = Vec::new();
        for _ in 0..field_size.x * field_size.y {
            if rng.gen::<f32>() > 0.1 {
                image.push(Data{
                    alive: false,
                    lifetime: 0.0,
                    t: 0.0,
                })
            } else {
                image.push(Data{
                    alive: true,
                    lifetime: 1.0,
                    t: 0.0
                })
            }
        }

        image
    }

    fn get_time(&self) -> f64 {
        self.glfw.get_time()
    }
}

/// Executes the application 
pub fn run() -> Result<(), Box<dyn std::error::Error + 'static>> {
    // Opens a new window and initializes glfw,opengl
    let mut app = Application::new()?;
    app.run()?;

    Ok(())
}
