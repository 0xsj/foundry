// Append-Only Key-Value Store — Reference Solution (Rust)
//
// Run tests: rustc --test main.rs && ./main

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Write};
use std::path::Path;

// ---------- Data types ----------

pub struct LogEntry {
    pub key: String,
    pub value: Option<String>,
}

// ---------- Serialization ----------

pub fn encode_entry(entry: &LogEntry) -> String {
    match &entry.value {
        Some(v) => format!("SET {} {}\n", entry.key, v),
        None    => format!("DEL {}\n", entry.key),
    }
}

pub fn decode_entry(line: &str) -> Option<LogEntry> {
    // splitn(3, ' ') gives at most 3 parts: ["SET", "key", "rest of value"]
    // This means values can contain spaces — only the first two spaces are split boundaries.
    let mut parts = line.splitn(3, ' ');

    match parts.next()? {
        "SET" => {
            let key = parts.next()?.to_string();
            let value = parts.next()?.to_string();
            Some(LogEntry { key, value: Some(value) })
        }
        "DEL" => {
            let key = parts.next()?.to_string();
            // DEL must have exactly one remaining token — the key
            if parts.next().is_some() {
                return None; // unexpected extra token
            }
            Some(LogEntry { key, value: None })
        }
        _ => None, // unrecognized prefix
    }
}

// ---------- LogStore ----------

pub struct LogStore<W: Write> {
    writer: W,
}

impl<W: Write> LogStore<W> {
    pub fn new(writer: W) -> Self {
        LogStore { writer }
    }

    pub fn set(&mut self, key: &str, value: &str) -> io::Result<()> {
        let entry = LogEntry {
            key: key.to_string(),
            value: Some(value.to_string()),
        };
        self.writer.write_all(encode_entry(&entry).as_bytes())
    }

