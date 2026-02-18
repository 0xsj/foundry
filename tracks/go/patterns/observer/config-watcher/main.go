// Package main demonstrates the Observer pattern using channel-based fan-out.
// A configuration watcher monitors a config source for changes and notifies
// all subscribers through dedicated channels. Each subscriber processes
// config updates independently at its own pace.
//
// Run: go run ./config-watcher/
package main

import (
	"context"
	"fmt"
	"log"
	"sync"
	"time"
)

// --- Config types ---

// Config represents application configuration that can change at runtime.
type Config struct {
	RateLimitRPS   int           `json:"rate_limit_rps"`
	CacheTTL       time.Duration `json:"cache_ttl"`
	FeatureFlags   map[string]bool
	TLSCertPath    string
	MaintenanceMode bool
}

// ConfigChange describes what changed between two config versions.
type ConfigChange struct {
	Previous  Config
	Current   Config
	ChangedAt time.Time
	Fields    []string // names of fields that changed
}

// --- Subscriber ---

// Subscriber receives config changes on a buffered channel.
// The Quit channel signals the subscriber to stop processing.
type Subscriber struct {
	ID      string
	Changes <-chan ConfigChange // read-only channel for the subscriber
	changes chan ConfigChange   // internal writable channel
	quit    chan struct{}
}

// --- ConfigWatcher: the subject ---

// ConfigWatcher polls a config source and notifies subscribers when changes occur.
type ConfigWatcher struct {
	mu          sync.RWMutex
	subscribers []*Subscriber
	current     Config
	nextID      int

	pollInterval time.Duration
	configLoader func() (Config, error) // function that loads current config
}

func NewConfigWatcher(loader func() (Config, error), pollInterval time.Duration) (*ConfigWatcher, error) {
	initial, err := loader()
	if err != nil {
		return nil, fmt.Errorf("loading initial config: %w", err)
	}
	return &ConfigWatcher{
		current:      initial,
		pollInterval: pollInterval,
		configLoader: loader,
	}, nil
}

// Subscribe creates a new subscriber with a buffered channel.
// The bufferSize determines how many unprocessed changes can queue up
// before the watcher starts dropping updates for this subscriber.
func (w *ConfigWatcher) Subscribe(name string, bufferSize int) *Subscriber {
	ch := make(chan ConfigChange, bufferSize)
	sub := &Subscriber{
		ID:      fmt.Sprintf("%s-%d", name, w.nextID),
		Changes: ch,
		changes: ch,
		quit:    make(chan struct{}),
	}

	w.mu.Lock()
	w.subscribers = append(w.subscribers, sub)
	w.nextID++
	w.mu.Unlock()

	log.Printf("[ConfigWatcher] subscriber %q registered (buffer=%d)", sub.ID, bufferSize)
	return sub
}

// Unsubscribe removes a subscriber and closes its channels.
func (w *ConfigWatcher) Unsubscribe(sub *Subscriber) {
	w.mu.Lock()
	defer w.mu.Unlock()

	for i, s := range w.subscribers {
		if s.ID == sub.ID {
			w.subscribers = append(w.subscribers[:i], w.subscribers[i+1:]...)
			close(s.quit)
			close(s.changes)
			log.Printf("[ConfigWatcher] subscriber %q removed", sub.ID)
			return
		}
	}
}

// Current returns the current config snapshot.
func (w *ConfigWatcher) Current() Config {
	w.mu.RLock()
	defer w.mu.RUnlock()
	return w.current
}

// Watch starts the polling loop. It blocks until the context is cancelled.
func (w *ConfigWatcher) Watch(ctx context.Context) error {
	ticker := time.NewTicker(w.pollInterval)
	defer ticker.Stop()

	log.Printf("[ConfigWatcher] watching for changes every %v", w.pollInterval)

	for {
		select {
		case <-ticker.C:
			newConfig, err := w.configLoader()
			if err != nil {
				log.Printf("[ConfigWatcher] error loading config: %v", err)
				continue // don't crash on transient errors
			}

			changed := detectChanges(w.current, newConfig)
			if len(changed) == 0 {
				continue // no changes
			}

			change := ConfigChange{
				Previous:  w.current,
				Current:   newConfig,
				ChangedAt: time.Now(),
				Fields:    changed,
			}

			w.mu.Lock()
			w.current = newConfig
			w.mu.Unlock()

			w.notify(change)

		case <-ctx.Done():
			log.Printf("[ConfigWatcher] shutting down: %v", ctx.Err())
			w.closeAll()
			return ctx.Err()
		}
	}
}

// notify fans out a config change to all subscribers.
func (w *ConfigWatcher) notify(change ConfigChange) {
	w.mu.RLock()
	subs := make([]*Subscriber, len(w.subscribers))
	copy(subs, w.subscribers)
	w.mu.RUnlock()

	for _, sub := range subs {
		select {
		case sub.changes <- change:
			log.Printf("[ConfigWatcher] notified %s: fields=%v", sub.ID, change.Fields)
		default:
			// Buffer full -- subscriber is processing too slowly.
			// In production, you might increment a dropped_events metric here.
			log.Printf("[ConfigWatcher] WARNING: dropped change for slow subscriber %s", sub.ID)
		}
	}
}

// closeAll closes all subscriber channels on shutdown.
func (w *ConfigWatcher) closeAll() {
	w.mu.Lock()
	defer w.mu.Unlock()
	for _, sub := range w.subscribers {
		close(sub.quit)
		close(sub.changes)
	}
	w.subscribers = nil
}

