package serializer

import (
	"bytes"
	"encoding/csv"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"time"
)

// ============================================================================
// DOMAIN TYPE
// ============================================================================

// Product is an item in the product catalog.
type Product struct {
	ID          string
	Name        string
	PriceCents  int64
	Stock       int
	Tags        []string
	CreatedAt   time.Time
	DeletedAt   *time.Time
}

// ============================================================================
// UNIX TIME WRAPPER
// ============================================================================

// UnixTime is a time.Time that marshals to/from a JSON integer (Unix seconds).
// Used by V1 responses which use epoch timestamps instead of RFC3339.
type UnixTime struct {
	time.Time
}

// MarshalJSON encodes the time as a Unix integer.
// The shadow struct trick is not needed here — we're just marshaling an int64.
func (u UnixTime) MarshalJSON() ([]byte, error) {
	return json.Marshal(u.Unix())
}

// UnmarshalJSON decodes a Unix integer into UnixTime.
// Must use pointer receiver to modify the actual value.
func (u *UnixTime) UnmarshalJSON(data []byte) error {
	var ts int64
	if err := json.Unmarshal(data, &ts); err != nil {
		return err
	}
	u.Time = time.Unix(ts, 0).UTC()
	return nil
}

// ============================================================================
// V1 RESPONSE
// ============================================================================

// V1Response is the flat JSON shape returned to V1 API clients.
// Uses UnixTime for created_at (integer), no deleted_at field.
// json:"-" on DeletedAt would also work, but simply not having the field at
// all is cleaner — the field never existed, so nothing to tag.
type V1Response struct {
	ID         string   `json:"id"`
	Name       string   `json:"name"`
	PriceCents int64    `json:"price_cents"`
	Stock      int      `json:"stock"`
	Tags       []string `json:"tags"`
	CreatedAt  UnixTime `json:"created_at"` // UnixTime → integer
	// DeletedAt intentionally absent — no json:"-" needed when the field doesn't exist
}

// NewV1Response converts a Product into a V1Response.
func NewV1Response(p Product) V1Response {
	return V1Response{
		ID:         p.ID,
		Name:       p.Name,
		PriceCents: p.PriceCents,
		Stock:      p.Stock,
		Tags:       p.Tags,
		CreatedAt:  UnixTime{p.CreatedAt},
	}
}

// ============================================================================
// V2 RESPONSE
// ============================================================================

// Links holds the hypermedia links for a V2 resource.
type Links struct {
	Self       string `json:"self"`
	Collection string `json:"collection"`
}

// Meta is embedded into V2Response.
// Embedding (not a named field) promotes api_version and links to the top JSON level.
// If we wrote `Meta Meta \`json:"meta"\``, they'd be nested under {"meta": {...}}.
type Meta struct {
	APIVersion string `json:"api_version"`
	Links      Links  `json:"links"`
}

// V2Response is the richer JSON shape for V2 API clients.
// Meta is embedded — its fields appear at the top level of the JSON object.
// DeletedAt is a pointer: nil → omitted, non-nil → RFC3339 string.
type V2Response struct {
	Meta                     // embedded — api_version and links promoted to top level
	ID         string        `json:"id"`
	Name       string        `json:"name"`
	PriceCents int64         `json:"price_cents"`
	Stock      int           `json:"stock"`
	Tags       []string      `json:"tags"`
	CreatedAt  time.Time     `json:"created_at"`              // RFC3339 (time.Time default)
	DeletedAt  *time.Time    `json:"deleted_at,omitempty"`    // nil → omit, non-nil → RFC3339
}

// NewV2Response converts a Product into a V2Response.
// baseURL is used to build the self and collection links.
func NewV2Response(p Product, baseURL string) V2Response {
	return V2Response{
		Meta: Meta{
			APIVersion: "v2",
			Links: Links{
				Self:       baseURL + "/products/" + p.ID,
				Collection: baseURL + "/products",
			},
		},
		ID:         p.ID,
		Name:       p.Name,
		PriceCents: p.PriceCents,
		Stock:      p.Stock,
		Tags:       p.Tags,
		CreatedAt:  p.CreatedAt,
		DeletedAt:  p.DeletedAt,
	}
}

// ============================================================================
// SERIALIZER
// ============================================================================

// Serializer converts Products into versioned JSON or CSV output.
type Serializer struct {
	baseURL string
}

// NewSerializer creates a Serializer that uses baseURL when constructing V2 links.
func NewSerializer(baseURL string) *Serializer {
	return &Serializer{baseURL: baseURL}
}

// SerializeJSON returns a pretty-printed JSON array of Products as the given version.
// Returns an error for unknown version strings.
func (s *Serializer) SerializeJSON(products []Product, version string) ([]byte, error) {
	switch version {
	case "v1":
		responses := make([]V1Response, len(products))
		for i, p := range products {
			responses[i] = NewV1Response(p)
		}
		return json.MarshalIndent(responses, "", "  ")

	case "v2":
		responses := make([]V2Response, len(products))
		for i, p := range products {
			responses[i] = NewV2Response(p, s.baseURL)
		}
		return json.MarshalIndent(responses, "", "  ")

	default:
		return nil, fmt.Errorf("unknown API version %q: must be v1 or v2", version)
	}
}

// csvHeaders defines the column order for CSV export.
var csvHeaders = []string{"id", "name", "price_cents", "stock", "tags", "created_at", "deleted_at"}

// SerializeCSV returns CSV bytes with a header row followed by one row per product.
// Tags are semicolon-joined; deleted_at is RFC3339 or empty string.
func (s *Serializer) SerializeCSV(products []Product) ([]byte, error) {
	var buf bytes.Buffer
	w := csv.NewWriter(&buf)

	if err := w.Write(csvHeaders); err != nil {
		return nil, fmt.Errorf("write CSV header: %w", err)
	}

	for _, p := range products {
		deletedAt := ""
		if p.DeletedAt != nil {
			deletedAt = p.DeletedAt.UTC().Format(time.RFC3339)
		}

		record := []string{
			p.ID,
			p.Name,
			strconv.FormatInt(p.PriceCents, 10),
			strconv.Itoa(p.Stock),
			strings.Join(p.Tags, ";"),
			p.CreatedAt.UTC().Format(time.RFC3339),
			deletedAt,
		}

		if err := w.Write(record); err != nil {
			return nil, fmt.Errorf("write CSV row for %s: %w", p.ID, err)
		}
	}

	// csv.Writer buffers internally — must flush before reading from buf.
	// Also must check for deferred write errors via Error().
	w.Flush()
	if err := w.Error(); err != nil {
		return nil, fmt.Errorf("flush CSV: %w", err)
	}

	return buf.Bytes(), nil
}
