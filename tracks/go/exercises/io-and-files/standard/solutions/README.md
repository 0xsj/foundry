# Solution: Streaming Log Processor

## Approach

The solution centers on one design decision: `Process(r io.Reader, w io.Writer)` instead of `ProcessFile(path string)`. By accepting interfaces rather than concrete file types, the same code works against a file, an in-memory buffer, stdin, an HTTP body, or a test fixture. `ProcessFile` is a thin convenience wrapper that opens the file and delegates to `Process`.

## Key Decisions

**bufio.Scanner for reading** — Scanner handles partial reads internally. You read lines with `scanner.Text()`, which strips the trailing newline. The scanner tracks its own buffer and refills it as needed — you never see the underlying `Read` calls.

**Always check scanner.Err()** — `scanner.Scan()` returns `false` for both EOF and mid-stream errors. Without checking `scanner.Err()` after the loop, you silently process incomplete data when there's an I/O failure. This is the most common bufio bug.

**json.NewEncoder(w)** — `Encode` writes the JSON object and appends `\n` in a single call. Cleaner than `Marshal` + `Write` + newline.

**Filter before redact** — No point redacting fields on an entry that won't be written. The `passesFilter` check comes before `redact`.

**parseLine returns the full Fields map** — Parsing into `map[string]string` handles any arbitrary fields without a rigid struct. `level` and `message` are pulled out as first-class fields; everything else lands in `Fields`.

## Variants

| Variant | File | Trade-off |
|---------|------|-----------|
| Reference | solution.go | Idiomatic bufio + json.Encoder |
| Streaming with TeeReader | variants/tee.go | Simultaneously writes to output and a byte counter — shows TeeReader composition |
| Channel pipeline | variants/pipeline.go | Parse/filter/write as separate goroutines with channels — shows concurrency at I/O layer |

## Performance Notes

- `bufio.Scanner` uses a 4KB default buffer, refilled as needed — constant memory use regardless of file size
- `json.Encoder` streams JSON without buffering the entire encoded form
- `ProcessFile` opens only one file descriptor and streams through it — no temp copies

For a 500MB log file, peak memory stays near 64KB (scanner buffer size). Contrast with `io.ReadAll` which would load 500MB into RAM.
