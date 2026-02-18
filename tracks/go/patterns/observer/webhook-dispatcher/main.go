// Package main demonstrates the Observer pattern using interface-based observers
// with concurrent delivery. A webhook dispatcher receives events and fans them
// out to registered webhook endpoints, delivering payloads over HTTP.
//
// Run: go run ./webhook-dispatcher/
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"sync"
	"time"
)

// --- Event types ---

// EventType identifies the kind of event being dispatched.
type EventType string

const (
	OrderCreated   EventType = "order.created"
	OrderShipped   EventType = "order.shipped"
	OrderCancelled EventType = "order.cancelled"
	PaymentFailed  EventType = "payment.failed"
)

// WebhookEvent is the payload delivered to observers.
type WebhookEvent struct {
	ID        string    `json:"id"`
	Type      EventType `json:"type"`
	Payload   any       `json:"payload"`
	Timestamp time.Time `json:"timestamp"`
}

// --- Observer interface ---

// WebhookHandler is the observer interface. Any type that can handle
// a webhook event satisfies this contract.
type WebhookHandler interface {
	// Handle processes a webhook event. Returns an error if delivery fails.
	Handle(ctx context.Context, event WebhookEvent) error

	// Endpoint returns a human-readable identifier for this handler.
	Endpoint() string
}

// --- Concrete observers ---

// HTTPWebhook simulates delivering a webhook payload to an HTTP endpoint.
type HTTPWebhook struct {
	URL     string
	Secret  string
	Timeout time.Duration
}

func (h *HTTPWebhook) Endpoint() string { return h.URL }

