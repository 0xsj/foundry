# Solution: Append-Only Key-Value Store

## Approach

The solution rests on two design decisions that directly exercise this module's key concepts:

**Generic over `W: Write`:** `LogStore<W: Write>` accepts any writer — `BufWriter<File>` in
production, `Cursor<Vec<u8>>` in tests, `Vec<u8>` for debugging. This is the trait-based
abstraction pattern for I/O: write to a trait, not a concrete type. The result is a storage
engine that's fully testable without a filesystem.

**`BufRead` for reading:** `read_log(reader: impl BufRead)` and `compact(input: impl BufRead, ...)`
accept any buffered reader. The production path wraps `File` in `BufReader`; the test path passes
`BufReader<Cursor<...>>`. No filesystem access in any test.

## Key Decisions

**Why `splitn(3, ' ')` in `decode_entry`?**
`splitn(n, delim)` splits into at most `n` parts — it stops splitting after finding `n-1`
delimiters. With `splitn(3, ' ')`, the result is `["CMD", "key", "rest of value"]`. This means
values can contain spaces (e.g., `"feature enabled for users"`) without any quoting or escaping.
Using plain `split` would break on the first space within the value.

**Why explicit `flush()` in `LogStore` instead of relying on drop?**
`BufWriter` flushes on drop, but the error is silently discarded. For a storage engine where
data loss is a serious bug, explicit `flush()` is required. The API exposes `flush()` and
documentation (and tests) enforce calling it. The same pattern applies to `File::sync_all()`
for durability guarantees (not implemented here — that's a separate topic in the async I/O
or durability module).

**Why `Option<String>` for `value` instead of an enum?**
A dedicated `enum LogEntry { Set { key, value }, Delete { key } }` would be cleaner and
more type-safe. The `Option<String>` approach was chosen to keep the starter simple and focus
on the I/O concepts. In production code, the enum variant is preferred.

**Why `compact` returns the count?**
Callers often want to know the compaction ratio — how many live entries remain vs. how many
log entries were processed. Returning the count makes this calculable without re-reading the
output. It also makes tests cleaner: `assert_eq!(count, 2)` is more readable than counting
lines in the output.

## Variant Approaches

| Approach | Trade-off |
|---|---|
| `LogStore<W: Write>` (reference) | Generic, zero-cost, requires `where W: Write` |
| `LogStore { writer: Box<dyn Write> }` | Dynamic dispatch, heap allocation, simpler type signature |
| `LogStore { writer: BufWriter<File> }` | Simple, but not testable without filesystem |
| Enum `LogEntry { Set(...), Del(...) }` | More idiomatic, eliminates `Option` confusion |
| Binary encoding (length-prefix) | More efficient, handles arbitrary bytes, harder to debug |

## Connection to Bigger Picture

The append-only log pattern appears in:
- **Kafka** — every topic partition is an append-only log
- **RocksDB/LevelDB WAL** — write-ahead log for crash safety
- **Redis AOF** — append-only file mode for persistence
- **Event sourcing** — architectural pattern for deriving state from immutable events

The `compact` function demonstrates **log compaction**: instead of ever overwriting old entries,
you periodically create a new, smaller log with only the current state. This trades write
amplification (every update is an append) for simplicity and crash safety.

The `LogStore<W: Write>` pattern previews the **dependency injection** principle you'll see
formalized in the patterns module: instead of instantiating dependencies internally, accept
them from outside. This makes components independently testable.
