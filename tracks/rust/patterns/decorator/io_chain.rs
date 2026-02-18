// Decorator Pattern: I/O Decorator Chain
//
// Demonstrates: Custom Read/Write decorators that compose with std library
// adapters. Builds CountingReader, LoggingWriter, and a ProgressReader
// that reports read progress as a percentage.
//
// Concepts: std::io::Read, std::io::Write, adapter composition,
// ownership in I/O chains, by_ref() for borrowing.
//
// Run: rustc io_chain.rs && ./io_chain

use std::io::{self, Read, Write, Cursor};

// ---------------------------------------------------------------------------
// CountingReader: tracks total bytes read through the decorator
// ---------------------------------------------------------------------------

struct CountingReader<R: Read> {
    inner: R,
    bytes_read: u64,
}

impl<R: Read> CountingReader<R> {
    fn new(inner: R) -> Self {
        CountingReader {
            inner,
            bytes_read: 0,
        }
    }

    fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    /// Consume the decorator and return the inner reader.
    /// Useful when you want to stop counting but continue reading.
    fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.bytes_read += n as u64;
        Ok(n)
    }
}

// ---------------------------------------------------------------------------
// ProgressReader: reports read progress as percentage of known total
// ---------------------------------------------------------------------------

struct ProgressReader<R: Read> {
    inner: R,
    total_bytes: u64,
    bytes_read: u64,
    label: String,
    last_reported_pct: u8,
}

impl<R: Read> ProgressReader<R> {
    fn new(inner: R, total_bytes: u64, label: &str) -> Self {
        ProgressReader {
            inner,
            total_bytes,
            bytes_read: 0,
            label: label.to_string(),
            last_reported_pct: 0,
        }
    }

    fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.bytes_read += n as u64;

        // Report progress every 10%
        if self.total_bytes > 0 {
            let pct = ((self.bytes_read as f64 / self.total_bytes as f64) * 100.0) as u8;
            let rounded = (pct / 10) * 10;
            if rounded > self.last_reported_pct || (n == 0 && self.bytes_read > 0) {
                println!(
                    "[{}] progress: {} / {} bytes ({}%)",
                    self.label, self.bytes_read, self.total_bytes, pct
                );
                self.last_reported_pct = rounded;
            }
        }

        Ok(n)
    }
}

// ---------------------------------------------------------------------------
// LoggingWriter: logs every write operation with byte count
// ---------------------------------------------------------------------------

struct LoggingWriter<W: Write> {
    inner: W,
    label: String,
    total_written: u64,
}

impl<W: Write> LoggingWriter<W> {
    fn new(inner: W, label: &str) -> Self {
        LoggingWriter {
            inner,
            label: label.to_string(),
            total_written: 0,
        }
    }

    fn total_written(&self) -> u64 {
        self.total_written
    }
}

impl<W: Write> Write for LoggingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.total_written += n as u64;

        // Show a preview of what was written (first 40 bytes max)
        let preview_len = n.min(40);
        let preview = String::from_utf8_lossy(&buf[..preview_len]);
        let suffix = if n > 40 { "..." } else { "" };
        println!(
            "[{}] wrote {} bytes: {:?}{}",
            self.label, n, preview, suffix
        );
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        println!("[{}] flush", self.label);
        self.inner.flush()
    }
}

// ---------------------------------------------------------------------------
// UppercaseWriter: transforms all written bytes to uppercase ASCII
// ---------------------------------------------------------------------------

struct UppercaseWriter<W: Write> {
    inner: W,
}

impl<W: Write> UppercaseWriter<W> {
    fn new(inner: W) -> Self {
        UppercaseWriter { inner }
    }
}

impl<W: Write> Write for UppercaseWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Transform to uppercase, then write to inner
        let upper: Vec<u8> = buf.iter().map(|b| b.to_ascii_uppercase()).collect();
        self.inner.write(&upper)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

// ---------------------------------------------------------------------------
// Main: demonstrate I/O decorator chains
// ---------------------------------------------------------------------------

