package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

// cmdFocus sets which language track is active. Everything else — status, the
// hook, what a session works on next — follows from this one value.
func cmdFocus(root string, args []string) error {
	prog, err := loadProgress(root)
	if err != nil {
		return err
	}
	if len(args) == 0 {
		fmt.Printf("\n  focus: %s\n\n", prog.Focus)
		return nil
	}
	lang, err := normLang(args[0])
	if err != nil {
		return err
	}
	spine, err := loadSpine(root)
	if err != nil {
		return err
	}
	prog.Focus = lang
	if err := saveProgress(root, prog); err != nil {
		return err
	}

	// Report where this track stands so switching is never disorienting.
	mods := spine.Modules[lang]
	done := 0
	for _, m := range mods {
		if mp := prog.Modules[lang+"/"+m.ID]; mp != nil && mp.Stage >= m.maxStage() {
			done++
		}
	}
	fmt.Printf("\n  focus: %s · %d/%d modules applied\n", lang, done, len(mods))
	fmt.Printf("  say \"go\" in Claude Code to start the session\n\n")
	return nil
}

// cmdStage records how far a module has moved through the five stages. The
// agent calls this at the end of a session so bookkeeping never means
// hand-editing JSON.
func cmdStage(root string, args []string) error {
	if len(args) < 3 {
		return fmt.Errorf("usage: foundry stage <lang> <module> <0-5>")
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
	n, err := strconv.Atoi(args[2])
	if err != nil || n < 0 || n > mod.maxStage() {
		return fmt.Errorf("stage must be 0-%d for %s, got %q", mod.maxStage(), mod.ID, args[2])
	}
	prog, err := loadProgress(root)
	if err != nil {
		return err
	}
	key := lang + "/" + mod.ID
	mp := prog.Modules[key]
	if mp == nil {
		mp = &ModuleProgress{Started: today()}
		prog.Modules[key] = mp
	}
	mp.Stage = n
	if n >= mod.maxStage() && mp.Completed == "" {
		mp.Completed = today()
	}
	if err := saveProgress(root, prog); err != nil {
		return err
	}
	fmt.Printf("\n  %s → stage %d (%s)\n\n", key, n, stageNames[n])
	return nil
}

func cmdGuide(root string) error {
	spine, err := loadSpine(root)
	if err != nil {
		return err
	}
	var langs []string
	for _, l := range spine.Languages {
		langs = append(langs, l)
	}

	fmt.Printf(`
  FOUNDRY — how to use it

  STARTING A TRACK
    foundry focus go            pick the language track (%s)
    say "go" in Claude Code     that is the whole session interface

    Everything else is written for you: the lesson, the example, your progress,
    the recall schedule, and the vault notes. There is no setup step per module.

  THE LOOP
    Each concept moves through five stages, spread across sessions.

      1 read        one screenful — what it is, why it exists
      2 mechanism   what actually happens underneath
      3 type along  retype example.go with it visible
      4 recall      retype it days later, from memory
      5 apply       build something small with it
      6 variants    the competing approaches, and when each wins

    Stage 4 is where retention actually comes from. It is scheduled, not
    requested — status tells you when it is due.

  DRILLS
    foundry drill go 00         create the file you type into
    foundry drill go 00 --blind same, but do not open the reference
    foundry check go 00         diff what you typed against the reference
    foundry drill go 00 --pass  clean run — doubles the interval
    foundry drill go 00 --lapse missed it — back to one day

    Comments and whitespace are forgiven. Names, order and structure are not.

  VARIANTS
    foundry variants go 10-errors        the design space for a module
    foundry variants add go 00 "..." "..."   record one you just hit

    Some things have one right answer. Most do not. Error handling, receivers,
    concurrency, serialization, class shapes — knowing which to reach for is
    the skill, and it is what separates writing code from reviewing it.

    Variants are seeded for the obvious modules but can be added to any of
    them. The first one opens stage 6 for that module. You write your
    judgement first, then compare against the reasoning.

  LEETCODE
    foundry lc                  dispense a problem
    foundry lc focus two-pointers   restrict to one pattern
    foundry lc done <id>        mark solved
    foundry lc list             progress by pattern
    foundry lc reset            wipe leetcode progress, nothing else

  INTERVIEW
    foundry iv                  dispense a question, create a blank answer file
    foundry iv ref <id>         reveal the reference — refuses until you write
    foundry iv done <id>        mark answered
    foundry iv focus python     restrict by language, topic or level
    foundry iv reset            wipe interview progress, keeps your answers

    Write the answer out in full. Answering in your head feels like knowing.

  BOOKKEEPING
    foundry status              where you are and what is due
    foundry stage go 00 3       record stage progress (the agent does this)

  MODULE NAMES
    Prefixes resolve, so "00" means 00-values-and-types. Languages accept
    short forms: go, ts, py, sc.

`, strings.Join(langs, ", "))
	return nil
}

// firstRun reports whether nothing has been started yet, so status can show
// orientation instead of an empty dashboard.
func firstRun(root string) bool {
	prog, err := loadProgress(root)
	if err != nil {
		return true
	}
	for _, mp := range prog.Modules {
		if mp.Stage > 0 {
			return false
		}
	}
	return true
}

func saveProgress(root string, p *Progress) error {
	return saveJSON(filepath.Join(root, "state", "progress.json"), p)
}

// cmdVariants prints the design space for a module: the credible approaches and
// the condition under which each is the right call. Reading them is stage 6.
func cmdVariants(root string, args []string) error {
	if len(args) > 0 && args[0] == "add" {
		return addVariant(root, args[1:])
	}
	if len(args) < 2 {
		return fmt.Errorf("usage: foundry variants <lang> <module>\n" +
			"       foundry variants add <lang> <module> \"<name>\" \"<when>\"")
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
	if len(mod.Variants) == 0 {
		fmt.Printf("\n  %s · %s — no variants recorded yet\n\n", lang, mod.Title)
		fmt.Printf("  that is a default, not a verdict. add one the moment a second\n")
		fmt.Printf("  credible approach comes up:\n\n")
		fmt.Printf("    foundry variants add %s %s \"<approach>\" \"<when it wins>\"\n\n",
			args[0], mod.ID)
		fmt.Printf("  the first one opens stage 6 for this module.\n\n")
		return nil
	}

	fmt.Printf("\n  %s · %s — %d ways to do it\n\n", lang, mod.Title, len(mod.Variants))
	width := 0
	for _, v := range mod.Variants {
		if len(v.Name) > width {
			width = len(v.Name)
		}
	}
	for _, v := range mod.Variants {
		fmt.Printf("  %-*s  %s\n", width, v.Name, wrapText(v.When, 74-width, strings.Repeat(" ", width+4)))
	}

	dir := filepath.Join(moduleDir(root, lang, mod.ID), "variants")
	take := filepath.Join(dir, "my-take.md")
	fmt.Printf("\n  implementations   %s\n", mustRel(root, dir))
	if _, err := os.Stat(take); err == nil {
		fmt.Printf("  your judgement    %s (written)\n\n", mustRel(root, take))
	} else {
		fmt.Printf("  your judgement    %s\n", mustRel(root, take))
		fmt.Printf("\n  pick one and argue for it before reading the reference reasoning.\n\n")
	}
	return nil
}

// addVariant records an approach discovered mid-session. Variants are seeded in
// foundry.json for the modules where the design space is obvious, but that seed
// is a starting point, not a whitelist — a second credible way to do something
// can surface in any module, and recording it there is how it stops being lost.
func addVariant(root string, args []string) error {
	if len(args) < 4 {
		return fmt.Errorf(`usage: foundry variants add <lang> <module> "<name>" "<when it wins>"`)
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
	name, when := strings.TrimSpace(args[2]), strings.TrimSpace(args[3])
	if name == "" || when == "" {
		return fmt.Errorf("both the approach and when it wins are required")
	}

	mods := spine.Modules[lang]
	for i := range mods {
		if mods[i].ID != mod.ID {
			continue
		}
		for _, v := range mods[i].Variants {
			if strings.EqualFold(v.Name, name) {
				return fmt.Errorf("%s already records %q", mod.ID, v.Name)
			}
		}
		opened := len(mods[i].Variants) == 0
		mods[i].Variants = append(mods[i].Variants, Variant{Name: name, When: when})
		if err := saveJSON(filepath.Join(root, "foundry.json"), spine); err != nil {
			return err
		}
		fmt.Printf("\n  %s · %s\n", lang, mod.Title)
		fmt.Printf("  + %s — %s\n", name, when)
		fmt.Printf("\n  now %d recorded", len(mods[i].Variants))
		if opened {
			fmt.Printf(" · stage 6 is now open for this module")
		}
		fmt.Printf("\n\n")
		return nil
	}
	return fmt.Errorf("could not locate %s in the spine", mod.ID)
}
