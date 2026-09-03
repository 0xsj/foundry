package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

var stageNames = []string{"untouched", "read", "mechanism", "type-along", "recall", "applied", "compared"}

// nextStageHint describes what a session should do next for a module.
func nextStageHint(stage int, mod Module) string {
	if stage >= mod.maxStage() {
		return "done"
	}
	return stageHint(stage)
}

func stageHint(stage int) string {
	switch stage {
	case 0:
		return "read → mechanism → type-along"
	case 1:
		return "mechanism → type-along"
	case 2:
		return "type-along"
	case 3:
		return "recall (scheduled)"
	case 4:
		return "apply"
	case 5:
		return "variants"
	default:
		return "done"
	}
}

func cmdStatus(root string) error {
	spine, err := loadSpine(root)
	if err != nil {
		return err
	}
	prog, err := loadProgress(root)
	if err != nil {
		return err
	}
	drills, err := loadDrills(root)
	if err != nil {
		return err
	}

	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()

	lang := prog.Focus
	if lang == "" {
		lang = "go"
	}
	mods := spine.Modules[lang]

	// The current module is the first that has not been applied.
	var current Module
	var currentStage int
	done := 0
	for _, m := range mods {
		mp := prog.Modules[lang+"/"+m.ID]
		st := 0
		if mp != nil {
			st = mp.Stage
		}
		if st >= m.maxStage() {
			done++
			continue
		}
		if current.ID == "" {
			current, currentStage = m, st
		}
	}

	tierName := ""
	for _, t := range spine.Tiers {
		if t.ID == current.Tier {
			tierName = t.Name
		}
	}

	fmt.Fprintf(w, "\nfoundry — %s · tier %d %s\n", lang, current.Tier, tierName)

	if current.ID == "" {
		fmt.Fprintf(w, "  every %s module is applied. switch focus or go deeper.\n\n", lang)
		return nil
	}
	fmt.Fprintf(w, "  now   %-28s %s\n", current.ID, nextStageHint(currentStage, current))

	// Recall drills that have come due.
	var due []string
	for _, d := range drills.Drills {
		if d.Due <= today() {
			due = append(due, d.Key)
		}
	}
	sort.Strings(due)
	switch len(due) {
	case 0:
		fmt.Fprintf(w, "  due   %-28s %s\n", "nothing", "no recall scheduled")
	default:
		short := make([]string, 0, len(due))
		for _, k := range due {
			short = append(short, strings.TrimPrefix(k, lang+"/"))
		}
		label := fmt.Sprintf("%d recall drill", len(due))
		if len(due) > 1 {
			label += "s"
		}
		fmt.Fprintf(w, "  due   %-28s %s\n", label, strings.Join(short, ", "))
	}

	// LeetCode and interview lines, only if those banks are present.
	if lc, err := loadLC(root); err == nil {
		fmt.Fprintf(w, "  lc    %-28s %d solved · %d remaining\n",
			lc.focusLabel(), len(lc.State.Solved), lc.remaining())
	}
	if iv, err := loadIV(root); err == nil {
		fmt.Fprintf(w, "  iv    %-28s %d answered · %d remaining\n",
			iv.focusLabel(), len(iv.State.Answered), iv.remaining())
	}

	fmt.Fprintf(w, "\n  %d/%d %s modules applied\n", done, len(mods), lang)
	if len(current.Mechanism) > 0 {
		fmt.Fprintf(w, "  next up covers: %s\n", strings.Join(current.Mechanism, " · "))
	}
	if len(current.Variants) > 0 {
		fmt.Fprintf(w, "  %d ways to do it: foundry variants %s %s\n",
			len(current.Variants), lang, current.ID)
	}

	// On a fresh track the dashboard alone is not enough — say what to do.
	if firstRun(root) {
		fmt.Fprintf(w, `
  ── nothing started yet ──────────────────────────────────

  say "go" in Claude Code and the session runs itself: the
  lesson gets written, you type the example, and progress,
  recall and notes are recorded for you.

  another track   foundry focus ts    (go, ts, py)
  the manual      foundry guide
`)
	} else {
		fmt.Fprintf(w, "  manual: foundry guide\n")
	}
	fmt.Fprintln(w)
	return nil
}

// cmdDrill scaffolds an attempt file, and with --pass records a clean recall.
func cmdDrill(root string, args []string) error {
	if len(args) < 2 {
		return fmt.Errorf("usage: foundry drill <lang> <module> [--blind|--pass|--lapse]")
	}
	lang, err := normLang(args[0])
	if err != nil {
		return err
	}
	spine, err := loadSpine(root)
	if err != nil {
		return err
	}
	mod, err := resolveModule(spine, lang, args[1])
	if err != nil {
		return err
	}
	flag := ""
	if len(args) > 2 {
		flag = args[2]
	}

	key := lang + "/" + mod.ID
	switch flag {
	case "--pass":
		return recordDrill(root, key, true)
	case "--lapse":
		return recordDrill(root, key, false)
	}

	refPath, err := exampleFile(root, lang, mod.ID)
	if err != nil {
		return err
	}
	attDir := filepath.Join(moduleDir(root, lang, mod.ID), "attempt")
	if err := os.MkdirAll(attDir, 0o755); err != nil {
		return err
	}
	attPath := filepath.Join(attDir, filepath.Base(refPath))
	if _, err := os.Stat(attPath); err == nil {
		fmt.Printf("\n  attempt already exists: %s\n", mustRel(root, attPath))
		fmt.Printf("  delete it to start over, or run: foundry check %s %s\n\n", args[0], mod.ID)
		return nil
	}
	if err := os.WriteFile(attPath, nil, 0o644); err != nil {
		return err
	}

	fmt.Printf("\n  %s · %s\n\n", lang, mod.Title)
	fmt.Printf("  type into   %s\n", mustRel(root, attPath))
	if flag == "--blind" {
		fmt.Printf("  from memory — do not open the reference\n")
	} else {
		fmt.Printf("  reference   %s\n", mustRel(root, refPath))
	}
	fmt.Printf("\n  then run    foundry check %s %s\n\n", args[0], mod.ID)
	return nil
}

// recordDrill applies the spacing schedule: a clean pass doubles the interval,
// a lapse drops it back to one day.
func recordDrill(root, key string, passed bool) error {
	drills, err := loadDrills(root)
	if err != nil {
		return err
	}
	var d *Drill
	for _, existing := range drills.Drills {
		if existing.Key == key {
			d = existing
			break
		}
	}
	if d == nil {
		d = &Drill{Key: key, Interval: 1}
		drills.Drills = append(drills.Drills, d)
	}
	if passed {
		d.Reps++
		if d.Interval < 1 {
			d.Interval = 1
		}
		d.Interval *= 2
		if d.Interval > 60 {
			d.Interval = 60
		}
	} else {
		d.Lapses++
		d.Interval = 1
	}
	d.Due = addDays(today(), d.Interval)
	if err := saveDrills(root, drills); err != nil {
		return err
	}
	verb := "lapsed"
	if passed {
		verb = "passed"
	}
	fmt.Printf("\n  %s %s · next recall in %d day(s), on %s\n\n", key, verb, d.Interval, d.Due)
	return nil
}
