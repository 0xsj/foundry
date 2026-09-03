package main

import (
	"encoding/json"
	"os"
	"testing"
)

// The variants command rewrites foundry.json, so marshalling it back must be
// byte-identical to what is on disk. If this fails, an edit would reformat the
// whole spine and bury the real change in noise.
func TestSpineRoundTrips(t *testing.T) {
	orig, err := os.ReadFile("../../foundry.json")
	if err != nil {
		t.Skip("not running from the repo")
	}
	var s Spine
	if err := json.Unmarshal(orig, &s); err != nil {
		t.Fatal(err)
	}
	out, err := marshalJSON(&s)
	if err != nil {
		t.Fatal(err)
	}
	if string(out) != string(orig) {
		for i := range out {
			if i >= len(orig) || out[i] != orig[i] {
				lo := i - 80
				if lo < 0 {
					lo = 0
				}
				hi := i + 80
				if hi > len(out) {
					hi = len(out)
				}
				t.Fatalf("diverges at byte %d:\ngot  ...%s...\nwant ...%s...",
					i, out[lo:hi], orig[lo:min(hi, len(orig))])
			}
		}
		t.Fatalf("length differs: got %d, want %d", len(out), len(orig))
	}
}
