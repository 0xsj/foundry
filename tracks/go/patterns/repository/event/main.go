// Repository Pattern: Event Store
//
// Demonstrates how the repository pattern works for an append-only event log.
// Unlike CRUD repositories, an event store only appends -- events are immutable
// once stored. Queries are by time range and aggregate ID.
//
// This is a simplified event sourcing pattern: instead of storing current state,
// you store the sequence of events that produced the state, and replay them
// to reconstruct it.
//
// Key ideas:
//   - Append-only interface (no Update or Delete)
//   - Query by time range and aggregate ID
//   - Events are immutable value types
//   - Repository abstraction works for non-CRUD storage patterns too
//
// Run: go run ./event/

package main

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"sync"
	"time"
)

// =============================================================================
// Domain Types
// =============================================================================

// Event represents an immutable domain event.
type Event struct {
	ID          string
	AggregateID string    // the entity this event belongs to (e.g., order ID)
	Type        string    // event type (e.g., "order.created", "payment.processed")
	Payload     []byte    // JSON-encoded event data
	OccurredAt  time.Time
	Version     int       // sequence number within the aggregate
}

// =============================================================================
// Repository Interface
// =============================================================================

// EventStore defines operations for an append-only event log.
// Notice: no Update, no Delete. Events are immutable facts.
type EventStore interface {
	// Append stores a new event. The store assigns the ID and timestamp.
	Append(ctx context.Context, event *Event) error

	// GetByAggregate returns all events for a given aggregate, ordered by version.
	GetByAggregate(ctx context.Context, aggregateID string) ([]*Event, error)

	// QueryByTimeRange returns events within the given time range, ordered by time.
	QueryByTimeRange(ctx context.Context, from, to time.Time) ([]*Event, error)

	// QueryByType returns all events of a given type, ordered by time.
	QueryByType(ctx context.Context, eventType string) ([]*Event, error)
}

// =============================================================================
// In-Memory Implementation
// =============================================================================

// MemoryEventStore stores events in a slice (append-only).
type MemoryEventStore struct {
	mu     sync.RWMutex
	events []*Event
	nextID int
}

var _ EventStore = (*MemoryEventStore)(nil)

func NewMemoryEventStore() *MemoryEventStore {
	return &MemoryEventStore{
		events: make([]*Event, 0),
	}
}

func (m *MemoryEventStore) Append(_ context.Context, event *Event) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	// Assign ID and timestamp (like a database would)
	m.nextID++
	event.ID = fmt.Sprintf("evt-%d", m.nextID)
	if event.OccurredAt.IsZero() {
		event.OccurredAt = time.Now()
	}

	// Assign version: count existing events for this aggregate + 1
	version := 0
	for _, e := range m.events {
		if e.AggregateID == event.AggregateID {
			version++
		}
	}
	event.Version = version + 1

	// Store a copy -- events are immutable, but we still copy to prevent
	// the caller from modifying the payload slice after appending.
	stored := *event
	stored.Payload = make([]byte, len(event.Payload))
	copy(stored.Payload, event.Payload)
	m.events = append(m.events, &stored)

	return nil
}

func (m *MemoryEventStore) GetByAggregate(_ context.Context, aggregateID string) ([]*Event, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	var result []*Event
	for _, e := range m.events {
		if e.AggregateID == aggregateID {
			copy := *e
			copy.Payload = make([]byte, len(e.Payload))
			copy2(copy.Payload, e.Payload)
			result = append(result, &copy)
		}
	}
	// Already ordered by version (appended in order)
	return result, nil
}

func (m *MemoryEventStore) QueryByTimeRange(_ context.Context, from, to time.Time) ([]*Event, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	var result []*Event
	for _, e := range m.events {
		if (e.OccurredAt.Equal(from) || e.OccurredAt.After(from)) &&
			(e.OccurredAt.Equal(to) || e.OccurredAt.Before(to)) {
			copy := *e
			copy.Payload = make([]byte, len(e.Payload))
			copy2(copy.Payload, e.Payload)
			result = append(result, &copy)
		}
	}
	return result, nil
}

func (m *MemoryEventStore) QueryByType(_ context.Context, eventType string) ([]*Event, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	var result []*Event
	for _, e := range m.events {
		if e.Type == eventType {
			copy := *e
			copy.Payload = make([]byte, len(e.Payload))
			copy2(copy.Payload, e.Payload)
			result = append(result, &copy)
		}
	}
	return result, nil
}

// copy2 is a helper to avoid shadowing the builtin copy.
func copy2(dst, src []byte) {
	copy(dst, src)
}

// =============================================================================
// Business Logic: Order Aggregate
// =============================================================================

// OrderService manages order lifecycle through events.
type OrderService struct {
	events EventStore
}

func NewOrderService(events EventStore) *OrderService {
	return &OrderService{events: events}
}

// OrderPayload is the JSON shape for order events.
type OrderPayload struct {
	OrderID    string  `json:"order_id"`
	CustomerID string  `json:"customer_id,omitempty"`
	Items      []Item  `json:"items,omitempty"`
	Total      float64 `json:"total,omitempty"`
	Reason     string  `json:"reason,omitempty"`
}

