use std::io::stdin;
use hive::engine::Engine;

fn main() {
    let mut engine = Engine::new();
    loop {
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => println!("{}", engine.handle_command(input.trim())),
            Err(e) => eprintln!("{}", e),
        }
    }
}
