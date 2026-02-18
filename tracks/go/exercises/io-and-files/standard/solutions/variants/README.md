# Solution Variants

## Reference vs Variants

| Variant | Approach | Best When |
|---------|----------|-----------|
| `solution.go` | Single-pass scan, filter, write | Default — simple, readable, efficient |
| `tee.go` (described below) | TeeReader to count bytes while writing | You need both output and metrics in one pass |
| Pipeline (goroutines) | Parse/filter/write as separate goroutines | High-throughput, CPU-bound transformation |

## TeeReader Variant (conceptual)

Instead of writing directly to `w`, wrap `w` in a `countingWriter` and connect it to the scanner's source using `io.TeeReader`. This lets you report total bytes written alongside the entry count — in a single pass, without storing anything extra.

```go
// TeeReader variant sketch
var bytesRead int64
teeSource := io.TeeReader(r, &countingWriter{&bytesRead})

scanner := bufio.NewScanner(teeSource)
// ... same loop ...
// At the end: bytesRead holds total input bytes consumed
```

Useful for: throughput metrics, progress bars on large files, audit logging of exact bytes processed.

## Pipeline Variant (conceptual)

Parse, filter, and write as three concurrent stages:

```go
// Stage 1: parse lines → entries channel
// Stage 2: filter entries → matched channel
// Stage 3: write matched entries to w

lines := make(chan string, 100)
entries := make(chan *LogEntry, 100)
matched := make(chan *LogEntry, 100)

go scanner(r, lines)
go parser(lines, entries, &summary)
go filter(entries, matched, cfg)
writer(matched, w, &summary)
```

Trade-off: more complex orchestration, but the three stages run in parallel — useful when parsing (JSON decode) is expensive and the pipeline is I/O + CPU bound.

For typical log processing, the single-pass approach in `solution.go` is faster due to lower goroutine overhead. Reach for the pipeline pattern when benchmarks show parsing is the bottleneck.
