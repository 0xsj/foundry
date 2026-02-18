// Package logparser parses semi-structured log lines into typed entries.
//
// Design decisions:
//   - Regexp patterns compiled at package level — they're static and expensive to compile.
//     Compiling inside ParseLogLine would be O(n×compile_cost) over a large batch.
//   - strings.Fields for initial tokenization — handles multiple spaces and tabs cleanly.
//   - strings.Cut for service extraction — cleaner than Index+slice.
//   - Pairs is always an initialized map (never nil) — callers can range safely.
//   - ParseLogBatch collects individual parse errors rather than failing fast;
//     real log pipelines should process what they can and report what they couldn't.
package logparser

import (
	"errors"
	"fmt"
	"regexp"
	"sort"
	"strings"
)

// Level represents a log severity level.
type Level string

const (
	LevelDebug Level = "DEBUG"
	LevelInfo  Level = "INFO"
	LevelWarn  Level = "WARN"
	LevelError Level = "ERROR"
)

// IsError reports whether the level is ERROR.
func (l Level) IsError() bool { return l == LevelError }

// IsWarn reports whether the level is WARN.
func (l Level) IsWarn() bool { return l == LevelWarn }

// validLevels is the set of accepted level values.
var validLevels = map[Level]bool{
	LevelDebug: true,
	LevelInfo:  true,
	LevelWarn:  true,
	LevelError: true,
}

// LogEntry holds the structured fields extracted from a single log line.
type LogEntry struct {
	Timestamp string            // RFC3339 timestamp string
	Level     Level             // DEBUG, INFO, WARN, ERROR
	Service   string            // service name (without trailing colon)
	Message   string            // free-form text before key=value pairs
	Pairs     map[string]string // extracted key=value pairs; always non-nil
}

// BatchResult holds the aggregated output of ParseLogBatch.
type BatchResult struct {
	Entries     []LogEntry // successfully parsed entries
	ErrorCount  int        // number of ERROR-level entries
	WarnCount   int        // number of WARN-level entries
	ParseErrors []string   // descriptions of lines that failed to parse
}

// ServicesWithErrors returns a sorted list of service names that have at
// least one ERROR-level entry in this batch.
func (r *BatchResult) ServicesWithErrors() []string {
	seen := make(map[string]struct{})
	for _, e := range r.Entries {
		if e.Level.IsError() {
			seen[e.Service] = struct{}{}
		}
	}
	result := make([]string, 0, len(seen))
	for svc := range seen {
		result = append(result, svc)
	}
	sort.Strings(result)
	return result
}

// Package-level compiled patterns — compiled once at program start.
// reTimestamp matches RFC3339 timestamps with Z or ±HH:MM timezone.
var reTimestamp = regexp.MustCompile(
	`^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:Z|[+-]\d{2}:\d{2})`)

// reKVPair matches key=value or key="quoted value".
// Group 1: key (word chars, hyphens, dots)
// Group 2: value (either "quoted string" or unquoted non-space token)
var reKVPair = regexp.MustCompile(`(\w[\w.-]*)=("(?:[^"\\]|\\.)*"|\S+)`)

// ParseLogLine parses a single log line into a LogEntry.
//
// Expected format:
//
//	2024-01-15T14:30:00Z LEVEL service-name: message key=value key="quoted value"
func ParseLogLine(line string) (LogEntry, error) {
	// Step 1: Extract the RFC3339 timestamp from the start of the line.
	// Using a regexp here because the timestamp has a fixed structure we can
	// validate precisely. strings.Fields alone can't distinguish a timestamp
	// from any other token.
	ts := reTimestamp.FindString(line)
	if ts == "" {
		return LogEntry{}, errors.New("missing or malformed timestamp")
	}

	// Trim the timestamp from the front, leaving the rest of the line.
	remainder := strings.TrimSpace(line[len(ts):])

	// Step 2: Tokenize with Fields (handles variable whitespace between tokens).
	tokens := strings.Fields(remainder)
	if len(tokens) < 2 {
		return LogEntry{}, fmt.Errorf("line too short: need at least level and service, got %q", remainder)
	}

	// Step 3: Validate the level token.
	level := Level(tokens[0])
	if !validLevels[level] {
		return LogEntry{}, fmt.Errorf("unknown log level %q: must be DEBUG, INFO, WARN, or ERROR", tokens[0])
	}

	// Step 4: Extract the service token (must end with ':').
	rawService := tokens[1]
	if !strings.HasSuffix(rawService, ":") {
		return LogEntry{}, fmt.Errorf("expected service token ending with ':', got %q", rawService)
	}
	service := strings.TrimSuffix(rawService, ":")

	// Step 5: Collect the tail — everything after the service token.
	// Find where the service token ends in the remainder string so we can
	// preserve exact whitespace in the message.
	serviceEnd := strings.Index(remainder, rawService) + len(rawService)
	tail := strings.TrimSpace(remainder[serviceEnd:])

	// Step 6: Separate message from key=value pairs.
	// The message is everything before the first key=value match.
	var message string
	loc := reKVPair.FindStringIndex(tail)
	if loc != nil {
		message = strings.TrimSpace(tail[:loc[0]])
	} else {
		message = tail
	}

	// Step 7: Extract all key=value pairs.
	pairs := make(map[string]string)
	matches := reKVPair.FindAllStringSubmatch(tail, -1)
	for _, m := range matches {
		key := m[1]
		val := m[2]
		// Strip surrounding quotes from quoted values.
		if len(val) >= 2 && val[0] == '"' && val[len(val)-1] == '"' {
			val = val[1 : len(val)-1]
		}
		pairs[key] = val
	}

	return LogEntry{
		Timestamp: ts,
		Level:     level,
		Service:   service,
		Message:   message,
		Pairs:     pairs,
	}, nil
}

// ParseLogBatch parses a slice of log lines.
//
// Blank lines are skipped silently. Individual parse errors are collected in
// BatchResult.ParseErrors rather than causing a fatal error — the caller
// receives whatever the parser could successfully extract.
func ParseLogBatch(lines []string) (BatchResult, error) {
	var result BatchResult
	result.Entries = make([]LogEntry, 0, len(lines))

	for i, line := range lines {
		// Skip blank lines silently — common in log files between sessions.
		if strings.TrimSpace(line) == "" {
			continue
		}

		entry, err := ParseLogLine(line)
		if err != nil {
			result.ParseErrors = append(result.ParseErrors,
				fmt.Sprintf("line %d: %v", i+1, err))
			continue
		}

		result.Entries = append(result.Entries, entry)
		if entry.Level.IsError() {
			result.ErrorCount++
		}
		if entry.Level.IsWarn() {
			result.WarnCount++
		}
	}

	return result, nil
}
