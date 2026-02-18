package serializer

import (
	"time"
)

// ============================================================================
// DOMAIN TYPE
// Product is the core data model. Serialization layers convert it into
// version-specific response shapes — the domain type itself never changes.
// ============================================================================

// Product is an item in the product catalog.
type Product struct {
	ID         string
	Name       string
	PriceCents int64
	Stock      int
	Tags       []string
	CreatedAt  time.Time
	DeletedAt  *time.Time // nil means not deleted
}

// ============================================================================
// UNIX TIME WRAPPER
// V1 uses Unix integer timestamps; V2 uses RFC3339 strings.
// UnixTime wraps time.Time and implements custom JSON marshaling.
// ============================================================================

// UnixTime is a time.Time that marshals to/from a JSON integer (Unix seconds).
type UnixTime struct {
	time.Time
}

// MarshalJSON encodes the time as a Unix integer timestamp.
// TODO: Implement. Must return a JSON integer, not a string.
func (u UnixTime) MarshalJSON() ([]byte, error) {
	return nil, nil
}

// UnmarshalJSON decodes a Unix integer timestamp into the UnixTime.
// TODO: Implement.
func (u *UnixTime) UnmarshalJSON(data []byte) error {
	return nil
}

// ============================================================================
// V1 RESPONSE
// Flat shape for V1 API clients.
// Fields: id, name, price_cents, stock, tags, created_at (Unix int)
// DeletedAt is never included in V1 output.
// ============================================================================

// V1Response is the JSON shape returned to V1 API clients.
type V1Response struct {
	// TODO: Add fields with correct json tags.
	// created_at should use UnixTime so it serializes as an integer.
	// DeletedAt must not appear in V1 JSON at all (not just omitted — never present).
}

// NewV1Response converts a Product into a V1Response.
// TODO: Implement.
func NewV1Response(p Product) V1Response {
	return V1Response{}
}

// ============================================================================
// V2 RESPONSE
// Richer shape for V2 API clients.
// Embeds Meta so api_version and links appear at the top JSON level.
// ============================================================================

// Links holds the hypermedia links for a V2 resource.
type Links struct {
	Self       string `json:"self"`
	Collection string `json:"collection"`
}

// Meta is embedded into V2Response. Its fields are promoted to the top level.
// Do not use a named field — that would create a nested "meta" key.
type Meta struct {
	// TODO: Add APIVersion string with json:"api_version"
	// TODO: Add Links Links with json:"links"
}

// V2Response is the JSON shape returned to V2 API clients.
// Meta is embedded — its fields (api_version, links) appear at the top JSON level.
type V2Response struct {
	Meta // embedded — fields promoted, not nested
	// TODO: Add fields with correct json tags.
	// created_at should be time.Time (RFC3339 default).
	// deleted_at should be *time.Time with omitempty (nil = omit, non-nil = include).
}

// NewV2Response converts a Product into a V2Response.
// baseURL is used to construct the self and collection links.
// e.g., baseURL = "https://api.example.com" →
//   self:       "https://api.example.com/products/prod_001"
//   collection: "https://api.example.com/products"
// TODO: Implement.
func NewV2Response(p Product, baseURL string) V2Response {
	return V2Response{}
}

// ============================================================================
// SERIALIZER
// Converts a slice of Products into the requested format and version.
// ============================================================================

// Serializer converts Products into versioned JSON or CSV output.
type Serializer struct {
	// TODO: store baseURL
}

// NewSerializer creates a Serializer using baseURL for constructing V2 links.
// TODO: Implement.
func NewSerializer(baseURL string) *Serializer {
	return &Serializer{}
}

// SerializeJSON returns a pretty-printed JSON array (2-space indent) of
// Products serialized as the given version ("v1" or "v2").
// Returns an error for unknown version strings.
// TODO: Implement.
func (s *Serializer) SerializeJSON(products []Product, version string) ([]byte, error) {
	return nil, nil
}

// SerializeCSV returns a CSV byte slice with a header row followed by
// one row per product.
//
// Columns (in order): id, name, price_cents, stock, tags, created_at, deleted_at
// - tags: semicolon-joined (e.g., "hardware;featured")
// - created_at: RFC3339 UTC
// - deleted_at: RFC3339 UTC, or empty string if nil
//
// TODO: Implement.
func (s *Serializer) SerializeCSV(products []Product) ([]byte, error) {
	return nil, nil
}
