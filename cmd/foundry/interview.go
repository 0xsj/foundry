package main

import (
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

type Question struct {
	ID        string   `json:"id"`
	Topic     string   `json:"topic"`
	Lang      string   `json:"lang"`
	Level     string   `json:"level"`
	Question  string   `json:"question"`
	Probes    []string `json:"probes"`
	Reference string   `json:"reference"`
}

type IVState struct {
	Focus    string   `json:"focus"`
	Current  string   `json:"current"`
	Answered []string `json:"answered"`
}

type IV struct {
	Questions []Question
	State     IVState
	root      string
}

func loadIV(root string) (*IV, error) {
	iv := &IV{root: root, State: IVState{Answered: []string{}}}
	if err := loadJSON(filepath.Join(root, "interview", "questions.json"), &iv.Questions); err != nil {
		return nil, err
	}
	err := loadJSON(filepath.Join(root, "interview", "state.json"), &iv.State)
	if err != nil && !os.IsNotExist(err) {
		return nil, err
	}
	if iv.State.Answered == nil {
		iv.State.Answered = []string{}
	}
	return iv, nil
}

func (iv *IV) save() error {
	return saveJSON(filepath.Join(iv.root, "interview", "state.json"), &iv.State)
}

func (iv *IV) isAnswered(id string) bool {
	for _, a := range iv.State.Answered {
		if a == id {
			return true
		}
	}
	return false
}

func (iv *IV) remaining() int {
	n := 0
	for _, q := range iv.Questions {
		if !iv.isAnswered(q.ID) {
			n++
		}
	}
	return n
}

func (iv *IV) focusLabel() string {
	if iv.State.Focus == "" {
		return "no focus"
	}
	return iv.State.Focus
}

func (iv *IV) find(id string) (Question, bool) {
	for _, q := range iv.Questions {
		if q.ID == id {
			return q, true
		}
	}
	return Question{}, false
}

var levelRank = map[string]int{"junior": 0, "mid": 1, "senior": 2}

// next dispenses an unanswered question, honouring a language or topic focus and
// working from junior upward so the bank stays approachable from day one.
func (iv *IV) next() (Question, bool) {
	var pool []Question
	for _, q := range iv.Questions {
		if iv.isAnswered(q.ID) {
			continue
		}
		if f := iv.State.Focus; f != "" && q.Lang != f && q.Topic != f && q.Level != f {
			continue
		}
		pool = append(pool, q)
	}
	if len(pool) == 0 {
		return Question{}, false
	}
	sort.Slice(pool, func(i, j int) bool {
		return levelRank[pool[i].Level] < levelRank[pool[j].Level]
	})
	window := 3
	if len(pool) < window {
		window = len(pool)
	}
	return pool[rand.New(rand.NewSource(time.Now().UnixNano())).Intn(window)], true
}

// answerPath is where the user writes. The reference is deliberately not in this
// file — you write first, then reveal, the same shape as a recall drill.
func (iv *IV) answerPath(id string) string {
	return filepath.Join(iv.root, "interview", "answers", id+".md")
}

func (iv *IV) writeAnswerStub(q Question) (string, error) {
	path := iv.answerPath(q.ID)
	if _, err := os.Stat(path); err == nil {
		return path, nil
	}
	var b strings.Builder
	fmt.Fprintf(&b, "# %s\n\n", q.Question)
	fmt.Fprintf(&b, "`%s` · `%s` · `%s`\n\n", q.Lang, q.Topic, q.Level)
	if len(q.Probes) > 0 {
		b.WriteString("Follow-ups you should also be able to answer:\n\n")
		for _, p := range q.Probes {
			fmt.Fprintf(&b, "- %s\n", p)
		}
		b.WriteString("\n")
	}
	b.WriteString("---\n\n" + answerHeading + "\n\n" + answerPlaceholder + "\n\n")
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return "", err
	}
	return path, os.WriteFile(path, []byte(b.String()), 0o644)
}

const (
	answerHeading     = "## My answer"
	answerPlaceholder = "_Write it out as if speaking to an interviewer, then reveal the reference._"
)

// hasWrittenAnswer reports whether anything real sits under the answer heading.
// The stub file exists from the moment a question is dispensed, so file
// existence alone is not evidence that the user has actually answered.
func hasWrittenAnswer(path string) bool {
	b, err := os.ReadFile(path)
	if err != nil {
		return false
	}
	_, after, found := strings.Cut(string(b), answerHeading)
	if !found {
		return true // hand-written file, take it at face value
	}
	for _, ln := range strings.Split(after, "\n") {
		t := strings.TrimSpace(ln)
		if t == "" || t == answerPlaceholder {
			continue
		}
		return true
	}
	return false
}

