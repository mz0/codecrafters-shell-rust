#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path::{Path, PathBuf};
use is_executable::IsExecutable;

pub const CMD_CD: &str = "cd";
pub const CMD_ECHO: &str = "echo";
pub const CMD_EXIT: &str = "exit";
pub const CMD_PWD: &str = "pwd";
pub const CMD_TYPE: &str = "type";

use shlib::rline::ShellHelper;
use shlib::parse::{parse, Command};

fn main() {
    let builtins = vec![CMD_CD, CMD_ECHO, CMD_EXIT, CMD_PWD, CMD_TYPE];
    let system_commands = get_all_executables(); // Scan PATH once
    let h = ShellHelper { builtins: builtins.clone(), system_commands };
    let mut rl = shlib::config::create_editor(h).unwrap();

    loop {
        // Prompt
        match rl.readline("$ ") {
            Ok(line) => {
                let cmd_line = line.trim();
                if cmd_line.is_empty() { continue; }

                _ = rl.add_history_entry(cmd_line);

                match parse(cmd_line) {
                    Command::SimpleCommand(cmd, args) => {
                        if cmd == CMD_EXIT { break }

                        if cmd == CMD_ECHO {
                            echo(&args);
                        } else if cmd == CMD_CD {
                            cd(&args)
                        } else if cmd == CMD_PWD {
                            pwd()
                        } else if cmd == CMD_TYPE {
                            type_of(&args, &builtins)
                        } else if let Some(exec_path) = find_executable_in_path(&cmd) {
                            _ = run_external_unix(exec_path, &cmd, &args);
                        } else {
                            println!("{cmd}: command not found")
                        }
                    },
                    Command::PipeCommand(_, _) => {
                        println!("Pipes not implemented yet");
                    },
                    Command::InvalidCommand(err) => {
                        println!("Error: {}", err);
                    }
                }
            },
            Err(_) => break, // Handles Ctrl+C / Ctrl+D
        }
    }
}

/// only Unix lets argv[0]=name substitution
fn run_external_unix(path: PathBuf, name: &str, args: &[String]) -> io::Result<i32> {
    use std::os::unix::process::CommandExt;
    let status = std::process::Command::new(path)
        .arg0(name)
        .args(args)
        .status()?;
    let exit_code = status.code().unwrap_or_else(|| { 128 }); // Terminated
    Ok(exit_code)
}

fn type_of(args: &[String], builtins: &[&str]) {
    if args.is_empty() {
        return;
    }
    let s = &args[0];
    if builtins.contains(&s.as_str()) {
        println!("{s} is a shell builtin");
        return
    }

    match find_executable_in_path(s) {
        Some(path) => println!("{s} is {}", path.display()),
        None => println!("{s}: not found"),
    }
}

fn echo(args: &[String]) {
    println!("{}", args.join(" "));
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

        if candidate.is_executable() {
            return Some(candidate);
        }
    }

    None
}

fn get_all_executables() -> Vec<String> {
    let mut execs = Vec::new();

    if let Some(path_var) = env::var_os("PATH") {
        for path in env::split_paths(&path_var) {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_executable() {
                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                            execs.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    execs.sort();
    execs.dedup(); // Remove duplicates (e.g., if 'ls' is in two PATH folders)
    execs
}

fn pwd() {
    let cwd = env::current_dir().unwrap();
    println!("{}", cwd.display());
}

fn cd(args: &[String]) {
    let cd_path = if args.is_empty() {
        "~"
    } else {
        &args[0]
    };

    let path: String = if cd_path == "~" {
        env::var_os("HOME").unwrap_or_else(|| ".".into()).to_string_lossy().into_owned()
    } else {
        String::from(cd_path)
    };
    if env::set_current_dir(&Path::new(&path)).is_err() {
        println!("cd: {}: No such file or directory", path);
    }
}
