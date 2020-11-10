use std::io::stdin;
use hive::engine::Engine;

fn main() {
    let mut engine = Engine::new();
    println!("{}", engine.handle_command("info").to_string());
    loop {
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => println!("{}", engine.handle_command(input.trim()).to_string()),
            Err(e) => eprintln!("{}", e),
        }
    }
}
