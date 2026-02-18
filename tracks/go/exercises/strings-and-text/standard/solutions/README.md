# Solution: Log Line Parser

## Approach

Parse in stages, each with a single responsibility:

1. **Timestamp extraction** — regexp anchored at `^` to find the structured timestamp at line start
2. **Token extraction** — `strings.Fields` to split the remainder (handles any whitespace)
3. **Level validation** — map lookup against the four known levels
4. **Service extraction** — `strings.HasSuffix(token, ":")` check, then `strings.TrimSuffix`
5. **Message/pairs split** — find where the first `key=value` match starts; everything before is the message
6. **Pair extraction** — `FindAllStringSubmatch` to capture all key=value pairs with quote handling

## Key Decisions

**Why regexp for the timestamp?** The timestamp has a precise structure that can be validated while extracting. `strings.Fields` alone would give you the token but not validate the format. The regexp also handles both `Z` and `±HH:MM` timezone formats in one pattern.

**Why `strings.Fields` instead of `strings.Split`?** Log lines from real services often have inconsistent spacing (look at the `INFO ` with two spaces in the test data). `Fields` splits on any run of whitespace and discards empty strings, giving clean tokens regardless of whitespace.

**Why compile regexps at package level?** `regexp.Compile` is expensive — it builds a finite automaton. Compiling inside `ParseLogLine` would pay that cost on every call. Package-level variables are compiled once at program start and reused for free.

**Why `map[Level]bool` for level validation?** A switch statement would work too. The map makes it easy to add new levels without modifying the validation logic — open/closed principle at small scale.

**Why collect parse errors instead of returning them?** A real log pipeline processes millions of lines. One malformed line shouldn't stop processing. Collecting errors lets the caller decide what to do: log them, alert if the error rate is high, or discard them.

## Performance

This parser allocates per call:
- `reTimestamp.FindString` allocates a substring
- `strings.Fields` allocates a `[]string`
- `make(map[string]string)` for Pairs
- `FindAllStringSubmatch` allocates for each match

For a high-throughput use case (100k+ lines/sec), you'd profile and possibly:
- Accept a `*LogEntry` to fill in-place (avoid the return copy)
- Use a pool of maps for Pairs
- Pre-allocate the []LogEntry in BatchResult

For typical use (dashboards, alerting, CI logs), this is more than fast enough.

## Variants

See `variants/` for alternative implementations:

| Variant | Approach | Trade-off |
|---|---|---|
| `streaming.go` | Returns entries via channel | Lower peak memory for huge batches |
| `indexed.go` | Pre-builds service→entries index | O(1) service lookups at query time |
