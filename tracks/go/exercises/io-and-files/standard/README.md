# Exercise: Streaming Log Processor

## Scenario

Your team ingests structured log files from a fleet of microservices. Each service writes one JSON log entry per line. The files can be hundreds of megabytes — too large to load into memory at once. You need a command-line tool that reads these log files **line by line**, filters entries by severity and time range, optionally transforms them (redacts PII fields), and writes matching entries to an output file while simultaneously counting matches for a summary report.

This is a production-pattern exercise: streaming I/O, Reader/Writer composition, proper resource cleanup, and real error handling.

## Brief

Implement a `LogProcessor` that:

1. Reads log entries from any `io.Reader` (a file, stdin, a test fixture)
2. Filters entries by minimum severity level and optional time range
3. Redacts a configurable list of field names from matching entries
4. Writes matching (and possibly redacted) entries to an `io.Writer`
5. Returns a `ProcessSummary` with counts of read, matched, and written entries

## Acceptance Criteria

- [ ] `LogProcessor` reads entries one at a time — never loads the entire file into memory
- [ ] `ProcessSummary` returned contains: `TotalRead`, `Matched`, `Written`, `Errors` counts
- [ ] Entries are filtered by `MinLevel` — levels are `DEBUG < INFO < WARN < ERROR`
- [ ] Entries with a `"level"` field below `MinLevel` are skipped (not written)
- [ ] If `RedactFields` is non-empty, matching entries have those fields replaced with `"[REDACTED]"` before writing
- [ ] Each written entry is followed by a newline
- [ ] `scanner.Err()` is always checked after the scan loop
- [ ] All opened files are closed (use `defer f.Close()`)
- [ ] `NewLogProcessor` and `Process` are the public API — see types below

## Types Provided (do not change)

```go
// LogEntry represents a single parsed log line.
type LogEntry struct {
    Level     string            // "DEBUG", "INFO", "WARN", "ERROR"
    Message   string
    Timestamp time.Time
    Fields    map[string]string // additional key-value pairs
    Raw       string            // original line (used if parse fails)
}

// ProcessConfig controls filtering and transformation behavior.
type ProcessConfig struct {
    MinLevel     string   // minimum level to include: "DEBUG", "INFO", "WARN", "ERROR"
    RedactFields []string // field names to redact in matched entries
}

// ProcessSummary holds counts from a processing run.
type ProcessSummary struct {
    TotalRead int
    Matched   int
    Written   int
    Errors    int // lines that failed to parse (still counted in TotalRead)
}
```

## API to Implement

```go
// NewLogProcessor creates a LogProcessor with the given config.
func NewLogProcessor(cfg ProcessConfig) *LogProcessor

// Process reads all entries from r, applies filters and transforms,
// and writes matching entries to w.
// Returns a summary of the run and any terminal error (not parse errors).
func (p *LogProcessor) Process(r io.Reader, w io.Writer) (ProcessSummary, error)

// ProcessFile is a convenience wrapper that opens the file at path,
// processes it, and writes results to w.
func (p *LogProcessor) ProcessFile(path string, w io.Writer) (ProcessSummary, error)
```

## Log Line Format

Each log line is JSON:

```json
{"level":"INFO","message":"request handled","latency_ms":"45","user_id":"u-8821","path":"/api/orders"}
{"level":"ERROR","message":"database timeout","latency_ms":"5001","user_id":"u-4401","path":"/api/users"}
{"level":"DEBUG","message":"cache miss","key":"order:8821"}
```

Fields guaranteed present: `level`, `message`. All other fields are optional.

Timestamp, if present, uses the field name `"ts"` in RFC3339 format.

Lines that are not valid JSON should be counted as errors and skipped (not written).

## Constraints

- Standard library only: `bufio`, `encoding/json`, `os`, `io`, `strings`
- `Process` must work on any `io.Reader` — not just `*os.File`
- If `MinLevel` is empty, treat it as `"DEBUG"` (pass everything)
- Redaction applies **after** filtering — redact fields in entries that pass the level filter
- Invalid log lines count toward `TotalRead` and `Errors`, but not `Matched` or `Written`

## Concepts Exercised

- `bufio.Scanner` for line-by-line streaming
- `io.Reader` / `io.Writer` as function parameters (decoupling from files)
- `io.TeeReader` for simultaneously reading and counting (optional extension)
- `defer f.Close()` in `ProcessFile`
- `scanner.Err()` pattern
- `encoding/json` for parsing and marshaling
- Proper error wrapping with `%w`

## Hints

<details>
<summary>Hint 1: Log level ordering</summary>

Map levels to integers for comparison:

```go
var levelOrder = map[string]int{
    "DEBUG": 0,
    "INFO":  1,
    "WARN":  2,
    "ERROR": 3,
}
```

An entry passes the filter if `levelOrder[entry.Level] >= levelOrder[p.cfg.MinLevel]`.
</details>

<details>
<summary>Hint 2: Parsing a log line</summary>

Use `encoding/json.Unmarshal` into a `map[string]string` first — it's flexible for unknown fields:

```go
var raw map[string]string
if err := json.Unmarshal([]byte(line), &raw); err != nil {
    summary.Errors++
    continue
}
entry := LogEntry{
    Level:   raw["level"],
    Message: raw["message"],
    Fields:  raw,
    Raw:     line,
}
delete(entry.Fields, "level")    // keep Fields as extra fields only
delete(entry.Fields, "message")
```
</details>

<details>
<summary>Hint 3: Writing entries back out</summary>

`encoding/json.Marshal` produces a flat JSON object. Use `json.NewEncoder(w)` to write directly to the writer:

```go
enc := json.NewEncoder(w)
// json.Encoder.Encode appends a newline automatically
if err := enc.Encode(entry); err != nil {
    return summary, fmt.Errorf("write entry: %w", err)
}
summary.Written++
```
</details>

<details>
<summary>Hint 4: ProcessFile cleanup pattern</summary>

```go
func (p *LogProcessor) ProcessFile(path string, w io.Writer) (ProcessSummary, error) {
    f, err := os.Open(path)
    if err != nil {
        return ProcessSummary{}, fmt.Errorf("open %s: %w", path, err)
    }
    defer f.Close()
    return p.Process(f, w)
}
```
</details>
