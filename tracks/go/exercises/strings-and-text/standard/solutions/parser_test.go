package logparser

import (
	"strings"
	"testing"
)

func TestParseLogLine_Basic(t *testing.T) {
	line := `2024-01-15T14:30:00Z ERROR payment-svc: connection refused host="db.prod" retries=3`
	entry, err := ParseLogLine(line)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if entry.Timestamp != "2024-01-15T14:30:00Z" {
		t.Errorf("Timestamp = %q, want %q", entry.Timestamp, "2024-01-15T14:30:00Z")
	}
	if entry.Level != LevelError {
		t.Errorf("Level = %q, want %q", entry.Level, LevelError)
	}
	if entry.Service != "payment-svc" {
		t.Errorf("Service = %q, want %q", entry.Service, "payment-svc")
	}
	if entry.Message != "connection refused" {
		t.Errorf("Message = %q, want %q", entry.Message, "connection refused")
	}
	if entry.Pairs["host"] != "db.prod" {
		t.Errorf("Pairs[host] = %q, want %q", entry.Pairs["host"], "db.prod")
	}
	if entry.Pairs["retries"] != "3" {
		t.Errorf("Pairs[retries] = %q, want %q", entry.Pairs["retries"], "3")
	}
}

func TestParseLogLine_AllLevels(t *testing.T) {
	tests := []struct {
		level    string
		expected Level
	}{
		{"DEBUG", LevelDebug},
		{"INFO", LevelInfo},
		{"WARN", LevelWarn},
		{"ERROR", LevelError},
	}

	for _, tc := range tests {
		line := "2024-01-15T14:30:00Z " + tc.level + " svc: msg"
		entry, err := ParseLogLine(line)
		if err != nil {
			t.Errorf("level %q: unexpected error: %v", tc.level, err)
			continue
		}
		if entry.Level != tc.expected {
			t.Errorf("level %q: got %q, want %q", tc.level, entry.Level, tc.expected)
		}
	}
}

func TestParseLogLine_InvalidLevel(t *testing.T) {
	line := "2024-01-15T14:30:00Z FATAL svc: msg"
	_, err := ParseLogLine(line)
	if err == nil {
		t.Error("expected error for invalid level, got nil")
	}
}

func TestParseLogLine_MissingTimestamp(t *testing.T) {
	line := "ERROR payment-svc: connection refused"
	_, err := ParseLogLine(line)
	if err == nil {
		t.Error("expected error for missing timestamp, got nil")
	}
}

func TestParseLogLine_MissingService(t *testing.T) {
	line := "2024-01-15T14:30:00Z ERROR"
	_, err := ParseLogLine(line)
	if err == nil {
		t.Error("expected error for missing service, got nil")
	}
}

func TestParseLogLine_QuotedValue(t *testing.T) {
	line := `2024-01-15T14:30:00Z INFO auth-svc: user login user="alice@example.com" region=us-east`
	entry, err := ParseLogLine(line)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if entry.Pairs["user"] != "alice@example.com" {
		t.Errorf("Pairs[user] = %q, want %q (quotes should be stripped)",
			entry.Pairs["user"], "alice@example.com")
	}
	if entry.Pairs["region"] != "us-east" {
		t.Errorf("Pairs[region] = %q, want %q", entry.Pairs["region"], "us-east")
	}
}

func TestParseLogLine_NoPairs(t *testing.T) {
	line := "2024-01-15T14:30:00Z INFO svc: server started"
	entry, err := ParseLogLine(line)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if entry.Message != "server started" {
		t.Errorf("Message = %q, want %q", entry.Message, "server started")
	}
	if entry.Pairs == nil {
		t.Error("Pairs should not be nil")
	}
	if len(entry.Pairs) != 0 {
		t.Errorf("Pairs should be empty, got %v", entry.Pairs)
	}
}

func TestParseLogLine_PairsOnly(t *testing.T) {
	line := "2024-01-15T14:30:00Z WARN svc: key=val other=x"
	entry, err := ParseLogLine(line)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if entry.Pairs["key"] != "val" {
		t.Errorf("Pairs[key] = %q, want %q", entry.Pairs["key"], "val")
	}
}

