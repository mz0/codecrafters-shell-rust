use crate::executables::find_executable_in_path;
use crate::history;

use std::collections::HashMap;
use std::env;
use std::io::{self, Write, Result};
use std::path::Path;
use std::sync::LazyLock;

type BuiltinFn = fn(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<()>;
static BUILTINS: LazyLock<HashMap<&'static str, BuiltinFn>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, BuiltinFn> = HashMap::new();
    m.insert(CMD_CD, cd);
    m.insert(CMD_ECHO, echo);
    m.insert(CMD_HISTORY, history);
    m.insert(CMD_PWD, pwd);
    m.insert(CMD_TYPE, type_of);
    m
});

pub fn run_builtin(
    cmd: &str,
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write
) -> Option<Result<()>> {
    // Look up the function in our map
    let fun = BUILTINS.get(cmd)?;
    Some(fun(args, stdout, stderr))
}

pub const CMD_CD: &str = "cd";
pub const CMD_ECHO: &str = "echo";
pub const CMD_EXIT: &str = "exit";
pub const CMD_HISTORY: &str = "history";
pub const CMD_PWD: &str = "pwd";
pub const CMD_TYPE: &str = "type";

pub fn all() -> Vec<&'static str> {
    vec![CMD_CD, CMD_ECHO, CMD_EXIT, CMD_HISTORY, CMD_PWD, CMD_TYPE]
}

pub fn type_of(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    let s = &args[0];
    if all().contains(&s.as_str()) {
        writeln!(stdout, "{s} is a shell builtin")?;
        return Ok(());
    }

    match find_executable_in_path(s) {
        Some(path) => writeln!(stdout, "{s} is {}", path.display())?,
        None => writeln!(stderr, "{s}: not found")?,
    }
    Ok(())
}

pub fn echo(args: &[String], stdout: &mut dyn Write, _stderr: &mut dyn Write) -> Result<()> {
    writeln!(stdout, "{}", args.join(" "))
}

pub fn history(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<()> {
    if let Some(first_arg) = args.first() {
        match first_arg.as_str() {
            "-r" => {
                if let Some(path_str) = args.get(1) {
                    let path = Path::new(path_str);
                    if let Err(e) = history::read_from_file(path) {
                        writeln!(stderr, "history: {}: {}", path.display(), e)?;
                    }
                } else {
                    writeln!(stderr, "history: -r: option requires an argument")?;
                }
            }
            "-w" => {
                if let Some(path_str) = args.get(1) {
                    let path = Path::new(path_str);
                    if let Err(e) = history::write_to_file(path) {
                        writeln!(stderr, "history: {}: {}", path.display(), e)?;
                    }
                } else {
                    writeln!(stderr, "history: -w: option requires an argument")?;
                }
            }
            "-a" => {
                if let Some(path_str) = args.get(1) {
                    let path = Path::new(path_str);
                    if let Err(e) = history::append_to_file(path) {
                        writeln!(stderr, "history: {}: {}", path.display(), e)?;
                    }
                } else {
                    writeln!(stderr, "history: -a: option requires an argument")?;
                }
            }
            _ => {
                // Not a flag, try to parse as a number for limit
                match first_arg.parse::<usize>() {
                    Ok(n) => history::print(stdout, Some(n)),
                    Err(_) => {
                        writeln!(stderr, "history: {}: numeric argument required", first_arg)?;
                    }
                }
            }
        }
    } else {
        // No arguments, print all history
        history::print(stdout, None);
    }
    Ok(())
}

pub fn pwd(_args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<()> {
    match env::current_dir() {
        Ok(cwd) => writeln!(stdout, "{}", cwd.display()),
        Err(e) => writeln!(stderr, "pwd: {}", e),
    }
}

pub fn cd(args: &[String], _stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<()> {
    let cd_path = if args.is_empty() { "~" } else { &args[0] };

    let path_str = if cd_path == "~" {
        // Try to get home directory, fallback to current dir "."
        env::var("HOME").unwrap_or_else(|_| ".".to_string())
    } else {
        cd_path.to_string()
    };

    let path = Path::new(&path_str);
    if let Err(e) = env::set_current_dir(path) {
        match e.kind() {
            io::ErrorKind::NotFound => writeln!(stderr, "cd: {}: No such file or directory", path.display())?,
            _ => writeln!(stderr, "cd: {}: {}", path.display(), e)?,
        }
    }
    Ok(())
}
