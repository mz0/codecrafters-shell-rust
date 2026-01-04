#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let cmd_echo = "echo";
    loop {
        // Prompt
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        // Capture the user's command in the "command" variable
        io::stdin().read_line(&mut command).unwrap();
        let cmd: &str = command.trim();
        if cmd == "exit" { break; }
        if cmd == cmd_echo {
            echo("");
            continue;
        }
        match cmd.split_once(char::is_whitespace) {
            Some((first_word, remainder)) => {
                if first_word == cmd_echo {
                    echo(remainder);
                };
            }
            None => {
                println!("{}: command not found", cmd)
            }
        }

    }
}

fn echo(s: &str) {
    println!("{}", s);
}