func (h *HTTPWebhook) Handle(ctx context.Context, event WebhookEvent) error {
	payload, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("marshal event: %w", err)
	}

	// In production, this would be an HTTP POST with HMAC signature.
	// Simulating delivery with a log line.
	log.Printf("[HTTP] POST %s | event=%s | payload=%d bytes", h.URL, event.Type, len(payload))

	// Simulate network latency
	select {
	case <-time.After(50 * time.Millisecond):
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

// SlackNotifier posts a formatted message to a Slack channel.
type SlackNotifier struct {
	Channel   string
	BotToken  string
	OnlyTypes []EventType
}

func (s *SlackNotifier) Endpoint() string { return fmt.Sprintf("slack:#%s", s.Channel) }

func (s *SlackNotifier) Handle(ctx context.Context, event WebhookEvent) error {
	// Filter: only handle configured event types
	if len(s.OnlyTypes) > 0 {
		matched := false
		for _, t := range s.OnlyTypes {
			if t == event.Type {
				matched = true
				break
			}
		}
		if !matched {
			return nil // silently skip events we don't care about
		}
	}

	log.Printf("[Slack] #%s | %s: %v", s.Channel, event.Type, event.Payload)
	return nil
}

// AuditLog records every event to an append-only log for compliance.
type AuditLog struct {
	mu      sync.Mutex
	entries []string
}

func (a *AuditLog) Endpoint() string { return "audit-log" }

func (a *AuditLog) Handle(_ context.Context, event WebhookEvent) error {
	entry := fmt.Sprintf("[%s] %s: %v",
		event.Timestamp.Format(time.RFC3339), event.Type, event.Payload)
	a.mu.Lock()
	a.entries = append(a.entries, entry)
	a.mu.Unlock()
	log.Printf("[Audit] recorded: %s", entry)
	return nil
}

func (a *AuditLog) Entries() []string {
	a.mu.Lock()
	defer a.mu.Unlock()
	out := make([]string, len(a.entries))
	copy(out, a.entries)
	return out
}

// --- Subject: WebhookDispatcher ---

// DeliveryResult captures the outcome of delivering to one observer.
type DeliveryResult struct {
	Endpoint string
	Error    error
	Duration time.Duration
}

// WebhookDispatcher is the subject that manages observers and dispatches events.
// It delivers events concurrently to all registered handlers.
type WebhookDispatcher struct {
	mu       sync.RWMutex
	handlers map[EventType][]WebhookHandler
	timeout  time.Duration
}

func NewWebhookDispatcher(timeout time.Duration) *WebhookDispatcher {
	return &WebhookDispatcher{
		handlers: make(map[EventType][]WebhookHandler),
		timeout:  timeout,
	}
}

// Subscribe registers a handler for one or more event types.
func (d *WebhookDispatcher) Subscribe(handler WebhookHandler, eventTypes ...EventType) {
	d.mu.Lock()
	defer d.mu.Unlock()
	for _, et := range eventTypes {
		d.handlers[et] = append(d.handlers[et], handler)
	}
	log.Printf("[Dispatcher] %s subscribed to %v", handler.Endpoint(), eventTypes)
}

// Unsubscribe removes a handler from all event types.
func (d *WebhookDispatcher) Unsubscribe(handler WebhookHandler) {
	d.mu.Lock()
	defer d.mu.Unlock()
	for et, handlers := range d.handlers {
		for i, h := range handlers {
			if h == handler {
				d.handlers[et] = append(handlers[:i], handlers[i+1:]...)
				break
			}
		}
	}
	log.Printf("[Dispatcher] %s unsubscribed", handler.Endpoint())
}

// Dispatch sends an event to all registered handlers concurrently.
// Returns delivery results for each handler.
func (d *WebhookDispatcher) Dispatch(ctx context.Context, event WebhookEvent) []DeliveryResult {
	// Snapshot handlers under read lock
	d.mu.RLock()
	handlers := make([]WebhookHandler, len(d.handlers[event.Type]))
	copy(handlers, d.handlers[event.Type])
	d.mu.RUnlock()

	if len(handlers) == 0 {
		return nil
	}

	// Deliver concurrently with per-handler timeout
	results := make([]DeliveryResult, len(handlers))
	var wg sync.WaitGroup

	for i, h := range handlers {
		wg.Add(1)
		go func(idx int, handler WebhookHandler) {
			defer wg.Done()
			start := time.Now()

			deliveryCtx, cancel := context.WithTimeout(ctx, d.timeout)
			defer cancel()

			err := handler.Handle(deliveryCtx, event)
			results[idx] = DeliveryResult{
				Endpoint: handler.Endpoint(),
				Error:    err,
				Duration: time.Since(start),
			}
		}(i, h)
	}

	wg.Wait()
	return results
}

// --- Main: wire it all together ---

func main() {
	dispatcher := NewWebhookDispatcher(5 * time.Second)

	// Register observers
	orderAPI := &HTTPWebhook{
		URL:     "https://partner-api.example.com/orders",
		Secret:  "whsec_abc123",
		Timeout: 3 * time.Second,
	}

	slack := &SlackNotifier{
		Channel:   "order-alerts",
		BotToken:  "xoxb-fake-token",
		OnlyTypes: []EventType{OrderCancelled, PaymentFailed}, // only critical events
	}

	audit := &AuditLog{}

	// Subscribe observers to event types
	dispatcher.Subscribe(orderAPI, OrderCreated, OrderShipped, OrderCancelled)
	dispatcher.Subscribe(slack, OrderCreated, OrderShipped, OrderCancelled, PaymentFailed)
	dispatcher.Subscribe(audit, OrderCreated, OrderShipped, OrderCancelled, PaymentFailed)

	fmt.Println("\n--- Dispatching order.created ---")
	results := dispatcher.Dispatch(context.Background(), WebhookEvent{
		ID:        "evt_001",
		Type:      OrderCreated,
		Payload:   map[string]any{"order_id": "ord_789", "total": 149.99},
		Timestamp: time.Now(),
	})
	printResults(results)

	fmt.Println("\n--- Dispatching payment.failed ---")
	results = dispatcher.Dispatch(context.Background(), WebhookEvent{
		ID:        "evt_002",
		Type:      PaymentFailed,
		Payload:   map[string]any{"order_id": "ord_789", "reason": "insufficient_funds"},
		Timestamp: time.Now(),
	})
	printResults(results)

	// Unsubscribe the partner API (they requested removal)
	dispatcher.Unsubscribe(orderAPI)

	fmt.Println("\n--- Dispatching order.cancelled (after unsubscribe) ---")
	results = dispatcher.Dispatch(context.Background(), WebhookEvent{
		ID:        "evt_003",
		Type:      OrderCancelled,
		Payload:   map[string]any{"order_id": "ord_789"},
		Timestamp: time.Now(),
	})
	printResults(results)

	fmt.Println("\n--- Audit Log Entries ---")
	for _, entry := range audit.Entries() {
		fmt.Println("  ", entry)
	}
}

func printResults(results []DeliveryResult) {
	for _, r := range results {
		status := "OK"
		if r.Error != nil {
			status = fmt.Sprintf("FAILED: %v", r.Error)
		}
		fmt.Printf("  %-45s %s (%v)\n", r.Endpoint, status, r.Duration.Round(time.Millisecond))
	}
}
