// Package main demonstrates an event-driven pipeline where components communicate
// exclusively through a Pub/Sub broker. Each stage reads from one topic and publishes
// to the next, creating a loosely coupled processing pipeline.
//
// Pipeline: Ingester -> Validator -> Enricher -> Storage
//
// Run: go run ./pipeline/
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"sync"
	"time"
)

// --- Message and Broker (simplified) ---

type Message struct {
	ID        string
	Topic     string
	Payload   []byte
	Timestamp time.Time
}

type Broker struct {
	mu   sync.RWMutex
	subs map[string][]chan Message
}

func NewBroker() *Broker {
	return &Broker{subs: make(map[string][]chan Message)}
}

func (b *Broker) Subscribe(topic string, bufSize int) <-chan Message {
	b.mu.Lock()
	defer b.mu.Unlock()
	ch := make(chan Message, bufSize)
	b.subs[topic] = append(b.subs[topic], ch)
	return ch
}

func (b *Broker) Publish(topic string, payload []byte) {
	b.mu.RLock()
	defer b.mu.RUnlock()

	msg := Message{
		ID:        fmt.Sprintf("msg-%d", time.Now().UnixNano()),
		Topic:     topic,
		Payload:   payload,
		Timestamp: time.Now(),
	}

	for _, ch := range b.subs[topic] {
		select {
		case ch <- msg:
		default:
			fmt.Printf("[Broker] WARN: dropped message on topic %q\n", topic)
		}
	}
}

func (b *Broker) Close() {
	b.mu.Lock()
	defer b.mu.Unlock()
	for _, chs := range b.subs {
		for _, ch := range chs {
			close(ch)
		}
	}
}

// --- Domain types ---

// WebhookEvent represents an incoming webhook that flows through the pipeline.
type WebhookEvent struct {
	EventID   string            `json:"event_id"`
	Source    string            `json:"source"`
	Type     string            `json:"type"`
	Payload  map[string]string `json:"payload"`
	Valid    bool              `json:"valid,omitempty"`
	Enriched map[string]string `json:"enriched,omitempty"`
	Error    string            `json:"error,omitempty"`
}

// --- Pipeline stages ---

// Ingester simulates receiving raw webhook payloads from external sources.
// It publishes raw events to the "webhook.raw" topic.
func Ingester(ctx context.Context, broker *Broker, events []WebhookEvent) {
	for _, evt := range events {
		select {
		case <-ctx.Done():
			return
		default:
		}

		data, _ := json.Marshal(evt)
		fmt.Printf("[Ingester] received event %s from %s\n", evt.EventID, evt.Source)
		broker.Publish("webhook.raw", data)

		// Simulate real-time ingestion delay
		time.Sleep(30 * time.Millisecond)
	}
	fmt.Println("[Ingester] all events ingested")
}

// Validator reads from "webhook.raw", validates the event, and publishes:
//   - Valid events to "webhook.validated"
//   - Invalid events to "webhook.rejected"
func Validator(ctx context.Context, broker *Broker, input <-chan Message, wg *sync.WaitGroup) {
	defer wg.Done()

	for {
		select {
		case msg, ok := <-input:
			if !ok {
				fmt.Println("[Validator] input closed, exiting")
				return
			}

			var evt WebhookEvent
			if err := json.Unmarshal(msg.Payload, &evt); err != nil {
				fmt.Printf("[Validator] failed to parse event: %v\n", err)
				continue
			}

			// Validation rules
			valid := true
			var reason string

			if evt.Source == "" {
				valid = false
				reason = "missing source"
			} else if evt.Type == "" {
				valid = false
				reason = "missing event type"
			} else if strings.Contains(evt.Source, "malicious") {
				valid = false
				reason = "blocked source"
			}

			if valid {
				evt.Valid = true
				data, _ := json.Marshal(evt)
				fmt.Printf("[Validator] %s PASSED validation\n", evt.EventID)
				broker.Publish("webhook.validated", data)
			} else {
				evt.Error = reason
				data, _ := json.Marshal(evt)
				fmt.Printf("[Validator] %s REJECTED: %s\n", evt.EventID, reason)
				broker.Publish("webhook.rejected", data)
			}

		case <-ctx.Done():
			return
		}
	}
}

