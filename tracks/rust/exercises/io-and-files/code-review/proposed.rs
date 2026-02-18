// Log Rotation — Proposed Implementation (Rust)
// Submitted for review: feat/log-rotation
//
// This compiles and passes the basic tests below.
// Review it for correctness, idiomatic style, and production readiness.
//
// Run: rustc --test proposed.rs && ./proposed

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Configuration for a rotating log.
pub struct RotatingLogger {
    log_path: String,               // Issue: should this be String or PathBuf?
    max_bytes: u64,
    archive_dir: String,            // Issue: same
}

impl RotatingLogger {
    pub fn new(log_path: String, max_bytes: u64, archive_dir: String) -> RotatingLogger {
        RotatingLogger {
            log_path,
            max_bytes,
            archive_dir,
        }
    }

    /// Write a log line. Rotates the log first if it has exceeded max_bytes.
    pub fn write_line(&mut self, line: &str) -> Result<(), String> {  // Issue: String error
        // Rotate if the file is too big
        if self.needs_rotation().unwrap() {   // Issue: unwrap
            self.rotate().unwrap();           // Issue: unwrap
        }

        // Open file, write line, close file — on every single write
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)   // Issue: opens and closes on every write call
            .unwrap();              // Issue: unwrap

        // Issue: no BufWriter — every writeln! is a direct syscall
        writeln!(file, "{}", line).unwrap();  // Issue: unwrap

        Ok(())
    }

    /// Check if the current log file exceeds max_bytes.
    fn needs_rotation(&self) -> Result<bool, String> {
        match fs::metadata(&self.log_path) {
            Ok(meta) => Ok(meta.len() >= self.max_bytes),
            Err(_) => Ok(false),  // File doesn't exist yet — no rotation needed
        }
    }

    /// Rename the current log to an archive path and start fresh.
    fn rotate(&mut self) -> Result<(), String> {
        // Build archive path: archive_dir + "/" + timestamp + ".log"
        // Issue: string concatenation for path construction
        let timestamp = "2026-02-18T10-30-00";  // Issue: hardcoded timestamp
        let archive_path = self.archive_dir.clone() + "/" + timestamp + ".log";

        // Issue: no check that archive_dir exists before renaming
        fs::rename(&self.log_path, &archive_path)
            .unwrap();  // Issue: unwrap — panics if archive_dir doesn't exist

        Ok(())
    }
}

/// Rotate a log file if it exceeds size_limit_bytes.
/// Returns true if rotation occurred.
///
/// Issue: this is a standalone function that duplicates RotatingLogger logic.
pub fn rotate_if_needed(log_path: &str, archive_dir: &str, size_limit_bytes: u64) -> bool {
    let size = fs::metadata(log_path)
        .map(|m| m.len())
        .unwrap_or(0);  // silently treats all errors as "no file"

    if size < size_limit_bytes {
        return false;
    }

    // Issue: string path concatenation
    let archive = archive_dir.to_string() + "/rotated-" + "timestamp" + ".log";

    // Issue: unwrap — panics on permission error, missing archive_dir, etc.
    fs::rename(log_path, &archive).unwrap();
    true
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_write_line_basic() {
        let dir = temp_dir("cr_test_basic");
        let log_path = dir.join("app.log").to_string_lossy().into_owned();
        let archive_dir = dir.join("archive").to_string_lossy().into_owned();
        fs::create_dir_all(&archive_dir).unwrap();

        let mut logger = RotatingLogger::new(log_path.clone(), 10_000, archive_dir);
        logger.write_line("hello").unwrap();
        logger.write_line("world").unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("world"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_rotate_if_needed_no_rotation_for_small_file() {
        let dir = temp_dir("cr_test_no_rotate");
        let log_path = dir.join("app.log");
        fs::write(&log_path, "small content").unwrap();
        let archive_dir = dir.join("archive").to_string_lossy().into_owned();
        fs::create_dir_all(&archive_dir).unwrap();

        let rotated = rotate_if_needed(
            &log_path.to_string_lossy(),
            &archive_dir,
            1_000_000,
        );
        assert!(!rotated);

        fs::remove_dir_all(&dir).ok();
    }
}

fn main() {
    println!("Log Rotation — Code Review Exercise");
    println!("Read the code, find the issues, fill in my-review.md.");
    println!("Run: rustc --test proposed.rs && ./proposed");
}
