package notification

import (
	"fmt"
	"log"
	"time"
)

// EventType represents different kinds of events in the system.
type EventType string

const (
	UserSignedUp   EventType = "user.signed_up"
	UserLoggedIn   EventType = "user.logged_in"
	OrderPlaced    EventType = "order.placed"
	OrderCompleted EventType = "order.completed"
	InvoiceSent    EventType = "invoice.sent"
)

// Event carries data about something that happened.
type Event struct {
	Type    EventType
	Data    map[string]interface{}
	Time    time.Time
}

// NotificationCenter is the central hub for events.
type NotificationCenter struct {
	observers []Observer
}

// Observer watches for events from the notification center.
type Observer struct {
	Name   string
	Center *NotificationCenter  // reference back to the center
	Types  []EventType
	Action func(Event)
}

// Global singleton -- one center for the whole app.
var DefaultCenter = &NotificationCenter{}

// Subscribe adds an observer to the center.
func (nc *NotificationCenter) Subscribe(name string, types []EventType, action func(Event)) {
	obs := Observer{
		Name:   name,
		Center: nc,
		Types:  types,
		Action: action,
	}
	nc.observers = append(nc.observers, obs)
	log.Printf("observer %q subscribed to %v", name, types)
}

// Publish sends an event to all interested observers.
func (nc *NotificationCenter) Publish(event Event) {
	log.Printf("publishing event: %s", event.Type)
	for i := 0; i < len(nc.observers); i++ {
		obs := nc.observers[i]
		for _, t := range obs.Types {
			if t == event.Type {
				obs.Action(event)
				break
			}
		}
	}
}

// PublishAsync sends an event asynchronously. Useful for non-critical events.
func (nc *NotificationCenter) PublishAsync(event Event) {
	go nc.Publish(event)
}

// NotifyAll sends an event to ALL observers regardless of type subscription.
func (nc *NotificationCenter) NotifyAll(event Event) {
	for _, obs := range nc.observers {
		obs.Action(event)
	}
}

// ObserverCount returns the number of registered observers.
func (nc *NotificationCenter) ObserverCount() int {
	return len(nc.observers)
}

// --- Concrete observer factories ---

// NewEmailNotifier creates an observer that sends emails on user events.
func NewEmailNotifier(center *NotificationCenter) {
	center.Subscribe("email-notifier", []EventType{UserSignedUp, OrderCompleted}, func(e Event) {
		email := e.Data["email"]
		fmt.Printf("[Email] Sending to %v: %s event\n", email, e.Type)
		// Simulate slow email sending
		time.Sleep(2 * time.Second)
	})
}

// NewDashboardUpdater creates an observer that updates the real-time dashboard.
func NewDashboardUpdater(center *NotificationCenter) {
	center.Subscribe("dashboard", []EventType{UserSignedUp, UserLoggedIn, OrderPlaced, OrderCompleted}, func(e Event) {
		fmt.Printf("[Dashboard] Updating for event: %s\n", e.Type)
	})
}

// NewWorkflowTrigger creates an observer that triggers automated workflows.
func NewWorkflowTrigger(center *NotificationCenter) {
	center.Subscribe("workflow", []EventType{OrderPlaced}, func(e Event) {
		fmt.Printf("[Workflow] Triggering fulfillment for order: %v\n", e.Data["order_id"])

		// After processing the order, publish a completion event.
		// This calls Publish from inside a Publish call.
		center.Publish(Event{
			Type: OrderCompleted,
			Data: e.Data,
			Time: time.Now(),
		})
	})
}

// NewAuditLogger creates an observer that logs all events for compliance.
func NewAuditLogger(center *NotificationCenter) {
	// nil types means "subscribe to all events"
	center.Subscribe("audit", nil, func(e Event) {
		fmt.Printf("[Audit] %s: %v\n", e.Type, e.Data)
	})
}

// --- Usage example ---

func Example() {
	center := &NotificationCenter{}

	NewEmailNotifier(center)
	NewDashboardUpdater(center)
	NewWorkflowTrigger(center)
	NewAuditLogger(center)

	center.Publish(Event{
		Type: UserSignedUp,
		Data: map[string]interface{}{
			"user_id": "usr_123",
			"email":   "user@example.com",
		},
		Time: time.Now(),
	})

	center.Publish(Event{
		Type: OrderPlaced,
		Data: map[string]interface{}{
			"order_id": "ord_456",
			"user_id":  "usr_123",
		},
		Time: time.Now(),
	})
}
