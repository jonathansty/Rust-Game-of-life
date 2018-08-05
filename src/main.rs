extern crate game_of_life;

fn main() {
    if let Err(failure) = game_of_life::run() {
        eprintln!("Application failed: {}", failure);
    }
}
