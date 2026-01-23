use std::env;
use std::io::{self};
use std::path::Path;

use shlib::{
    builtins, external, history, pipeline,
    parse::{parse, Command},
    executables::{find_executable_in_path, get_all_executables},
    rline::ShellHelper,
};

fn main() {
    let system_commands = get_all_executables(); // Scan PATH once
    let h = ShellHelper { builtins: builtins::all().clone(), system_commands };
    let mut rl = shlib::create_editor(h).unwrap();

    if let Ok(histfile) = env::var("HISTFILE") {
        let path = Path::new(&histfile);
        if let Ok(count) = history::read_from_file(path) {
            let recent = history::get_recent(count);
            for cmd in recent {
                _ = rl.add_history_entry(cmd.as_str());
            }
        }
    }

    loop {
        // Prompt
        match rl.readline("$ ") {
            Ok(line) => {
                let cmd_line = line.trim();
                if cmd_line.is_empty() { continue; }

                _ = rl.add_history_entry(cmd_line);
                history::add(cmd_line);

                match parse(cmd_line) {
                    Command::SimpleCommand(cmd, args) => {
                        if cmd == builtins::CMD_EXIT {
                            if let Ok(histfile) = env::var("HISTFILE") {
                                _ = history::append_to_file(Path::new(&histfile));
                            }
                            break
                        }

                        let mut stdout = io::stdout();
                        let mut stderr = io::stderr();
                        if cmd == builtins::CMD_HISTORY {
                            _ = builtins::run_builtin(&*cmd, &args, &mut stdout, &mut stderr);
                            // TODO cmd == "history -r .." -> update rustyline history
                        } else if builtins::all().contains(&&*cmd) {
                            _ = builtins::run_builtin(&*cmd, &args, &mut stdout, &mut stderr);
                        } else if let Some(exec_path) = find_executable_in_path(&cmd) {
                            _ = external::run_unix(exec_path, &cmd, &args);
                        } else {
                            eprintln!("{cmd}: command not found")
                        }
                    },
                    Command::PipeCommand(commands) => {
                        pipeline::run_pipeline(&commands);
                    },
                    c @ Command::RedirectCommand(_, _, _) => {
                        pipeline::run_pipeline(&vec![c]);
                    },
                    Command::InvalidCommand(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            },
            Err(_) => {
                if let Ok(histfile) = env::var("HISTFILE") {
                    _ = history::append_to_file(Path::new(&histfile));
                }
                break
            }, // Handles Ctrl+C / Ctrl+D
        }
    }
}
