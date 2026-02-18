// Package main demonstrates real-world pointer patterns:
// - Constructor returning *T
// - Optional fields with *T
// - Pointer to loop variable gotcha
// - Pointer to interface: rarely what you want
// - sync.Pool for allocation reuse
//
// Run with: go run ./patterns/
package main

import (
	"bytes"
	"fmt"
	"sync"
	"time"
)

// ============================================================================
// CONSTRUCTOR PATTERN: NewXxx returns *T
// ============================================================================

type RateLimiter struct {
	name     string
	limit    int           // requests per window
	window   time.Duration
	burst    int           // max burst above limit
	mu       sync.Mutex
	tokens   int
	lastReset time.Time
}

// NewRateLimiter is the only way to create a valid RateLimiter.
// Returns *RateLimiter because:
//   - All mutating methods use pointer receivers
//   - Callers share the same instance (shared state)
//   - Zero value RateLimiter is not meaningful (missing required fields)
func NewRateLimiter(name string, limit int, window time.Duration) (*RateLimiter, error) {
	if name == "" {
		return nil, fmt.Errorf("rate limiter name is required")
	}
	if limit <= 0 {
		return nil, fmt.Errorf("limit must be positive, got %d", limit)
	}
	if window <= 0 {
		return nil, fmt.Errorf("window must be positive, got %v", window)
	}
	return &RateLimiter{
		name:      name,
		limit:     limit,
		window:    window,
		burst:     limit * 2, // default burst is 2x limit
		tokens:    limit,
		lastReset: time.Now(),
	}, nil
}

func (r *RateLimiter) Allow() bool {
	r.mu.Lock()
	defer r.mu.Unlock()

	if time.Since(r.lastReset) >= r.window {
		r.tokens = r.limit
		r.lastReset = time.Now()
	}

	if r.tokens <= 0 {
		return false
	}
	r.tokens--
	return true
}

func (r *RateLimiter) Name() string { return r.name }

func constructorPattern() {
	fmt.Println("--- constructor pattern ---")

	rl, err := NewRateLimiter("api-gateway", 10, time.Second)
	if err != nil {
		fmt.Printf("error: %v\n", err)
		return
	}

	fmt.Printf("created: %s (limit=%d)\n", rl.Name(), rl.limit)

	// Invalid constructor calls
	_, err = NewRateLimiter("", 10, time.Second)
	fmt.Printf("empty name error: %v\n", err)

	_, err = NewRateLimiter("test", -5, time.Second)
	fmt.Printf("negative limit error: %v\n", err)

	fmt.Println()
}

// ============================================================================
// OPTIONAL FIELDS WITH *T
// *T as "value may not be present" (like TypeScript's T | undefined)
// ============================================================================

type JobConfig struct {
	Name      string
	Command   string
	Schedule  string        // cron expression
	Timeout   *time.Duration // nil = no timeout (run until completion)
	MaxRetries *int          // nil = use system default (3)
	Tags       *[]string     // nil = no tags
	NotifyOn   *string       // nil = no notification
}

func runJob(cfg JobConfig) {
	const defaultMaxRetries = 3
	const defaultTimeout = 5 * time.Minute

	maxRetries := defaultMaxRetries
	if cfg.MaxRetries != nil {
		maxRetries = *cfg.MaxRetries
	}

	timeout := defaultTimeout
	if cfg.Timeout != nil {
		timeout = *cfg.Timeout
	}

	var tagList []string
	if cfg.Tags != nil {
		tagList = *cfg.Tags
	}

	fmt.Printf("job: %s\n", cfg.Name)
	fmt.Printf("  command:    %s\n", cfg.Command)
	fmt.Printf("  schedule:   %s\n", cfg.Schedule)
	fmt.Printf("  timeout:    %v\n", timeout)
	fmt.Printf("  maxRetries: %d\n", maxRetries)
	fmt.Printf("  tags:       %v\n", tagList)
	if cfg.NotifyOn != nil {
		fmt.Printf("  notify:     %s\n", *cfg.NotifyOn)
	} else {
		fmt.Printf("  notify:     (disabled)\n")
	}
}

// Helper: &int(n) in one step
func intPtr(n int) *int { return &n }
func durationPtr(d time.Duration) *time.Duration { return &d }
func stringPtr(s string) *string { return &s }
func stringSlicePtr(ss []string) *[]string { return &ss }

func optionalFields() {
	fmt.Println("--- optional fields with *T ---")

	// Minimal config — all optional fields use defaults
	minimalJob := JobConfig{
		Name:     "cleanup-old-sessions",
		Command:  "/usr/bin/cleanup",
		Schedule: "0 2 * * *", // 2am daily
	}
	runJob(minimalJob)

	fmt.Println()

	// Full config — all optional fields specified
	fullJob := JobConfig{
		Name:       "sync-analytics",
		Command:    "/usr/bin/sync",
		Schedule:   "*/5 * * * *", // every 5 minutes
		Timeout:    durationPtr(30 * time.Second),
		MaxRetries: intPtr(0), // 0 retries — intentional, not "use default"
		Tags:       stringSlicePtr([]string{"analytics", "sync", "critical"}),
		NotifyOn:   stringPtr("ops@example.com"),
	}
	runJob(fullJob)

	fmt.Println()
}

