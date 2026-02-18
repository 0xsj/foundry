// Testing I/O Code — Cursor, Trait Abstraction, and In-Memory I/O (Rust)
//
// Shows how to write functions that are testable without touching the filesystem,
// using Cursor<Vec<u8>> as a drop-in for files in tests.
//
// Run: rustc --test testing_io.rs && ./testing_io

use std::io::{self, BufRead, BufReader, Cursor, Read, Write};
use std::path::Path;

// ---- 1. Accept traits, not concrete types ----
//
// The key insight: if your function accepts `impl BufRead` instead of `File`,
// tests can pass a Cursor<&[u8]> — no temp files, no cleanup, no I/O in tests.
//
// Rule: accept the most general trait that satisfies your needs.
// - Need lines()?           → impl BufRead
// - Need raw bytes?         → impl Read
// - Need to write?          → impl Write
// - Need to seek?           → impl Read + Seek

/// Parse CSV records from any buffered reader.
/// Returns (headers, rows) where each row is a Vec of field values.
fn parse_csv(reader: impl BufRead) -> io::Result<(Vec<String>, Vec<Vec<String>>)> {
    let mut lines = reader.lines();

    // First line is headers
    let header_line = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "empty CSV"))??;

    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Remaining lines are data rows
    let mut rows = Vec::new();
    for line in lines {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
        rows.push(fields);
    }

    Ok((headers, rows))
}

/// Write records in CSV format to any writer.
fn write_csv(writer: &mut impl Write, headers: &[&str], rows: &[Vec<String>]) -> io::Result<()> {
    writeln!(writer, "{}", headers.join(","))?;
    for row in rows {
        writeln!(writer, "{}", row.join(","))?;
    }
    Ok(())
}

// ---- 2. Cursor<Vec<u8>> as a writable in-memory buffer ----
//
// Cursor<Vec<u8>> is the standard tool for capturing writes in tests.
// After writing, call into_inner() to get the bytes back.

fn render_status_page(writer: &mut impl Write, services: &[(&str, bool)]) -> io::Result<()> {
    writeln!(writer, "# Service Health")?;
    writeln!(writer, "")?;
    for (name, healthy) in services {
        let status = if *healthy { "OK" } else { "DOWN" };
        writeln!(writer, "- {}: {}", name, status)?;
    }
    Ok(())
}

// ---- 3. Abstracting over file path and in-memory source ----
//
// Sometimes you need to open files in production but use in-memory data in tests.
// The trait-based approach handles this cleanly: production code passes a BufReader<File>,
// tests pass a BufReader<Cursor<&[u8]>>.
//
// The production entry point converts the path to a BufRead:

fn process_log_file(path: &Path) -> io::Result<usize> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    count_error_lines(reader)
}

// The testable inner function accepts any BufRead:
fn count_error_lines(reader: impl BufRead) -> io::Result<usize> {
    Ok(reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|line| line.contains("ERROR"))
        .count())
}

// ---- 4. Reading captured output back as a string ----
//
// After writing to a Cursor<Vec<u8>>, use String::from_utf8 or
// from_utf8_lossy to inspect the result in tests.

fn format_metrics(writer: &mut impl Write, metrics: &[(&str, f64)]) -> io::Result<()> {
    for (name, value) in metrics {
        // Prometheus text format
        writeln!(writer, "# TYPE {} gauge", name)?;
        writeln!(writer, "{} {}", name, value)?;
    }
    Ok(())
}

// ---- 5. Two-phase Cursor: write then read back ----
//
// You can write to a Cursor<Vec<u8>>, then rewind and read back from it.
// This is useful for testing round-trip serialization.

fn round_trip_demo() -> io::Result<String> {
    let mut buf = Cursor::new(Vec::new());

    // Write phase
    write_csv(
        &mut buf,
        &["id", "name", "score"],
        &[
            vec!["1".into(), "alice".into(), "98".into()],
            vec!["2".into(), "bob".into(), "87".into()],
        ],
    )?;

    // Read phase — rewind to start before reading
    buf.set_position(0);
    let reader = BufReader::new(buf);
    let (headers, rows) = parse_csv(reader)?;

    Ok(format!(
        "headers: {:?}, {} rows",
        headers,
        rows.len()
    ))
}

