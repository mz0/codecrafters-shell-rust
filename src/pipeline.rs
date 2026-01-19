use std::process::Stdio;

use shlib::executables::find_executable_in_path;
use shlib::parse::Command;
use crate::external::prepare_unix_command;

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
}
