// Run with: go run ./json-basics/
//
// Demonstrates: Marshal, Unmarshal, struct tags, omitempty, pointers for nullable fields,
// embedded struct promotion, and the zero-value omitempty trap.
package main

import (
	"encoding/json"
	"fmt"
	"log"
)

// ============================================================================
// Basic struct tags: rename, omitempty, skip
// ============================================================================

// WebhookEvent represents an inbound event from a third-party service.
// The JSON keys use snake_case (external convention);
// Go fields use PascalCase (Go convention).
type WebhookEvent struct {
	EventType  string `json:"event_type"`           // rename
	Source     string `json:"source"`               // rename
	ExternalID string `json:"external_id"`          // rename
	Signature  string `json:"signature,omitempty"`  // omit when empty
	InternalID string `json:"-"`                    // never in JSON
}

func marshalUnmarshalBasics() {
	fmt.Println("=== Marshal / Unmarshal ===")

	event := WebhookEvent{
		EventType:  "payment.completed",
		Source:     "stripe",
		ExternalID: "evt_3abc",
		// Signature is empty — will be omitted from JSON
		InternalID: "internal-only", // never appears in JSON
	}

	// Marshal to JSON bytes
	data, err := json.Marshal(event)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("JSON: %s\n", data)
	// → {"event_type":"payment.completed","source":"stripe","external_id":"evt_3abc"}
	// Note: Signature omitted (omitempty), InternalID omitted (json:"-")

	// Unmarshal back — with an extra unknown field in the input
	incoming := []byte(`{
		"event_type": "refund.created",
		"source":     "stripe",
		"external_id":"evt_9xyz",
		"unknown_future_field": "ignored silently"
	}`)

	var received WebhookEvent
	if err := json.Unmarshal(incoming, &received); err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Received: %+v\n", received)
	// unknown_future_field is silently ignored — forward-compatible by default
}

// ============================================================================
// The omitempty zero-value trap with integers
// ============================================================================

// JobResult shows the omitempty pitfall: exit code 0 (success) is the zero
// value for int, so it gets omitted — exactly when you need it most.
type JobResult struct {
	JobID    string `json:"job_id"`
	ExitCode int    `json:"exit_code,omitempty"` // DANGEROUS: omits exit code 0
	Output   string `json:"output,omitempty"`    // safe: empty output is meaningless
}

// JobResultSafe fixes the problem with a pointer: nil = "not set", &0 = "explicitly 0".
type JobResultSafe struct {
	JobID    string `json:"job_id"`
	ExitCode *int   `json:"exit_code,omitempty"` // nil omitted; pointer-to-0 serializes as 0
	Output   string `json:"output,omitempty"`
}

func omitemptyTrap() {
	fmt.Println("\n=== omitempty zero-value trap ===")

	exitCode := 0 // success

	bad := JobResult{JobID: "job-1", ExitCode: exitCode}
	good := JobResultSafe{JobID: "job-1", ExitCode: &exitCode}

	badData, _ := json.Marshal(bad)
	goodData, _ := json.Marshal(good)

	fmt.Printf("Bad  (int,   exit_code=0): %s\n", badData)
	// → {"job_id":"job-1"} — exit_code silently dropped!

	fmt.Printf("Good (*int,  exit_code=0): %s\n", goodData)
	// → {"job_id":"job-1","exit_code":0} — preserved

	// Omitting when explicitly not set:
	notSet := JobResultSafe{JobID: "job-2"} // ExitCode is nil
	notSetData, _ := json.Marshal(notSet)
	fmt.Printf("Not set (*int, nil):       %s\n", notSetData)
	// → {"job_id":"job-2"} — omitted because nil, which is the intent
}

// ============================================================================
// Embedded structs and promoted fields
// ============================================================================

// ResponseMeta holds fields that appear in every API response.
// Embedding it into response types promotes these fields to the top level.
type ResponseMeta struct {
	RequestID string `json:"request_id"`
	Version   int    `json:"api_version"`
	DurationMs int64 `json:"duration_ms,omitempty"`
}

// UserResponse is the payload for /users/:id.
// The embedded ResponseMeta fields appear at the top level in JSON —
// there is no nested "response_meta" key.
type UserResponse struct {
	ResponseMeta            // embedded — fields promoted
	UserID    string        `json:"user_id"`
	Email     string        `json:"email"`
	Name      string        `json:"name"`
}

// ItemResponse demonstrates that a named field creates a nested object.
type ItemResponse struct {
	Meta   ResponseMeta `json:"meta"` // named field — creates nested {"meta": {...}}
	ItemID string       `json:"item_id"`
	Name   string       `json:"name"`
}

func embeddedStructs() {
	fmt.Println("\n=== Embedded structs: promoted vs nested ===")

	user := UserResponse{
		ResponseMeta: ResponseMeta{
			RequestID:  "req-abc-123",
			Version:    2,
			DurationMs: 45,
		},
		UserID: "usr_001",
		Email:  "alice@example.com",
		Name:   "Alice",
	}

	item := ItemResponse{
		Meta: ResponseMeta{
			RequestID: "req-def-456",
			Version:   2,
		},
		ItemID: "item_001",
		Name:   "Widget Pro",
	}

	userData, _ := json.MarshalIndent(user, "", "  ")
	itemData, _ := json.MarshalIndent(item, "", "  ")

	fmt.Println("Embedded (promoted):")
	fmt.Println(string(userData))
	// request_id, api_version, duration_ms are at the top level

	fmt.Println("\nNamed field (nested):")
	fmt.Println(string(itemData))
	// meta: { request_id, api_version } is a nested object
}

// ============================================================================
// The json:",string" option
// ============================================================================

// SnowflakeID represents a Twitter-style 64-bit ID.
// JavaScript's Number type can't safely represent integers above 2^53,
// so large IDs are transmitted as strings to avoid precision loss.
type SnowflakeID struct {
	UserID    int64  `json:"user_id,string"`    // "12345678901234567" in JSON
	PostID    int64  `json:"post_id,string"`    // also a string
	Score     int    `json:"score"`             // regular number, small enough
}

func stringOption() {
	fmt.Println("\n=== json:\",string\" for JS-safe large integers ===")

	s := SnowflakeID{
		UserID: 1234567890123456789,
		PostID: 9007199254740993, // 2^53 + 1: above JS safe integer range
		Score:  42,
	}
	data, _ := json.Marshal(s)
	fmt.Printf("%s\n", data)
	// → {"user_id":"1234567890123456789","post_id":"9876543210987654321","score":42}
	// Strings survive JavaScript without precision loss; score is a regular number.
}

func main() {
	marshalUnmarshalBasics()
	omitemptyTrap()
	embeddedStructs()
	stringOption()
}