fn main() {
    println!("=== COUNTING READER ===\n");

    // Simulate a data source (in production, this would be a file or network stream)
    let data = b"The quick brown fox jumps over the lazy dog. This sentence has more bytes.";
    let source = Cursor::new(data.as_slice());

    // Decorate with counting
    let mut reader = CountingReader::new(source);

    // Read in small chunks to see counting in action
    let mut buf = [0u8; 20];
    let mut all_data = Vec::new();

    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                all_data.extend_from_slice(&buf[..n]);
                println!(
                    "Read {} bytes (total: {}): {:?}",
                    n,
                    reader.bytes_read(),
                    String::from_utf8_lossy(&buf[..n])
                );
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
    println!("Total bytes read: {}\n", reader.bytes_read());

    println!("=== PROGRESS READER ===\n");

    let data = b"Loading configuration data from remote source. \
                 This simulates a large download with progress tracking. \
                 Each chunk reports how far we've gotten through the total.";
    let total = data.len() as u64;
    let source = Cursor::new(data.as_slice());

    // Wrap in ProgressReader
    let mut reader = ProgressReader::new(source, total, "config-download");
    let mut output = String::new();
    // Read in 30-byte chunks to see progress updates
    let mut chunk_buf = [0u8; 30];
    loop {
        match reader.read(&mut chunk_buf) {
            Ok(0) => break,
            Ok(n) => {
                output.push_str(&String::from_utf8_lossy(&chunk_buf[..n]));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
    println!("Read complete: {} bytes\n", reader.bytes_read());

    println!("=== COMPOSED READ CHAIN ===\n");

    // Chain decorators: CountingReader wrapping a Take adapter
    // Read at most 25 bytes from the source, and count what we read
    let data = b"First part of the stream. Second part that won't be read.";
    let source = Cursor::new(data.as_slice());

    // Compose: Take (limit to 25 bytes) -> CountingReader
    let limited = source.take(25);
    let mut counted = CountingReader::new(limited);

    let mut result = String::new();
    counted.read_to_string(&mut result).unwrap();
    println!("Limited + counted read: {:?}", result);
    println!("Bytes counted: {}\n", counted.bytes_read());

    println!("=== LOGGING WRITER ===\n");

    let mut output_buf: Vec<u8> = Vec::new();

    {
        let mut writer = LoggingWriter::new(&mut output_buf, "response");

        writer.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
        writer.write_all(b"Content-Type: application/json\r\n").unwrap();
        writer.write_all(b"\r\n").unwrap();
        writer.write_all(b"{\"status\": \"ok\", \"data\": [1, 2, 3]}").unwrap();
        writer.flush().unwrap();

        println!("Total bytes written: {}", writer.total_written());
    }

    println!(
        "\nFinal output buffer ({} bytes):\n{}",
        output_buf.len(),
        String::from_utf8_lossy(&output_buf)
    );

    println!("\n=== COMPOSED WRITE CHAIN ===\n");

    // Chain: UppercaseWriter -> LoggingWriter -> Vec<u8>
    let mut output_buf: Vec<u8> = Vec::new();

    {
        let logger = LoggingWriter::new(&mut output_buf, "pipeline");
        let mut upper = UppercaseWriter::new(logger);

        upper.write_all(b"hello world from the decorator chain").unwrap();
        upper.flush().unwrap();
    }

    println!(
        "\nFinal output: {:?}",
        String::from_utf8_lossy(&output_buf)
    );

    println!("\n=== BY_REF: BORROW INSTEAD OF CONSUME ===\n");

    // Demonstrate by_ref() to avoid consuming the reader
    let data = b"HEADER:config-v2\nkey1=value1\nkey2=value2\nkey3=value3";
    let mut source = Cursor::new(data.as_slice());

    // Read just the first 16 bytes (the header) without consuming source
    {
        let mut header_buf = [0u8; 16];
        source.by_ref().take(16).read_exact(&mut header_buf).unwrap();
        println!("Header: {:?}", String::from_utf8_lossy(&header_buf));
    }

    // source is still usable -- read the rest
    let mut rest = String::new();
    source.read_to_string(&mut rest).unwrap();
    println!("Body: {:?}", rest);
}
