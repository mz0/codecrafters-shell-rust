#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        // Prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Capture the user's command in the "command" variable
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        println!("{}: command not found", command.trim())
    }
}
