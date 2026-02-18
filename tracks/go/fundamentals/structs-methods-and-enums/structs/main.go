// Run with: go run ./structs/
package main

import (
	"fmt"
	"unsafe"
)

// ============================================================================
// BASIC STRUCT DEFINITIONS
// A struct groups related fields into one named type.
// ============================================================================

// WebhookEvent represents an incoming webhook from an external service.
// All fields are exported (uppercase) — visible outside the package.
type WebhookEvent struct {
	ID        string
	Source    string
	EventType string
	Payload   string
	Timestamp int64
	Delivered bool
}

// DeliveryAddress is a small struct used as a nested field.
type DeliveryAddress struct {
	Street  string
	City    string
	Country string
}

// ShipmentNotification demonstrates nested structs and field tags.
// Tags are read by encoding/json, validation libraries, ORMs, etc.
type ShipmentNotification struct {
	OrderID  string          `json:"order_id" validate:"required"`
	Carrier  string          `json:"carrier"`
	Tracking string          `json:"tracking_number,omitempty"` // omitted if empty
	Address  DeliveryAddress `json:"delivery_address"`
	Weight   float64         `json:"weight_kg"`
	Fragile  bool            `json:"fragile"`
}

// ============================================================================
// ZERO VALUES
// Every struct field is zero-initialized when not explicitly set.
// ============================================================================

func demoZeroValues() {
	fmt.Println("=== Zero Values ===")

	// Zero value of a struct: all fields set to their zero values.
	var e WebhookEvent
	fmt.Printf("ID:        %q\n", e.ID)        // ""
	fmt.Printf("Timestamp: %d\n", e.Timestamp)  // 0
	fmt.Printf("Delivered: %v\n", e.Delivered)  // false

	// Zero value is immediately usable — set fields one at a time.
	e.ID = "evt-001"
	e.Source = "stripe"
	e.EventType = "payment.completed"
	fmt.Printf("After update: %+v\n\n", e)
}

// ============================================================================
// STRUCT LITERALS
// Two styles: named fields (preferred) and positional (avoid in prod code).
// ============================================================================

func demoLiterals() {
	fmt.Println("=== Struct Literals ===")

	// Named fields — preferred. Unspecified fields get zero values.
	// Order doesn't matter. Safe to add fields to the struct later.
	e1 := WebhookEvent{
		ID:        "evt-002",
		Source:    "github",
		EventType: "push",
		Payload:   `{"ref":"refs/heads/main"}`,
	}
	// Delivered is false (zero value), Timestamp is 0 (zero value).
	fmt.Printf("Named literal: %+v\n", e1)

	// Positional — must match declaration order exactly.
	// Fragile: adding a field to the struct breaks all positional literals.
	// Only acceptable for small, very stable structs (sometimes in tests).
	e2 := WebhookEvent{"evt-003", "slack", "message", `{"text":"hi"}`, 1700000000, false}
	fmt.Printf("Positional literal: %+v\n\n", e2)

	// Nested struct literal
	shipment := ShipmentNotification{
		OrderID:  "ord-789",
		Carrier:  "UPS",
		Tracking: "1Z999AA10123456784",
		Address: DeliveryAddress{
			Street:  "123 Main St",
			City:    "Portland",
			Country: "US",
		},
		Weight: 2.5,
	}
	fmt.Printf("Nested: %+v\n\n", shipment)
}

// ============================================================================
// STRUCT SIZE AND MEMORY LAYOUT
// Fields are laid out contiguously in memory. Alignment padding may be added.
// ============================================================================

// Well-ordered struct — fields go from largest to smallest alignment.
// No padding wasted.
type CompactConfig struct {
	Timeout  int64   // 8 bytes
	MaxConns int32   // 4 bytes
	Port     int16   // 2 bytes
	Debug    bool    // 1 byte
	_        [1]byte // 1 byte explicit pad to 16-byte total
}

// Poorly-ordered struct — compiler adds hidden padding to satisfy alignment.
// This wastes memory compared to CompactConfig.
type PaddedConfig struct {
	Debug    bool    // 1 byte + 7 bytes hidden padding
	Timeout  int64   // 8 bytes
	Port     int16   // 2 bytes + 2 bytes hidden padding
	MaxConns int32   // 4 bytes
}

func demoMemoryLayout() {
	fmt.Println("=== Memory Layout ===")
	fmt.Printf("CompactConfig size: %d bytes\n", unsafe.Sizeof(CompactConfig{}))
	fmt.Printf("PaddedConfig size:  %d bytes\n", unsafe.Sizeof(PaddedConfig{}))

	// Field offsets — where each field starts within the struct
	var p PaddedConfig
	fmt.Printf("PaddedConfig.Debug    offset: %d\n", unsafe.Offsetof(p.Debug))
	fmt.Printf("PaddedConfig.Timeout  offset: %d\n", unsafe.Offsetof(p.Timeout))
	fmt.Printf("PaddedConfig.Port     offset: %d\n", unsafe.Offsetof(p.Port))
	fmt.Printf("PaddedConfig.MaxConns offset: %d\n\n", unsafe.Offsetof(p.MaxConns))
}

// ============================================================================
// ANONYMOUS STRUCTS
// Struct types defined inline — no named type. Useful for tests and one-off shapes.
// ============================================================================

func demoAnonymousStructs() {
	fmt.Println("=== Anonymous Structs ===")

	// Table-driven test cases — a classic use of anonymous structs.
	testCases := []struct {
		input    string
		wantLen  int
		wantZero bool
	}{
		{"hello", 5, false},
		{"", 0, true},
		{"café", 4, false},
	}

	for _, tc := range testCases {
		fmt.Printf("input=%q wantLen=%d wantZero=%v\n", tc.input, tc.wantLen, tc.wantZero)
	}
	fmt.Println()

	// One-off response shape — no need to define a named type for this.
	apiResponse := struct {
		Status  int    `json:"status"`
		Message string `json:"message"`
	}{
		Status:  200,
		Message: "webhook accepted",
	}
	fmt.Printf("API response: %+v\n\n", apiResponse)
}

// ============================================================================
// FIELD TAGS — what they look like, not how they're read (that's reflection)
// ============================================================================

// AuditRecord shows real-world field tag usage.
// json: controls encoding/json serialization name
// db: controls sqlx/gorm column name
// validate: controls go-playground/validator rules
type AuditRecord struct {
	ID        int64  `json:"id"          db:"id"         validate:"required"`
	UserID    int64  `json:"user_id"     db:"user_id"    validate:"required,gt=0"`
	Action    string `json:"action"      db:"action"     validate:"required,oneof=create update delete"`
	Resource  string `json:"resource"    db:"resource"   validate:"required"`
	IP        string `json:"ip_address"  db:"ip_address" validate:"ip"`
	CreatedAt int64  `json:"created_at"  db:"created_at"`
	hidden    string // unexported — ignored by json, db, validate
}

func demoFieldTags() {
	fmt.Println("=== Field Tags ===")

	r := AuditRecord{
		ID:       1,
		UserID:   42,
		Action:   "update",
		Resource: "notification/123",
		IP:       "192.168.1.1",
	}

	// Tags don't affect normal struct usage at all — they're metadata.
	fmt.Printf("AuditRecord: id=%d action=%s resource=%s\n\n",
		r.ID, r.Action, r.Resource)
}

func main() {
	demoZeroValues()
	demoLiterals()
	demoMemoryLayout()
	demoAnonymousStructs()
	demoFieldTags()
}
