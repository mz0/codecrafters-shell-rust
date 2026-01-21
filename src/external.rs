use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Prepares a Command object with the executable path and arguments.
/// Handles argv[0] setting.
pub(crate) fn prepare_unix_command(path: &Path, name: &str, args: &[String]) -> Command {
    let mut cmd = Command::new(path);
    cmd.arg0(name).args(args);
    cmd
}

/// Runs an external command, inheriting stdin/stdout/stderr.
pub fn run_unix(path: PathBuf, name: &str, args: &[String]) -> io::Result<i32> {
    let status = prepare_unix_command(&path, name, args).status()?;
    Ok(status.code().unwrap_or(128))
}
