package indexer

import (
	"reflect"
	"testing"
)

// ============================================================================
// Bug 1 Tests: Frequency
// ============================================================================

func TestFrequency_Strings(t *testing.T) {
	input := []string{"go", "rust", "go", "zig", "go", "rust"}
	got := Frequency(input)
	want := map[string]int{"go": 3, "rust": 2, "zig": 1}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("Frequency(strings) = %v, want %v", got, want)
	}
}

func TestFrequency_Ints(t *testing.T) {
	input := []int{1, 2, 2, 3, 3, 3}
	got := Frequency(input)
	if got[1] != 1 || got[2] != 2 || got[3] != 3 {
		t.Errorf("Frequency(ints) = %v, unexpected counts", got)
	}
}

func TestFrequency_Empty(t *testing.T) {
	got := Frequency([]string{})
	if len(got) != 0 {
		t.Errorf("Frequency(empty) = %v, want empty map", got)
	}
}

// ============================================================================
// Bug 2 Tests: WrapAll
// ============================================================================

func TestWrapAll_String(t *testing.T) {
	input := []string{"user", "order", "product"}
	got := WrapAll(input, "<", ">")
	want := []string{"<user>", "<order>", "<product>"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("WrapAll(string) = %v, want %v", got, want)
	}
}

func TestWrapAll_Tag(t *testing.T) {
	// Tag has underlying type string — WrapAll must accept it
	tags := []Tag{"critical", "high", "low"}
	got := WrapAll(tags, "[", "]")
	want := []Tag{"[critical]", "[high]", "[low]"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("WrapAll(Tag) = %v, want %v", got, want)
	}
}

func TestWrapAll_EventName(t *testing.T) {
	events := []EventName{"user.login", "user.logout", "order.placed"}
	got := WrapAll(events, "evt:", "")
	want := []EventName{"evt:user.login", "evt:user.logout", "evt:order.placed"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("WrapAll(EventName) = %v, want %v", got, want)
	}
}

// ============================================================================
// Bug 3 Tests: SumScores
// ============================================================================

func TestSumScores_Latency(t *testing.T) {
	latencies := []LatencyScore{
		{ms: 12.5},
		{ms: 8.3},
		{ms: 15.2},
	}

	got := SumScores(latencies)
	want := 36.0

	if got != want {
		t.Errorf("SumScores(latencies) = %.2f, want %.2f", got, want)
	}
}

func TestSumScores_ErrorRate(t *testing.T) {
	rates := []ErrorRateScore{
		{pct: 1.5},
		{pct: 2.3},
		{pct: 0.8},
	}

	got := SumScores(rates)
	want := 4.6

	// Float comparison with tolerance
	if got < want-0.001 || got > want+0.001 {
		t.Errorf("SumScores(error rates) = %.4f, want %.4f", got, want)
	}
}

func TestSumScores_Empty(t *testing.T) {
	got := SumScores([]LatencyScore{})
	if got != 0.0 {
		t.Errorf("SumScores(empty) = %.2f, want 0.0", got)
	}
}
