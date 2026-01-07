#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let cmd_echo = "echo";
    let cmd_exit = "exit";
    let cmd_type = "type";
    let builtins = vec![cmd_echo, cmd_exit, cmd_type];
    loop {
        // Prompt
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        // Capture the user's command in the "command" variable
        io::stdin().read_line(&mut command).unwrap();
        let cmd: &str = command.trim();
        if cmd == cmd_exit { break }
        if cmd == cmd_echo {
            echo("");
            continue
        }
        match cmd.split_once(char::is_whitespace) {
            Some((first_word, remainder)) => {
                if first_word == cmd_echo {
                    echo(remainder);
                    continue
                }
                if first_word == cmd_type && builtins.contains(&remainder) {
                    println!("{} is a shell builtin", remainder)
                } else {
                    println!("{}: not found", remainder)
                }
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