    pub fn delete(&mut self, key: &str) -> io::Result<()> {
        let entry = LogEntry {
            key: key.to_string(),
            value: None,
        };
        self.writer.write_all(encode_entry(&entry).as_bytes())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

// ---------- Reading ----------

pub fn read_log(reader: impl BufRead) -> io::Result<HashMap<String, String>> {
    let mut state: HashMap<String, String> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        // Silently skip malformed lines — partial writes during crashes are possible
        if let Some(entry) = decode_entry(&line) {
            match entry.value {
                Some(v) => { state.insert(entry.key, v); }
                None    => { state.remove(&entry.key); }
            }
        }
    }

    Ok(state)
}

// ---------- Compaction ----------

pub fn compact(input: impl BufRead, output: &mut impl Write) -> io::Result<usize> {
    // Read current state — this scans the entire log
    let state = read_log(input)?;

    // Produce deterministic output by sorting keys
    let mut keys: Vec<&String> = state.keys().collect();
    keys.sort();

    let mut count = 0;
    for key in keys {
        let entry = LogEntry {
            key: key.clone(),
            value: Some(state[key].clone()),
        };
        output.write_all(encode_entry(&entry).as_bytes())?;
        count += 1;
    }

    Ok(count)
}

// ---------- Production entry point ----------

pub fn open_log(path: impl AsRef<Path>) -> io::Result<LogStore<BufWriter<File>>> {
    let file = OpenOptions::new()
        .create(true)   // create if it doesn't exist
        .append(true)   // every write goes to end of file — never truncates
        .open(path)?;

    // Wrap in BufWriter: accumulate writes in memory, flush in 8 KB chunks
    Ok(LogStore::new(BufWriter::new(file)))
}

// ---------- Tests (identical to starter) ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_set() {
        let entry = LogEntry {
            key: "user:1001".to_string(),
            value: Some("alice".to_string()),
        };
        let line = encode_entry(&entry);
        assert!(line.ends_with('\n'));
        let decoded = decode_entry(line.trim()).unwrap();
        assert_eq!(decoded.key, "user:1001");
        assert_eq!(decoded.value.as_deref(), Some("alice"));
    }

    #[test]
    fn test_encode_decode_del() {
        let entry = LogEntry {
            key: "user:1002".to_string(),
            value: None,
        };
        let line = encode_entry(&entry);
        let decoded = decode_entry(line.trim()).unwrap();
        assert_eq!(decoded.key, "user:1002");
        assert!(decoded.value.is_none());
    }

    #[test]
    fn test_decode_malformed_returns_none() {
        assert!(decode_entry("").is_none());
        assert!(decode_entry("UNKNOWN key value").is_none());
        assert!(decode_entry("SET").is_none());
        assert!(decode_entry("SET key").is_none());
        assert!(decode_entry("DEL").is_none());
    }

    #[test]
    fn test_decode_value_with_spaces() {
        let decoded = decode_entry("SET flag:1 feature enabled for users").unwrap();
        assert_eq!(decoded.key, "flag:1");
        assert_eq!(decoded.value.as_deref(), Some("feature enabled for users"));
    }

    #[test]
    fn test_logstore_set_produces_lines() {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut store = LogStore::new(&mut buf);
            store.set("k1", "v1").unwrap();
            store.set("k2", "v2").unwrap();
            store.flush().unwrap();
        }
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains("k1"));
        assert!(output.contains("v1"));
        assert!(output.contains("k2"));
        assert!(output.contains("v2"));
    }

    #[test]
    fn test_logstore_delete_produces_del_line() {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut store = LogStore::new(&mut buf);
            store.set("k1", "v1").unwrap();
            store.delete("k1").unwrap();
            store.flush().unwrap();
        }
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains("DEL"));
        assert!(output.contains("k1"));
    }

    #[test]
    fn test_read_log_basic_state() {
        let log = b"SET user:1 alice\nSET user:2 bob\nSET user:3 carol\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert_eq!(state.get("user:1").map(String::as_str), Some("alice"));
        assert_eq!(state.get("user:2").map(String::as_str), Some("bob"));
        assert_eq!(state.get("user:3").map(String::as_str), Some("carol"));
    }

    #[test]
    fn test_read_log_later_write_wins() {
        let log = b"SET flag:dark_mode false\nSET flag:dark_mode true\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert_eq!(state.get("flag:dark_mode").map(String::as_str), Some("true"));
    }

    #[test]
    fn test_read_log_del_removes_key() {
        let log = b"SET user:1 alice\nDEL user:1\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert!(!state.contains_key("user:1"));
    }

    #[test]
    fn test_read_log_del_then_set() {
        let log = b"SET user:1 alice\nDEL user:1\nSET user:1 alice-reborn\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert_eq!(state.get("user:1").map(String::as_str), Some("alice-reborn"));
    }

    #[test]
    fn test_read_log_skips_malformed() {
        let log = b"SET user:1 alice\nBADLINE\nSET user:2 bob\n";
        let reader = BufReader::new(Cursor::new(log));
        let state = read_log(reader).unwrap();
        assert_eq!(state.len(), 2);
    }

    #[test]
    fn test_read_log_empty() {
        let reader = BufReader::new(Cursor::new(b""));
        let state = read_log(reader).unwrap();
        assert!(state.is_empty());
    }

    #[test]
    fn test_compact_deduplicates() {
        let log = b"SET k1 v1\nSET k2 v2\nSET k1 v1-updated\n";
        let reader = BufReader::new(Cursor::new(log));
        let mut output = Cursor::new(Vec::new());
        let count = compact(reader, &mut output).unwrap();
        assert_eq!(count, 2);
        let out_str = String::from_utf8(output.into_inner()).unwrap();
        assert_eq!(out_str.matches("k1").count(), 1);
        assert!(out_str.contains("v1-updated"));
    }

    #[test]
    fn test_compact_removes_deleted_keys() {
        let log = b"SET k1 v1\nSET k2 v2\nDEL k1\n";
        let reader = BufReader::new(Cursor::new(log));
        let mut output = Cursor::new(Vec::new());
        let count = compact(reader, &mut output).unwrap();
        assert_eq!(count, 1);
        let out_str = String::from_utf8(output.into_inner()).unwrap();
        assert!(!out_str.contains("k1"));
        assert!(out_str.contains("k2"));
    }

    #[test]
    fn test_compact_output_is_sorted() {
        let log = b"SET zebra last\nSET alpha first\nSET middle mid\n";
        let reader = BufReader::new(Cursor::new(log));
        let mut output = Cursor::new(Vec::new());
        compact(reader, &mut output).unwrap();
        let out_str = String::from_utf8(output.into_inner()).unwrap();
        let lines: Vec<&str> = out_str.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("alpha"));
        assert!(lines[1].contains("middle"));
        assert!(lines[2].contains("zebra"));
    }

    #[test]
    fn test_write_then_read_roundtrip() {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut store = LogStore::new(&mut buf);
            store.set("config:timeout", "30s").unwrap();
            store.set("config:retries", "3").unwrap();
            store.set("config:timeout", "60s").unwrap();
            store.delete("config:retries").unwrap();
            store.set("config:debug", "false").unwrap();
            store.flush().unwrap();
        }
        buf.set_position(0);
        let state = read_log(BufReader::new(buf)).unwrap();
        assert_eq!(state.get("config:timeout").map(String::as_str), Some("60s"));
        assert!(!state.contains_key("config:retries"));
        assert_eq!(state.get("config:debug").map(String::as_str), Some("false"));
        assert_eq!(state.len(), 2);
    }
}

fn main() {
    println!("Append-Only Key-Value Store — Reference Solution");
    println!("Run: rustc --test main.rs && ./main");
}
