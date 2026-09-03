package main

import "testing"

func texts(ls []line) []string {
	out := make([]string, len(ls))
	for i, l := range ls {
		out[i] = l.text
	}
	return out
}

func eq(t *testing.T, got, want []string) {
	t.Helper()
	if len(got) != len(want) {
		t.Fatalf("got %d lines %q, want %d %q", len(got), got, len(want), want)
	}
	for i := range got {
		if got[i] != want[i] {
			t.Errorf("line %d: got %q, want %q", i, got[i], want[i])
		}
	}
}

func TestSignificantGo(t *testing.T) {
	src := "package main\n" +
		"// a comment\n" +
		"/* block\n   spanning */\n" +
		"func main() {\n" +
		"\turl := \"https://x.dev//not-a-comment\" // trailing\n" +
		"\n" +
		"}\n"
	eq(t, texts(significant(src, syntaxFor["go"])), []string{
		"package main",
		"func main() {",
		`url := "https://x.dev//not-a-comment"`,
		"}",
	})
}

func TestSignificantPython(t *testing.T) {
	src := "import os\n" +
		"# a comment\n" +
		"fmt = \"count: #1\"  # trailing\n" +
		"def main():\n" +
		"    return os.getcwd()\n"
	eq(t, texts(significant(src, syntaxFor["python"])), []string{
		"import os",
		`fmt = "count: #1"`,
		"def main():",
		"return os.getcwd()",
	})
}

// A Python triple-quoted string must survive as code rather than being torn
// apart by the single-quote delimiters it contains.
func TestPythonTripleQuoted(t *testing.T) {
	src := "doc = \"\"\"line one\n# not a comment\nline two\"\"\"\n"
	got := texts(significant(src, syntaxFor["python"]))
	eq(t, got, []string{`doc = """line one`, "# not a comment", `line two"""`})
}

func TestSignificantTypeScript(t *testing.T) {
	src := "const re = /a\\/b/; // trailing\n" +
		"const tpl = `raw // inside`;\n"
	eq(t, texts(significant(src, syntaxFor["typescript"])), []string{
		"const re = /a\\/b/;",
		"const tpl = `raw // inside`;",
	})
}

func TestWhitespaceForgiven(t *testing.T) {
	a := significant("x   :=    1\n", syntaxFor["go"])
	b := significant("\t\tx := 1\n", syntaxFor["go"])
	if a[0].text != b[0].text {
		t.Errorf("whitespace not normalised: %q vs %q", a[0].text, b[0].text)
	}
}

func TestDiffDetectsMissingAndExtra(t *testing.T) {
	ref := significant("a := 1\nb := 2\nc := 3\n", syntaxFor["go"])
	att := significant("a := 1\nc := 3\nd := 4\n", syntaxFor["go"])

	var missing, extra int
	for _, o := range diffLines(ref, att) {
		switch o.kind {
		case opMissing:
			missing++
		case opExtra:
			extra++
		}
	}
	if missing != 1 || extra != 1 {
		t.Errorf("got %d missing / %d extra, want 1 / 1", missing, extra)
	}
}

func TestDiffCleanWhenIdentical(t *testing.T) {
	src := "a := 1\nb := 2\n"
	ref := significant(src, syntaxFor["go"])
	for _, o := range diffLines(ref, significant(src, syntaxFor["go"])) {
		if o.kind != opSame {
			t.Errorf("identical input reported a difference: %+v", o)
		}
	}
}

// Line numbers must point back into the original file, not the stripped copy.
func TestLineNumbersSurviveStripping(t *testing.T) {
	src := "package main\n\n// gap\n\nvar x = 1\n"
	got := significant(src, syntaxFor["go"])
	if got[len(got)-1].no != 5 {
		t.Errorf("got line %d for `var x = 1`, want 5", got[len(got)-1].no)
	}
}

func TestUnterminatedStringDoesNotHang(t *testing.T) {
	if got := significant("x := \"unterminated\n", syntaxFor["go"]); len(got) == 0 {
		t.Error("expected at least one line from unterminated input")
	}
}

func TestSignificantScala(t *testing.T) {
	src := "// leading comment\n" +
		"val host = \"127.0.0.1\" // trailing\n" +
		"val url = s\"http://$host//api\"\n"
	eq(t, texts(significant(src, syntaxFor["scala"])), []string{
		`val host = "127.0.0.1"`,
		`val url = s"http://$host//api"`,
	})
}

// Scala nests block comments, so the inner */ must not end the outer comment.
func TestScalaNestedBlockComment(t *testing.T) {
	src := "val a = 1\n/* outer /* inner */ still comment */\nval b = 2\n"
	eq(t, texts(significant(src, syntaxFor["scala"])), []string{"val a = 1", "val b = 2"})
}

// Go does not nest, so the first */ closes and the trailing text is code.
func TestGoBlockCommentDoesNotNest(t *testing.T) {
	src := "/* outer /* inner */\nvar b = 2\n"
	got := texts(significant(src, syntaxFor["go"]))
	if len(got) != 1 || got[0] != "var b = 2" {
		t.Errorf("got %q, want [\"var b = 2\"]", got)
	}
}

func TestScalaTripleQuoted(t *testing.T) {
	src := "val q = \"\"\"has // slashes and /* stars */\"\"\"\n"
	eq(t, texts(significant(src, syntaxFor["scala"])), []string{
		`val q = """has // slashes and /* stars */"""`,
	})
}
