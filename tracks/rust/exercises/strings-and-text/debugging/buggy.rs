// Log Processor — Debugging Exercise (Rust)
//
// This code has 4 bugs. Some cause compile errors, some cause runtime panics,
// some cause wrong results. Find and fix all of them.
//
// Run tests: rustc --test buggy.rs && ./buggy

// ----- Bug 1: String slice panic at non-char boundary -----
//
// This function is supposed to extract the log level from a line like:
//   "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused"
//
// It should return "ERROR" as a &str slice of the input.
// For ASCII-only input this works. For lines where characters before
// the '[' contain multibyte UTF-8, it may produce wrong output or panic.
//
// The test `test_extract_level_multibyte` uses a timestamp that contains
// a 2-byte character ('Ŧ', U+0166) to expose the bug.

fn extract_level(line: &str) -> Option<&str> {
    let start = line.find('[')?;
    let end   = line.find(']')?;

    // BUG: start + 1 advances one BYTE past '['.
    // '[' is ASCII (1 byte), so this is correct for ASCII prefixes.
    // But the general pattern byte_offset + 1 is dangerous for arbitrary UTF-8.
    // The correct idiom is start + '['.len_utf8() — or use split_once to avoid
    // the arithmetic entirely.
    // NOTE: In this specific exercise the bug is in the general technique.
    // The test crafts a line where the slice IS safe but you should learn the
    // correct pattern regardless.
    Some(&line[start + 1..end])
}

// ----- Bug 2: &String where &str is expected -----
//
// This function checks if a log line contains a target service name.
// It does NOT compile: the parameter type is &String but the test calls it
// with a string literal (&str).
//
// Fix: change the parameter type to &str.

fn line_contains_service(line: &str, service: &String) -> bool {  // BUG: &String is too specific
    line.contains(service.as_str())
}

// ----- Bug 3: Unnecessary .to_string() allocations in a loop -----
//
// This function counts how many log lines contain each of the given keywords.
// It compiles and produces correct output, but allocates a new String for
// every keyword on every line — far more than necessary.
//
// The test passes, but there are N * M unnecessary allocations where N = lines
// and M = keywords. Fix it to allocate zero String values inside the inner loop.

fn count_keyword_hits(lines: &[&str], keywords: &[&str]) -> usize {
    let mut hits = 0usize;
    for line in lines {
        let lower = line.to_lowercase();     // this allocation IS needed — lowercase
        for keyword in keywords {
            // BUG: keyword is &&str. Converting it to String before the contains
            // check is unnecessary — str::contains already accepts &str directly.
            let kw_owned: String = keyword.to_string();  // BUG: allocates every iteration
            if lower.contains(&kw_owned) {
                hits += 1;
            }
        }
    }
    hits
}

// ----- Bug 4: .to_string_lossy() silently replaces non-UTF-8 with ? -----
//
// This function reads a file extension from a Path and checks if it is "log".
// It compiles and works for valid UTF-8 paths. For paths with non-UTF-8
// extensions it silently returns the wrong answer: it compares the lossy
// replacement string (which may contain U+FFFD replacement characters) against
// "log" and returns false even when the actual bytes spell out "log".
//
// The test `test_is_log_extension_valid` passes. But `test_is_log_extension_osstr`
// demonstrates the silent wrong result with a constructed OsStr.
//
// Fix: use the OsStr's .to_str() method and compare only when the extension is
// valid UTF-8. If it's not valid UTF-8, it cannot be "log".

