// Package main demonstrates fan-out and fan-in patterns using Pub/Sub.
//
// Fan-out: One publisher broadcasts events to multiple subscribers with different
// processing speeds. Shows backpressure handling with buffered channels, timeouts,
// and how slow subscribers affect (or don't affect) fast ones.
//
// Fan-in: Multiple worker goroutines publish results to a single aggregator.
//
// Run: go run ./fanout/
package main

import (
	"context"
	"fmt"
	"math/rand"
	"sync"
	"sync/atomic"
	"time"
)

// --- Message type ---

type Message struct {
	ID        int
	Topic     string
	Payload   string
	Timestamp time.Time
}

// --- Broker with delivery stats ---

type DeliveryStats struct {
	Delivered atomic.Int64
	Dropped   atomic.Int64
	TimedOut  atomic.Int64
}

type Broker struct {
	mu    sync.RWMutex
	subs  map[string][]chan Message
	stats DeliveryStats
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

// PublishWithTimeout sends a message to all subscribers of a topic.
// Uses a per-subscriber timeout to avoid blocking on slow consumers.
func (b *Broker) PublishWithTimeout(topic string, msg Message, timeout time.Duration) {
	b.mu.RLock()
	defer b.mu.RUnlock()

	msg.Topic = topic
	msg.Timestamp = time.Now()

	for _, ch := range b.subs[topic] {
		select {
		case ch <- msg:
			b.stats.Delivered.Add(1)
		case <-time.After(timeout):
			b.stats.TimedOut.Add(1)
		}
	}
}

// PublishNonBlocking sends a message to all subscribers, dropping if buffer is full.
func (b *Broker) PublishNonBlocking(topic string, msg Message) {
	b.mu.RLock()
	defer b.mu.RUnlock()

	msg.Topic = topic
	msg.Timestamp = time.Now()

	for _, ch := range b.subs[topic] {
		select {
		case ch <- msg:
			b.stats.Delivered.Add(1)
		default:
			b.stats.Dropped.Add(1)
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

// --- Fan-Out Demo ---

func demoFanOut() {
	fmt.Println("========================================")
	fmt.Println("  FAN-OUT: One Publisher, Many Subscribers")
	fmt.Println("========================================\n")

	broker := NewBroker()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	var wg sync.WaitGroup

	// Three subscribers with different processing speeds and buffer sizes
	fastSub := broker.Subscribe("events", 5)   // small buffer, fast processor
	mediumSub := broker.Subscribe("events", 10) // medium buffer, medium processor
	slowSub := broker.Subscribe("events", 3)    // tiny buffer, slow processor

	// Fast subscriber: processes immediately
	wg.Add(1)
	go func() {
		defer wg.Done()
		count := 0
		for {
			select {
			case msg, ok := <-fastSub:
				if !ok {
					fmt.Printf("  [Fast]   done, processed %d messages\n", count)
					return
				}
				count++
				fmt.Printf("  [Fast]   msg %d: %s\n", msg.ID, msg.Payload)
				// No delay -- processes instantly
			case <-ctx.Done():
				return
			}
		}
	}()

	// Medium subscriber: 50ms per message
	wg.Add(1)
	go func() {
		defer wg.Done()
		count := 0
		for {
			select {
			case msg, ok := <-mediumSub:
				if !ok {
					fmt.Printf("  [Medium] done, processed %d messages\n", count)
					return
				}
				count++
				fmt.Printf("  [Medium] msg %d: %s\n", msg.ID, msg.Payload)
				time.Sleep(50 * time.Millisecond)
			case <-ctx.Done():
				return
			}
		}
	}()

	// Slow subscriber: 200ms per message -- will experience drops
	wg.Add(1)
	go func() {
		defer wg.Done()
		count := 0
		for {
			select {
			case msg, ok := <-slowSub:
				if !ok {
					fmt.Printf("  [Slow]   done, processed %d messages\n", count)
					return
				}
				count++
				fmt.Printf("  [Slow]   msg %d: %s\n", msg.ID, msg.Payload)
				time.Sleep(200 * time.Millisecond)
			case <-ctx.Done():
				return
			}
		}
	}()

	// Publisher: sends 15 messages rapidly
	time.Sleep(20 * time.Millisecond) // let subscribers start
	fmt.Println("--- Publishing 15 messages ---\n")

	for i := 1; i <= 15; i++ {
		broker.PublishNonBlocking("events", Message{
			ID:      i,
			Payload: fmt.Sprintf("event-%d", i),
		})
		time.Sleep(10 * time.Millisecond) // slight delay between publishes
	}

	// Wait for processing to complete
	time.Sleep(1 * time.Second)

	broker.Close()
	wg.Wait()

	fmt.Printf("\n--- Fan-Out Stats ---\n")
	fmt.Printf("  Delivered: %d\n", broker.stats.Delivered.Load())
	fmt.Printf("  Dropped:   %d\n", broker.stats.Dropped.Load())
	fmt.Printf("  Timed Out: %d\n", broker.stats.TimedOut.Load())
}

// --- Fan-In Demo ---

func demoFanIn() {
	fmt.Println("\n\n========================================")
	fmt.Println("  FAN-IN: Many Publishers, One Subscriber")
	fmt.Println("========================================\n")

	broker := NewBroker()

	// Single aggregator subscribing to "results"
	resultsCh := broker.Subscribe("results", 50)

	var wg sync.WaitGroup

	// Launch 5 worker goroutines, each publishing results
	numWorkers := 5
	for i := 0; i < numWorkers; i++ {
		wg.Add(1)
		go func(workerID int) {
			defer wg.Done()

			// Each worker processes 3 items
			for j := 1; j <= 3; j++ {
				// Simulate variable processing time
				processingTime := time.Duration(rand.Intn(100)+50) * time.Millisecond
				time.Sleep(processingTime)

				broker.PublishNonBlocking("results", Message{
					ID:      workerID*100 + j,
					Payload: fmt.Sprintf("worker-%d-result-%d (took %v)", workerID, j, processingTime),
				})
			}
			fmt.Printf("  [Worker %d] finished all tasks\n", workerID)
		}(i)
	}

	// Aggregator: collect all results
	var collected []string
	var aggWg sync.WaitGroup
	aggWg.Add(1)
	go func() {
		defer aggWg.Done()
		for msg := range resultsCh {
			collected = append(collected, msg.Payload)
			fmt.Printf("  [Aggregator] received: %s\n", msg.Payload)
		}
	}()

	// Wait for all workers to finish, then close broker
	wg.Wait()
	fmt.Println("\n  All workers done. Closing broker...")

	// Brief pause to let any in-flight messages reach the aggregator
	time.Sleep(50 * time.Millisecond)
	broker.Close()
	aggWg.Wait()

	fmt.Printf("\n--- Fan-In Results ---\n")
	fmt.Printf("  Total results collected: %d (expected: %d)\n", len(collected), numWorkers*3)
	fmt.Printf("  Delivered: %d\n", broker.stats.Delivered.Load())
	fmt.Printf("  Dropped:   %d\n", broker.stats.Dropped.Load())
}

// --- Fan-Out with Timeout Demo ---

func demoFanOutWithTimeout() {
	fmt.Println("\n\n========================================")
	fmt.Println("  FAN-OUT WITH TIMEOUT")
	fmt.Println("========================================\n")

	broker := NewBroker()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	var wg sync.WaitGroup

	// Subscriber with tiny buffer (will fill up quickly)
	blockedSub := broker.Subscribe("urgent", 1)

	wg.Add(1)
	go func() {
		defer wg.Done()
		count := 0
		for {
			select {
			case msg, ok := <-blockedSub:
				if !ok {
					fmt.Printf("  [Blocked] done, processed %d messages\n", count)
					return
				}
				count++
				fmt.Printf("  [Blocked] processing msg %d (takes 500ms)...\n", msg.ID)
				time.Sleep(500 * time.Millisecond) // very slow
			case <-ctx.Done():
				return
			}
		}
	}()

	time.Sleep(20 * time.Millisecond)

	fmt.Println("--- Publishing with 50ms timeout ---\n")
	for i := 1; i <= 5; i++ {
		start := time.Now()
		broker.PublishWithTimeout("urgent", Message{
			ID:      i,
			Payload: fmt.Sprintf("urgent-%d", i),
		}, 50*time.Millisecond)
		elapsed := time.Since(start)
		fmt.Printf("  Publish msg %d took %v\n", i, elapsed.Round(time.Millisecond))
	}

	time.Sleep(2 * time.Second)
	broker.Close()
	wg.Wait()

	fmt.Printf("\n--- Timeout Stats ---\n")
	fmt.Printf("  Delivered: %d\n", broker.stats.Delivered.Load())
	fmt.Printf("  Timed Out: %d\n", broker.stats.TimedOut.Load())
}

func main() {
	demoFanOut()
	demoFanIn()
	demoFanOutWithTimeout()
}
