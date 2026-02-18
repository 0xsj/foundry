// Package leaderboard tracks player scores across multiple game modes.
package leaderboard

import "sort"

// Entry represents a single player's score in a game mode.
type Entry struct {
	PlayerID string
	Score    int
	Rank     int // calculated field — set by RankAll()
}

// Leaderboard manages scores per game mode.
type Leaderboard struct {
	scores map[string][]Entry // game mode → entries
}

// New creates a new Leaderboard.
func New() *Leaderboard {
	// BUG 1: scores map is never initialized.
	// Any call to AddScore or Import before calling any read method panics.
	return &Leaderboard{}
}

// AddScore adds or updates a player's score in a game mode.
// If the player already has a score in this mode, it is replaced.
func (lb *Leaderboard) AddScore(mode, playerID string, score int) {
	entries := lb.scores[mode]

	// Linear scan to find and update existing player
	for _, e := range entries {
		if e.PlayerID == playerID {
			// BUG 2: e is a copy — this updates the copy, not the slice element.
			// After the loop, the original entry in lb.scores[mode] is unchanged.
			e.Score = score
			return
		}
	}
	// Player not found — append new entry
	lb.scores[mode] = append(lb.scores[mode], Entry{PlayerID: playerID, Score: score})
}

// TopN returns the top n players in a game mode, sorted by score descending.
// If n > number of players, returns all players.
func (lb *Leaderboard) TopN(mode string, n int) []Entry {
	entries := lb.scores[mode]
	if len(entries) == 0 {
		return nil // BUG 3: should return []Entry{}, not nil
	}

	// Sort a copy — don't mutate the stored order
	// BUG 4: This does NOT make a copy. It sorts lb.scores[mode] directly
	// because entries is a slice header sharing the same backing array.
	sorted := entries
	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].Score > sorted[j].Score
	})

	if n > len(sorted) {
		n = len(sorted)
	}
	return sorted[:n]
}

// RankAll assigns rank numbers to all players in a game mode (1 = highest score).
// Modifies entries in place.
func (lb *Leaderboard) RankAll(mode string) {
	entries := lb.scores[mode]
	sort.Slice(entries, func(i, j int) bool {
		return entries[i].Score > entries[j].Score
	})
	// BUG 5: range loop copies each entry — setting e.Rank modifies the copy.
	// The Rank field in lb.scores[mode] is never updated.
	for rank, e := range entries {
		e.Rank = rank + 1
		_ = e
	}
}

// HasPlayer reports whether a player has a score in any game mode.
// BUG 6: This is O(n*m) — linear scan over all entries in all modes.
// With a player index (map[string]bool), this would be O(1).
func (lb *Leaderboard) HasPlayer(playerID string) bool {
	for _, entries := range lb.scores {
		for _, e := range entries {
			if e.PlayerID == playerID {
				return true
			}
		}
	}
	return false
}

// Import bulk-loads entries for a game mode, replacing any existing data.
// The caller's slice is stored directly — not copied.
func (lb *Leaderboard) Import(mode string, entries []Entry) {
	// BUG 7: Storing the caller's slice directly. The caller retains a reference
	// to the same backing array. If the caller modifies their slice (or appends
	// within capacity), lb.scores[mode] is silently corrupted.
	lb.scores[mode] = entries
}

// MergeMode merges entries from srcMode into dstMode.
// Players in srcMode that are not in dstMode are added.
// Players in both modes keep their dstMode score.
func (lb *Leaderboard) MergeMode(dstMode, srcMode string) {
	src := lb.scores[srcMode]
	for _, e := range src {
		// HasPlayer is O(n*m) — called once per source entry.
		// With a per-mode player index, this would be O(k).
		if !lb.playerInMode(dstMode, e.PlayerID) {
			lb.scores[dstMode] = append(lb.scores[dstMode], e)
		}
	}
}

func (lb *Leaderboard) playerInMode(mode, playerID string) bool {
	for _, e := range lb.scores[mode] {
		if e.PlayerID == playerID {
			return true
		}
	}
	return false
}

// ScoreStats returns the min, max, and average score for a game mode.
func (lb *Leaderboard) ScoreStats(mode string) (min, max int, avg float64) {
	entries := lb.scores[mode]
	if len(entries) == 0 {
		return 0, 0, 0
	}
	min = entries[0].Score
	max = entries[0].Score
	sum := 0
	for _, e := range entries {
		if e.Score < min {
			min = e.Score
		}
		if e.Score > max {
			max = e.Score
		}
		sum += e.Score
	}
	avg = float64(sum) / float64(len(entries))
	return min, max, avg
}
