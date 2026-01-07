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
        let mut raw_input = String::new();
        // Capture the user's command
        io::stdin().read_line(&mut raw_input).unwrap();
        let cmd: &str = raw_input.trim();
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
