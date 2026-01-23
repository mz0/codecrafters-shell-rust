use std::process::{Stdio, Child};
use std::thread;
use std::fs::OpenOptions;

use crate::executables::find_executable_in_path;
use crate::parse::{Command, RedirectKind};
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
        let (cmd_ref, redirects) = unwrap_command(&commands[i]);

        let is_last = i == commands.len() - 1;
        let is_builtin = match cmd_ref {
            Command::SimpleCommand(c, _) => builtins::all().contains(&c.as_str()),
            _ => false,
        };

        if !is_builtin {
            // External
            let (cmd_name, args) = match cmd_ref {
                Command::SimpleCommand(c, a) => (c, a),
                _ => { i += 1; continue; }
            };

            let path = match find_executable_in_path(cmd_name) {
                Some(p) => p,
                None => {
                    eprintln!("{}: command not found", cmd_name);
                    return;
                }
            };

            let mut stdout_target = if is_last { Stdio::inherit() } else { Stdio::piped() };
            let mut stderr_target = Stdio::inherit();

            for (path, kind) in redirects.iter().rev() {
                if let Ok(f) = open_redirect_file(path, kind) {
                    match kind {
                        RedirectKind::Stdout | RedirectKind::StdoutAppend => stdout_target = Stdio::from(f),
                        RedirectKind::Stderr | RedirectKind::StderrAppend => stderr_target = Stdio::from(f),
                        RedirectKind::Both | RedirectKind::BothAppend => {
                            if let Ok(f2) = f.try_clone() {
                                stdout_target = Stdio::from(f);
                                stderr_target = Stdio::from(f2);
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to open redirect file: {}", path);
                    return;
                }
            }

            match prepare_unix_command(&path, cmd_name, args)
                .stdin(input_source)
                .stdout(stdout_target)
                .stderr(stderr_target)
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

            let (cmd_name, args) = match cmd_ref {
                Command::SimpleCommand(c, a) => (c, a),
                _ => { i += 1; continue; }
            };

            // Resolve redirects for builtin
            let mut stdout_redirect: Option<std::fs::File> = None;
            let mut stderr_redirect: Option<std::fs::File> = None;

            for (path, kind) in redirects.iter().rev() {
                if let Ok(f) = open_redirect_file(path, kind) {
                    match kind {
                        RedirectKind::Stdout | RedirectKind::StdoutAppend => stdout_redirect = Some(f),
                        RedirectKind::Stderr | RedirectKind::StderrAppend => stderr_redirect = Some(f),
                        RedirectKind::Both | RedirectKind::BothAppend => {
                            if let Ok(f2) = f.try_clone() {
                                stdout_redirect = Some(f);
                                stderr_redirect = Some(f2);
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to open redirect file: {}", path);
                    return;
                }
            }

            if is_last {
                let mut stdout: Box<dyn std::io::Write> = if let Some(f) = stdout_redirect {
                    Box::new(f)
                } else {
                    Box::new(std::io::stdout())
                };
                let mut stderr: Box<dyn std::io::Write> = if let Some(f) = stderr_redirect {
                    Box::new(f)
                } else {
                    Box::new(std::io::stderr())
                };

                let _ = builtins::run_builtin(cmd_name, args, &mut stdout, &mut stderr);
                i += 1;
            } else {
                // Look ahead
                let next_cmd = &commands[i+1];
                let (next_cmd_ref, next_redirects) = unwrap_command(next_cmd);
                let next_is_builtin = match next_cmd_ref {
                    Command::SimpleCommand(c, _) => builtins::all().contains(&c.as_str()),
                    _ => false,
                };

                if next_is_builtin {
                    // Builtin | Builtin
                    // Run current to sink
                    let mut stdout: Box<dyn std::io::Write> = if let Some(f) = stdout_redirect {
                        Box::new(f)
                    } else {
                        Box::new(std::io::sink())
                    };
                    let mut stderr: Box<dyn std::io::Write> = if let Some(f) = stderr_redirect {
                        Box::new(f)
                    } else {
                        Box::new(std::io::stderr())
                    };

                    let _ = builtins::run_builtin(cmd_name, args, &mut stdout, &mut stderr);

                    i += 1;
                } else {
                    // Builtin | External
                    // Spawn External (next_cmd)
                    let (next_name, next_args) = match next_cmd_ref {
                        Command::SimpleCommand(c, a) => (c, a),
                        _ => { i += 2; continue; }
                    };

                    let next_path = match find_executable_in_path(next_name) {
                        Some(p) => p,
                        None => {
                            eprintln!("{}: command not found", next_name);
                            return;
                        }
                    };

                    let next_is_last = (i + 1) == commands.len() - 1;
                    let mut next_stdout = if next_is_last { Stdio::inherit() } else { Stdio::piped() };
                    let mut next_stderr = Stdio::inherit();

                    for (path, kind) in next_redirects.iter().rev() {
                        if let Ok(f) = open_redirect_file(path, kind) {
                            match kind {
                                RedirectKind::Stdout | RedirectKind::StdoutAppend => next_stdout = Stdio::from(f),
                                RedirectKind::Stderr | RedirectKind::StderrAppend => next_stderr = Stdio::from(f),
                                RedirectKind::Both | RedirectKind::BothAppend => {
                                    if let Ok(f2) = f.try_clone() {
                                        next_stdout = Stdio::from(f);
                                        next_stderr = Stdio::from(f2);
                                    }
                                }
                            }
                        }
                    }

                    match prepare_unix_command(&next_path, next_name, next_args)
                        .stdin(Stdio::piped()) // We will write to this
                        .stdout(next_stdout)
                        .stderr(next_stderr)
                        .spawn()
                    {
                        Ok(mut child) => {
                            // Run current builtin writing to child.stdin in a separate thread
                            // to avoid deadlock if the child produces output that fills the pipe
                            // before we can read it (in the next iteration).
                            if let Some(child_stdin) = child.stdin.take() {
                                let cmd_name_owned = cmd_name.clone();
                                let args_owned = args.clone();

                                // Move redirects to thread
                                let stdout_redirect = stdout_redirect;
                                let stderr_redirect = stderr_redirect;

                                thread::spawn(move || {
                                    let mut stdout: Box<dyn std::io::Write> = if let Some(f) = stdout_redirect {
                                        Box::new(f)
                                    } else {
                                        Box::new(child_stdin)
                                    };
                                    let mut stderr: Box<dyn std::io::Write> = if let Some(f) = stderr_redirect {
                                        Box::new(f)
                                    } else {
                                        Box::new(std::io::stderr())
                                    };
                                    let _ = builtins::run_builtin(&cmd_name_owned, &args_owned, &mut stdout, &mut stderr);
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

fn unwrap_command(mut cmd: &Command) -> (&Command, Vec<(&str, &RedirectKind)>) {
    let mut redirects = Vec::new();
    while let Command::RedirectCommand(inner, path, kind) = cmd {
        redirects.push((path.as_str(), kind));
        cmd = inner;
    }
    (cmd, redirects)
}

fn open_redirect_file(path: &str, kind: &RedirectKind) -> std::io::Result<std::fs::File> {
    let mut opts = OpenOptions::new();
    opts.create(true).write(true);
    match kind {
        RedirectKind::Stdout | RedirectKind::Stderr | RedirectKind::Both => { opts.truncate(true); },
        RedirectKind::StdoutAppend | RedirectKind::StderrAppend | RedirectKind::BothAppend => { opts.append(true); },
    }
    opts.open(path)
}
