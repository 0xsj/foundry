// Package main demonstrates the Observer pattern using callback-based observers.
// An audit event system records user actions and notifies multiple logging
// backends (console, JSON file, metrics) using simple function callbacks.
//
// This is the simplest observer approach in Go -- a slice of functions called
// sequentially when events occur. It's the closest analog to Node.js EventEmitter.
//
// Run: go run ./audit-logger/
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"
)

// --- Event types ---

// AuditAction describes what the user did.
type AuditAction string

const (
	ActionLogin         AuditAction = "user.login"
	ActionLogout        AuditAction = "user.logout"
	ActionPasswordReset AuditAction = "user.password_reset"
	ActionRoleChange    AuditAction = "user.role_change"
	ActionDataExport    AuditAction = "data.export"
	ActionDataDelete    AuditAction = "data.delete"
	ActionConfigChange  AuditAction = "config.change"
	ActionAPIKeyCreated AuditAction = "api_key.created"
	ActionAPIKeyRevoked AuditAction = "api_key.revoked"
)

// Severity classifies the audit event importance.
type Severity int

const (
	SeverityInfo Severity = iota
	SeverityWarning
	SeverityCritical
)

func (s Severity) String() string {
	switch s {
	case SeverityInfo:
		return "INFO"
	case SeverityWarning:
		return "WARNING"
	case SeverityCritical:
		return "CRITICAL"
	default:
		return "UNKNOWN"
	}
}

// AuditEvent is the payload passed to all observers.
type AuditEvent struct {
	ID        string            `json:"id"`
	Action    AuditAction       `json:"action"`
	Severity  Severity          `json:"severity"`
	UserID    string            `json:"user_id"`
	IP        string            `json:"ip"`
	Metadata  map[string]string `json:"metadata,omitempty"`
	Timestamp time.Time         `json:"timestamp"`
}

// --- Callback-based observer system ---

// AuditHandler is the observer callback type.
type AuditHandler func(AuditEvent)

// namedHandler wraps a handler with an ID for unsubscription.
type namedHandler struct {
	id int
	fn AuditHandler
}

// AuditBus is the subject that manages callback observers.
type AuditBus struct {
	mu       sync.RWMutex
	handlers map[AuditAction][]namedHandler
	global   []namedHandler // handlers that receive ALL events
	nextID   int
}

func NewAuditBus() *AuditBus {
	return &AuditBus{
		handlers: make(map[AuditAction][]namedHandler),
	}
}

// On registers a handler for a specific action. Returns an unsubscribe function.
// This mirrors the Node.js pattern: emitter.on('event', handler)
func (b *AuditBus) On(action AuditAction, handler AuditHandler) func() {
	b.mu.Lock()
	id := b.nextID
	b.nextID++
	b.handlers[action] = append(b.handlers[action], namedHandler{id: id, fn: handler})
	b.mu.Unlock()

	// Return cleanup function -- same pattern as React useEffect cleanup
	// or Go's context.WithCancel returning a cancel func.
	return func() {
		b.mu.Lock()
		defer b.mu.Unlock()
		handlers := b.handlers[action]
		for i, h := range handlers {
			if h.id == id {
				b.handlers[action] = append(handlers[:i], handlers[i+1:]...)
				return
			}
		}
	}
}

// OnAll registers a handler that receives every event regardless of action.
// Returns an unsubscribe function.
func (b *AuditBus) OnAll(handler AuditHandler) func() {
	b.mu.Lock()
	id := b.nextID
	b.nextID++
	b.global = append(b.global, namedHandler{id: id, fn: handler})
	b.mu.Unlock()

	return func() {
		b.mu.Lock()
		defer b.mu.Unlock()
		for i, h := range b.global {
			if h.id == id {
				b.global = append(b.global[:i], b.global[i+1:]...)
				return
			}
		}
	}
}

// Emit publishes an event to matching handlers and global handlers.
// Handlers are called synchronously in registration order.
func (b *AuditBus) Emit(event AuditEvent) {
	// Snapshot handlers under read lock, then call outside the lock.
	// This prevents deadlocks if a handler calls On() or unsubscribes.
	b.mu.RLock()
	specific := make([]namedHandler, len(b.handlers[event.Action]))
	copy(specific, b.handlers[event.Action])
	global := make([]namedHandler, len(b.global))
	copy(global, b.global)
	b.mu.RUnlock()

	// Call global handlers first (they often include logging/metrics)
	for _, h := range global {
		h.fn(event)
	}
	// Then action-specific handlers
	for _, h := range specific {
		h.fn(event)
	}
}

// --- Concrete observer callbacks ---

// ConsoleLogger prints formatted audit events to stdout.
func ConsoleLogger(severityFilter Severity) AuditHandler {
	return func(e AuditEvent) {
		if e.Severity < severityFilter {
			return
		}
		fmt.Printf("[%s] %-10s | %-25s | user=%-8s | ip=%-15s",
			e.Timestamp.Format("15:04:05"),
			e.Severity,
			e.Action,
			e.UserID,
			e.IP,
		)
		if len(e.Metadata) > 0 {
			parts := make([]string, 0, len(e.Metadata))
			for k, v := range e.Metadata {
				parts = append(parts, fmt.Sprintf("%s=%s", k, v))
			}
			fmt.Printf(" | %s", strings.Join(parts, ", "))
		}
		fmt.Println()
	}
}

