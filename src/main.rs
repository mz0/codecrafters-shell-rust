#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        // Prompt
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        // Capture the user's command in the "command" variable
        io::stdin().read_line(&mut command).unwrap();
        if command.trim() == "exit" { break; }
        println!("{}: command not found", command.trim())
    }
}
