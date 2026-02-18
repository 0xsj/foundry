# Standard Exercise: Append-Only Key-Value Store

## Scenario

You're building the storage engine for a lightweight feature flag service. The store uses an
**append-only log** on disk: every write appends a new entry, every read scans the log to find
the latest value for a key. This design is simple to implement correctly, crash-safe (no partial
writes that corrupt existing data), and the backing pattern for systems like Kafka, RocksDB WAL,
and Redis AOF.

The storage layer must be abstracted over a `Write` trait so it can be tested without touching
the filesystem (using `Cursor<Vec<u8>>`), and the reader must use buffered I/O to handle large
logs efficiently.

## Brief

Implement a `LogStore` that writes key-value entries to an append-only log, and a `LogReader`
that scans a log and returns the latest value for a given key. Add a `compact` function that
reads an existing log and produces a deduplicated version (latest value wins per key).

## Acceptance Criteria

### 1. `LogEntry` struct

```rust
pub struct LogEntry {
    pub key: String,
    pub value: Option<String>,  // None = tombstone (deletion)
}
```

### 2. `encode_entry` and `decode_entry` (serialization)

- `encode_entry(entry: &LogEntry) -> String`
  - Encodes as a single line: `SET key value\n` for inserts, `DEL key\n` for tombstones
  - The key and value must not contain newlines (assume valid input)
- `decode_entry(line: &str) -> Option<LogEntry>`
  - Parses a single line back into a `LogEntry`
  - Returns `None` for malformed lines (graceful — don't panic)

### 3. `LogStore<W: Write>` struct

- Wraps a `W: Write` — the write target (file, Vec<u8>, etc.)
- `LogStore::new(writer: W) -> LogStore<W>`
- `fn set(&mut self, key: &str, value: &str) -> io::Result<()>` — write SET entry
- `fn delete(&mut self, key: &str) -> io::Result<()>` — write DEL entry
- `fn flush(&mut self) -> io::Result<()>` — flush underlying writer

### 4. `LogReader` function

- `fn read_log(reader: impl BufRead) -> io::Result<HashMap<String, String>>`
- Scans the entire log, applying entries in order (later entries override earlier)
- Tombstone (`DEL`) entries remove a key from the result
- Returns the final state as a `HashMap<String, String>`

### 5. `compact` function

- `fn compact(input: impl BufRead, output: &mut impl Write) -> io::Result<usize>`
- Reads the full log using `read_log`, writes only the final live entries to `output`
- Returns the number of entries written (not lines read)
- Entries in the output should be in sorted key order (deterministic output)

### 6. `LogStore` with `BufWriter` in production

- `fn open_log(path: impl AsRef<Path>) -> io::Result<LogStore<BufWriter<File>>>`
  - Opens (or creates) a log file in append mode
  - Wraps it in `BufWriter` for efficient writes
  - Returns a `LogStore` ready to use

## Constraints

- No external crates — stdlib only
- All provided tests must pass
- `LogStore` must be generic over `W: Write` — not hardcoded to `File`
- `read_log` must accept `impl BufRead` — not hardcoded to `File`
- `compact` output must be in sorted key order (makes tests deterministic)

## Hints

<details>
<summary>Hint 1: Entry encoding format</summary>

Two formats, one per line:
```
SET user:1001 alice
DEL user:1002
```

For `encode_entry`: use `format!` with a match on `entry.value`.
For `decode_entry`: `line.splitn(3, ' ')` splits into at most 3 parts — handles values with spaces.

</details>

<details>
<summary>Hint 2: LogStore generic write</summary>

```rust
pub struct LogStore<W: Write> {
    writer: W,
}

impl<W: Write> LogStore<W> {
    pub fn new(writer: W) -> Self {
        LogStore { writer }
    }

    pub fn set(&mut self, key: &str, value: &str) -> io::Result<()> {
        let entry = LogEntry { key: key.to_string(), value: Some(value.to_string()) };
        let line = encode_entry(&entry);
        self.writer.write_all(line.as_bytes())
    }
}
```

</details>

<details>
<summary>Hint 3: read_log with HashMap</summary>

```rust
pub fn read_log(reader: impl BufRead) -> io::Result<HashMap<String, String>> {
    let mut state: HashMap<String, String> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        if let Some(entry) = decode_entry(&line) {
            match entry.value {
                Some(v) => { state.insert(entry.key, v); }
                None    => { state.remove(&entry.key); }
            }
        }
    }
    Ok(state)
}
```

</details>

<details>
<summary>Hint 4: compact with sorted output</summary>

```rust
pub fn compact(input: impl BufRead, output: &mut impl Write) -> io::Result<usize> {
    let state = read_log(input)?;
    let mut keys: Vec<&String> = state.keys().collect();
    keys.sort();
    let mut count = 0;
    for key in keys {
        let entry = LogEntry { key: key.clone(), value: Some(state[key].clone()) };
        output.write_all(encode_entry(&entry).as_bytes())?;
        count += 1;
    }
    Ok(count)
}
```

</details>

<details>
<summary>Hint 5: open_log with BufWriter</summary>

```rust
use std::fs::{File, OpenOptions};
use std::io::BufWriter;

pub fn open_log(path: impl AsRef<Path>) -> io::Result<LogStore<BufWriter<File>>> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    Ok(LogStore::new(BufWriter::new(file)))
}
```

</details>
