# Exercise: Log Line Parser

## Scenario

Your platform ingests structured log output from a dozen microservices, but the format is not perfectly consistent. Each line has a timestamp prefix and a severity level, followed by a free-form message that may include `key=value` pairs and quoted strings. You need a parser that extracts the structured fields reliably so a downstream system can index and alert on them.

## Brief

Implement a `ParseLogLine` function and supporting types that extract structured data from log lines. Then implement a `ParseLogBatch` function that processes a slice of lines and aggregates statistics.

## Log Line Format

```
2024-01-15T14:30:00Z ERROR payment-svc: connection refused host="db.prod" retries=3 duration_ms=1504
2024-01-15T14:30:01Z INFO  auth-svc: user login user="alice@example.com" region=us-east latency_ms=42
2024-01-15T14:30:02Z WARN  inventory: rate limit exceeded limit=1000 current=1043
```

Fields:
- **Timestamp**: RFC3339 format at the start of the line (required)
- **Level**: `ERROR`, `WARN`, `INFO`, `DEBUG` — second token (required)
- **Service**: service name — third token (required, ends with `:`)
- **Message**: human-readable text — everything after the service until the first `key=value` pair
- **Pairs**: zero or more `key=value` or `key="quoted value"` pairs (optional)

## Acceptance Criteria

- [ ] `ParseLogLine(line string) (LogEntry, error)` — parse a single log line
  - Returns error if timestamp is missing or malformed
  - Returns error if level is not one of DEBUG/INFO/WARN/ERROR
  - Returns error if service token is missing
  - All key=value pairs are extracted into `Entry.Pairs` map
  - Quoted values have their surrounding quotes removed
  - Numeric values in pairs remain as strings (the caller can parse if needed)
- [ ] `LogEntry` struct with: `Timestamp`, `Level`, `Service`, `Message string`, `Pairs map[string]string`
- [ ] `Level` type: string-backed, with `IsError()`, `IsWarn()` methods
- [ ] `ParseLogBatch(lines []string) (BatchResult, error)` — parse multiple lines
  - Skips blank lines silently
  - Returns a `BatchResult` with: `Entries []LogEntry`, `ErrorCount`, `WarnCount`, `ParseErrors []string`
  - Parsing errors for individual lines are collected in `ParseErrors` (not returned as a fatal error)
- [ ] `BatchResult.ServicesWithErrors() []string` — returns sorted list of service names that have at least one ERROR entry

## Constraints

- Standard library only
- Use `regexp` for key=value extraction — compile patterns at package level (not inside functions)
- Use `strings` for splitting the line into tokens
- Use `strconv` if you need to validate or convert numeric fields
- The parser must handle lines where the message is empty (only key=value pairs after service)
- Values with spaces must be quoted: `region=us-east` (no quotes needed), `message="something went wrong"` (quotes needed)

## Hints

<details>
<summary>Hint 1: Overall parsing strategy</summary>

Parse in stages:
1. Extract the timestamp using a regexp (it's at a known position and has a fixed format)
2. Use `strings.Fields` to tokenize the rest of the line
3. Walk the tokens: find level, find service (ends with `:`), collect the rest
4. Use a regexp to extract key=value pairs from the tail

```go
var reTimestamp = regexp.MustCompile(`^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:Z|[+-]\d{2}:\d{2})`)
var reKVPair = regexp.MustCompile(`(\w[\w.-]*)=("(?:[^"\\]|\\.)*"|\S+)`)
```
</details>

<details>
<summary>Hint 2: Extracting key=value pairs</summary>

`FindAllStringSubmatch` returns all matches with capture groups:

```go
matches := reKVPair.FindAllStringSubmatch(tail, -1)
for _, m := range matches {
    key := m[1]
    val := m[2]
    // Strip surrounding quotes from quoted values
    if len(val) >= 2 && val[0] == '"' && val[len(val)-1] == '"' {
        val = val[1 : len(val)-1]
    }
    pairs[key] = val
}
```
</details>

<details>
<summary>Hint 3: Extracting the message (text before key=value pairs)</summary>

After identifying the service token, find where key=value pairs begin and take everything before the first one as the message:

```go
// Find the position of the first key=value pair
loc := reKVPair.FindStringIndex(tail)
var message string
if loc != nil {
    message = strings.TrimSpace(tail[:loc[0]])
} else {
    message = strings.TrimSpace(tail)
}
```
</details>

<details>
<summary>Hint 4: ServicesWithErrors sort</summary>

```go
import "sort"

func (r *BatchResult) ServicesWithErrors() []string {
    seen := make(map[string]struct{})
    for _, e := range r.Entries {
        if e.Level.IsError() {
            seen[e.Service] = struct{}{}
        }
    }
    result := make([]string, 0, len(seen))
    for svc := range seen {
        result = append(result, svc)
    }
    sort.Strings(result)
    return result
}
```
</details>
