use std::process::{Stdio, Child};
use std::thread;

use crate::executables::find_executable_in_path;
use crate::parse::Command;
use crate::builtins;
use crate::external::prepare_unix_command;

pub fn run_pipeline(commands: &[Command]) {
    if commands.is_empty() {
        return;
    }

    let mut children: Vec<Child> = Vec::new();
    let mut input_source: Stdio = Stdio::inherit();
    let mut i = 0;

    while i < commands.len() {
        let cmd = &commands[i];
        let is_last = i == commands.len() - 1;
        let is_builtin = match cmd {
            Command::SimpleCommand(c, _) => builtins::all().contains(&c.as_str()),
            _ => false,
        };

        if !is_builtin {
            // External
            let (cmd_name, args) = match cmd {
                Command::SimpleCommand(c, a) => (c, a),
                _ => unreachable!(),
            };

            let path = match find_executable_in_path(cmd_name) {
                Some(p) => p,
                None => {
                    eprintln!("{}: command not found", cmd_name);
                    return;
                }
            };

            let stdout_target = if is_last { Stdio::inherit() } else { Stdio::piped() };

            match prepare_unix_command(&path, cmd_name, args)
                .stdin(input_source)
                .stdout(stdout_target)
                .spawn()
            {
                Ok(mut child) => {
                    if !is_last {
                        input_source = Stdio::from(child.stdout.take().unwrap());
                    } else {
                        input_source = Stdio::inherit();
                    }
                    children.push(child);
                },
                Err(e) => {
                    eprintln!("Failed to start {}: {}", cmd_name, e);
                    return;
                }
            }
            i += 1;
        } else {
            // Builtin
            // Builtins ignore stdin, so we close the previous pipe if any.
            input_source = Stdio::inherit();

            let (cmd_name, args) = match cmd {
                Command::SimpleCommand(c, a) => (c, a),
                _ => unreachable!(),
            };

            if is_last {
                let mut stdout = std::io::stdout();
                let mut stderr = std::io::stderr();
                let _ = builtins::run_builtin(cmd_name, args, &mut stdout, &mut stderr);
                i += 1;
            } else {
                // Look ahead
                let next_cmd = &commands[i+1];
                let next_is_builtin = match next_cmd {
                    Command::SimpleCommand(c, _) => builtins::all().contains(&c.as_str()),
                    _ => false,
                };

                if next_is_builtin {
                    // Builtin | Builtin
                    // Run current to sink
                    let mut sink = std::io::sink();
                    let mut stderr = std::io::stderr();
                    let _ = builtins::run_builtin(cmd_name, args, &mut sink, &mut stderr);

                    i += 1;
                } else {
                    // Builtin | External
                    // Spawn External (next_cmd)
                    let (next_name, next_args) = match next_cmd {
                        Command::SimpleCommand(c, a) => (c, a),
                        _ => unreachable!(),
                    };

                    let next_path = match find_executable_in_path(next_name) {
                        Some(p) => p,
                        None => {
                            eprintln!("{}: command not found", next_name);
                            return;
                        }
                    };

                    let next_is_last = (i + 1) == commands.len() - 1;
                    let next_stdout = if next_is_last { Stdio::inherit() } else { Stdio::piped() };

                    match prepare_unix_command(&next_path, next_name, next_args)
                        .stdin(Stdio::piped()) // We will write to this
                        .stdout(next_stdout)
                        .spawn()
                    {
                        Ok(mut child) => {
                            // Run current builtin writing to child.stdin in a separate thread
                            // to avoid deadlock if the child produces output that fills the pipe
                            // before we can read it (in the next iteration).
                            if let Some(mut child_stdin) = child.stdin.take() {
                                let cmd_name_owned = cmd_name.clone();
                                let args_owned = args.clone();
                                thread::spawn(move || {
                                    let mut stderr = std::io::stderr();
                                    let _ = builtins::run_builtin(&cmd_name_owned, &args_owned, &mut child_stdin, &mut stderr);
                                });
                            }

                            // Update input_source for i+2
                            if !next_is_last {
                                input_source = Stdio::from(child.stdout.take().unwrap());
                            } else {
                                input_source = Stdio::inherit();
                            }

                            children.push(child);
                        },
                        Err(e) => {
                            eprintln!("Failed to start {}: {}", next_name, e);
                            return;
                        }
                    }

                    // We handled i and i+1
                    i += 2;
                }
            }
        }
    }

    for mut child in children {
        let _ = child.wait();
    }
}
