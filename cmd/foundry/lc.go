package main

import (
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"sort"
	"time"
)

type LCProblem struct {
	ID         string `json:"id"`
	Num        int    `json:"num"`
	Title      string `json:"title"`
	Difficulty string `json:"difficulty"`
	Pattern    string `json:"pattern"`
	URL        string `json:"url"`
}

type LCState struct {
	Focus   string   `json:"focus"`
	Current string   `json:"current"`
	Solved  []string `json:"solved"`
}

type LC struct {
	Problems []LCProblem
	State    LCState
	root     string
}

func loadLC(root string) (*LC, error) {
	lc := &LC{root: root, State: LCState{Solved: []string{}}}
	if err := loadJSON(filepath.Join(root, "leetcode", "problems.json"), &lc.Problems); err != nil {
		return nil, err
	}
	err := loadJSON(filepath.Join(root, "leetcode", "state.json"), &lc.State)
	if err != nil && !os.IsNotExist(err) {
		return nil, err
	}
	if lc.State.Solved == nil {
		lc.State.Solved = []string{}
	}
	return lc, nil
}

func (lc *LC) save() error {
	return saveJSON(filepath.Join(lc.root, "leetcode", "state.json"), &lc.State)
}

func (lc *LC) isSolved(id string) bool {
	for _, s := range lc.State.Solved {
		if s == id {
			return true
		}
	}
	return false
}

func (lc *LC) remaining() int {
	n := 0
	for _, p := range lc.Problems {
		if !lc.isSolved(p.ID) {
			n++
		}
	}
	return n
}

func (lc *LC) focusLabel() string {
	if lc.State.Focus == "" {
		return "no pattern focus"
	}
	return lc.State.Focus
}

var difficultyRank = map[string]int{"easy": 0, "medium": 1, "hard": 2}

// next dispenses an unsolved problem. It prefers the current pattern focus and
// the easiest remaining tier, then picks randomly among the top candidates so
// the same problem is not served every time.
func (lc *LC) next() (LCProblem, bool) {
	var pool []LCProblem
	for _, p := range lc.Problems {
		if lc.isSolved(p.ID) {
			continue
		}
		if lc.State.Focus != "" && p.Pattern != lc.State.Focus {
			continue
		}
		pool = append(pool, p)
	}
	if len(pool) == 0 {
		// Focus exhausted, so fall back to everything unsolved.
		for _, p := range lc.Problems {
			if !lc.isSolved(p.ID) {
				pool = append(pool, p)
			}
		}
	}
	if len(pool) == 0 {
		return LCProblem{}, false
	}
	sort.Slice(pool, func(i, j int) bool {
		if difficultyRank[pool[i].Difficulty] != difficultyRank[pool[j].Difficulty] {
			return difficultyRank[pool[i].Difficulty] < difficultyRank[pool[j].Difficulty]
		}
		return pool[i].Num < pool[j].Num
	})
	window := min(3, len(pool))
	return pool[rand.New(rand.NewSource(time.Now().UnixNano())).Intn(window)], true
}

func cmdLC(root string, args []string) error {
	lc, err := loadLC(root)
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("leetcode/problems.json is missing")
		}
		return err
	}

	sub := "next"
	if len(args) > 0 {
		sub = args[0]
	}

	switch sub {
	case "next":
		p, ok := lc.next()
		if !ok {
			fmt.Printf("\n  all %d problems solved. foundry lc reset to go again.\n\n", len(lc.Problems))
			return nil
		}
		lc.State.Current = p.ID
		if err := lc.save(); err != nil {
			return err
		}
		fmt.Printf("\n  %d. %s\n", p.Num, p.Title)
		fmt.Printf("  %s · %s\n", p.Difficulty, p.Pattern)
		fmt.Printf("  %s\n\n", p.URL)
		fmt.Printf("  solve in   leetcode/solutions/{go,ts}/%s.{go,ts}\n", p.ID)
		fmt.Printf("  then       foundry lc done %s\n\n", p.ID)
		return nil

	case "done":
		id := lc.State.Current
		if len(args) > 1 {
			id = args[1]
		}
		if id == "" {
			return fmt.Errorf("nothing in progress: foundry lc done <id>")
		}
		if lc.isSolved(id) {
			fmt.Printf("\n  %s already solved\n\n", id)
			return nil
		}
		lc.State.Solved = append(lc.State.Solved, id)
		if lc.State.Current == id {
			lc.State.Current = ""
		}
		if err := lc.save(); err != nil {
			return err
		}
		fmt.Printf("\n  %s solved · %d/%d\n\n", id, len(lc.State.Solved), len(lc.Problems))
		return nil

	case "focus":
		if len(args) < 2 {
			return fmt.Errorf("usage: foundry lc focus <pattern|none>")
		}
		if args[1] == "none" {
			lc.State.Focus = ""
		} else {
			lc.State.Focus = args[1]
		}
		if err := lc.save(); err != nil {
			return err
		}
		fmt.Printf("\n  focus: %s\n\n", lc.focusLabel())
		return nil

	case "reset":
		lc.State = LCState{Solved: []string{}}
		if err := lc.save(); err != nil {
			return err
		}
		fmt.Printf("\n  leetcode progress reset · %d problems available\n", len(lc.Problems))
		fmt.Printf("  nothing else was touched\n\n")
		return nil

	case "list":
		byPattern := map[string][]LCProblem{}
		for _, p := range lc.Problems {
			byPattern[p.Pattern] = append(byPattern[p.Pattern], p)
		}
		var pats []string
		for k := range byPattern {
			pats = append(pats, k)
		}
		sort.Strings(pats)
		fmt.Println()
		for _, pat := range pats {
			ps := byPattern[pat]
			solved := 0
			for _, p := range ps {
				if lc.isSolved(p.ID) {
					solved++
				}
			}
			fmt.Printf("  %-22s %d/%d\n", pat, solved, len(ps))
		}
		fmt.Println()
		return nil
	}
	return fmt.Errorf("unknown lc subcommand %q (next, done, focus, reset, list)", sub)
}

// addDays shifts a YYYY-MM-DD date forward, used by the drill scheduler.
func addDays(date string, days int) string {
	t, err := time.Parse(dateFmt, date)
	if err != nil {
		t = time.Now()
	}
	return t.AddDate(0, 0, days).Format(dateFmt)
}