func TestParseLogLine_TimezoneOffset(t *testing.T) {
	line := "2024-01-15T14:30:00+05:30 INFO svc: msg"
	entry, err := ParseLogLine(line)
	if err != nil {
		t.Fatalf("unexpected error for timezone offset timestamp: %v", err)
	}
	if entry.Timestamp != "2024-01-15T14:30:00+05:30" {
		t.Errorf("Timestamp = %q, want %q", entry.Timestamp, "2024-01-15T14:30:00+05:30")
	}
}

func TestParseLogLine_PairsNotNil(t *testing.T) {
	line := `2024-01-15T14:30:00Z INFO svc: no pairs here`
	entry, err := ParseLogLine(line)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if entry.Pairs == nil {
		t.Error("Pairs should be an empty map, not nil")
	}
}

func TestLevelMethods(t *testing.T) {
	if !LevelError.IsError() {
		t.Error("ERROR.IsError() should be true")
	}
	if LevelWarn.IsError() {
		t.Error("WARN.IsError() should be false")
	}
	if !LevelWarn.IsWarn() {
		t.Error("WARN.IsWarn() should be true")
	}
	if LevelInfo.IsWarn() {
		t.Error("INFO.IsWarn() should be false")
	}
}

var batchLines = []string{
	`2024-01-15T14:30:00Z ERROR payment-svc: connection refused host="db.prod" retries=3`,
	`2024-01-15T14:30:01Z INFO  auth-svc: user login user="alice@example.com"`,
	`2024-01-15T14:30:02Z WARN  inventory: rate limit limit=1000 current=1043`,
	`2024-01-15T14:30:03Z ERROR auth-svc: token expired user="bob@example.com"`,
	``,
	`not a log line`,
	`2024-01-15T14:30:04Z DEBUG payment-svc: retrying`,
}

func TestParseLogBatch_Counts(t *testing.T) {
	result, err := ParseLogBatch(batchLines)
	if err != nil {
		t.Fatalf("unexpected fatal error: %v", err)
	}
	if len(result.Entries) != 5 {
		t.Errorf("Entries len = %d, want 5", len(result.Entries))
	}
	if result.ErrorCount != 2 {
		t.Errorf("ErrorCount = %d, want 2", result.ErrorCount)
	}
	if result.WarnCount != 1 {
		t.Errorf("WarnCount = %d, want 1", result.WarnCount)
	}
	if len(result.ParseErrors) != 1 {
		t.Errorf("ParseErrors len = %d, want 1", len(result.ParseErrors))
	}
}

func TestParseLogBatch_ServicesWithErrors(t *testing.T) {
	result, _ := ParseLogBatch(batchLines)
	services := result.ServicesWithErrors()

	if len(services) != 2 {
		t.Errorf("ServicesWithErrors len = %d, want 2", len(services))
	}
	if len(services) == 2 {
		if services[0] != "auth-svc" || services[1] != "payment-svc" {
			t.Errorf("ServicesWithErrors = %v, want [auth-svc payment-svc]", services)
		}
	}
}

func TestParseLogBatch_BlankLinesSkipped(t *testing.T) {
	lines := []string{"", "   ", `2024-01-15T14:30:00Z INFO svc: msg`, ""}
	result, _ := ParseLogBatch(lines)
	if len(result.Entries) != 1 {
		t.Errorf("blank lines should be skipped; got %d entries, want 1", len(result.Entries))
	}
	if len(result.ParseErrors) != 0 {
		t.Errorf("blank lines should not generate parse errors; got %v", result.ParseErrors)
	}
}

func TestParseLogBatch_AllErrorLines(t *testing.T) {
	lines := []string{
		`2024-01-15T14:30:00Z ERROR svc-a: msg`,
		`2024-01-15T14:30:00Z ERROR svc-b: msg`,
		`2024-01-15T14:30:00Z ERROR svc-a: msg again`,
	}
	result, _ := ParseLogBatch(lines)
	services := result.ServicesWithErrors()
	if len(services) != 2 {
		t.Errorf("want 2 unique error services, got %v", services)
	}
	if strings.Join(services, ",") != "svc-a,svc-b" {
		t.Errorf("services not sorted: %v", services)
	}
}
