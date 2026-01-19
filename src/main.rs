#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;

// Import from our library
use shlib::{
    builtins,
    parse::{parse, Command},
    executables::{find_executable_in_path, get_all_executables},
    rline::ShellHelper,
};

fn main() {
    let builtins_list = builtins::all();
    let system_commands = get_all_executables(); // Scan PATH once
    let h = ShellHelper { builtins: builtins_list.clone(), system_commands };
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
                            _ = builtins::echo(&args, &mut stdout);
                        } else if cmd == builtins::CMD_CD {
                            builtins::cd(&args)
                        } else if cmd == builtins::CMD_PWD {
                            _ = builtins::pwd(&mut stdout, &mut stderr)
                        } else if cmd == builtins::CMD_TYPE {
                            _ = builtins::type_of(&args, &builtins_list, &mut stdout)
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
