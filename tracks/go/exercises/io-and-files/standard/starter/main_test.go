package logprocessor

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// ============================================================================
// Test helpers
// ============================================================================

// logLines builds a multiline log input from individual JSON maps.
func logLines(entries ...map[string]string) string {
	var sb strings.Builder
	for _, e := range entries {
		b, _ := json.Marshal(e)
		sb.Write(b)
		sb.WriteByte('\n')
	}
	return sb.String()
}

func entry(level, message string, extra ...string) map[string]string {
	m := map[string]string{"level": level, "message": message}
	for i := 0; i+1 < len(extra); i += 2 {
		m[extra[i]] = extra[i+1]
	}
	return m
}

// ============================================================================
// Process: basic filtering
// ============================================================================

func TestProcess_AllPassWhenMinLevelEmpty(t *testing.T) {
	input := logLines(
		entry("DEBUG", "cache miss"),
		entry("INFO", "request ok"),
		entry("WARN", "high latency"),
		entry("ERROR", "db timeout"),
	)

	p := NewLogProcessor(ProcessConfig{MinLevel: ""})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if summary.TotalRead != 4 {
		t.Errorf("TotalRead = %d, want 4", summary.TotalRead)
	}
	if summary.Matched != 4 {
		t.Errorf("Matched = %d, want 4 (empty MinLevel passes everything)", summary.Matched)
	}
	if summary.Written != 4 {
		t.Errorf("Written = %d, want 4", summary.Written)
	}
}