// detectChanges compares two configs and returns the names of changed fields.
func detectChanges(old, new Config) []string {
	var changed []string
	if old.RateLimitRPS != new.RateLimitRPS {
		changed = append(changed, "RateLimitRPS")
	}
	if old.CacheTTL != new.CacheTTL {
		changed = append(changed, "CacheTTL")
	}
	if old.TLSCertPath != new.TLSCertPath {
		changed = append(changed, "TLSCertPath")
	}
	if old.MaintenanceMode != new.MaintenanceMode {
		changed = append(changed, "MaintenanceMode")
	}
	// Feature flags: check for any difference
	if !mapsEqual(old.FeatureFlags, new.FeatureFlags) {
		changed = append(changed, "FeatureFlags")
	}
	return changed
}

func mapsEqual(a, b map[string]bool) bool {
	if len(a) != len(b) {
		return false
	}
	for k, v := range a {
		if bv, ok := b[k]; !ok || bv != v {
			return false
		}
	}
	return true
}

// --- Subscriber processors ---

// runRateLimiter reacts to rate limit config changes.
func runRateLimiter(sub *Subscriber) {
	for {
		select {
		case change, ok := <-sub.Changes:
			if !ok {
				log.Printf("[RateLimiter] channel closed, shutting down")
				return
			}
			for _, f := range change.Fields {
				if f == "RateLimitRPS" {
					log.Printf("[RateLimiter] updating rate limit: %d -> %d RPS",
						change.Previous.RateLimitRPS, change.Current.RateLimitRPS)
				}
			}
		case <-sub.quit:
			log.Printf("[RateLimiter] quit signal received")
			return
		}
	}
}

// runCacheManager reacts to cache TTL and feature flag changes.
func runCacheManager(sub *Subscriber) {
	for {
		select {
		case change, ok := <-sub.Changes:
			if !ok {
				log.Printf("[CacheManager] channel closed, shutting down")
				return
			}
			for _, f := range change.Fields {
				switch f {
				case "CacheTTL":
					log.Printf("[CacheManager] cache TTL changed: %v -> %v",
						change.Previous.CacheTTL, change.Current.CacheTTL)
				case "FeatureFlags":
					log.Printf("[CacheManager] feature flags changed, flushing cache")
				}
			}
		case <-sub.quit:
			log.Printf("[CacheManager] quit signal received")
			return
		}
	}
}

// runTLSReloader reacts to TLS certificate path changes.
func runTLSReloader(sub *Subscriber) {
	for {
		select {
		case change, ok := <-sub.Changes:
			if !ok {
				log.Printf("[TLSReloader] channel closed, shutting down")
				return
			}
			for _, f := range change.Fields {
				if f == "TLSCertPath" {
					log.Printf("[TLSReloader] reloading TLS cert: %s -> %s",
						change.Previous.TLSCertPath, change.Current.TLSCertPath)
				}
			}
		case <-sub.quit:
			log.Printf("[TLSReloader] quit signal received")
			return
		}
	}
}

// --- Main: simulate config changes ---

func main() {
	// Simulate a config source that changes over time.
	configVersion := 0
	configs := []Config{
		{
			RateLimitRPS:    100,
			CacheTTL:        5 * time.Minute,
			FeatureFlags:    map[string]bool{"dark_mode": false, "beta_api": false},
			TLSCertPath:     "/etc/certs/server.pem",
			MaintenanceMode: false,
		},
		{
			RateLimitRPS:    200, // changed
			CacheTTL:        5 * time.Minute,
			FeatureFlags:    map[string]bool{"dark_mode": false, "beta_api": false},
			TLSCertPath:     "/etc/certs/server.pem",
			MaintenanceMode: false,
		},
		{
			RateLimitRPS:    200,
			CacheTTL:        10 * time.Minute, // changed
			FeatureFlags:    map[string]bool{"dark_mode": true, "beta_api": false}, // changed
			TLSCertPath:     "/etc/certs/server.pem",
			MaintenanceMode: false,
		},
		{
			RateLimitRPS:    200,
			CacheTTL:        10 * time.Minute,
			FeatureFlags:    map[string]bool{"dark_mode": true, "beta_api": true},
			TLSCertPath:     "/etc/certs/server-2026.pem", // changed
			MaintenanceMode: false,
		},
	}

	var mu sync.Mutex
	loader := func() (Config, error) {
		mu.Lock()
		defer mu.Unlock()
		cfg := configs[configVersion]
		return cfg, nil
	}

	watcher, err := NewConfigWatcher(loader, 100*time.Millisecond)
	if err != nil {
		log.Fatal(err)
	}

	// Create subscribers with different buffer sizes
	rateLimiterSub := watcher.Subscribe("rate-limiter", 10)
	cacheSub := watcher.Subscribe("cache-manager", 10)
	tlsSub := watcher.Subscribe("tls-reloader", 5)

	// Start subscriber goroutines
	go runRateLimiter(rateLimiterSub)
	go runCacheManager(cacheSub)
	go runTLSReloader(tlsSub)

	// Start watcher in background
	ctx, cancel := context.WithCancel(context.Background())
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		watcher.Watch(ctx)
	}()

	// Simulate config changes over time
	for i := 1; i < len(configs); i++ {
		time.Sleep(250 * time.Millisecond)
		mu.Lock()
		configVersion = i
		mu.Unlock()
		fmt.Printf("\n=== Config version %d applied ===\n", i)
	}

	// Let the last notification propagate
	time.Sleep(200 * time.Millisecond)

	// Unsubscribe TLS reloader (simulating a component shutdown)
	watcher.Unsubscribe(tlsSub)

	// Clean shutdown
	time.Sleep(100 * time.Millisecond)
	cancel()
	wg.Wait()

	fmt.Println("\n=== Config watcher shut down cleanly ===")
}
