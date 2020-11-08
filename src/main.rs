use std::io::stdin;
use hive::engine::Engine;

fn main() {
    let mut engine = Engine::new();
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        println!("{}", engine.handle_command(input.trim()).to_string());
    }
}
