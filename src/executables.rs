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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_find_executable_with_single_quotes() {
        // Create a unique temporary directory
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("shell_test_{}", timestamp));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let exe_name = "exe with 'single quotes'";
        let exe_path = temp_dir.join(exe_name);

        // Create the file
        File::create(&exe_path).expect("Failed to create file");

        // Make it executable
        let mut perms = fs::metadata(&exe_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&exe_path, perms).expect("Failed to set permissions");

        // Save original PATH
        let original_path = env::var_os("PATH");

        // Set PATH to the temp dir
        unsafe { env::set_var("PATH", &temp_dir); }

        let result = find_executable_in_path(exe_name);

        // Restore PATH
        if let Some(path) = original_path {
            unsafe { env::set_var("PATH", path); }
        } else {
            unsafe { env::remove_var("PATH"); }
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);

        assert_eq!(result, Some(exe_path));
    }
}
