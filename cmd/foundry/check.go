package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// commentSyntax describes how a language marks comments and strings. Go and
// TypeScript share a shape; Python does not, so the stripper is parameterised
// rather than assuming C-style syntax.
type commentSyntax struct {
	line       string
	blockOpen  string
	blockClose string
	nestBlocks bool     // Scala nests block comments; C-family languages do not
	strings    []string // delimiters, longest first so """ beats "
}

var syntaxFor = map[string]commentSyntax{
	"go":         {line: "//", blockOpen: "/*", blockClose: "*/", strings: []string{`"`, "'", "`"}},
	"typescript": {line: "//", blockOpen: "/*", blockClose: "*/", strings: []string{`"`, "'", "`"}},
	"python":     {line: "#", strings: []string{`"""`, `'''`, `"`, `'`}},
	"scala": {line: "//", blockOpen: "/*", blockClose: "*/", nestBlocks: true,
		strings: []string{`"""`, `"`, `'`}},
}

// stripComments removes comments while respecting string literals, so a // in a
// URL or a # in a format string is not mistaken for a comment. Newlines inside
// stripped regions are preserved to keep reported line numbers honest.
func stripComments(src string, syn commentSyntax) string {
	var out strings.Builder
	i := 0
	for i < len(src) {
		rest := src[i:]

		if syn.blockOpen != "" && strings.HasPrefix(rest, syn.blockOpen) {
			end := blockCommentEnd(src, i, syn)
			out.WriteString(strings.Repeat("\n", strings.Count(src[i:end], "\n")))
			i = end
			continue
		}

		if syn.line != "" && strings.HasPrefix(rest, syn.line) {
			j := strings.IndexByte(rest, '\n')
			if j < 0 {
				break
			}
			i += j // leave the newline for the next pass
			continue
		}

		if delim := matchDelim(rest, syn.strings); delim != "" {
			end := closingDelim(src, i+len(delim), delim)
			out.WriteString(src[i:end])
			i = end
			continue
		}

		out.WriteByte(src[i])
		i++
	}
	return out.String()
}

// blockCommentEnd returns the index just past the comment opening at start.
// Scala nests block comments, so a depth counter is needed rather than a search
// for the first close delimiter. An unterminated comment runs to end of input.
func blockCommentEnd(src string, start int, syn commentSyntax) int {
	depth := 0
	i := start
	for i < len(src) {
		switch {
		case strings.HasPrefix(src[i:], syn.blockOpen) && (depth == 0 || syn.nestBlocks):
			depth++
			i += len(syn.blockOpen)
		case strings.HasPrefix(src[i:], syn.blockClose):
			depth--
			i += len(syn.blockClose)
			if depth == 0 {
				return i
			}
		default:
			i++
		}
	}
	return len(src)
}

func matchDelim(rest string, delims []string) string {
	for _, d := range delims {
		if strings.HasPrefix(rest, d) {
			return d
		}
	}
	return ""
}

// closingDelim returns the index just past the delimiter that closes a string
// opened at from, honouring backslash escapes. An unterminated literal runs to
// end of input rather than panicking.
func closingDelim(src string, from int, delim string) int {
	for i := from; i < len(src); i++ {
		if src[i] == '\\' {
			i++
			continue
		}
		if strings.HasPrefix(src[i:], delim) {
			return i + len(delim)
		}
	}
	return len(src)
}

// line is a significant line of code paired with its position in the original file.
type line struct {
	no   int
	text string
}

// significant returns code lines with comments and blank lines removed and
// interior whitespace collapsed. Names, order and structure still count — only
// formatting and commentary are forgiven.
func significant(src string, syn commentSyntax) []line {
	var out []line
	for i, raw := range strings.Split(stripComments(src, syn), "\n") {
		t := strings.Join(strings.Fields(raw), " ")
		if t == "" {
			continue
		}
		out = append(out, line{no: i + 1, text: t})
	}
	return out
}

type opKind int

const (
	opSame opKind = iota
	opMissing
	opExtra
)

type op struct {
	kind opKind
	ref  line
	att  line
}

// diffLines is a standard LCS backtrack. Examples are 15-40 lines, so the
// quadratic table is irrelevant and the exact alignment is worth having.
func diffLines(ref, att []line) []op {
	n, m := len(ref), len(att)
	lcs := make([][]int, n+1)
	for i := range lcs {
		lcs[i] = make([]int, m+1)
	}
	for i := n - 1; i >= 0; i-- {
		for j := m - 1; j >= 0; j-- {
			if ref[i].text == att[j].text {
				lcs[i][j] = lcs[i+1][j+1] + 1
			} else if lcs[i+1][j] >= lcs[i][j+1] {
				lcs[i][j] = lcs[i+1][j]
			} else {
				lcs[i][j] = lcs[i][j+1]
			}
		}
	}
	var ops []op
	i, j := 0, 0
	for i < n && j < m {
		switch {
		case ref[i].text == att[j].text:
			ops = append(ops, op{kind: opSame, ref: ref[i], att: att[j]})
			i, j = i+1, j+1
		case lcs[i+1][j] >= lcs[i][j+1]:
			ops = append(ops, op{kind: opMissing, ref: ref[i]})
			i++
		default:
			ops = append(ops, op{kind: opExtra, att: att[j]})
			j++
		}
	}
	for ; i < n; i++ {
		ops = append(ops, op{kind: opMissing, ref: ref[i]})
	}
	for ; j < m; j++ {
		ops = append(ops, op{kind: opExtra, att: att[j]})
	}
	return ops
}

func cmdCheck(root string, args []string) error {
	if len(args) < 2 {
		return fmt.Errorf("usage: foundry check <lang> <module>")
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
	refPath, err := exampleFile(root, lang, mod.ID)
	if err != nil {
		return err
	}
	attPath := filepath.Join(moduleDir(root, lang, mod.ID), "attempt", filepath.Base(refPath))
	attBytes, err := os.ReadFile(attPath)
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("no attempt yet — run: foundry drill %s %s", args[0], mod.ID)
		}
		return err
	}
	refBytes, err := os.ReadFile(refPath)
	if err != nil {
		return err
	}

	syn := syntaxFor[lang]
	ref := significant(string(refBytes), syn)
	att := significant(string(attBytes), syn)
	ops := diffLines(ref, att)

	var missing, extra int
	for _, o := range ops {
		switch o.kind {
		case opMissing:
			missing++
		case opExtra:
			extra++
		}
	}

	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()

	fmt.Fprintf(w, "\n  %s · %s\n", lang, mod.Title)
	fmt.Fprintf(w, "  %s\n\n", mustRel(root, attPath))

	if missing == 0 && extra == 0 {
		fmt.Fprintf(w, "  clean — %d/%d lines\n\n", len(ref), len(ref))
		fmt.Fprintf(w, "  mark it: foundry drill %s %s --pass\n\n", args[0], mod.ID)
		return nil
	}

	for _, o := range ops {
		switch o.kind {
		case opMissing:
			fmt.Fprintf(w, "  - %3d  %s\n", o.ref.no, o.ref.text)
		case opExtra:
			fmt.Fprintf(w, "  + %3d  %s\n", o.att.no, o.att.text)
		}
	}
	matched := len(ref) - missing
	fmt.Fprintf(w, "\n  %d/%d lines · %d missing · %d extra\n", matched, len(ref), missing, extra)
	fmt.Fprintf(w, "  - is in the reference and not in yours; + is yours and not in the reference\n\n")
	return nil
}
