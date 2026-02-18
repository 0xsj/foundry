// Package main demonstrates an in-process message broker with topic-based routing,
// wildcard subscriptions, thread-safe operations, and graceful shutdown.
//
// Run: go run ./broker/
package main

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"time"
)

// --- Message types ---

// Message represents a payload delivered through the broker.
type Message struct {
	ID        string
	Topic     string
	Payload   []byte
	Timestamp time.Time
	Metadata  map[string]string
}

// --- Subscription ---

// Subscription represents a subscriber's connection to the broker.
// It holds the channel for receiving messages and metadata for management.
type Subscription struct {
	id      string
	pattern string          // topic pattern (may include wildcards)
	ch      chan Message     // delivery channel
	done    chan struct{}    // closed when subscription is cancelled
}

// Ch returns the receive-only channel for reading messages.
func (s *Subscription) Ch() <-chan Message {
	return s.ch
}

// --- Broker ---

// Broker is a thread-safe, in-process message broker supporting topic-based
// routing with wildcard patterns. It provides at-most-once delivery semantics
// with non-blocking sends to subscriber channels.
type Broker struct {
	mu            sync.RWMutex
	subscriptions map[string]*Subscription // subscription ID -> subscription
	closed        bool
	nextID        int
}

// NewBroker creates a new message broker ready to accept subscriptions.
func NewBroker() *Broker {
	return &Broker{
		subscriptions: make(map[string]*Subscription),
	}
}

// Subscribe registers a new subscriber for the given topic pattern.
// The pattern can include wildcards:
//   - "order.created"  -- exact match
//   - "order.*"        -- matches any single segment (e.g., order.created, order.shipped)
//   - "order.>"        -- matches any number of segments (e.g., order.created, order.us.east.created)
//
// bufSize controls the channel buffer size (backpressure tolerance).
// Returns a Subscription that the caller reads from via sub.Ch().
func (b *Broker) Subscribe(pattern string, bufSize int) (*Subscription, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	if b.closed {
		return nil, fmt.Errorf("broker is closed")
	}

	b.nextID++
	id := fmt.Sprintf("sub-%d", b.nextID)

	sub := &Subscription{
		id:      id,
		pattern: pattern,
		ch:      make(chan Message, bufSize),
		done:    make(chan struct{}),
	}

	b.subscriptions[id] = sub
	fmt.Printf("[Broker] %s subscribed to %q (buffer=%d)\n", id, pattern, bufSize)
	return sub, nil
}

// Unsubscribe removes a subscription and closes its channel.
func (b *Broker) Unsubscribe(sub *Subscription) {
	b.mu.Lock()
	defer b.mu.Unlock()

	if _, exists := b.subscriptions[sub.id]; exists {
		close(sub.ch)
		close(sub.done)
		delete(b.subscriptions, sub.id)
		fmt.Printf("[Broker] %s unsubscribed from %q\n", sub.id, sub.pattern)
	}
}

// Publish sends a message to all subscribers whose pattern matches the topic.
// Uses non-blocking sends: if a subscriber's buffer is full, the message is dropped
// for that subscriber (at-most-once delivery).
// Returns the number of subscribers that received the message and the number that dropped it.
func (b *Broker) Publish(topic string, payload []byte, metadata map[string]string) (delivered, dropped int) {
	b.mu.RLock()
	defer b.mu.RUnlock()

	if b.closed {
		return 0, 0
	}

	msg := Message{
		ID:        fmt.Sprintf("msg-%d", time.Now().UnixNano()),
		Topic:     topic,
		Payload:   payload,
		Timestamp: time.Now(),
		Metadata:  metadata,
	}

	for _, sub := range b.subscriptions {
		if !matchTopic(sub.pattern, topic) {
			continue
		}

		select {
		case sub.ch <- msg:
			delivered++
		default:
			dropped++
		}
	}

	return delivered, dropped
}

// Close shuts down the broker: closes all subscriber channels and prevents new publishes.
func (b *Broker) Close() {
	b.mu.Lock()
	defer b.mu.Unlock()

	if b.closed {
		return
	}
	b.closed = true

	for id, sub := range b.subscriptions {
		close(sub.ch)
		close(sub.done)
		delete(b.subscriptions, id)
	}

	fmt.Println("[Broker] closed")
}

// --- Topic matching ---