func wrapText(s string, width int, indent string) string {
	var out strings.Builder
	line := 0
	for _, w := range strings.Fields(s) {
		if line > 0 && line+1+len(w) > width {
			out.WriteString("\n" + indent)
			line = 0
		} else if line > 0 {
			out.WriteString(" ")
			line++
		}
		out.WriteString(w)
		line += len(w)
	}
	return out.String()
}

func cmdIV(root string, args []string) error {
	iv, err := loadIV(root)
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("interview/questions.json is missing")
		}
		return err
	}

	sub := "next"
	if len(args) > 0 {
		sub = args[0]
	}

	switch sub {
	case "next":
		q, ok := iv.next()
		if !ok {
			if iv.State.Focus != "" {
				return fmt.Errorf("no unanswered questions matching focus %q — try: foundry iv focus none",
					iv.State.Focus)
			}
			fmt.Printf("\n  all %d questions answered. foundry iv reset to go again.\n\n", len(iv.Questions))
			return nil
		}
		path, err := iv.writeAnswerStub(q)
		if err != nil {
			return err
		}
		iv.State.Current = q.ID
		if err := iv.save(); err != nil {
			return err
		}
		fmt.Printf("\n  %s · %s · %s\n\n", q.Lang, q.Topic, q.Level)
		fmt.Printf("  %s\n\n", wrapText(q.Question, 68, "  "))
		for _, p := range q.Probes {
			fmt.Printf("    ↳ %s\n", wrapText(p, 64, "      "))
		}
		fmt.Printf("\n  write in   %s\n", mustRel(root, path))
		fmt.Printf("  reveal     foundry iv ref %s\n\n", q.ID)
		return nil

	case "ref":
		id := iv.State.Current
		if len(args) > 1 {
			id = args[1]
		}
		if id == "" {
			return fmt.Errorf("nothing in progress: foundry iv ref <id>")
		}
		q, ok := iv.find(id)
		if !ok {
			return fmt.Errorf("no question %q", id)
		}
		if !hasWrittenAnswer(iv.answerPath(id)) {
			return fmt.Errorf("nothing written yet — answer it first in %s",
				mustRel(root, iv.answerPath(id)))
		}
		fmt.Printf("\n  reference — %s\n\n", q.ID)
		fmt.Printf("  %s\n\n", wrapText(q.Reference, 68, "  "))
		fmt.Printf("  mark it    foundry iv done %s\n\n", q.ID)
		return nil

	case "done":
		id := iv.State.Current
		if len(args) > 1 {
			id = args[1]
		}
		if id == "" {
			return fmt.Errorf("nothing in progress: foundry iv done <id>")
		}
		if !iv.isAnswered(id) {
			iv.State.Answered = append(iv.State.Answered, id)
		}
		if iv.State.Current == id {
			iv.State.Current = ""
		}
		if err := iv.save(); err != nil {
			return err
		}
		fmt.Printf("\n  %s answered · %d/%d\n\n", id, len(iv.State.Answered), len(iv.Questions))
		return nil

	case "focus":
		if len(args) < 2 {
			return fmt.Errorf("usage: foundry iv focus <lang|topic|level|none>")
		}
		if args[1] == "none" {
			iv.State.Focus = ""
		} else {
			iv.State.Focus = args[1]
		}
		if err := iv.save(); err != nil {
			return err
		}
		fmt.Printf("\n  focus: %s\n\n", iv.focusLabel())
		return nil

	case "reset":
		iv.State = IVState{Answered: []string{}}
		if err := iv.save(); err != nil {
			return err
		}
		fmt.Printf("\n  interview progress reset · %d questions available\n", len(iv.Questions))
		fmt.Printf("  your written answers in interview/answers/ were kept\n\n")
		return nil

	case "list":
		byLang := map[string][]Question{}
		for _, q := range iv.Questions {
			byLang[q.Lang] = append(byLang[q.Lang], q)
		}
		var langs []string
		for k := range byLang {
			langs = append(langs, k)
		}
		sort.Strings(langs)
		fmt.Println()
		for _, l := range langs {
			qs := byLang[l]
			done := 0
			for _, q := range qs {
				if iv.isAnswered(q.ID) {
					done++
				}
			}
			fmt.Printf("  %-12s %d/%d\n", l, done, len(qs))
		}
		fmt.Println()
		return nil
	}
	return fmt.Errorf("unknown iv subcommand %q (next, ref, done, focus, reset, list)", sub)
}
