use std::io::{self};

mod pipeline;
mod external;

// Import from our library
use shlib::{
    builtins,
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

                match parse(cmd_line) {
                    Command::SimpleCommand(cmd, args) => {
                        if cmd == builtins::CMD_EXIT { break }

                        let mut stdout = io::stdout();
                        let mut stderr = io::stderr();
                        if cmd == builtins::CMD_ECHO {
                            _ = builtins::echo(&args, &mut stdout, &mut stderr);
                        } else if cmd == builtins::CMD_CD {
                            _ = builtins::cd(&args, &mut stdout, &mut stderr);
                        } else if cmd == builtins::CMD_PWD {
                            _ = builtins::pwd(&args, &mut stdout, &mut stderr);
                        } else if cmd == builtins::CMD_TYPE {
                            _ = builtins::type_of(&args, &mut stdout, &mut stderr);
                        } else if let Some(exec_path) = find_executable_in_path(&cmd) {
                            _ = external::run_unix(exec_path, &cmd, &args);
                        } else {
                            eprintln!("{cmd}: command not found")
                        }
                    },
                    Command::PipeCommand(left, right) => {
                        pipeline::run_pipeline(&left, &right);
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