// matchTopic checks if a subscription pattern matches a published topic.
//
// Supports:
//   - Exact match:    "order.created" matches "order.created"
//   - Single wildcard: "order.*" matches "order.created" but NOT "order.us.created"
//   - Multi wildcard:  "order.>" matches "order.created" AND "order.us.created"
func matchTopic(pattern, topic string) bool {
	if pattern == topic {
		return true
	}

	patternParts := strings.Split(pattern, ".")
	topicParts := strings.Split(topic, ".")

	for i, part := range patternParts {
		if part == ">" {
			// ">" matches everything from this point forward
			return i < len(topicParts)
		}
		if i >= len(topicParts) {
			return false
		}
		if part == "*" {
			continue // matches any single segment
		}
		if part != topicParts[i] {
			return false
		}
	}

	return len(patternParts) == len(topicParts)
}

// --- Main: demonstrate the broker ---

func main() {
	broker := NewBroker()

	// Create a context for coordinating shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// --- Subscribe ---

	// Exact topic subscriber: only order.created
	orderSub, _ := broker.Subscribe("order.created", 10)

	// Wildcard subscriber: all order events
	allOrdersSub, _ := broker.Subscribe("order.*", 10)

	// Multi-level wildcard: everything
	auditSub, _ := broker.Subscribe(">", 20)

	// --- Start subscriber goroutines ---

	var wg sync.WaitGroup

	// Subscriber 1: handles new orders
	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			select {
			case msg, ok := <-orderSub.Ch():
				if !ok {
					fmt.Println("[OrderHandler] channel closed, exiting")
					return
				}
				fmt.Printf("[OrderHandler] received %s: %s\n", msg.Topic, string(msg.Payload))
			case <-ctx.Done():
				return
			}
		}
	}()

	// Subscriber 2: monitors all order activity
	wg.Add(1)
	go func() {
		defer wg.Done()
		count := 0
		for {
			select {
			case msg, ok := <-allOrdersSub.Ch():
				if !ok {
					fmt.Printf("[OrderMonitor] channel closed, processed %d messages\n", count)
					return
				}
				count++
				fmt.Printf("[OrderMonitor] [%d] %s: %s\n", count, msg.Topic, string(msg.Payload))
			case <-ctx.Done():
				return
			}
		}
	}()

	// Subscriber 3: audit log for everything
	wg.Add(1)
	go func() {
		defer wg.Done()
		for {
			select {
			case msg, ok := <-auditSub.Ch():
				if !ok {
					fmt.Println("[AuditLog] channel closed, exiting")
					return
				}
				fmt.Printf("[AuditLog] %s | topic=%s | %s\n",
					msg.Timestamp.Format("15:04:05.000"), msg.Topic, string(msg.Payload))
			case <-ctx.Done():
				return
			}
		}
	}()

	// Give goroutines a moment to start
	time.Sleep(50 * time.Millisecond)

	// --- Publish events ---

	fmt.Println("\n=== Publishing order.created ===")
	d, dr := broker.Publish("order.created",
		[]byte(`{"order_id":"ord-001","total":149.99}`),
		map[string]string{"source": "api"},
	)
	fmt.Printf("  delivered=%d dropped=%d\n", d, dr)

	time.Sleep(50 * time.Millisecond)

	fmt.Println("\n=== Publishing order.shipped ===")
	d, dr = broker.Publish("order.shipped",
		[]byte(`{"order_id":"ord-001","carrier":"fedex"}`),
		nil,
	)
	fmt.Printf("  delivered=%d dropped=%d\n", d, dr)

	time.Sleep(50 * time.Millisecond)

	fmt.Println("\n=== Publishing payment.failed (no order subscribers match) ===")
	d, dr = broker.Publish("payment.failed",
		[]byte(`{"order_id":"ord-001","reason":"declined"}`),
		nil,
	)
	fmt.Printf("  delivered=%d dropped=%d\n", d, dr)

	time.Sleep(50 * time.Millisecond)

	// --- Unsubscribe one subscriber ---

	fmt.Println("\n=== Unsubscribing OrderHandler ===")
	broker.Unsubscribe(orderSub)

	fmt.Println("\n=== Publishing order.created (after unsubscribe) ===")
	d, dr = broker.Publish("order.created",
		[]byte(`{"order_id":"ord-002","total":89.00}`),
		nil,
	)
	fmt.Printf("  delivered=%d dropped=%d\n", d, dr)

	time.Sleep(50 * time.Millisecond)

	// --- Shutdown ---

	fmt.Println("\n=== Shutting down broker ===")
	broker.Close()

	// Wait for all subscriber goroutines to exit
	wg.Wait()
	fmt.Println("\nAll subscribers exited cleanly.")
}
