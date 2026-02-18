// Chain of Responsibility: Log Level Routing
//
// Demonstrates a chain where each handler decides whether to process
// AND whether to pass along to the next handler. Unlike the approval
// chain (where only one handler processes), log handlers can each
// independently decide to process the entry.
//
// Console handler: processes all levels, always passes along
// File handler: processes warn and above, always passes along
// Alert handler: processes error only, sends alert
//
// Run: go run ./logging/
package main

import (
	"fmt"
	"strings"
	"time"
)

// --- Domain Types ---

// Level represents log severity
type Level int

const (
	Debug Level = iota
	Info
	Warn
	Error
	Fatal
)

func (l Level) String() string {
	switch l {
	case Debug:
		return "DEBUG"
	case Info:
		return "INFO"
	case Warn:
		return "WARN"
	case Error:
		return "ERROR"
	case Fatal:
		return "FATAL"
	default:
		return "UNKNOWN"
	}
}

// LogEntry represents a single log event
type LogEntry struct {
	Timestamp time.Time
	Level     Level
	Service   string
	Message   string
	Fields    map[string]string
}

func (e LogEntry) String() string {
	fields := ""
	if len(e.Fields) > 0 {
		parts := make([]string, 0, len(e.Fields))
		for k, v := range e.Fields {
			parts = append(parts, fmt.Sprintf("%s=%s", k, v))
		}
		fields = " " + strings.Join(parts, " ")
	}
	return fmt.Sprintf("[%s] %s %s: %s%s",
		e.Timestamp.Format("15:04:05"),
		e.Level,
		e.Service,
		e.Message,
		fields,
	)
}

// --- Chain of Responsibility (Slice-Based with Multi-Handler Processing) ---

// LogHandler processes a log entry and returns true if the chain should continue.
// Unlike validation chains (where the first failure stops), log handlers
// typically BOTH process AND pass along.
type LogHandler struct {
	Name       string
	MinLevel   Level
	Process    func(entry LogEntry)
	StopChain  bool // if true, this handler stops the chain after processing
}

// LogPipeline runs log entries through a chain of handlers.
type LogPipeline struct {
	handlers []LogHandler
}

func NewLogPipeline(handlers ...LogHandler) *LogPipeline {
	return &LogPipeline{handlers: handlers}
}

func (p *LogPipeline) Log(entry LogEntry) {
	for _, h := range p.handlers {
		if entry.Level >= h.MinLevel {
			h.Process(entry)
			if h.StopChain {
				return
			}
		}
	}
}

// --- Concrete Handlers ---

// NewConsoleHandler logs everything to stdout.
func NewConsoleHandler() LogHandler {
	return LogHandler{
		Name:     "console",
		MinLevel: Debug,
		Process: func(entry LogEntry) {
			fmt.Printf("  [CONSOLE] %s\n", entry)
		},
		StopChain: false, // always pass to next handler
	}
}

// NewFileHandler logs warnings and above to a "file" (simulated with print).
func NewFileHandler(filename string) LogHandler {
	var lineCount int
	return LogHandler{
		Name:     "file:" + filename,
		MinLevel: Warn,
		Process: func(entry LogEntry) {
			lineCount++
			fmt.Printf("  [FILE:%s#%d] %s\n", filename, lineCount, entry)
		},
		StopChain: false,
	}
}

// NewAlertHandler sends alerts for errors and above.
func NewAlertHandler(channel string) LogHandler {
	return LogHandler{
		Name:     "alert:" + channel,
		MinLevel: Error,
		Process: func(entry LogEntry) {
			fmt.Printf("  [ALERT -> %s] %s: %s (service: %s)\n",
				channel, entry.Level, entry.Message, entry.Service)
		},
		StopChain: false,
	}
}

// NewDropHandler silently drops messages below a threshold.
// Useful as the first handler to filter noise before expensive processing.
func NewDropHandler(minLevel Level) LogHandler {
	return LogHandler{
		Name:     "drop-filter",
		MinLevel: Debug, // receives everything
		Process: func(entry LogEntry) {
			// This handler stops the chain if the level is too low.
			// The actual filtering is done by the StopChain logic below.
		},
		// Note: We can't use StopChain directly because we want conditional stopping.
		// Instead, we'll use a wrapper approach.
		StopChain: false,
	}
}

