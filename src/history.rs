use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static HISTORY: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

const MAX_HISTORY_SIZE: usize = 1000;
const MAX_COMMAND_LEN: usize = 1024;

/// Adds a command to the global history.
/// Should be called from your main loop after reading input.
pub fn add(command: &str) {
    if command.trim().is_empty() {
        return;
    }

    let history_mutex = HISTORY.get_or_init(|| Mutex::new(Vec::new()));
    let mut history = history_mutex.lock().unwrap();

    // Truncate long commands if necessary
    let cmd_string = if command.len() > MAX_COMMAND_LEN {
        format!("{}...", &command[..MAX_COMMAND_LEN])
    } else {
        command.to_string()
    };

    history.push(cmd_string);
    if history.len() > MAX_HISTORY_SIZE {
        history.remove(0);
    }
}

/// Prints the current history to stdout.
/// To be called by the `history` builtin.
pub fn print(stdout: &mut dyn Write, limit: Option<usize>) {
    let history_mutex = HISTORY.get_or_init(|| Mutex::new(Vec::new()));
    let history = history_mutex.lock().unwrap();

    let start_index = match limit {
        Some(n) => history.len().saturating_sub(n),
        None => 0,
    };

    for (i, command) in history.iter().enumerate().skip(start_index) {
        let _ = writeln!(stdout, "{:5}  {}", i + 1, command);
    }
}

/// Writes the current history to a file.
pub fn write_to_file(path: &Path) -> std::io::Result<()> {
    let history_mutex = HISTORY.get_or_init(|| Mutex::new(Vec::new()));
    let history = history_mutex.lock().unwrap();

    let mut file = File::create(path)?;
    for command in history.iter() {
        writeln!(file, "{}", command)?;
    }
    Ok(())
}

/// Reads history from a file, appending to the current history.
pub fn read_from_file(path: &Path) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        add(&line?);
    }
    Ok(())
}
