# Variants: API Response Serializer

## stream.go — Streaming with io.Writer

Instead of returning `[]byte`, `SerializeJSONStream` writes directly to an `io.Writer`. This avoids allocating a large byte slice in memory, which matters when:
- Streaming to an HTTP response writer directly (no buffering needed)
- Handling very large product catalogs
- Writing to a file in one pass

```go
func (s *Serializer) SerializeJSONStream(products []Product, version string, w io.Writer) error {
    enc := json.NewEncoder(w)
    enc.SetIndent("", "  ")
    switch version {
    case "v1":
        responses := make([]V1Response, len(products))
        for i, p := range products {
            responses[i] = NewV1Response(p)
        }
        return enc.Encode(responses)
    case "v2":
        // ...
    }
}
```

**Trade-off:** The `[]byte` API is simpler for tests (compare bytes directly) and for cases where you need to inspect the output before writing it. The `io.Writer` API is better for large payloads and HTTP handlers. Many real APIs offer both — `Marshal` (returns bytes) and `Encode` (writes to writer).

## flatten.go — Custom MarshalJSON Instead of Separate Response Type

Instead of `V1Response` as a separate struct, implement `MarshalJSON` directly on `Product` and use a build tag or option to select the version. This approach:

- Keeps the domain type and its JSON serialization co-located
- Avoids the allocation of a new slice of response types
- But tightly couples the domain type to a specific serialization format

This is generally the wrong trade-off for APIs with multiple versions. The separate response type approach (reference solution) is more maintainable because each response type owns its own serialization contract independently.

**Rule of thumb:** Domain types should not implement format-specific `MarshalJSON` unless there is only ever one valid serialization of the type. If you need "the same data, but V1 uses Unix timestamps and V2 uses RFC3339", use separate response types.

## Comparison

| Approach | Pros | Cons |
|----------|------|------|
| Reference (separate structs) | Clear version boundary, testable independently, clean | Allocation per version conversion |
| Streaming (io.Writer) | No intermediate allocation, HTTP-friendly | Can't inspect result before writing |
| Custom MarshalJSON on domain | Less code, co-located | Couples domain to format, hard to version |
