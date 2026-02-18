# Solution: API Response Serializer

## Approach

The solution separates concerns cleanly: the `Product` domain type never changes shape. Two response types — `V1Response` and `V2Response` — handle the version-specific serialization.

The key decisions:

1. **`UnixTime` custom marshaler** — wraps `time.Time` and overrides `MarshalJSON`/`UnmarshalJSON` to produce a JSON integer. The implementation is a two-liner: marshal the result of `.Unix()`, unmarshal into `int64` then convert with `time.Unix()`.

2. **V2 embedding for field promotion** — `Meta` is embedded (not a named field) inside `V2Response`. This promotes `api_version` and `links` to the top JSON level. Using a named field like `Meta Meta \`json:"meta"\`` would produce a nested object `{"meta": {...}}`, which is not what V2 requires.

3. **`*time.Time` for nullable DeletedAt** — V2 uses a pointer so that `omitempty` skips the field when nil and includes it (as RFC3339) when set. An `omitempty` on a `time.Time` value (non-pointer) would never omit — structs are never considered "empty" by `encoding/json`.

4. **`SerializeJSON` switch** — builds the appropriate slice type per version, then uses `json.MarshalIndent`. Error on unknown version is explicit rather than silent fallback.

5. **CSV Flush discipline** — `csv.Writer` buffers internally. The solution calls `w.Flush()` and checks `w.Error()` before returning. Forgetting this is the most common CSV bug.

## Key Decisions

| Decision | Why |
|----------|-----|
| Shadow struct in `MarshalJSON` | Would be needed if calling `json.Marshal(m)` inside the method — avoids infinite recursion. For `UnixTime` we marshal a plain `int64`, so no shadow struct needed. |
| `Meta` embedded (not named) | Named field creates nesting in JSON. Embedding promotes fields. V2 spec requires top-level `api_version` and `links`. |
| `*time.Time` with `omitempty` | `time.Time{}` is not a "zero" for omitempty (structs are never empty). Pointer is nil or non-nil, which is exactly the presence/absence distinction needed. |
| Error on unknown version | Explicit error instead of default/fallback prevents silent behavioral drift when new versions are added. |
| `bytes.Buffer` for CSV | Allows writing CSV to memory and returning `[]byte`. The CSV writer wraps it via `io.Writer`. |

## Variants

See `variants/` for:
- `stream.go` — `SerializeJSON` using `json.Encoder` (streaming to `io.Writer`) instead of `MarshalIndent`
- `flatten.go` — V1 using a custom `MarshalJSON` instead of a separate response struct (shows the trade-off)

## Complexity

- `SerializeJSON`: O(n) time and space — one pass over the product slice to build response slice, one pass for JSON encoding
- `SerializeCSV`: O(n) time and space — one pass with buffered I/O

Both are dominated by the encoding work, not the data transformation.
