// Package main demonstrates type assertions, type switches, and the nil
// interface trap — the three subtler aspects of Go interfaces.
//
// Topics:
// - Type assertions: safe (comma-ok) and unsafe (panicking) forms
// - Type switches: dispatching on concrete type
// - The nil interface trap: why (interface)(nil typed value) != nil
// - any (empty interface) and its use cases
// - Interface upgrade via type assertion
//
// Run with: go run ./type-assertions/
package main

import (
	"errors"
	"fmt"
)

// ============================================================================
// TYPES AND INTERFACES FOR EXAMPLES
// ============================================================================

// StorageEvent is the base event interface.
type StorageEvent interface {
	EventType() string
	Key() string
}

// WriteEvent is emitted when a key is written.
type WriteEvent struct {
	key   string
	value []byte
	size  int
}

func (w WriteEvent) EventType() string { return "write" }
func (w WriteEvent) Key() string       { return w.key }
func (w WriteEvent) Value() []byte     { return w.value }
func (w WriteEvent) Size() int         { return w.size }

// DeleteEvent is emitted when a key is deleted.
type DeleteEvent struct {
	key string
}

func (d DeleteEvent) EventType() string { return "delete" }
func (d DeleteEvent) Key() string       { return d.key }

// ExpireEvent is emitted when a key's TTL expires.
type ExpireEvent struct {
	key string
	ttl int // seconds the key was alive
}

func (e ExpireEvent) EventType() string { return "expire" }
func (e ExpireEvent) Key() string       { return e.key }
func (e ExpireEvent) TTL() int          { return e.ttl }

// ============================================================================
// TYPE ASSERTIONS
//
// Extract the concrete type from an interface.
// Two forms: panicking (use when you're certain) and safe (use when uncertain).
// ============================================================================

func demonstrateTypeAssertions() {
	fmt.Println("--- Type Assertions ---")

	var event StorageEvent = WriteEvent{
		key:   "user:42",
		value: []byte(`{"name":"alice"}`),
		size:  18,
	}

	// SAFE form: two-return (comma-ok). Use this almost always.
	if we, ok := event.(WriteEvent); ok {
		fmt.Printf("write event: key=%s size=%d\n", we.Key(), we.Size())
	}

	// SAFE assertion to a different (wrong) type: ok=false, no panic.
	if de, ok := event.(DeleteEvent); ok {
		fmt.Printf("delete event: key=%s\n", de.Key())
	} else {
		fmt.Println("not a DeleteEvent (as expected)")
	}

	// UNSAFE form: single-return. Panics if assertion fails.
	// Only use when you have a compile-time guarantee of the type.
	we := event.(WriteEvent) // guaranteed to work — we just set it above
	fmt.Printf("unsafe assertion worked: key=%s\n", we.Key())

	// Demonstrate the panic: uncomment to see it
	// _ = event.(DeleteEvent) // PANIC: interface conversion: main.WriteEvent is not main.DeleteEvent

	// ASSERT TO INTERFACE: check if a value satisfies a more specific interface.
	// Useful for optional capabilities (interface upgrades).
	type Sizer interface {
		Size() int
	}
	if sizer, ok := event.(Sizer); ok {
		fmt.Printf("event has Size(): %d bytes\n", sizer.Size())
	}

	fmt.Println()
}

// ============================================================================
// TYPE SWITCHES
//
// Clean way to handle multiple possible concrete types behind one interface.
// Each case arm binds the variable to the concrete type.
// ============================================================================

// processEvent handles each event type differently.
// This is idiomatic: type switch on the interface, access concrete fields in each arm.
func processEvent(event StorageEvent) {
	switch e := event.(type) {
	case WriteEvent:
		// e is WriteEvent here — can access e.Size(), e.Value()
		fmt.Printf("  WRITE: key=%s value=%q bytes=%d\n", e.Key(), e.Value(), e.Size())

	case DeleteEvent:
		// e is DeleteEvent here
		fmt.Printf("  DELETE: key=%s\n", e.Key())

	case ExpireEvent:
		// e is ExpireEvent here — can access e.TTL()
		fmt.Printf("  EXPIRE: key=%s after %ds\n", e.Key(), e.TTL())

	case nil:
		// The interface itself was nil — not just a nil concrete value
		fmt.Println("  NIL event received")

	default:
		// Handles any future event types we haven't accounted for yet.
		// %T prints the concrete type name.
		fmt.Printf("  UNKNOWN event type: %T key=%s\n", e, e.Key())
	}
}

func demonstrateTypeSwitch() {
	fmt.Println("--- Type Switch ---")

	events := []StorageEvent{
		WriteEvent{key: "session:abc", value: []byte("token"), size: 5},
		DeleteEvent{key: "session:xyz"},
		ExpireEvent{key: "rate-limit:ip:1.2.3.4", ttl: 60},
		nil, // nil interface — handled by case nil
	}

	for _, e := range events {
		processEvent(e)
	}
	fmt.Println()
}

// ============================================================================
// THE NIL INTERFACE TRAP
//
// The most confusing thing about Go interfaces.
// An interface value is nil only if both its type and value are nil.
// A nil concrete pointer wrapped in an interface is NOT nil.
// ============================================================================

// NetworkError is a concrete error type.
type NetworkError struct {
	Code    int
	Message string
}

func (e *NetworkError) Error() string {
	return fmt.Sprintf("network error %d: %s", e.Code, e.Message)
}

// BUG: returns a non-nil interface holding a nil *NetworkError.
// The caller's nil check will fail!
func connectBug(failWithTypedNil bool) error {
	var netErr *NetworkError // nil *NetworkError

	if failWithTypedNil {
		// We return the typed nil — but wrapped in error interface,
		// this creates: (type=*NetworkError, value=nil)
		// That interface is NOT nil.
		return netErr
	}
	return nil
}

