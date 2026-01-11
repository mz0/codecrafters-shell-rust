#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let cmd_echo = "echo";
    let cmd_exit = "exit";
    let cmd_pwd = "pwd";
    let cmd_type = "type";
    let builtins = vec![cmd_echo, cmd_exit, cmd_pwd, cmd_type];
    loop {
        // Prompt
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut raw_input = String::new();

        // Capture the user's input
        io::stdin().read_line(&mut raw_input).unwrap();
        let cmd: &str = raw_input.trim();

        if cmd == cmd_exit { break }

        let (first_word, remainder) = cmd.split_once(char::is_whitespace)
            .unwrap_or((cmd, ""));

        if first_word == cmd_echo {
            echo(remainder)
        } else if first_word == cmd_pwd {
            pwd()
        } else if first_word == cmd_type {
            type_of(remainder, &builtins)
        } else if let Some(exec_path) = find_executable_in_path(first_word) {
            let argv = tokenize(remainder);
            _ = run_external_unix(exec_path, first_word, &argv);
        } else {
            println!("{}: command not found", cmd)
        }
    }
}

// extra simple tokenizer for now
fn tokenize(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn run_external_unix(path: PathBuf, name: &str, args: &[String]) -> io::Result<i32> {
    use std::os::unix::process::CommandExt;
    let status = std::process::Command::new(path)
        .arg0(name)
        .args(args)
        .status()?;
    let exit_code = status.code().unwrap_or_else(|| { 128 }); // Terminated
    Ok(exit_code)
}

fn type_of(s: &str, builtins: &[&str]) {
    if builtins.contains(&s) {
        println!("{} is a shell builtin", s);
        return
    }

    match find_executable_in_path(s) {
        Some(path) => println!("{} is {}", s, path.display()),
        None => println!("{}: not found", s),
    }
}

fn echo(s: &str) {
    println!("{}", s);
}

fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let paths = env::split_paths(&path_var);

    for dir in paths {
        // Empty PATH entries mean current directory
        let dir = if dir.as_os_str().is_empty() {
            Path::new(".").to_path_buf()
        } else {
            dir
        };

        let candidate = dir.join(name);

        if is_executable(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn is_executable(path: &Path) -> bool {
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };

    use std::os::unix::fs::PermissionsExt;
    metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0)
}

fn pwd() {
    let cwd = env::current_dir().unwrap();
    println!("{}", cwd.display());
}
