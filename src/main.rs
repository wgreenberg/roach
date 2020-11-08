use std::io::stdin;
use hive::engine::Engine;

fn main() {
    let mut engine = Engine::new();
    println!("{}", engine.handle_command("info").to_string());
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        println!("{}", engine.handle_command(input.trim()).to_string());
    }
}