// JSONLogger collects events as JSON lines (in production, this writes to a file).
func JSONLogger() (AuditHandler, func() []string) {
	var mu sync.Mutex
	var lines []string

	handler := func(e AuditEvent) {
		data, err := json.Marshal(e)
		if err != nil {
			log.Printf("json marshal error: %v", err)
			return
		}
		mu.Lock()
		lines = append(lines, string(data))
		mu.Unlock()
	}

	getLines := func() []string {
		mu.Lock()
		defer mu.Unlock()
		out := make([]string, len(lines))
		copy(out, lines)
		return out
	}

	return handler, getLines
}

// MetricsCounter counts events by action and severity.
func MetricsCounter() (AuditHandler, func() map[string]int) {
	var mu sync.Mutex
	counts := make(map[string]int)

	handler := func(e AuditEvent) {
		key := fmt.Sprintf("%s.%s", e.Action, e.Severity)
		mu.Lock()
		counts[key]++
		mu.Unlock()
	}

	getCounts := func() map[string]int {
		mu.Lock()
		defer mu.Unlock()
		out := make(map[string]int, len(counts))
		for k, v := range counts {
			out[k] = v
		}
		return out
	}

	return handler, getCounts
}

// SecurityAlertHandler only fires for critical severity events.
func SecurityAlertHandler() AuditHandler {
	return func(e AuditEvent) {
		if e.Severity != SeverityCritical {
			return
		}
		fmt.Printf("*** SECURITY ALERT *** %s by user %s from %s\n",
			e.Action, e.UserID, e.IP)
	}
}

// --- Main ---

func main() {
	bus := NewAuditBus()

	// Register global observers (receive all events)
	jsonHandler, getJSONLines := JSONLogger()
	metricsHandler, getMetrics := MetricsCounter()

	unsubJSON := bus.OnAll(jsonHandler)
	_ = bus.OnAll(metricsHandler)
	_ = bus.OnAll(ConsoleLogger(SeverityInfo))

	// Register action-specific observers
	unsubSecurity := bus.On(ActionPasswordReset, SecurityAlertHandler())
	_ = bus.On(ActionDataDelete, SecurityAlertHandler())
	_ = bus.On(ActionAPIKeyRevoked, SecurityAlertHandler())

	fmt.Println("=== Emitting audit events ===\n")

	now := time.Now()
	eventID := 0
	nextID := func() string {
		eventID++
		return fmt.Sprintf("evt_%03d", eventID)
	}

	// Normal login
	bus.Emit(AuditEvent{
		ID: nextID(), Action: ActionLogin, Severity: SeverityInfo,
		UserID: "usr_42", IP: "192.168.1.10",
		Metadata:  map[string]string{"method": "password"},
		Timestamp: now,
	})

	// Suspicious password reset
	bus.Emit(AuditEvent{
		ID: nextID(), Action: ActionPasswordReset, Severity: SeverityCritical,
		UserID: "usr_42", IP: "203.0.113.50",
		Metadata:  map[string]string{"source": "unknown_device"},
		Timestamp: now.Add(5 * time.Minute),
	})

	// Data export
	bus.Emit(AuditEvent{
		ID: nextID(), Action: ActionDataExport, Severity: SeverityWarning,
		UserID: "usr_99", IP: "10.0.0.5",
		Metadata:  map[string]string{"records": "50000", "format": "csv"},
		Timestamp: now.Add(10 * time.Minute),
	})

	// Config change
	bus.Emit(AuditEvent{
		ID: nextID(), Action: ActionConfigChange, Severity: SeverityWarning,
		UserID: "admin_1", IP: "10.0.0.1",
		Metadata:  map[string]string{"field": "rate_limit", "old": "100", "new": "200"},
		Timestamp: now.Add(15 * time.Minute),
	})

	// Demonstrate unsubscribe
	fmt.Println("\n=== Unsubscribing JSON logger and security alerts for password reset ===\n")
	unsubJSON()
	unsubSecurity()

	// This event won't be captured by JSON logger or security alert for password reset
	bus.Emit(AuditEvent{
		ID: nextID(), Action: ActionPasswordReset, Severity: SeverityCritical,
		UserID: "usr_77", IP: "198.51.100.23",
		Metadata:  map[string]string{"source": "api"},
		Timestamp: now.Add(20 * time.Minute),
	})

	// Print collected data
	fmt.Println("\n=== JSON Log (4 events, 5th was after unsubscribe) ===")
	for i, line := range getJSONLines() {
		fmt.Printf("  %d: %s\n", i+1, line)
	}

	fmt.Println("\n=== Metrics ===")
	for key, count := range getMetrics() {
		fmt.Printf("  %-40s %d\n", key, count)
	}
}
