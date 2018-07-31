extern crate gl;
extern crate glfw;

extern crate gpu_accelerated_fluid;


use std::io;
use std::io::prelude::*;
#[allow(dead_code)]
fn pause(){
    write!(io::stdout(),"Press any key to continue...").unwrap();
    io::stdout().flush().expect("Failed to flush.");

    let _ = io::stdin().read(&mut [0u8]).unwrap();
}
fn main() {
    gpu_accelerated_fluid::run();

    // pause();
}
