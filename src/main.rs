use std::io::{self};

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
                        if cmd == builtins::CMD_EXIT { break }

                        let mut stdout = io::stdout();
                        let mut stderr = io::stderr();
                        if builtins::all().contains(&&*cmd) {
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
                    Command::InvalidCommand(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            },
            Err(_) => break, // Handles Ctrl+C / Ctrl+D
        }
    }
}
