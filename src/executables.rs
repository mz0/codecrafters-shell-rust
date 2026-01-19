use is_executable::IsExecutable;
use std::env;
use std::path::{Path, PathBuf};

pub fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let paths = env::split_paths(&path_var);

    for dir in paths {
        // Empty PATH entries mean current directory
        let dir = if dir.as_os_str().is_empty() {
            Path::new(".").to_path_buf()
        } else {
            dir
        };

        let candidate = dir.join(name);

        if candidate.is_executable() {
            return Some(candidate);
        }
    }

    None
}

pub fn get_all_executables() -> Vec<String> {
    let mut execs = Vec::new();

    if let Some(path_var) = env::var_os("PATH") {
        for path in env::split_paths(&path_var) {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_executable() {
                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                            execs.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    execs.sort();
    execs.dedup(); // Remove duplicates (e.g., if 'ls' is in two PATH folders)
    execs
}
