#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path::{Path, PathBuf};
use is_executable::IsExecutable;
use rustyline::{Config, CompletionType, EditMode};
use rustyline::config::BellStyle;

pub const CMD_CD: &str = "cd";
pub const CMD_ECHO: &str = "echo";
pub const CMD_EXIT: &str = "exit";
pub const CMD_PWD: &str = "pwd";
pub const CMD_TYPE: &str = "type";

mod rline;
use rline::ShellHelper;

fn main() {
    let builtins = vec![CMD_CD, CMD_ECHO, CMD_EXIT, CMD_PWD, CMD_TYPE];
    let system_commands = get_all_executables(); // Scan PATH once
    let h = ShellHelper { builtins: builtins.clone(), system_commands };
    let rlc = Config::builder()
    .completion_type(CompletionType::List) // default: Emacs-style, cycles through candidates
    .bell_style(BellStyle::Audible)
    .edit_mode(EditMode::Emacs) // e.g. Ctrl-A, Home - Move cursor to the beginning of line
    .max_history_size(1000)
    .unwrap().completion_prompt_limit(200) // trigger alert when completion is too ambiguous
    .build();
    let mut rl = rustyline::Editor::<ShellHelper, _>::with_config(rlc).unwrap();
    rl.set_helper(Some(h));

    loop {
        // Prompt
        match rl.readline("$ ") {
            Ok(line) => {
                let cmd = line.trim();
                if cmd.is_empty() { continue; }

                _ = rl.add_history_entry(cmd);
                if cmd == CMD_EXIT { break }

                let (first_word, remainder) = cmd.split_once(char::is_whitespace)
                    .unwrap_or((cmd, ""));

                if first_word == CMD_ECHO {
                    echo(remainder)
                } else if first_word == CMD_CD {
                    cd(remainder)
                } else if first_word == CMD_PWD {
                    pwd()
                } else if first_word == CMD_TYPE {
                    type_of(remainder, &builtins)
                } else if let Some(exec_path) = find_executable_in_path(first_word) {
                    let argv = tokenize(remainder);
                    _ = run_external_unix(exec_path, first_word, &argv);
                } else {
                    println!("{}: command not found", cmd)
                }
            },
            Err(_) => break, // Handles Ctrl+C / Ctrl+D
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

fn cd(cd_path: &str) {
    if cd_path.is_empty() { return }
    let path: String = if cd_path == "~" {
        env::var_os("HOME").unwrap_or_else(|| ".".into()).to_string_lossy().into_owned()
    } else {
        String::from(cd_path)
    };
    if env::set_current_dir(&Path::new(&path)).is_err() {
        println!("cd: {}: No such file or directory", path);
    }
}