// FIX: returns a truly nil interface when there's no error.
func connectFixed(shouldFail bool) error {
	if shouldFail {
		return &NetworkError{Code: 503, Message: "service unavailable"}
	}
	return nil // untyped nil — both type and value are nil
}

func demonstrateNilInterfaceTrap() {
	fmt.Println("--- Nil Interface Trap ---")

	// The trap:
	err := connectBug(true)
	fmt.Printf("connectBug() returned: %v\n", err)
	fmt.Printf("err == nil:            %v (SURPRISING — we returned a nil pointer!)\n", err == nil)
	fmt.Printf("concrete value is nil: %v\n", err.(*NetworkError) == nil)
	// err is not nil (the interface has a type), but the underlying value is nil.
	// Calling err.Error() would panic.
	fmt.Println()

	// Why this happens:
	// When you return a *NetworkError (even nil) as an error interface,
	// Go creates: interface{type: *NetworkError, value: nil}
	// == nil checks BOTH type and value. Type is *NetworkError ≠ nil.
	// So the interface is not nil.

	// The fix:
	err2 := connectFixed(false)
	fmt.Printf("connectFixed() returned: %v\n", err2)
	fmt.Printf("err2 == nil:             %v (correct)\n", err2 == nil)
	fmt.Println()

	// How to check for nil concrete value inside a non-nil interface:
	// Use a type assertion, then check the concrete value.
	err3 := connectBug(true) // non-nil interface, nil concrete
	if ne, ok := err3.(*NetworkError); ok {
		if ne == nil {
			fmt.Println("interface is non-nil but holds a nil *NetworkError")
			fmt.Println("this is the trap: the if err != nil check passed, but we have no error data")
		}
	}
	fmt.Println()

	// Practical rule: when returning interfaces, always return explicit nil.
	// Never return a typed nil — use conditional return.
	fmt.Println("Rule: return nil (untyped), never return a typed nil via an interface")
	fmt.Println()
}

// ============================================================================
// THE EMPTY INTERFACE: any
//
// any = interface{} — holds any value. Useful but loses type safety.
// Use sparingly; prefer type parameters (generics) in Go 1.18+.
// ============================================================================

// Metadata holds arbitrary key-value pairs with any value type.
// This is a legitimate use of any: we genuinely don't know what values
// callers will store here.
type Metadata map[string]any

func (m Metadata) String(key string) (string, bool) {
	v, ok := m[key]
	if !ok {
		return "", false
	}
	s, ok := v.(string)
	return s, ok
}

func (m Metadata) Int(key string) (int, bool) {
	v, ok := m[key]
	if !ok {
		return 0, false
	}
	i, ok := v.(int)
	return i, ok
}

// formatValue formats any value for display.
// Type switch on any is a common pattern for serialization/formatting.
func formatValue(v any) string {
	switch val := v.(type) {
	case string:
		return fmt.Sprintf("%q", val)
	case int:
		return fmt.Sprintf("%d", val)
	case float64:
		return fmt.Sprintf("%.2f", val)
	case bool:
		if val {
			return "true"
		}
		return "false"
	case []byte:
		return fmt.Sprintf("bytes(%d)", len(val))
	case nil:
		return "null"
	case error:
		return fmt.Sprintf("error(%s)", val.Error())
	default:
		return fmt.Sprintf("%T(%v)", val, val)
	}
}

func demonstrateAny() {
	fmt.Println("--- empty interface (any) ---")

	meta := Metadata{
		"user_id":    "u-42",
		"request_ms": 145,
		"cache_hit":  true,
		"threshold":  0.95,
		"raw":        []byte("binary payload"),
	}

	for key, val := range meta {
		fmt.Printf("  %-15s = %s\n", key, formatValue(val))
	}
	fmt.Println()

	// Type-safe accessors via type assertion
	if id, ok := meta.String("user_id"); ok {
		fmt.Printf("user_id (string): %s\n", id)
	}
	if ms, ok := meta.Int("request_ms"); ok {
		fmt.Printf("request_ms (int): %d\n", ms)
	}
	fmt.Println()
}

// ============================================================================
// ERRORS.AS: TYPE ASSERTION IN THE ERROR CHAIN
//
// errors.As traverses the error chain and does a type assertion.
// This is the preferred way to extract specific error types.
// ============================================================================

func demonstrateErrorsAs() {
	fmt.Println("--- errors.As (type assertion in error chain) ---")

	// Wrap a NetworkError in a higher-level error
	original := &NetworkError{Code: 429, Message: "too many requests"}
	wrapped := fmt.Errorf("fetching config: %w", original)
	doubleWrapped := fmt.Errorf("initializing service: %w", wrapped)

	// errors.As walks the chain, finds *NetworkError, and type-asserts into it.
	var netErr *NetworkError
	if errors.As(doubleWrapped, &netErr) {
		fmt.Printf("Found NetworkError in chain: code=%d msg=%s\n", netErr.Code, netErr.Message)
	}

	// errors.Is checks for value equality (or Unwrap chain)
	if errors.Is(doubleWrapped, original) {
		fmt.Println("original error found in chain via errors.Is")
	}
	fmt.Println()
}

// ============================================================================
// MAIN
// ============================================================================

func main() {
	fmt.Println("=== Type Assertions, Type Switches, Nil Interface Trap ===")
	fmt.Println()

	demonstrateTypeAssertions()
	demonstrateTypeSwitch()
	demonstrateNilInterfaceTrap()
	demonstrateAny()
	demonstrateErrorsAs()
}
