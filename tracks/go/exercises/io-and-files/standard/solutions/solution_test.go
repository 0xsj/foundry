package logprocessor

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

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
		t.Errorf("Matched = %d, want 4", summary.Matched)
	}
}

func TestProcess_FilterByMinLevel_Info(t *testing.T) {
	input := logLines(
		entry("DEBUG", "verbose"),
		entry("INFO", "normal"),
		entry("WARN", "slow"),
		entry("ERROR", "fatal"),
	)
	p := NewLogProcessor(ProcessConfig{MinLevel: "INFO"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if summary.Matched != 3 {
		t.Errorf("Matched = %d, want 3 (INFO, WARN, ERROR)", summary.Matched)
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
		t.Errorf("Matched = %d, want 1", summary.Matched)
	}
}

func TestProcess_AllFiltered(t *testing.T) {
	input := logLines(entry("DEBUG", "a"), entry("INFO", "b"))
	p := NewLogProcessor(ProcessConfig{MinLevel: "WARN"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if summary.Matched != 0 {
		t.Errorf("Matched = %d, want 0", summary.Matched)
	}
}

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
		t.Errorf("Matched = %d, want 1", summary.Matched)
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
		t.Errorf("expected all zeros, got %+v", summary)
	}
}

func TestProcess_RedactSingleField(t *testing.T) {
	input := logLines(entry("INFO", "user login", "user_id", "u-8821", "path", "/login"))
	p := NewLogProcessor(ProcessConfig{MinLevel: "INFO", RedactFields: []string{"user_id"}})
	var out bytes.Buffer
	_, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	var result map[string]string
	if err := json.Unmarshal(bytes.TrimRight(out.Bytes(), "\n"), &result); err != nil {
		t.Fatalf("output not valid JSON: %v\nOutput: %s", err, out.String())
	}
	if result["user_id"] != "[REDACTED]" {
		t.Errorf("user_id = %q, want \"[REDACTED]\"", result["user_id"])
	}
	if result["path"] != "/login" {
		t.Errorf("path = %q, want \"/login\"", result["path"])
	}
}

func TestProcess_RedactMultipleFields(t *testing.T) {
	input := logLines(entry("WARN", "pii access", "user_id", "u-001", "email", "alice@example.com", "ip", "1.2.3.4"))
	p := NewLogProcessor(ProcessConfig{MinLevel: "WARN", RedactFields: []string{"user_id", "email"}})
	var out bytes.Buffer
	_, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	outputLine := strings.SplitN(strings.TrimSpace(out.String()), "\n", 2)[0]
	var result map[string]string
	if err := json.Unmarshal([]byte(outputLine), &result); err != nil {
		t.Fatalf("output not valid JSON: %v", err)
	}
	if result["user_id"] != "[REDACTED]" {
		t.Errorf("user_id = %q, want \"[REDACTED]\"", result["user_id"])
	}
	if result["email"] != "[REDACTED]" {
		t.Errorf("email = %q, want \"[REDACTED]\"", result["email"])
	}
	if result["ip"] != "1.2.3.4" {
		t.Errorf("ip = %q, want \"1.2.3.4\"", result["ip"])
	}
}

func TestProcess_OutputIsNewlineDelimited(t *testing.T) {
	input := logLines(entry("INFO", "first"), entry("INFO", "second"), entry("INFO", "third"))
	p := NewLogProcessor(ProcessConfig{MinLevel: "INFO"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(input), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	lines := strings.Split(strings.TrimRight(out.String(), "\n"), "\n")
	if len(lines) != summary.Written {
		t.Errorf("output has %d lines, want %d", len(lines), summary.Written)
	}
}

func TestProcess_OutputIsValidJSON(t *testing.T) {
	input := logLines(entry("ERROR", "something broke", "code", "503"))
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
			t.Errorf("output line not valid JSON: %v\nLine: %q", err, line)
		}
	}
}

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
	if summary.Matched != 2 {
		t.Errorf("Matched = %d, want 2 (INFO + ERROR)", summary.Matched)
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

func TestProcess_WorksOnInMemoryReader(t *testing.T) {
	data := logLines(entry("WARN", "test warning"))
	p := NewLogProcessor(ProcessConfig{MinLevel: "WARN"})
	var out bytes.Buffer
	summary, err := p.Process(strings.NewReader(data), &out)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if summary.Matched != 1 {
		t.Errorf("Matched = %d, want 1", summary.Matched)
	}
}
