package debugging

import (
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// TestDataRace_ConcurrentSubscribeAndPublish exposes BUG 1.
// Run with: go test -race -run TestDataRace
//
// The race detector should flag concurrent access to b.handlers
// because Subscribe writes without holding the mutex.
func TestDataRace_ConcurrentSubscribeAndPublish(t *testing.T) {
	bus := NewEventBus()

	var wg sync.WaitGroup

	// Concurrently subscribe handlers
	for i := 0; i < 50; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			bus.Subscribe(PaymentProcessed, func(e Event) {
				// just receive the event
			})
		}()
	}

	// Concurrently publish events
	for i := 0; i < 50; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			bus.Publish(Event{
				Type:      PaymentProcessed,
				Payload:   map[string]string{"amount": "100"},
				Timestamp: time.Now(),
			})
		}()
	}

	wg.Wait()
}

// TestDeadlock_HandlerSubscribesDuringPublish exposes BUG 2.
// Once BUG 1 is fixed (adding mutex to Subscribe), this test will deadlock
// because Publish holds the mutex and the handler calls Subscribe which also
// needs the mutex. The fix for BUG 2 requires releasing the lock before
// calling handlers (copy-on-read pattern).
func TestDeadlock_HandlerSubscribesDuringPublish(t *testing.T) {
	bus := NewEventBus()

	// Register a handler that tries to subscribe another handler during notification.
	// This is a realistic scenario: a fraud detection handler might want to add
	// additional monitoring when suspicious activity is detected.
	bus.Subscribe(PaymentProcessed, func(e Event) {
		bus.Subscribe(PaymentFailed, func(e Event) {
			// fraud monitoring handler
		})
	})

	done := make(chan struct{})
	go func() {
		bus.Publish(Event{
			Type:      PaymentProcessed,
			Payload:   map[string]string{"amount": "10000"},
			Timestamp: time.Now(),
		})
		close(done)
	}()

	select {
	case <-done:
		// Success: no deadlock
	case <-time.After(2 * time.Second):
		t.Fatal("deadlock detected: Publish did not complete within 2 seconds")
	}
}

// TestGoroutineLeak_UnsubscribeDoesNotCloseChannel exposes BUG 3.
// After unsubscribing, the goroutine reading from the channel should exit.
func TestGoroutineLeak_UnsubscribeDoesNotCloseChannel(t *testing.T) {
	bus := NewEventBus()

	sub := bus.SubscribeChannel(PaymentProcessed, 10)

	// Start a goroutine that reads from the subscriber channel.
	// In production, this would be a long-running consumer.
	var exited atomic.Bool
	go func() {
		for range sub.Events {
			// process events
		}
		// This should execute after the channel is closed.
		exited.Store(true)
	}()

	// Send one event
	bus.Publish(Event{
		Type:      PaymentProcessed,
		Payload:   map[string]string{"amount": "50"},
		Timestamp: time.Now(),
	})

	// Unsubscribe -- the goroutine should exit
	bus.UnsubscribeChannel(PaymentProcessed, sub)

	// Give the goroutine time to exit
	time.Sleep(100 * time.Millisecond)

	if !exited.Load() {
		t.Error("goroutine leak: subscriber goroutine did not exit after unsubscribe")
	}
}

// TestNilCallbackPanic exposes BUG 4.
// When SubscribeWithFilter is called with a nil filter, the wrapped handler is nil.
// Calling a nil function panics.
func TestNilCallbackPanic(t *testing.T) {
	bus := NewEventBus()

	// Subscribe with nil filter -- this should still work
	// (no filter means accept all events)
	handler := func(e Event) {
		// process event
	}
	bus.SubscribeWithFilter(PaymentProcessed, nil, handler)

	// This should NOT panic
	defer func() {
		if r := recover(); r != nil {
			t.Fatalf("unexpected panic: %v", r)
		}
	}()

	bus.Publish(Event{
		Type:      PaymentProcessed,
		Payload:   map[string]string{"amount": "25"},
		Timestamp: time.Now(),
	})
}

// TestNilHandlerPanic also exposes BUG 4 from a different angle.
// When both filter and handler interact, a nil handler inside the filter
// wrapper causes a panic.
func TestNilHandlerPanic(t *testing.T) {
	bus := NewEventBus()

	// Subscribe with a filter but nil handler -- should be caught
	bus.SubscribeWithFilter(PaymentProcessed, func(e Event) bool {
		return true
	}, nil)

	defer func() {
		if r := recover(); r != nil {
			t.Fatalf("unexpected panic from nil handler: %v", r)
		}
	}()

	bus.Publish(Event{
		Type:      PaymentProcessed,
		Payload:   map[string]string{"amount": "75"},
		Timestamp: time.Now(),
	})
}

// TestAllBugsFixed verifies the system works correctly after all fixes.
func TestAllBugsFixed(t *testing.T) {
	bus := NewEventBus()

	var received atomic.Int64

	// Subscribe a normal handler
	bus.Subscribe(PaymentProcessed, func(e Event) {
		received.Add(1)
	})

	// Subscribe a handler that subscribes during notification (no deadlock)
	bus.Subscribe(PaymentProcessed, func(e Event) {
		bus.Subscribe(PaymentFailed, func(e Event) {
			received.Add(1)
		})
	})

	// Subscribe a channel subscriber
	sub := bus.SubscribeChannel(PaymentProcessed, 10)
	var chanExited atomic.Bool
	go func() {
		for range sub.Events {
			received.Add(1)
		}
		chanExited.Store(true)
	}()

	// Subscribe with nil filter (no panic)
	bus.SubscribeWithFilter(PaymentProcessed, nil, func(e Event) {
		received.Add(1)
	})

	// Publish an event
	done := make(chan struct{})
	go func() {
		bus.Publish(Event{
			Type:      PaymentProcessed,
			Payload:   map[string]string{"amount": "100"},
			Timestamp: time.Now(),
		})
		close(done)
	}()

	select {
	case <-done:
		// No deadlock
	case <-time.After(2 * time.Second):
		t.Fatal("deadlock: Publish did not complete")
	}

	// Wait for channel delivery
	time.Sleep(50 * time.Millisecond)

	// Unsubscribe channel -- goroutine should exit
	bus.UnsubscribeChannel(PaymentProcessed, sub)
	time.Sleep(100 * time.Millisecond)

	if !chanExited.Load() {
		t.Error("channel subscriber goroutine did not exit")
	}

	// Verify events were received
	count := received.Load()
	if count < 3 {
		t.Errorf("expected at least 3 handler invocations, got %d", count)
	}
}
