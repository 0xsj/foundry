// Read/Write Traits — I/O Fundamentals (Rust)
//
// Demonstrates Read, Write, BufReader, BufWriter, and trait-based I/O abstraction.
// All examples use in-memory buffers so no files are required to run.
//
// Run: rustc read_write.rs && ./read_write

use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Read, Write};

// ---- 1. Writing to any impl Write ----
//
// By accepting `impl Write`, this function works with files, sockets, Vec<u8>,
// BufWriter, stdout — anything that implements Write. No changes needed at
// the call site when you switch the backing writer.

fn write_report(writer: &mut impl Write, title: &str, entries: &[(&str, u64)]) -> io::Result<()> {
    writeln!(writer, "=== {} ===", title)?;
    for (label, value) in entries {
        writeln!(writer, "  {:<20} {}", label, value)?;
    }
    writeln!(writer, "  {:-<28}", "")?;
    let total: u64 = entries.iter().map(|(_, v)| v).sum();
    writeln!(writer, "  {:<20} {}", "TOTAL", total)?;
    Ok(())
}

// ---- 2. Reading from any impl BufRead ----
//
// BufRead instead of Read gives us .lines() — essential for text processing.
// Using BufRead as the bound (not File or BufReader<File>) makes the function
// testable with Cursor<&[u8]> without touching the filesystem.

fn count_lines_containing(reader: impl BufRead, pattern: &str) -> io::Result<usize> {
    let mut count = 0;
    for line in reader.lines() {
        let line = line?;
        if line.contains(pattern) {
            count += 1;
        }
    }
    Ok(count)
}

// ---- 3. BufWriter flushes in chunks — explicit flush required ----
//
// BufWriter accumulates writes in an 8 KB buffer. The buffer is flushed to
// the underlying writer when:
//   (a) the buffer fills up
//   (b) flush() is called explicitly
//   (c) the BufWriter is dropped
//
// Case (c) is the trap: flush-on-drop silently discards errors.
// Always call flush() explicitly if the write must succeed.

fn demonstrate_bufwriter() -> io::Result<()> {
    let mut output: Vec<u8> = Vec::new();

    {
        let mut writer = BufWriter::new(&mut output);

        // These writes go to the internal buffer, not to `output` yet
        for i in 0..5 {
            writeln!(writer, "record-{:04}", i)?;
        }

        // Explicit flush — errors surface here, not silently in drop
        writer.flush()?;
    } // writer dropped here; flush already done

    println!(
        "[BufWriter] wrote {} bytes across 5 lines",
        output.len()
    );
    println!("[BufWriter] first line: {}", String::from_utf8_lossy(&output).lines().next().unwrap_or(""));
    Ok(())
}

// ---- 4. BufReader wraps any Read — adds line iteration and efficient buffering ----

fn demonstrate_bufreader() -> io::Result<()> {
    // Simulate file contents in memory using Cursor<&[u8]>
    let log_data = b"2026-02-18 INFO  service started\n\
                     2026-02-18 ERROR database unreachable\n\
                     2026-02-18 INFO  retrying connection\n\
                     2026-02-18 ERROR max retries exceeded\n\
                     2026-02-18 WARN  falling back to cache\n";

    // BufReader wraps the Cursor — adds buffering and BufRead trait
    let reader = BufReader::new(Cursor::new(log_data));
    let error_count = count_lines_containing(reader, "ERROR")?;

    println!("[BufReader] found {} ERROR lines", error_count);
    Ok(())
}

// ---- 5. Read chaining with take() and chain() ----
//
// take(n) limits a reader to n bytes.
// chain(other) concatenates two readers end-to-end.
// Both adapters are lazy — no data is copied until you actually read.

fn demonstrate_adapters() -> io::Result<()> {
    let header = b"HEADER|";
    let body = b"payload data here";

    // Read header bytes followed by body bytes — no allocation until read
    let mut combined = Vec::new();
    Cursor::new(header).chain(Cursor::new(body)).read_to_end(&mut combined)?;
    println!("[chain]  combined: {}", String::from_utf8_lossy(&combined));

    // Limit reading to first 7 bytes
    let mut limited = Vec::new();
    Cursor::new(b"truncate this after seven").take(7).read_to_end(&mut limited)?;
    println!("[take]   first 7 bytes: {}", String::from_utf8_lossy(&limited));

    Ok(())
}

// ---- 6. io::copy — zero-allocation stream transfer ----
//
// io::copy reads from a Read and writes to a Write using an 8 KB internal buffer.
// Equivalent to: loop { read chunk; write chunk } but with no heap allocation.
// Works identically for files, sockets, or in-memory buffers.

fn demonstrate_copy() -> io::Result<()> {
    let source = b"streaming data: 1 2 3 4 5 6 7 8 9 10";
    let mut destination: Vec<u8> = Vec::new();

    let bytes_copied = io::copy(&mut Cursor::new(source), &mut destination)?;

    println!("[copy]   copied {} bytes", bytes_copied);
    println!("[copy]   destination: {}", String::from_utf8_lossy(&destination));
    Ok(())
}

fn main() -> io::Result<()> {
    // --- write_report to an in-memory Vec<u8> ---
    let metrics = [
        ("requests_total", 142_831u64),
        ("errors_total", 23u64),
        ("cache_hits", 98_541u64),
    ];

    let mut report_buf: Vec<u8> = Vec::new();
    write_report(&mut report_buf, "Service Metrics", &metrics)?;
    println!("{}", String::from_utf8_lossy(&report_buf));

    // --- write_report to stdout directly (same function, different writer) ---
    let stdout = io::stdout();
    let mut locked = stdout.lock();
    write_report(&mut locked, "Same Function, stdout Writer", &metrics[..1])?;
    locked.flush()?;
    drop(locked);
    println!();

    demonstrate_bufwriter()?;
    println!();

    demonstrate_bufreader()?;
    println!();

    demonstrate_adapters()?;
    println!();

    demonstrate_copy()?;

    Ok(())
}
