use std::process::Stdio;

use shlib::executables::find_executable_in_path;
use shlib::parse::Command;
use crate::external::prepare_unix_command;
use crate::builtins;

pub fn run_pipeline(left: &Command, right: &Command) {
    let (left_cmd, left_args) = match left {
        Command::SimpleCommand(cmd, args) => (cmd, args),
        _ => {
            eprintln!("Complex commands in pipes not supported yet");
            return;
        }
    };

    let (right_cmd, right_args) = match right {
        Command::SimpleCommand(cmd, args) => (cmd, args),
        _ => {
            eprintln!("Complex commands in pipes not supported yet");
            return;
        }
    };

    let left_is_builtin = builtins::all().contains(&left_cmd.as_str());
    let right_is_builtin = builtins::all().contains(&right_cmd.as_str());

    if !left_is_builtin && !right_is_builtin {
        let left_path = match find_executable_in_path(left_cmd) {
            Some(path) => path,
            None => {
                eprintln!("{}: command not found", left_cmd);
                return;
            }
        };

        let right_path = match find_executable_in_path(right_cmd) {
            Some(path) => path,
            None => {
                eprintln!("{}: command not found", right_cmd);
                return;
            }
        };

        let mut left_child = match prepare_unix_command(&left_path, left_cmd, left_args)
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Failed to start {}: {}", left_cmd, e);
                return;
            }
        };

        let left_stdout = left_child.stdout.take().expect("Failed to open stdout");

        let mut right_child = match prepare_unix_command(&right_path, right_cmd, right_args)
            .stdin(Stdio::from(left_stdout))
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Failed to start {}: {}", right_cmd, e);
                let _ = left_child.kill();
                return;
            }
        };

        let _ = left_child.wait();
        let _ = right_child.wait();
    } else if right_is_builtin {
        let mut left_child = None;

        if left_is_builtin {
            let mut sink = std::io::sink();
            let mut stderr = std::io::stderr();
            let _ = builtins::run_builtin(left_cmd, left_args, &mut sink, &mut stderr);
        } else {
            if let Some(left_path) = find_executable_in_path(left_cmd) {
                match prepare_unix_command(&left_path, left_cmd, left_args)
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    Ok(mut c) => {
                        // Drop the read end of the pipe immediately to simulate closed stdin on the right
                        drop(c.stdout.take());
                        left_child = Some(c);
                    },
                    Err(e) => {
                        eprintln!("Failed to start {}: {}", left_cmd, e);
                        return;
                    }
                };
            } else {
                eprintln!("{}: command not found", left_cmd);
            }
        }

        let mut stdout = std::io::stdout();
        let mut stderr = std::io::stderr();
        let _ = builtins::run_builtin(right_cmd, right_args, &mut stdout, &mut stderr);

        if let Some(mut child) = left_child {
            let _ = child.wait();
        }
    } else {
        // Left is builtin, Right is external
        let right_path = match find_executable_in_path(right_cmd) {
            Some(path) => path,
            None => {
                eprintln!("{}: command not found", right_cmd);
                return;
            }
        };

        let mut right_child = match prepare_unix_command(&right_path, right_cmd, right_args)
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Failed to start {}: {}", right_cmd, e);
                return;
            }
        };

        if let Some(mut right_stdin) = right_child.stdin.take() {
            let mut stderr = std::io::stderr();
            let _ = builtins::run_builtin(left_cmd, left_args, &mut right_stdin, &mut stderr);
        }

        let _ = right_child.wait();
    }
}