type Item struct {
	ProductID string  `json:"product_id"`
	Quantity  int     `json:"quantity"`
	Price     float64 `json:"price"`
}

func (s *OrderService) CreateOrder(ctx context.Context, orderID, customerID string, items []Item) error {
	total := 0.0
	for _, item := range items {
		total += item.Price * float64(item.Quantity)
	}

	payload, _ := json.Marshal(OrderPayload{
		OrderID:    orderID,
		CustomerID: customerID,
		Items:      items,
		Total:      total,
	})

	return s.events.Append(ctx, &Event{
		AggregateID: orderID,
		Type:        "order.created",
		Payload:     payload,
	})
}

func (s *OrderService) ProcessPayment(ctx context.Context, orderID string, amount float64) error {
	payload, _ := json.Marshal(OrderPayload{
		OrderID: orderID,
		Total:   amount,
	})

	return s.events.Append(ctx, &Event{
		AggregateID: orderID,
		Type:        "payment.processed",
		Payload:     payload,
	})
}

func (s *OrderService) ShipOrder(ctx context.Context, orderID string) error {
	payload, _ := json.Marshal(OrderPayload{
		OrderID: orderID,
	})

	return s.events.Append(ctx, &Event{
		AggregateID: orderID,
		Type:        "order.shipped",
		Payload:     payload,
	})
}

func (s *OrderService) CancelOrder(ctx context.Context, orderID, reason string) error {
	payload, _ := json.Marshal(OrderPayload{
		OrderID: orderID,
		Reason:  reason,
	})

	return s.events.Append(ctx, &Event{
		AggregateID: orderID,
		Type:        "order.cancelled",
		Payload:     payload,
	})
}

// GetOrderHistory returns the full event history for an order.
func (s *OrderService) GetOrderHistory(ctx context.Context, orderID string) ([]*Event, error) {
	return s.events.GetByAggregate(ctx, orderID)
}

// =============================================================================
// Main
// =============================================================================

func main() {
	ctx := context.Background()
	store := NewMemoryEventStore()
	svc := NewOrderService(store)

	fmt.Println("=== Repository Pattern: Event Store ===")
	fmt.Println(strings.Repeat("-", 55))

	// Create and process two orders
	fmt.Println("\n1. Creating orders:")

	items1 := []Item{
		{ProductID: "laptop-1", Quantity: 1, Price: 1299.99},
		{ProductID: "mouse-1", Quantity: 2, Price: 29.99},
	}
	svc.CreateOrder(ctx, "order-001", "customer-alice", items1)
	fmt.Println("   Created order-001 (laptop + 2 mice)")

	items2 := []Item{
		{ProductID: "keyboard-1", Quantity: 1, Price: 149.99},
	}
	svc.CreateOrder(ctx, "order-002", "customer-bob", items2)
	fmt.Println("   Created order-002 (keyboard)")

	// Process order-001 through its lifecycle
	fmt.Println("\n2. Processing order-001:")
	svc.ProcessPayment(ctx, "order-001", 1359.97)
	fmt.Println("   Payment processed")
	svc.ShipOrder(ctx, "order-001")
	fmt.Println("   Order shipped")

	// Cancel order-002
	fmt.Println("\n3. Cancelling order-002:")
	svc.CancelOrder(ctx, "order-002", "customer changed mind")
	fmt.Println("   Order cancelled")

	// Query: full history for order-001
	fmt.Println("\n4. Event history for order-001:")
	history, _ := svc.GetOrderHistory(ctx, "order-001")
	for _, e := range history {
		fmt.Printf("   v%d [%s] %s — %s\n", e.Version, e.ID, e.Type, e.OccurredAt.Format("15:04:05.000"))
	}

	// Query: full history for order-002
	fmt.Println("\n5. Event history for order-002:")
	history, _ = svc.GetOrderHistory(ctx, "order-002")
	for _, e := range history {
		fmt.Printf("   v%d [%s] %s — %s\n", e.Version, e.ID, e.Type, e.OccurredAt.Format("15:04:05.000"))
	}

	// Query: all cancellation events
	fmt.Println("\n6. All cancellation events:")
	cancellations, _ := store.QueryByType(ctx, "order.cancelled")
	for _, e := range cancellations {
		var p OrderPayload
		json.Unmarshal(e.Payload, &p)
		fmt.Printf("   [%s] %s — reason: %s\n", e.ID, e.AggregateID, p.Reason)
	}

	// Query: all events in a time range (last 10 seconds)
	fmt.Println("\n7. All events in last 10 seconds:")
	now := time.Now()
	recent, _ := store.QueryByTimeRange(ctx, now.Add(-10*time.Second), now)
	for _, e := range recent {
		fmt.Printf("   [%s] %s / %s (v%d)\n", e.ID, e.AggregateID, e.Type, e.Version)
	}

	fmt.Println(strings.Repeat("-", 55))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- Event store interface is append-only: no Update, no Delete")
	fmt.Println("- Events are immutable facts — the repository enforces this")
	fmt.Println("- Queries are by aggregate ID, time range, or event type")
	fmt.Println("- The same repository pattern works for non-CRUD storage")
	fmt.Println("- Business logic (OrderService) doesn't know how events are stored")
}
