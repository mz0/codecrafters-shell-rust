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

        // Capture the user's cinput
        io::stdin().read_line(&mut raw_input).unwrap();
        let cmd: &str = raw_input.trim();

        if cmd == cmd_exit { break }

        let (first_word, remainder) = cmd.split_once(char::is_whitespace)
            .unwrap_or((cmd, ""));

        if first_word == cmd_echo {
            echo(remainder)
        } else if first_word == cmd_type {
            type_of(remainder, &builtins)
        } else {
            println!("{}: command not found", cmd)
        }
    }
}

fn type_of(s: &str, builtins: &[&str]) {
    if builtins.contains(&s) {
        println!("{} is a shell builtin", s)
    } else {
        println!("{}: not found", s)
    }
}

fn echo(s: &str) {
    println!("{}", s);
}
