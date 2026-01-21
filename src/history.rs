use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

struct History {
    commands: Vec<String>,
    unsaved_idx: usize,
}

static HISTORY: OnceLock<Mutex<History>> = OnceLock::new();

const MAX_HISTORY_SIZE: usize = 1000;
const MAX_COMMAND_LEN: usize = 1024;

/// Adds a command to the global history.
/// Should be called from your main loop after reading input.
pub fn add(command: &str) {
    let history_mutex = HISTORY.get_or_init(|| Mutex::new(History { commands: Vec::new(), unsaved_idx: 0 }));
    let mut history = history_mutex.lock().unwrap();
    add_internal(&mut history, command);
}

fn add_internal(history: &mut History, command: &str) {
    if command.trim().is_empty() {
        return;
    }

    // Truncate long commands if necessary
    let cmd_string = if command.len() > MAX_COMMAND_LEN {
        format!("{}...", &command[..MAX_COMMAND_LEN])
    } else {
        command.to_string()
    };

    history.commands.push(cmd_string);
    if history.commands.len() > MAX_HISTORY_SIZE {
        history.commands.remove(0);
        if history.unsaved_idx > 0 {
            history.unsaved_idx -= 1;
        }
    }
}

/// Prints the current history to stdout.
/// To be called by the `history` builtin.
pub fn print(stdout: &mut dyn Write, limit: Option<usize>) {
    let history_mutex = HISTORY.get_or_init(|| Mutex::new(History { commands: Vec::new(), unsaved_idx: 0 }));
    let history = history_mutex.lock().unwrap();

    let start_index = match limit {
        Some(n) => history.commands.len().saturating_sub(n),
        None => 0,
    };

    for (i, command) in history.commands.iter().enumerate().skip(start_index) {
        let _ = writeln!(stdout, "{:5}  {}", i + 1, command);
    }
}

/// Writes the current history to a file.
pub fn write_to_file(path: &Path) -> std::io::Result<()> {
    let history_mutex = HISTORY.get_or_init(|| Mutex::new(History { commands: Vec::new(), unsaved_idx: 0 }));
    let mut history = history_mutex.lock().unwrap();

    let mut file = File::create(path)?;
    for command in history.commands.iter() {
        writeln!(file, "{}", command)?;
    }
    // We wrote everything, so everything is now saved.
    history.unsaved_idx = history.commands.len();
    Ok(())
}

/// Reads history from a file, appending to the current history.
pub fn read_from_file(path: &Path) -> std::io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let history_mutex = HISTORY.get_or_init(|| Mutex::new(History { commands: Vec::new(), unsaved_idx: 0 }));
    let mut history = history_mutex.lock().unwrap();

    // If we are reading history and everything before was saved (e.g. startup),
    // we treat the loaded commands as saved as well.
    let was_fully_saved = history.unsaved_idx == history.commands.len();

    let mut count = 0;
    for line in reader.lines() {
        add_internal(&mut history, &line?);
        count += 1;
    }

    if was_fully_saved {
        history.unsaved_idx = history.commands.len();
    }
    Ok(count)
}

/// Appends the current history to a file.
pub fn append_to_file(path: &Path) -> std::io::Result<()> {
    let history_mutex = HISTORY.get_or_init(|| Mutex::new(History { commands: Vec::new(), unsaved_idx: 0 }));
    let mut history = history_mutex.lock().unwrap();

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    // Only write commands that haven't been saved yet
    for command in history.commands.iter().skip(history.unsaved_idx) {
        writeln!(file, "{}", command)?;
    }
    history.unsaved_idx = history.commands.len();
    Ok(())
}

/// Returns the last `n` commands from the history.
pub fn get_recent(n: usize) -> Vec<String> {
    let history_mutex = HISTORY.get_or_init(|| Mutex::new(History { commands: Vec::new(), unsaved_idx: 0 }));
    let history = history_mutex.lock().unwrap();
    let start_index = history.commands.len().saturating_sub(n);
    history.commands[start_index..].to_vec()
}
