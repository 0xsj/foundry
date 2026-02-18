# Exercise: API Response Serializer

## Scenario

Your team maintains a public REST API that is undergoing a versioning migration. V1 clients receive a flat response shape; V2 clients receive a richer shape with embedded metadata and nested resource links. Both versions share the same core data — a `Product` — but the serialized JSON differs. The same endpoint must also support CSV export for BI tooling that ingests reports via bulk download.

You are building the serialization layer: a `Serializer` that accepts a list of products, a version, and a format, and produces the correctly shaped output.

## Brief

Implement a product API response serializer with the following types and behaviors:

1. **`Product`** — core domain type: `ID string`, `Name string`, `PriceCents int64`, `Stock int`, `Tags []string`, `CreatedAt time.Time`, `DeletedAt *time.Time` (nullable)

2. **`V1Response`** — flat JSON shape for V1 clients:
   - Fields: `id`, `name`, `price_cents`, `stock`, `tags`, `created_at` (Unix timestamp, not RFC3339)
   - `DeletedAt` does not appear in V1

3. **`V2Response`** — richer JSON shape for V2 clients:
   - Fields: `id`, `name`, `price_cents`, `stock`, `tags`, `created_at` (RFC3339), `deleted_at` (RFC3339, omitted if nil)
   - Embedded `Meta` struct with `api_version` (always `"v2"`) and `links` object containing `self` and `collection` URL strings
   - `Meta` fields appear at the top level (embedded, not nested under a `"meta"` key)

4. **`UnixTime`** — a `time.Time` wrapper that marshals/unmarshals as a Unix integer (used by V1)

5. **`Serializer`** — the main type:
   - `NewSerializer(baseURL string) *Serializer`
   - `SerializeJSON(products []Product, version string) ([]byte, error)` — returns pretty-printed JSON array
   - `SerializeCSV(products []Product) ([]byte, error)` — CSV with header row

6. **CSV columns** (in order): `id`, `name`, `price_cents`, `stock`, `tags` (semicolon-joined), `created_at` (RFC3339), `deleted_at` (RFC3339 or empty string)

### Shape Examples

**V1 JSON** for a single product:
```json
{
  "id": "prod_001",
  "name": "Widget Pro",
  "price_cents": 4999,
  "stock": 100,
  "tags": ["hardware", "featured"],
  "created_at": 1708000000
}
```

**V2 JSON** for the same product:
```json
{
  "api_version": "v2",
  "links": {
    "self": "https://api.example.com/products/prod_001",
    "collection": "https://api.example.com/products"
  },
  "id": "prod_001",
  "name": "Widget Pro",
  "price_cents": 4999,
  "stock": 100,
  "tags": ["hardware", "featured"],
  "created_at": "2026-02-18T10:00:00Z"
}
```

Note that `api_version` and `links` appear at the top level — they are promoted from an embedded struct, not nested under a key.

## Acceptance Criteria

- [ ] `UnixTime` implements `json.Marshaler` and `json.Unmarshaler`; marshals as an integer, not a string
- [ ] `V1Response` uses `UnixTime` for `created_at`; `DeletedAt` is never present in V1 JSON
- [ ] `V2Response` embeds a `Meta` struct whose fields (`api_version`, `links`) are promoted to the top level
- [ ] `V2Response.DeletedAt` is `*time.Time` and uses `omitempty` — omitted when nil, RFC3339 when set
- [ ] `SerializeJSON("v1")` returns a JSON array of `V1Response` objects, pretty-printed (2-space indent)
- [ ] `SerializeJSON("v2")` returns a JSON array of `V2Response` objects, pretty-printed (2-space indent)
- [ ] `SerializeJSON` returns an error for unknown version strings
- [ ] `SerializeCSV` returns a byte slice with a header row followed by one data row per product
- [ ] CSV `tags` field joins the slice with `;` as separator
- [ ] CSV `deleted_at` is an empty string when `nil`, RFC3339 string when set
- [ ] All tests in `starter/main_test.go` pass

## Constraints

- Standard library only — no third-party packages
- `Serializer` must be initialized with `NewSerializer` — do not use package-level state
- `UnixTime` must use a custom `MarshalJSON` / `UnmarshalJSON` — do not rely on `time.Time`'s default behavior
- V2 `Meta` embedded struct must use embedding (not a named field) so its fields appear at the top JSON level

## Concepts Exercised

- Struct tags: `json:"name"`, `omitempty`, `json:"-"`
- Custom `MarshalJSON` / `UnmarshalJSON` using shadow structs
- Embedded structs and JSON field promotion
- Pointer fields for nullable values (`*time.Time`)
- `json.MarshalIndent` for pretty-printing
- `encoding/csv` with manual column mapping and `Flush()`
- Version-specific response shaping without mutating the domain type

## Hints

<details>
<summary>Hint 1: UnixTime implementation</summary>

Use a shadow struct inside MarshalJSON to avoid infinite recursion:
```go
func (u UnixTime) MarshalJSON() ([]byte, error) {
    return json.Marshal(u.Unix()) // json.Marshal an int64
}

func (u *UnixTime) UnmarshalJSON(data []byte) error {
    var ts int64
    if err := json.Unmarshal(data, &ts); err != nil {
        return err
    }
    u.Time = time.Unix(ts, 0).UTC()
    return nil
}
```
</details>

<details>
<summary>Hint 2: Embedding for promoted JSON fields</summary>

To get `api_version` and `links` at the top level of V2 JSON, embed the `Meta` struct without a field name:
```go
type V2Response struct {
    Meta                      // embedded — fields promoted
    ID    string `json:"id"`
    // ...
}
```
If you write `Meta Meta \`json:"meta"\``, it creates a nested `"meta"` key — not what V2 requires.
</details>

<details>
<summary>Hint 3: CSV tags field</summary>

`strings.Join(product.Tags, ";")` produces the semicolon-joined string.
Use `strconv.FormatInt(product.PriceCents, 10)` to convert the int64 to a string.
</details>

<details>
<summary>Hint 4: Nullable deleted_at in CSV</summary>

```go
deletedAt := ""
if product.DeletedAt != nil {
    deletedAt = product.DeletedAt.UTC().Format(time.RFC3339)
}
```
</details>
