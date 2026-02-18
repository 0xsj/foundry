// Log Processor — Debugging Exercise (Rust / I/O and Files)
//
// This code compiles (mostly) but has 4 bugs — one per function.
// Read the symptoms in debugging/README.md, find the root cause, and fix it.
//
// Run: rustc --test buggy.rs && ./buggy

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

// ---- Bug 1: BufWriter silently discards data ----
//
// write_summary writes a report to a file. The function returns Ok(()),
// the file is created — but it's always 0 bytes. No error is ever returned.
//
// Hint: BufWriter accumulates writes in an 8 KB buffer. When does that buffer
// get written to the underlying file? What happens when BufWriter is dropped?

pub fn write_summary(path: &Path, lines: &[String]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for line in lines {
        writeln!(writer, "{}", line)?;
    }

    Ok(())
    // BUG: the BufWriter is dropped here without flushing.
    // Drop calls flush() but any flush error is silently discarded.
    // Small writes that fit in the 8 KB buffer are never written to disk.
}

// ---- Bug 2: Reading a file byte-by-byte without BufReader ----
//
// scan_log_slow counts lines containing a search term. It works correctly
// but is catastrophically slow on large files.
//
// Hint: File implements Read. Read::lines() requires BufRead, not Read.
// What trait provides lines()? What's missing here?

pub fn scan_log_slow(path: &Path, term: &str) -> io::Result<usize> {
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut content = String::new();
    // BUG: reading the entire file into memory at once defeats the purpose of streaming.
    // For a 10 GB log file this allocates 10 GB of RAM.
    // The fix: wrap file in BufReader and iterate with .lines() instead of read_to_string.
    file.read_to_string(&mut content)?;

    let count = content
        .lines()
        .filter(|line| line.contains(term))
        .count();

    Ok(count)
}

// ---- Bug 3: unwrap() in library code ----
//
// open_report_file opens or creates a report file. But it panics when the
// path doesn't exist or when permissions are denied — instead of returning
// an error that the caller can handle.
//
// Hint: Library functions must never panic on expected failure cases.
// What should replace unwrap() here?

pub fn open_report_file(path: &Path) -> File {
    // BUG: unwrap() here panics the entire process if the file can't be opened.
    // Library code must propagate errors — the caller decides how to handle them.
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap()
}

// ---- Bug 4: String concatenation for paths ----
//
// build_archive_path constructs a path to an archive file.
// On Windows, the separator is `\` but this code always uses `/`.
// On any OS, this approach is fragile — a trailing slash in base_dir
// would produce double slashes like `logs//2026-02-18.tar.gz`.
//
// Hint: What type in std::path is designed to compose paths safely?

pub fn build_archive_path(base_dir: &str, date: &str) -> String {
    // BUG: string concatenation for paths is platform-specific and fragile.
    // This breaks on Windows (wrong separator) and is sensitive to trailing slashes.
    base_dir.to_string() + "/" + date + ".tar.gz"
}

// ---- Tests ----
//
// Note on Bugs 3 and 4:
// These bugs change return types (File -> io::Result<File>, String -> PathBuf).
// The tests below are written for the FIXED signatures and will not compile until
// you apply the fix. That compile error is intentional — it shows you exactly what
// the type should be. Bugs 1 and 2 compile but produce wrong output at runtime.

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: write_summary must actually write content
    // RUNTIME BUG — compiles, but file is empty after the call
    #[test]
    fn test_write_summary_creates_content() {
        let dir = std::env::temp_dir().join("foundry_debug_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("summary.txt");

        let lines = vec![
            "errors: 42".to_string(),
            "warnings: 7".to_string(),
            "processed: 1000".to_string(),
        ];

        write_summary(&path, &lines).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(
            !content.is_empty(),
            "file is empty — BufWriter was not flushed before drop"
        );
        assert!(content.contains("errors: 42"));
        assert!(content.contains("processed: 1000"));

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    // Test 2: scan_log_slow — correct results, but catastrophically slow on large files
    // RUNTIME BUG — compiles, returns correct results, but reads entire file into RAM
    #[test]
    fn test_scan_log_slow_counts_correctly() {
        let dir = std::env::temp_dir().join("foundry_debug_scan");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("app.log");

        fs::write(
            &path,
            "INFO  start\nERROR disk full\nINFO  retry\nERROR max retries\nINFO  done\n",
        )
        .unwrap();

        let count = scan_log_slow(&path, "ERROR").unwrap();
        assert_eq!(count, 2, "expected 2 ERROR lines");

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }

    // Test 3: open_report_file must return io::Result<File>, not panic
    // TYPE BUG — the test below requires open_report_file to return io::Result<File>.
    // It will not compile until you change the return type and remove unwrap().
    #[test]
    fn test_open_report_file_returns_result() {
        let dir = std::env::temp_dir().join("foundry_debug_report");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("report.log");

        // After the fix: open_report_file returns io::Result<File>
        let result: io::Result<File> = open_report_file(&path);
        assert!(result.is_ok(), "expected Ok(File)");

        // A bad path must return Err, not panic
        let bad_path = Path::new("/nonexistent/deeply/nested/report.log");
        let bad_result: io::Result<File> = open_report_file(bad_path);
        assert!(bad_result.is_err(), "expected Err for missing parent directory");

        fs::remove_dir_all(&dir).ok();
    }

    // Test 4: build_archive_path must return PathBuf and use OS-native separators
    // TYPE BUG — the test below requires build_archive_path to return PathBuf.
    // It will not compile until you change the return type to PathBuf.
    #[test]
    fn test_build_archive_path_correct_separator() {
        // After the fix: build_archive_path returns PathBuf
        let path: PathBuf = build_archive_path("/var/log/archives", "2026-02-18");

        assert_eq!(
            path.file_name().and_then(|n| n.to_str()),
            Some("2026-02-18.tar.gz"),
            "file name component should be '2026-02-18.tar.gz'"
        );
        assert_eq!(
            path.parent().and_then(|p| p.to_str()),
            Some("/var/log/archives"),
            "parent directory should be '/var/log/archives'"
        );
    }

    #[test]
    fn test_build_archive_path_no_double_slash() {
        let path: PathBuf = build_archive_path("/var/log/archives/", "2026-02-18");
        let path_str = path.to_string_lossy();
        assert!(
            !path_str.contains("//"),
            "path must not contain double slash: {}",
            path_str
        );
    }
}

fn main() {
    println!("Fix the bugs, then run: rustc --test buggy.rs && ./buggy");
}