func TestProcess_FilterByMinLevel_Info(t *testing.T) {
	input := logLines(
		entry("DEBUG", "verbose detail"),
		entry("INFO", "normal operation"),
		entry("WARN", "slow response"),
		entry("ERROR", "fatal crash"),
	)

	p := NewLogProcessor(ProcessConfig{MinLevel: "INFO"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if summary.TotalRead != 4 {
		t.Errorf("TotalRead = %d, want 4", summary.TotalRead)
	}
	if summary.Matched != 3 {
		t.Errorf("Matched = %d, want 3 (INFO, WARN, ERROR)", summary.Matched)
	}
	if summary.Written != 3 {
		t.Errorf("Written = %d, want 3", summary.Written)
	}
}

func TestProcess_FilterByMinLevel_Error(t *testing.T) {
	input := logLines(
		entry("DEBUG", "trace"),
		entry("INFO", "ok"),
		entry("WARN", "careful"),
		entry("ERROR", "broken"),
	)

	p := NewLogProcessor(ProcessConfig{MinLevel: "ERROR"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if summary.Matched != 1 {
		t.Errorf("Matched = %d, want 1 (only ERROR)", summary.Matched)
	}
}

func TestProcess_AllFiltered(t *testing.T) {
	input := logLines(
		entry("DEBUG", "a"),
		entry("INFO", "b"),
	)

	p := NewLogProcessor(ProcessConfig{MinLevel: "WARN"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if summary.Matched != 0 {
		t.Errorf("Matched = %d, want 0", summary.Matched)
	}
	if out.Len() != 0 {
		t.Errorf("output should be empty when nothing matches, got %q", out.String())
	}
}

// ============================================================================
// Process: invalid lines
// ============================================================================

func TestProcess_InvalidLinesCountedAsErrors(t *testing.T) {
	input := "not json at all\n" +
		`{"level":"INFO","message":"valid"}` + "\n" +
		`{broken json` + "\n"

	p := NewLogProcessor(ProcessConfig{MinLevel: "DEBUG"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if summary.TotalRead != 3 {
		t.Errorf("TotalRead = %d, want 3", summary.TotalRead)
	}
	if summary.Errors != 2 {
		t.Errorf("Errors = %d, want 2", summary.Errors)
	}
	if summary.Matched != 1 {
		t.Errorf("Matched = %d, want 1 (only the valid INFO line)", summary.Matched)
	}
}

func TestProcess_EmptyInputReturnsZeroSummary(t *testing.T) {
	p := NewLogProcessor(ProcessConfig{MinLevel: "INFO"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(""), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if summary.TotalRead != 0 || summary.Matched != 0 || summary.Written != 0 || summary.Errors != 0 {
		t.Errorf("expected all zeros for empty input, got %+v", summary)
	}
}

// ============================================================================
// Process: redaction
// ============================================================================

func TestProcess_RedactSingleField(t *testing.T) {
	input := logLines(
		entry("INFO", "user login", "user_id", "u-8821", "path", "/login"),
	)

	p := NewLogProcessor(ProcessConfig{
		MinLevel:     "INFO",
		RedactFields: []string{"user_id"},
	})
	var out bytes.Buffer
	_, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Parse the output line and verify redaction
	var result map[string]string
	if err := json.Unmarshal(bytes.TrimRight(out.Bytes(), "\n"), &result); err != nil {
		t.Fatalf("output is not valid JSON: %v\nOutput: %s", err, out.String())
	}

	if result["user_id"] != "[REDACTED]" {
		t.Errorf("user_id = %q, want \"[REDACTED]\"", result["user_id"])
	}
	if result["path"] != "/login" {
		t.Errorf("path = %q, want \"/login\" (should not be redacted)", result["path"])
	}
}

func TestProcess_RedactMultipleFields(t *testing.T) {
	input := logLines(
		entry("WARN", "pii access", "user_id", "u-001", "email", "alice@example.com", "ip", "1.2.3.4"),
	)

	p := NewLogProcessor(ProcessConfig{
		MinLevel:     "WARN",
		RedactFields: []string{"user_id", "email"},
	})
	var out bytes.Buffer
	_, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// The output might be multiple JSON lines — take the first
	outputLine := strings.SplitN(strings.TrimSpace(out.String()), "\n", 2)[0]
	var result map[string]string
	if err := json.Unmarshal([]byte(outputLine), &result); err != nil {
		t.Fatalf("output not valid JSON: %v\nLine: %q", err, outputLine)
	}

	if result["user_id"] != "[REDACTED]" {
		t.Errorf("user_id = %q, want \"[REDACTED]\"", result["user_id"])
	}
	if result["email"] != "[REDACTED]" {
		t.Errorf("email = %q, want \"[REDACTED]\"", result["email"])
	}
	if result["ip"] != "1.2.3.4" {
		t.Errorf("ip = %q, want \"1.2.3.4\" (not in RedactFields)", result["ip"])
	}
}

func TestProcess_RedactOnlyMatchingEntries(t *testing.T) {
	// DEBUG entries are filtered out before redaction even applies
	input := logLines(
		entry("DEBUG", "trace", "user_id", "u-001"),
		entry("INFO", "event", "user_id", "u-002"),
	)

	p := NewLogProcessor(ProcessConfig{
		MinLevel:     "INFO",
		RedactFields: []string{"user_id"},
	})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if summary.Written != 1 {
		t.Errorf("Written = %d, want 1", summary.Written)
	}

	// Output should only contain the INFO line, redacted
	var result map[string]string
	outputLine := strings.TrimSpace(out.String())
	if err := json.Unmarshal([]byte(outputLine), &result); err != nil {
		t.Fatalf("output not valid JSON: %v", err)
	}
	if result["user_id"] != "[REDACTED]" {
		t.Errorf("user_id = %q, want \"[REDACTED]\"", result["user_id"])
	}
}

// ============================================================================
// Process: output format
// ============================================================================

func TestProcess_OutputIsNewlineDelimited(t *testing.T) {
	input := logLines(
		entry("INFO", "first"),
		entry("INFO", "second"),
		entry("INFO", "third"),
	)

	p := NewLogProcessor(ProcessConfig{MinLevel: "INFO"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	lines := strings.Split(strings.TrimRight(out.String(), "\n"), "\n")
	if len(lines) != summary.Written {
		t.Errorf("output has %d lines, want %d (one per written entry)", len(lines), summary.Written)
	}
}

func TestProcess_OutputIsValidJSON(t *testing.T) {
	input := logLines(
		entry("ERROR", "something broke", "code", "503"),
	)

	p := NewLogProcessor(ProcessConfig{MinLevel: "ERROR"})
	var out bytes.Buffer
	_, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	for _, line := range strings.Split(strings.TrimRight(out.String(), "\n"), "\n") {
		if line == "" {
			continue
		}
		var m map[string]interface{}
		if err := json.Unmarshal([]byte(line), &m); err != nil {
			t.Errorf("output line is not valid JSON: %v\nLine: %q", err, line)
		}
	}
}

// ============================================================================
// ProcessFile
// ============================================================================

func TestProcessFile_ReadsFromDisk(t *testing.T) {
	dir := t.TempDir()
	logPath := filepath.Join(dir, "test.log")

	content := logLines(
		entry("INFO", "startup"),
		entry("ERROR", "crash"),
		entry("DEBUG", "trace"),
	)
	if err := os.WriteFile(logPath, []byte(content), 0644); err != nil {
		t.Fatalf("setup: %v", err)
	}

	p := NewLogProcessor(ProcessConfig{MinLevel: "INFO"})
	var out bytes.Buffer
	summary, err := p.ProcessFile(logPath, &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if summary.TotalRead != 3 {
		t.Errorf("TotalRead = %d, want 3", summary.TotalRead)
	}
	if summary.Matched != 2 { // INFO + ERROR, not DEBUG
		t.Errorf("Matched = %d, want 2", summary.Matched)
	}
}

func TestProcessFile_NonexistentFileReturnsError(t *testing.T) {
	p := NewLogProcessor(ProcessConfig{})
	var out bytes.Buffer
	_, err := p.ProcessFile("/no/such/file.log", &out)
	if err == nil {
		t.Error("expected error for nonexistent file, got nil")
	}
}

// ============================================================================
// Process: streaming — works on any io.Reader, not just files
// ============================================================================

func TestProcess_WorksOnInMemoryReader(t *testing.T) {
	// This verifies Process is not coupled to *os.File
	data := logLines(entry("WARN", "test warning"))
	p := NewLogProcessor(ProcessConfig{MinLevel: "WARN"})
	var out bytes.Buffer

	// Pass a strings.Reader — not a file
	summary, err := p.Process(strings.NewReader(data), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if summary.Matched != 1 {
		t.Errorf("Matched = %d, want 1", summary.Matched)
	}
}
