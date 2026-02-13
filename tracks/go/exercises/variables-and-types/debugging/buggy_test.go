package tracker

import (
	"testing"
	"unicode/utf8"
)

func TestNewTracker_DoesNotPanic(t *testing.T) {
	tr := NewTracker()
	// BUG 1: This should not panic. If it does, the map isn't initialized.
	tr.Record("ping", `{"status":"ok"}`)

	stats, ok := tr.Get("ping")
	if !ok {
		t.Fatal("expected ping event to be tracked")
	}
	if stats.Count != 1 {
		t.Errorf("Count = %d, want 1", stats.Count)
	}
}

func TestRecord_CountsAccumulate(t *testing.T) {
	tr := NewTracker()
	tr.Record("push", `{"ref":"main"}`)
	tr.Record("push", `{"ref":"dev"}`)
	tr.Record("push", `{"ref":"feature"}`)

	// BUG 2: Count should be 3 after three Record calls for the same type.
	stats, ok := tr.Get("push")
	if !ok {
		t.Fatal("expected push event to be tracked")
	}
	if stats.Count != 3 {
		t.Errorf("Count = %d, want 3", stats.Count)
	}
	if stats.LastPayload != `{"ref":"feature"}` {
		t.Errorf("LastPayload = %q, want %q", stats.LastPayload, `{"ref":"feature"}`)
	}
}

func TestRecord_MultipleEventTypes(t *testing.T) {
	tr := NewTracker()
	tr.Record("push", `{"ref":"main"}`)
	tr.Record("ping", `{"zen":"keep it simple"}`)
	tr.Record("push", `{"ref":"dev"}`)

	push, _ := tr.Get("push")
	ping, _ := tr.Get("ping")

	if push.Count != 2 {
		t.Errorf("push Count = %d, want 2", push.Count)
	}
	if ping.Count != 1 {
		t.Errorf("ping Count = %d, want 1", ping.Count)
	}
}

func TestCharCount_ASCII(t *testing.T) {
	s := "hello world"
	got := CharCount(s)
	want := utf8.RuneCountInString(s)
	if got != want {
		t.Errorf("CharCount(%q) = %d, want %d characters", s, got, want)
	}
}

func TestCharCount_Unicode(t *testing.T) {
	// BUG 3: These strings have fewer characters than bytes.
	tests := []struct {
		input string
		want  int
	}{
		{"café", 4},           // é is 2 bytes in UTF-8
		{"日本語", 3},            // each CJK char is 3 bytes
		{"hello 🌍", 7},       // emoji is 4 bytes
		{"résumé", 6},         // two accented chars
	}

	for _, tt := range tests {
		got := CharCount(tt.input)
		if got != tt.want {
			t.Errorf("CharCount(%q) = %d, want %d characters (len gives bytes: %d)",
				tt.input, got, tt.want, len(tt.input))
		}
	}
}

func TestTracker_PayloadSizeIsChars(t *testing.T) {
	tr := NewTracker()
	tr.Record("deploy", `{"env":"プロダクション"}`)

	stats, _ := tr.Get("deploy")
	// The dashboard displays "chars" — so PayloadSize should be character count, not byte count.
	wantChars := utf8.RuneCountInString(`{"env":"プロダクション"}`)
	if stats.PayloadSize != wantChars {
		t.Errorf("PayloadSize = %d bytes, want %d chars", stats.PayloadSize, wantChars)
	}
}
