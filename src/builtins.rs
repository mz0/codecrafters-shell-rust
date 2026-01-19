use crate::executables::find_executable_in_path;
use std::env;
use std::path::Path;

pub const CMD_CD: &str = "cd";
pub const CMD_ECHO: &str = "echo";
pub const CMD_EXIT: &str = "exit";
pub const CMD_PWD: &str = "pwd";
pub const CMD_TYPE: &str = "type";

pub fn all() -> Vec<&'static str> {
    vec![CMD_CD, CMD_ECHO, CMD_EXIT, CMD_PWD, CMD_TYPE]
}

pub fn type_of(args: &[String], builtins: &[&str]) {
    if args.is_empty() {
        return;
    }
    let s = &args[0];
    if builtins.contains(&s.as_str()) {
        println!("{s} is a shell builtin");
        return;
    }

    match find_executable_in_path(s) {
        Some(path) => println!("{s} is {}", path.display()),
        None => println!("{s}: not found"),
    }
}

pub fn echo(args: &[String]) {
    println!("{}", args.join(" "));
}

pub fn pwd() {
    match env::current_dir() {
        Ok(cwd) => println!("{}", cwd.display()),
        Err(e) => eprintln!("pwd: {}", e),
    }
}

pub fn cd(args: &[String]) {
    let cd_path = if args.is_empty() { "~" } else { &args[0] };

    let path_str = if cd_path == "~" {
        // Try to get home directory, fallback to current dir "."
        env::var("HOME").unwrap_or_else(|_| ".".to_string())
    } else {
        cd_path.to_string()
    };

    let path = Path::new(&path_str);
    if let Err(e) = env::set_current_dir(path) {
        println!("cd: {}: {}", path.display(), e);
    }
}