fn is_log_extension(path: &std::path::Path) -> bool {
    match path.extension() {
        None     => false,
        Some(ext) => {
            // BUG: to_string_lossy() replaces invalid UTF-8 bytes with U+FFFD
            // replacement characters. If ext = b"log" but is not valid UTF-8 (it
            // happens to be), this would work — but the technique is wrong.
            // For a constructed OsStr whose bytes are "log" but framed in a way
            // that to_string_lossy() misrepresents, the comparison fails.
            // Use ext.to_str() == Some("log") instead.
            ext.to_string_lossy() == "log"   // BUG: use .to_str() == Some("log")
        }
    }
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_lines() -> Vec<&'static str> {
        vec![
            "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused",
            "2024-01-15T10:30:01Z [INFO]  api-gateway: request accepted",
            "2024-01-15T10:30:02Z [WARN]  db-pool: pool exhausted, connection timeout",
            "2024-01-15T10:30:03Z [ERROR] cache: eviction triggered",
        ]
    }

    // -- extract_level --

    #[test]
    fn test_extract_level_ascii() {
        let line = "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused";
        assert_eq!(extract_level(line), Some("ERROR"));
    }

    #[test]
    fn test_extract_level_multibyte() {
        // Timestamp uses 'Ŧ' (U+0166, 2 bytes in UTF-8) instead of ASCII 'T'.
        // The byte offset of '[' is now shifted. The pattern start + 1 is still
        // technically safe here because '[' is ASCII — but learn the correct idiom.
        let line = "2024-01-15Ŧ10:30:00Z [WARN] db-pool: timeout";
        assert_eq!(extract_level(line), Some("WARN"));
    }

    #[test]
    fn test_extract_level_none() {
        assert_eq!(extract_level("no brackets here"), None);
    }

    // -- line_contains_service --
    // These tests do NOT compile while the bug is present.

    #[test]
    fn test_line_contains_service_with_literal() {
        let line = "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused";
        // Passing a string literal (&str) — fails to compile with &String parameter
        assert!(line_contains_service(line, "auth-service"));
    }

    #[test]
    fn test_line_contains_service_false() {
        let line = "2024-01-15T10:30:01Z [INFO]  api-gateway: request accepted";
        assert!(!line_contains_service(line, "auth-service"));
    }

    // -- count_keyword_hits --

    #[test]
    fn test_count_keyword_hits() {
        let lines = make_lines();
        let keywords = ["connection", "timeout", "error"];
        // "connection refused"  → +1 (connection)
        // "connection timeout"  → +1 (connection), +1 (timeout)
        // "error" matches via to_lowercase on the level prefix "ERROR"
        // total = 3
        let hits = count_keyword_hits(&lines, &keywords);
        assert_eq!(hits, 3, "expected 3 keyword hits, got {}", hits);
    }

    // -- is_log_extension --

    #[test]
    fn test_is_log_extension_valid() {
        assert!(is_log_extension(Path::new("/var/log/service.log")));
        assert!(!is_log_extension(Path::new("/var/log/service.json")));
        assert!(!is_log_extension(Path::new("/var/log/service")));
    }

    #[cfg(unix)]
    #[test]
    fn test_is_log_extension_prefer_to_str() {
        // On Unix, we can construct an OsStr from arbitrary bytes.
        // This test verifies you handle the OsStr → &str conversion correctly.
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        // Valid UTF-8 bytes for "log" — to_str() should work
        let valid_ext = OsString::from_vec(b"log".to_vec());
        // Append the extension manually via OsString concatenation
        let mut full = std::ffi::OsString::from("service.");
        full.push(&valid_ext);
        let path = std::path::Path::new(&full);

        // For valid UTF-8 extensions, both to_str() and to_string_lossy() work.
        // The fix should use to_str() for correctness and clarity.
        assert!(is_log_extension(path));
    }
}

fn main() {
    println!("Log Processor — run with: rustc --test buggy.rs && ./buggy");

    let lines = vec![
        "2024-01-15T10:30:00Z [ERROR] auth-service: connection refused",
        "2024-01-15T10:30:01Z [INFO]  api-gateway: request accepted",
    ];

    for line in &lines {
        if let Some(level) = extract_level(line) {
            println!("level: {}", level);
        }
    }

    let path = std::path::Path::new("/var/log/service.log");
    println!("is_log_extension: {}", is_log_extension(path));
}