// Enricher reads from "webhook.validated", adds metadata, and publishes to "webhook.enriched".
func Enricher(ctx context.Context, broker *Broker, input <-chan Message, wg *sync.WaitGroup) {
	defer wg.Done()

	// Simulated lookup table (in production: database, cache, or API call)
	sourceMetadata := map[string]map[string]string{
		"github": {
			"provider":  "GitHub",
			"tier":      "enterprise",
			"region":    "us-east-1",
		},
		"stripe": {
			"provider":  "Stripe",
			"tier":      "premium",
			"region":    "us-west-2",
		},
	}

	for {
		select {
		case msg, ok := <-input:
			if !ok {
				fmt.Println("[Enricher] input closed, exiting")
				return
			}

			var evt WebhookEvent
			if err := json.Unmarshal(msg.Payload, &evt); err != nil {
				continue
			}

			// Enrich with metadata lookup
			evt.Enriched = make(map[string]string)
			if meta, ok := sourceMetadata[evt.Source]; ok {
				for k, v := range meta {
					evt.Enriched[k] = v
				}
			}
			evt.Enriched["processed_at"] = time.Now().Format(time.RFC3339)

			data, _ := json.Marshal(evt)
			fmt.Printf("[Enricher] %s enriched with %d fields\n", evt.EventID, len(evt.Enriched))
			broker.Publish("webhook.enriched", data)

		case <-ctx.Done():
			return
		}
	}
}

// Storage reads from "webhook.enriched" and persists events.
// It also reads from "webhook.rejected" to store rejected events for analysis.
func Storage(ctx context.Context, enrichedInput <-chan Message, rejectedInput <-chan Message, wg *sync.WaitGroup) {
	defer wg.Done()

	stored := 0
	rejected := 0

	for {
		select {
		case msg, ok := <-enrichedInput:
			if !ok {
				enrichedInput = nil // prevent busy-looping on closed channel
				if rejectedInput == nil {
					fmt.Printf("[Storage] all channels closed, stored=%d rejected=%d\n", stored, rejected)
					return
				}
				continue
			}

			var evt WebhookEvent
			if err := json.Unmarshal(msg.Payload, &evt); err != nil {
				continue
			}
			stored++
			fmt.Printf("[Storage] STORED %s (source=%s, type=%s, enriched=%v)\n",
				evt.EventID, evt.Source, evt.Type, evt.Enriched)

		case msg, ok := <-rejectedInput:
			if !ok {
				rejectedInput = nil
				if enrichedInput == nil {
					fmt.Printf("[Storage] all channels closed, stored=%d rejected=%d\n", stored, rejected)
					return
				}
				continue
			}

			var evt WebhookEvent
			if err := json.Unmarshal(msg.Payload, &evt); err != nil {
				continue
			}
			rejected++
			fmt.Printf("[Storage] REJECTED %s logged (reason=%s)\n", evt.EventID, evt.Error)

		case <-ctx.Done():
			fmt.Printf("[Storage] shutdown, stored=%d rejected=%d\n", stored, rejected)
			return
		}
	}
}

// --- Main: wire the pipeline together ---

func main() {
	broker := NewBroker()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	var wg sync.WaitGroup

	// Subscribe each stage to its input topic
	rawCh := broker.Subscribe("webhook.raw", 20)
	validatedCh := broker.Subscribe("webhook.validated", 20)
	enrichedCh := broker.Subscribe("webhook.enriched", 20)
	rejectedCh := broker.Subscribe("webhook.rejected", 20)

	// Start pipeline stages (reverse order so subscribers are ready before publishers)
	wg.Add(1)
	go Storage(ctx, enrichedCh, rejectedCh, &wg)

	wg.Add(1)
	go Enricher(ctx, broker, validatedCh, &wg)

	wg.Add(1)
	go Validator(ctx, broker, rawCh, &wg)

	// Simulate incoming webhook events
	events := []WebhookEvent{
		{EventID: "evt-001", Source: "github", Type: "push", Payload: map[string]string{"repo": "foundry", "branch": "main"}},
		{EventID: "evt-002", Source: "stripe", Type: "payment.completed", Payload: map[string]string{"amount": "9900", "currency": "usd"}},
		{EventID: "evt-003", Source: "malicious-bot", Type: "spam", Payload: map[string]string{"content": "buy stuff"}},
		{EventID: "evt-004", Source: "", Type: "unknown", Payload: map[string]string{}}, // missing source
		{EventID: "evt-005", Source: "github", Type: "pull_request.merged", Payload: map[string]string{"pr": "42", "repo": "foundry"}},
	}

	fmt.Println("=== Pipeline Starting ===\n")

	// Ingester runs in foreground (blocks until all events are sent)
	Ingester(ctx, broker, events)

	// Give the pipeline time to process all events through all stages
	time.Sleep(500 * time.Millisecond)

	// Shut down: close broker channels, which cascades through the pipeline
	fmt.Println("\n=== Shutting Down ===")
	broker.Close()
	wg.Wait()

	fmt.Println("\n=== Pipeline Complete ===")
}