// NewConditionalDropHandler demonstrates a handler that conditionally stops the chain.
func NewConditionalDropHandler(minLevel Level) LogHandler {
	return LogHandler{
		Name:     "filter",
		MinLevel: Debug,
		Process: func(entry LogEntry) {
			if entry.Level < minLevel {
				fmt.Printf("  [FILTER] dropped %s message (below %s threshold)\n",
					entry.Level, minLevel)
			}
		},
		StopChain: false, // we handle this differently -- see FilteredPipeline below
	}
}

// FilteredPipeline demonstrates a pipeline with conditional short-circuiting.
type FilteredPipeline struct {
	minLevel Level
	handlers []LogHandler
}

func NewFilteredPipeline(minLevel Level, handlers ...LogHandler) *FilteredPipeline {
	return &FilteredPipeline{
		minLevel: minLevel,
		handlers: handlers,
	}
}

func (p *FilteredPipeline) Log(entry LogEntry) {
	// Gate: reject entries below minimum level before running the chain
	if entry.Level < p.minLevel {
		fmt.Printf("  [GATE] dropped %s message (pipeline minimum: %s)\n",
			entry.Level, p.minLevel)
		return
	}

	for _, h := range p.handlers {
		if entry.Level >= h.MinLevel {
			h.Process(entry)
			if h.StopChain {
				return
			}
		}
	}
}

// --- Main ---

func main() {
	fmt.Println("=== Chain of Responsibility: Log Level Routing ===\n")

	now := time.Now()

	// Build the logging chain
	pipeline := NewLogPipeline(
		NewConsoleHandler(),
		NewFileHandler("app.log"),
		NewAlertHandler("#ops-alerts"),
	)

	entries := []LogEntry{
		{
			Timestamp: now,
			Level:     Debug,
			Service:   "auth-svc",
			Message:   "checking token cache",
			Fields:    map[string]string{"cache_size": "142"},
		},
		{
			Timestamp: now.Add(1 * time.Second),
			Level:     Info,
			Service:   "api-gateway",
			Message:   "request processed",
			Fields:    map[string]string{"path": "/api/users", "status": "200", "duration_ms": "45"},
		},
		{
			Timestamp: now.Add(2 * time.Second),
			Level:     Warn,
			Service:   "rate-limiter",
			Message:   "client approaching rate limit",
			Fields:    map[string]string{"client_ip": "10.0.0.5", "remaining": "3"},
		},
		{
			Timestamp: now.Add(3 * time.Second),
			Level:     Error,
			Service:   "payment-svc",
			Message:   "payment processing failed",
			Fields:    map[string]string{"order_id": "ORD-789", "error": "gateway_timeout"},
		},
		{
			Timestamp: now.Add(4 * time.Second),
			Level:     Fatal,
			Service:   "db-pool",
			Message:   "connection pool exhausted",
			Fields:    map[string]string{"active": "100", "max": "100"},
		},
	}

	fmt.Println("--- Standard Pipeline (all handlers) ---\n")
	for _, entry := range entries {
		fmt.Printf("Log: %s [%s]\n", entry.Message, entry.Level)
		pipeline.Log(entry)
		fmt.Println()
	}

	// Demonstrate filtered pipeline
	fmt.Println("--- Filtered Pipeline (Warn+ only) ---\n")
	filtered := NewFilteredPipeline(Warn,
		NewConsoleHandler(),
		NewFileHandler("errors.log"),
		NewAlertHandler("#critical"),
	)

	for _, entry := range entries {
		fmt.Printf("Log: %s [%s]\n", entry.Message, entry.Level)
		filtered.Log(entry)
		fmt.Println()
	}

	// Show handler routing summary
	fmt.Println("--- Handler Routing Summary ---")
	fmt.Println()
	fmt.Println("| Level | Console | File | Alert |")
	fmt.Println("|-------|---------|------|-------|")
	for _, level := range []Level{Debug, Info, Warn, Error, Fatal} {
		console := "yes"
		file := "no"
		alert := "no"
		if level >= Warn {
			file = "yes"
		}
		if level >= Error {
			alert = "yes"
		}
		fmt.Printf("| %-5s | %-7s | %-4s | %-5s |\n", level, console, file, alert)
	}
}
