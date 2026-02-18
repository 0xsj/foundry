// Package logprocessor implements a streaming log processor.
// Run tests: go test -v ./...
//
// Design decisions:
//   - Process accepts io.Reader/io.Writer — not *os.File — so it works with any source/sink.
//     Tests use strings.Reader; callers use os.File or http.ResponseWriter. Same code.
//   - bufio.Scanner is used for line-by-line reading. The important invariant: if scanner.Scan()
//     returns false, you MUST check scanner.Err(). It's false on both EOF (fine) and error (not fine).
//   - Redaction is applied after filtering. If an entry doesn't pass the level filter, there's
//     no point redacting it — it's not going anywhere.
//   - json.NewEncoder(w) is used for output — it appends '\n' automatically after each Encode call,
//     giving us newline-delimited JSON with no extra work.
package logprocessor

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"time"
)

// LogEntry represents a single parsed log line.
type LogEntry struct {
	Level     string
	Message   string
	Timestamp time.Time
	Fields    map[string]string
	Raw       string
}

// ProcessConfig controls filtering and transformation behavior.
type ProcessConfig struct {
	MinLevel     string
	RedactFields []string
}

// ProcessSummary holds counts from a processing run.
type ProcessSummary struct {
	TotalRead int
	Matched   int
	Written   int
	Errors    int
}

// levelOrder maps severity names to integers for comparison.
var levelOrder = map[string]int{
	"DEBUG": 0,
	"INFO":  1,
	"WARN":  2,
	"ERROR": 3,
}

// LogProcessor reads, filters, and writes log entries.
type LogProcessor struct {
	cfg     ProcessConfig
	minRank int // numeric equivalent of cfg.MinLevel, pre-computed
}

// NewLogProcessor creates a LogProcessor with the given config.
func NewLogProcessor(cfg ProcessConfig) *LogProcessor {
	minRank := 0 // empty MinLevel → DEBUG rank → pass everything
	if rank, ok := levelOrder[cfg.MinLevel]; ok {
		minRank = rank
	}
	return &LogProcessor{
		cfg:     cfg,
		minRank: minRank,
	}
}

// Process reads all entries from r line by line, applies filters and transforms,
// and writes matching entries to w.
//
// Reading is done through bufio.Scanner — it handles partial reads and buffer management
// internally. We never load the entire file into memory.
//
// Key pattern: scanner.Scan() returns false on BOTH EOF and error.
// Always call scanner.Err() after the loop to distinguish them.
func (p *LogProcessor) Process(r io.Reader, w io.Writer) (ProcessSummary, error) {
	var summary ProcessSummary
	enc := json.NewEncoder(w) // json.Encoder.Encode writes JSON + '\n' in one call

	scanner := bufio.NewScanner(r)
	for scanner.Scan() {
		line := scanner.Text()
		summary.TotalRead++

		entry, err := parseLine(line)
		if err != nil {
			summary.Errors++
			continue // skip invalid lines, don't stop processing
		}

		// Level filter
		if !p.passesFilter(entry) {
			continue
		}
		summary.Matched++

		// Redaction (mutates the Fields map — that's fine, we parsed a fresh copy)
		p.redact(entry)

		// Write the entry as JSON + newline
		if err := enc.Encode(p.toMap(entry)); err != nil {
			return summary, fmt.Errorf("write entry: %w", err)
		}
		summary.Written++
	}

	// This is the critical check — scanner.Scan() returns false for both EOF and error.
	// Without this, we silently ignore mid-stream I/O failures.
	if err := scanner.Err(); err != nil {
		return summary, fmt.Errorf("scan: %w", err)
	}

	return summary, nil
}

// ProcessFile opens the file at path and calls Process with the open file and w.
// Note: defer f.Close() goes immediately after the error check — not at the top of the function.
func (p *LogProcessor) ProcessFile(path string, w io.Writer) (ProcessSummary, error) {
	f, err := os.Open(path)
	if err != nil {
		return ProcessSummary{}, fmt.Errorf("open %s: %w", path, err)
	}
	defer f.Close() // for read-only files, ignoring Close() error is acceptable

	return p.Process(f, w)
}

// passesFilter returns true if the entry's level meets the minimum threshold.
func (p *LogProcessor) passesFilter(entry *LogEntry) bool {
	rank, ok := levelOrder[entry.Level]
	if !ok {
		// Unknown level — treat as DEBUG (lowest)
		rank = 0
	}
	return rank >= p.minRank
}

// redact replaces the values of configured fields with "[REDACTED]".
// Operates on the Fields map directly — no copy needed since we own this entry.
func (p *LogProcessor) redact(entry *LogEntry) {
	for _, field := range p.cfg.RedactFields {
		if _, exists := entry.Fields[field]; exists {
			entry.Fields[field] = "[REDACTED]"
		}
	}
}

// toMap converts a LogEntry to a flat map for JSON encoding.
// level and message are always included; other fields are merged in.
func (p *LogProcessor) toMap(entry *LogEntry) map[string]string {
	out := make(map[string]string, len(entry.Fields)+2)
	for k, v := range entry.Fields {
		out[k] = v
	}
	out["level"] = entry.Level
	out["message"] = entry.Message
	return out
}

// parseLine parses a JSON log line into a LogEntry.
// Returns an error if the line is not valid JSON.
func parseLine(line string) (*LogEntry, error) {
	// Parse into a flat map — this handles unknown fields naturally
	// without needing a rigid struct that would miss extra fields.
	var raw map[string]string
	if err := json.Unmarshal([]byte(line), &raw); err != nil {
		return nil, fmt.Errorf("parse: %w", err)
	}

	entry := &LogEntry{
		Level:   raw["level"],
		Message: raw["message"],
		Fields:  make(map[string]string),
		Raw:     line,
	}

	// Copy extra fields (everything except level and message)
	for k, v := range raw {
		if k != "level" && k != "message" && k != "ts" {
			entry.Fields[k] = v
		}
	}

	// Parse optional timestamp
	if ts, ok := raw["ts"]; ok {
		if t, err := time.Parse(time.RFC3339, ts); err == nil {
			entry.Timestamp = t
		}
	}

	return entry, nil
}