// ============================================================================
// POINTER TO LOOP VARIABLE — CLASSIC GOTCHA
// In Go < 1.22, loop variable is shared across iterations.
// In Go 1.22+, each iteration has its own variable.
// The fix (explicit shadowing) is still good practice for goroutines.
// ============================================================================

func loopVariableGotcha() {
	fmt.Println("--- loop variable gotcha ---")

	// Correct approach: shadow the loop variable to create a new one per iteration
	names := []string{"alice", "bob", "carol"}
	ptrs := make([]*string, len(names))

	for i, name := range names {
		name := name // shadow: new 'name' variable scoped to this iteration
		ptrs[i] = &name
	}

	fmt.Print("correct (shadowed): ")
	for _, p := range ptrs {
		fmt.Printf("%s ", *p)
	}
	fmt.Println()

	// Alternative: use index to take address of slice element directly
	ptrs2 := make([]*string, len(names))
	for i := range names {
		ptrs2[i] = &names[i] // address of the slice element — always unique
	}

	fmt.Print("correct (slice addr): ")
	for _, p := range ptrs2 {
		fmt.Printf("%s ", *p)
	}
	fmt.Println()

	// The goroutine version — where this matters most
	// Without shadowing, all goroutines may capture the same loop variable
	var wg sync.WaitGroup
	results := make(chan string, len(names))

	for _, n := range names {
		n := n // IMPORTANT: shadow before goroutine captures it
		wg.Add(1)
		go func() {
			defer wg.Done()
			results <- fmt.Sprintf("processed: %s", n)
		}()
	}

	wg.Wait()
	close(results)
	for r := range results {
		fmt.Println(r)
	}

	fmt.Println()
}

// ============================================================================
// POINTER TO INTERFACE — RARELY WHAT YOU WANT
// An interface already holds an internal pointer to the concrete value.
// Wrapping in *Interface adds indirection with no benefit,
// and creates the non-nil-interface-with-nil-value trap.
// ============================================================================

type Processor interface {
	Process(data string) (string, error)
}

type JSONProcessor struct{ indent bool }

func (p *JSONProcessor) Process(data string) (string, error) {
	return fmt.Sprintf(`{"data": "%s", "indent": %v}`, data, p.indent), nil
}

func useProcessor(p Processor) string {
	if p == nil {
		return "(nil processor)"
	}
	result, _ := p.Process("hello")
	return result
}

func ptrToInterface() {
	fmt.Println("--- pointer to interface ---")

	proc := &JSONProcessor{indent: true}

	// Correct: pass the concrete *JSONProcessor as Processor
	result := useProcessor(proc)
	fmt.Printf("correct: %s\n", result)

	// Wrong: *Processor — extra pointer level, rarely needed
	var iface Processor = proc
	ifacePtr := &iface // *Processor — almost never what you want
	// To call through *Processor you'd need: (*ifacePtr).Process(...)
	_ = ifacePtr
	fmt.Println("*Processor created — extra level of indirection, not useful")

	// The nil interface trap:
	// A *JSONProcessor that is nil, when assigned to an interface, creates
	// a non-nil interface value (it has type info) whose underlying pointer is nil.
	var nilProc *JSONProcessor = nil
	var iface2 Processor = nilProc // iface2 != nil even though nilProc == nil

	fmt.Printf("nilProc == nil: %v\n", nilProc == nil)           // true
	fmt.Printf("iface2 == nil: %v\n", iface2 == nil)              // false! has type info
	fmt.Printf("useProcessor(iface2) would panic if we called Process\n")
	// useProcessor(iface2) would pass the non-nil check and then panic on Process

	// Safe approach: compare the concrete pointer before assigning to interface
	if nilProc != nil {
		var iface3 Processor = nilProc
		fmt.Println(useProcessor(iface3))
	} else {
		fmt.Println("nilProc is nil — not assigning to interface")
	}

	fmt.Println()
}

// ============================================================================
// sync.Pool: REUSING ALLOCATIONS
// ============================================================================

var bufPool = sync.Pool{
	New: func() any {
		// Called when pool is empty — allocate a new buffer
		return &bytes.Buffer{}
	},
}

// processEvent simulates a hot path that processes many events.
// Using sync.Pool avoids allocating a new bytes.Buffer for every call.
func processEvent(eventType, payload string) string {
	// Borrow a buffer from the pool
	buf := bufPool.Get().(*bytes.Buffer)
	// Always reset before use — previous user may have left content
	buf.Reset()
	// Return to pool when done — deferred so it runs even on panic
	defer bufPool.Put(buf)

	buf.WriteString("[")
	buf.WriteString(eventType)
	buf.WriteString("] ")
	buf.WriteString(payload)
	buf.WriteString(" at ")
	buf.WriteString(time.Now().Format("15:04:05"))

	return buf.String()
}

func syncPoolDemo() {
	fmt.Println("--- sync.Pool: reusing allocations ---")

	events := []struct{ t, p string }{
		{"USER_LOGIN", "user_id=42"},
		{"PAGE_VIEW", "path=/dashboard"},
		{"API_CALL", "endpoint=/api/metrics"},
	}

	for _, e := range events {
		result := processEvent(e.t, e.p)
		fmt.Printf("  %s\n", result)
	}

	fmt.Println("(each processEvent call reused a pooled *bytes.Buffer)")
	fmt.Println()
}

func main() {
	constructorPattern()
	optionalFields()
	loopVariableGotcha()
	ptrToInterface()
	syncPoolDemo()
}
