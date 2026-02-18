// Package debugging implements an event notification system for a payment
// processing service. There are four bugs in this file related to concurrency,
// lifecycle management, and nil safety.
package debugging

import (
	"fmt"
	"sync"
	"time"
)

// EventType identifies the kind of event.
type EventType string

const (
	PaymentProcessed EventType = "payment.processed"
	PaymentFailed    EventType = "payment.failed"
	RefundIssued     EventType = "refund.issued"
)

// Event is the payload delivered to handlers.
type Event struct {
	Type      EventType
	Payload   map[string]string
	Timestamp time.Time
}

// HandlerFunc is a callback that handles events.
type HandlerFunc func(Event)

// ChannelSubscriber receives events via a channel.
type ChannelSubscriber struct {
	ID     string
	Events chan Event
}

// EventBus manages handlers and channel subscribers for events.
type EventBus struct {
	mu       sync.Mutex
	handlers map[EventType][]HandlerFunc
	chanSubs map[EventType][]*ChannelSubscriber
}

func NewEventBus() *EventBus {
	return &EventBus{
		handlers: make(map[EventType][]HandlerFunc),
		chanSubs: make(map[EventType][]*ChannelSubscriber),
	}
}

// Subscribe adds a handler for an event type.
func (b *EventBus) Subscribe(eventType EventType, handler HandlerFunc) {
	b.handlers[eventType] = append(b.handlers[eventType], handler)
}

// Publish sends an event to all handlers and channel subscribers.
func (b *EventBus) Publish(event Event) {
	b.mu.Lock()
	defer b.mu.Unlock()

	for _, h := range b.handlers[event.Type] {
		h(event)
	}

	for _, sub := range b.chanSubs[event.Type] {
		sub.Events <- event
	}
}

// SubscribeChannel creates a channel subscriber for an event type.
func (b *EventBus) SubscribeChannel(eventType EventType, bufferSize int) *ChannelSubscriber {
	b.mu.Lock()
	defer b.mu.Unlock()

	sub := &ChannelSubscriber{
		ID:     fmt.Sprintf("sub-%d", time.Now().UnixNano()),
		Events: make(chan Event, bufferSize),
	}
	b.chanSubs[eventType] = append(b.chanSubs[eventType], sub)
	return sub
}

// UnsubscribeChannel removes a channel subscriber.
func (b *EventBus) UnsubscribeChannel(eventType EventType, sub *ChannelSubscriber) {
	b.mu.Lock()
	defer b.mu.Unlock()

	subs := b.chanSubs[eventType]
	for i, s := range subs {
		if s.ID == sub.ID {
			b.chanSubs[eventType] = append(subs[:i], subs[i+1:]...)
			return
		}
	}
}

// SubscribeWithFilter adds a handler that only processes events matching a filter.
// If filter is nil, the handler should receive all events.
func (b *EventBus) SubscribeWithFilter(eventType EventType, filter func(Event) bool, handler HandlerFunc) {
	var wrappedHandler HandlerFunc
	if filter != nil {
		wrappedHandler = func(e Event) {
			if filter(e) {
				handler(e)
			}
		}
	}
	b.handlers[eventType] = append(b.handlers[eventType], wrappedHandler)
}
