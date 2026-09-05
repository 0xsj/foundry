package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// ---- curriculum spine (authored, read-only) ----

type Spine struct {
	Version   int                 `json:"version"`
	Languages []string            `json:"languages"`
	Tiers     []Tier              `json:"tiers"`
	Modules   map[string][]Module `json:"modules"`
}

type Tier struct {
	ID   int    `json:"id"`
	Name string `json:"name"`
	Goal string `json:"goal"`
}

type Module struct {
	ID        string    `json:"id"`
	Tier      int       `json:"tier"`
	Title     string    `json:"title"`
	Mechanism []string  `json:"mechanism"`
	Stdlib    []string  `json:"stdlib"`
	Variants  []Variant `json:"variants,omitempty"`
}

// Variant is one credible way to solve what a module teaches, paired with the
// condition under which it is the right choice. Modules with no real design
// space carry none and finish at stage 5.
type Variant struct {
	Name string `json:"name"`
	When string `json:"when"`
}

// maxStage is 6 for modules with a design space worth comparing, 5 otherwise.
func (m Module) maxStage() int {
	if len(m.Variants) > 0 {
		return 6
	}
	return 5
}

// ---- progress (mutable) ----

type Progress struct {
	Focus    string                     `json:"focus"`
	Started  string                     `json:"started"`
	Sessions int                        `json:"sessions"`
	Modules  map[string]*ModuleProgress `json:"modules"`
}

// ModuleProgress tracks how far a module has moved through the five stages.
// Stage 0 means untouched; 5 means applied.
type ModuleProgress struct {
	Stage     int    `json:"stage"`
	Started   string `json:"started,omitempty"`
	Completed string `json:"completed,omitempty"`
}

// ---- drill schedule (mutable) ----

type Drills struct {
	Drills []*Drill `json:"drills"`
}

// Drill is one scheduled recall of a module's example file. Interval doubles on a
// clean pass and resets on a lapse, which is enough spacing without a full SM-2.
type Drill struct {
	Key      string `json:"key"` // "go/00-values-and-types"
	Due      string `json:"due"` // YYYY-MM-DD
	Reps     int    `json:"reps"`
	Lapses   int    `json:"lapses"`
	Interval int    `json:"interval"` // days
}

const dateFmt = "2006-01-02"

func today() string { return time.Now().Format(dateFmt) }

// ---- io helpers ----

func loadJSON(path string, v any) error {
	b, err := os.ReadFile(path)
	if err != nil {
		return err
	}
	return json.Unmarshal(b, v)
}

// marshalJSON is the single encoder for everything this tool writes. HTML
// escaping is off so & and < stay readable in the spine, and the em dashes in
// variant text stay raw UTF-8 rather than \u2014.
func marshalJSON(v any) ([]byte, error) {
	var buf bytes.Buffer
	enc := json.NewEncoder(&buf)
	enc.SetIndent("", "  ")
	enc.SetEscapeHTML(false)
	if err := enc.Encode(v); err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

func saveJSON(path string, v any) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	b, err := marshalJSON(v)
	if err != nil {
		return err
	}
	return os.WriteFile(path, b, 0o644)
}

func loadSpine(root string) (*Spine, error) {
	var s Spine
	if err := loadJSON(filepath.Join(root, "foundry.json"), &s); err != nil {
		return nil, fmt.Errorf("reading foundry.json: %w", err)
	}
	return &s, nil
}

// loadProgress tolerates a missing file so a fresh clone works with no setup.
func loadProgress(root string) (*Progress, error) {
	p := &Progress{Focus: "go", Started: today(), Modules: map[string]*ModuleProgress{}}
	err := loadJSON(filepath.Join(root, "state", "progress.json"), p)
	if err != nil && !os.IsNotExist(err) {
		return nil, err
	}
	if p.Modules == nil {
		p.Modules = map[string]*ModuleProgress{}
	}
	return p, nil
}

func loadDrills(root string) (*Drills, error) {
	d := &Drills{}
	err := loadJSON(filepath.Join(root, "state", "drills.json"), d)
	if err != nil && !os.IsNotExist(err) {
		return nil, err
	}
	return d, nil
}

func saveDrills(root string, d *Drills) error {
	return saveJSON(filepath.Join(root, "state", "drills.json"), d)
}

// ---- resolution ----

// normLang accepts the shorthand a person actually types.
func normLang(s string) (string, error) {
	switch strings.ToLower(s) {
	case "go", "golang":
		return "go", nil
	case "ts", "typescript":
		return "typescript", nil
	case "py", "python":
		return "python", nil
	case "sc", "scala":
		return "scala", nil
	case "hs", "haskell":
		return "haskell", nil
	}
	return "", fmt.Errorf("unknown language %q (use go, ts, py, sc or hs)", s)
}

// resolveModule matches on full id first, then unique prefix, so "00" works.
func resolveModule(s *Spine, lang, q string) (Module, error) {
	mods := s.Modules[lang]
	for _, m := range mods {
		if m.ID == q {
			return m, nil
		}
	}
	var hits []Module
	for _, m := range mods {
		if strings.HasPrefix(m.ID, q) {
			hits = append(hits, m)
		}
	}
	switch len(hits) {
	case 1:
		return hits[0], nil
	case 0:
		return Module{}, fmt.Errorf("no %s module matching %q", lang, q)
	default:
		var ids []string
		for _, m := range hits {
			ids = append(ids, m.ID)
		}
		return Module{}, fmt.Errorf("%q is ambiguous: %s", q, strings.Join(ids, ", "))
	}
}

func moduleDir(root, lang, id string) string {
	return filepath.Join(root, "tracks", lang, id)
}

// sourceExt maps a track language to the extension its example file uses.
var sourceExt = map[string]string{
	"go":         "go",
	"typescript": "ts",
	"python":     "py",
	"scala":      "scala",
	"haskell":    "hs",
}

// exampleFile finds the reference file a drill targets. Each module has exactly
// one, named for the language.
func exampleFile(root, lang, id string) (string, error) {
	ext, ok := sourceExt[lang]
	if !ok {
		return "", fmt.Errorf("no file extension known for %q", lang)
	}
	p := filepath.Join(moduleDir(root, lang, id), "example."+ext)
	if _, err := os.Stat(p); err != nil {
		return "", fmt.Errorf("no reference at %s — the module has not been written yet",
			mustRel(root, p))
	}
	return p, nil
}

func mustRel(root, p string) string {
	r, err := filepath.Rel(root, p)
	if err != nil {
		return p
	}
	return r
}
