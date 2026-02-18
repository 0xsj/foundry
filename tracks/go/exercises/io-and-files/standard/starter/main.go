// Package logprocessor implements a streaming log processor.
// Run tests: go test -v ./...
package logprocessor

import (
	"io"
	"time"
)

// ============================================================================
// TYPES — provided, do not change
// ============================================================================

// LogEntry represents a single parsed log line.
type LogEntry struct {
	Level     string            // "DEBUG", "INFO", "WARN", "ERROR"
	Message   string
	Timestamp time.Time
	Fields    map[string]string // additional key-value pairs
	Raw       string            // original line (used if parse fails)
}

// ProcessConfig controls filtering and transformation behavior.
type ProcessConfig struct {
	MinLevel     string   // minimum level to include: "DEBUG", "INFO", "WARN", "ERROR"
	RedactFields []string // field names to redact in matched entries
}

// ProcessSummary holds counts from a processing run.
type ProcessSummary struct {
	TotalRead int
	Matched   int
	Written   int
	Errors    int // lines that failed to parse (still counted in TotalRead)
}

// levelOrder maps severity names to integers for comparison.
// An entry passes the level filter if its level >= the configured MinLevel.
var levelOrder = map[string]int{
	"DEBUG": 0,
	"INFO":  1,
	"WARN":  2,
	"ERROR": 3,
}

// ============================================================================
// LogProcessor — implement below
// ============================================================================

// LogProcessor reads, filters, and writes log entries.
type LogProcessor struct {
	// TODO: add fields
}

// NewLogProcessor creates a LogProcessor with the given config.
func NewLogProcessor(cfg ProcessConfig) *LogProcessor {
	// TODO: Implement
	_ = cfg
	return &LogProcessor{}
}

// Process reads all entries from r line by line, applies filters and transforms,
// and writes matching entries to w.
//
// Implementation notes:
//   - Use bufio.Scanner to read r line by line (never load all into memory)
//   - Parse each line as JSON into a map[string]string
//   - Filter by MinLevel using levelOrder map above
//   - Redact fields listed in cfg.RedactFields by replacing their value with "[REDACTED]"
//   - Write each matching entry as JSON followed by a newline
//   - Check scanner.Err() after the loop
//   - Count everything in ProcessSummary
func (p *LogProcessor) Process(r io.Reader, w io.Writer) (ProcessSummary, error) {
	// TODO: Implement
	_ = r
	_ = w
	return ProcessSummary{}, nil
}

// ProcessFile opens the file at path and calls Process with the open file and w.
// Closes the file when done.
func (p *LogProcessor) ProcessFile(path string, w io.Writer) (ProcessSummary, error) {
	// TODO: Implement
	// Hint: os.Open, defer f.Close(), then call p.Process(f, w)
	_ = path
	_ = w
	return ProcessSummary{}, nil
}