// ---- Tests — demonstrate Cursor-based testing ----

#[cfg(test)]
mod tests {
    use super::*;

    // Test parse_csv without any files
    #[test]
    fn test_parse_csv_basic() {
        let input = b"name,age,city\nalice,30,london\nbob,25,berlin\n";
        let reader = BufReader::new(Cursor::new(input));
        let (headers, rows) = parse_csv(reader).unwrap();

        assert_eq!(headers, vec!["name", "age", "city"]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["alice", "30", "london"]);
        assert_eq!(rows[1], vec!["bob", "25", "berlin"]);
    }

    #[test]
    fn test_parse_csv_empty_body() {
        let input = b"col1,col2\n";
        let reader = BufReader::new(Cursor::new(input));
        let (headers, rows) = parse_csv(reader).unwrap();

        assert_eq!(headers, vec!["col1", "col2"]);
        assert!(rows.is_empty());
    }

    #[test]
    fn test_parse_csv_empty_input_errors() {
        let reader = BufReader::new(Cursor::new(b""));
        let result = parse_csv(reader);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
    }

    // Test write_csv captures output correctly
    #[test]
    fn test_write_csv_output() {
        let mut buf = Cursor::new(Vec::new());
        write_csv(
            &mut buf,
            &["id", "status"],
            &[
                vec!["1".into(), "active".into()],
                vec!["2".into(), "inactive".into()],
            ],
        )
        .unwrap();

        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(output, "id,status\n1,active\n2,inactive\n");
    }

    // Test the error counter without files
    #[test]
    fn test_count_error_lines() {
        let log = b"2026-02-18 INFO  start\n\
                    2026-02-18 ERROR disk full\n\
                    2026-02-18 WARN  retry\n\
                    2026-02-18 ERROR max retries\n";
        let reader = BufReader::new(Cursor::new(log));
        let count = count_error_lines(reader).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_error_lines_zero() {
        let log = b"INFO start\nINFO stop\n";
        let reader = BufReader::new(Cursor::new(log));
        assert_eq!(count_error_lines(reader).unwrap(), 0);
    }

    // Test status page rendering
    #[test]
    fn test_render_status_page() {
        let mut buf = Cursor::new(Vec::new());
        render_status_page(
            &mut buf,
            &[("auth", true), ("database", false), ("cache", true)],
        )
        .unwrap();

        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains("auth: OK"));
        assert!(output.contains("database: DOWN"));
        assert!(output.contains("cache: OK"));
    }

    // Test metrics format
    #[test]
    fn test_format_metrics() {
        let mut buf = Cursor::new(Vec::new());
        format_metrics(
            &mut buf,
            &[("http_requests_total", 1234.0), ("error_rate", 0.02)],
        )
        .unwrap();

        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains("# TYPE http_requests_total gauge"));
        assert!(output.contains("http_requests_total 1234"));
        assert!(output.contains("error_rate 0.02"));
    }

    // Test round-trip: write CSV then read it back
    #[test]
    fn test_round_trip() {
        let result = round_trip_demo().unwrap();
        assert!(result.contains("headers:"));
        assert!(result.contains("2 rows"));
    }
}

fn main() -> io::Result<()> {
    println!("=== Testing I/O Code with Cursor ===");
    println!();

    // Demonstrate write to in-memory buffer
    let services = [("auth", true), ("payments", true), ("recommendations", false)];
    let mut buf = Cursor::new(Vec::new());
    render_status_page(&mut buf, &services)?;
    let page = String::from_utf8(buf.into_inner()).unwrap();
    print!("[status_page]\n{}", page);

    // Demonstrate round-trip
    let result = round_trip_demo()?;
    println!("[round_trip] {}", result);

    println!();
    println!("Run `rustc --test testing_io.rs && ./testing_io` to run the test suite.");
    Ok(())
}
