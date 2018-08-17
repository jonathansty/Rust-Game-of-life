extern crate glfw; 

use super::*;

#[derive(Default)]
pub struct Loading{
    timer : f32
}
impl AppState for Loading{
    fn activate(&mut self, _ : &mut GameData){
        println!("loading");
    }

    fn deactivate(&mut self, _ : &mut GameData){
        println!("Exiting loading!");
    }

    fn handle_event(&mut self, _ : &mut GameData, _ : glfw::WindowEvent) -> Trans {
      Trans::None  
    }

    fn update(&mut self, _data : &mut GameData) -> Trans {
        self.timer = self.timer + 0.01;
        if self.timer > 20.0
        {
            println!("Starting transition");
            return Trans::Transition(Box::new(GameOfLife{}));
        }

        // Default do none
        Trans::None  
    }
}

pub struct GameOfLife{
}

impl AppState for GameOfLife{
    fn activate(&mut self, data : &mut GameData){
        println!("Starting game of life. {}", data.sample_int);
    }

    fn deactivate(&mut self, data : &mut GameData){
        println!("Stopping game of life. {}", data.sample_int);
    }

    fn handle_event(&mut self, _ : &mut GameData, _ : glfw::WindowEvent) -> Trans {
      Trans::None  
    }

    fn update(&mut self, _ : &mut GameData) -> Trans {
      Trans::None  
    }

}