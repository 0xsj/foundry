package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

// toolchain is how one track actually gets run. The probe is a complete program
// that must print "ok" — the point is proving the language executes here, not
// that some binary happens to exist on PATH.
type toolchain struct {
	lang    string
	bin     string
	version []string
	run     []string // the file path is appended
	probe   string
}

var toolchains = []toolchain{
	{
		lang: "go", bin: "go",
		version: []string{"go", "version"},
		run:     []string{"go", "run"},
		probe:   "package main\nimport \"fmt\"\nfunc main() { fmt.Println(\"ok\", 1+2) }\n",
	},
	{
		lang: "typescript", bin: "bun",
		version: []string{"bun", "--version"},
		run:     []string{"bun"},
		probe:   "const xs: number[] = [1, 2, 3];\nconsole.log(\"ok\", xs.reduce((a, b) => a + b, 0));\n",
	},
	{
		lang: "python", bin: "python3",
		version: []string{"python3", "--version"},
		run:     []string{"python3"},
		probe:   "xs = [1, 2, 3]\nprint(\"ok\", sum(xs))\n",
	},
	{
		lang: "scala", bin: "scala-cli",
		version: []string{"scala-cli", "version", "--cli-version"},
		run:     []string{"scala-cli", "run"},
		probe: "//> using scala 3.8.4\n//> using jvm 25\n" +
			"@main def hi(): Unit = println(s\"ok ${List(1,2,3).sum}\")\n",
	},
	{
		// runghc compiles to a temp binary and runs it, so this proves the
		// compiler works, not just that an interpreter loaded the file.
		lang: "haskell", bin: "runghc",
		version: []string{"ghc", "--numeric-version"},
		run:     []string{"runghc"},
		probe: "import Data.List (foldl')\n" +
			"main :: IO ()\n" +
			"main = putStrLn (\"ok \" ++ show (foldl' (+) 0 [1, 2, 3 :: Int]))\n",
	},
}

// cmdDoctor runs a real program in every track and reports what happened. A
// track that cannot execute is worse than a missing one: the lesson gets
// written, the drill passes on text, and the code was never once run.
func cmdDoctor(root string, args []string) error {
	spine, err := loadSpine(root)
	if err != nil {
		return err
	}
	want := map[string]bool{}
	for _, l := range spine.Languages {
		want[l] = true
	}

	dir, err := os.MkdirTemp("", "foundry-doctor")
	if err != nil {
		return err
	}
	defer os.RemoveAll(dir)

	fmt.Printf("\n  running a real program in each track\n\n")
	failed := 0
	for _, tc := range toolchains {
		if !want[tc.lang] {
			continue
		}
		status, detail := probeToolchain(dir, tc)
		if status != "ok" {
			failed++
		}
		fmt.Printf("  %-11s %-4s %s\n", tc.lang, status, detail)
	}

	fmt.Println()
	if failed > 0 {
		return fmt.Errorf("%d of %d tracks cannot run code", failed, len(want))
	}
	fmt.Printf("  all %d tracks execute\n\n", len(want))
	return nil
}

func probeToolchain(dir string, tc toolchain) (status, detail string) {
	if _, err := exec.LookPath(tc.bin); err != nil {
		return "MISS", tc.bin + " not on PATH"
	}

	ver := "unknown"
	if out, err := exec.Command(tc.version[0], tc.version[1:]...).Output(); err == nil {
		ver = strings.TrimSpace(strings.SplitN(string(out), "\n", 2)[0])
	}

	ext := sourceExt[tc.lang]
	path := filepath.Join(dir, "probe_"+tc.lang+"."+ext)
	if err := os.WriteFile(path, []byte(tc.probe), 0o644); err != nil {
		return "FAIL", err.Error()
	}

	start := time.Now()
	cmd := exec.Command(tc.run[0], append(append([]string{}, tc.run[1:]...), path)...)
	cmd.Dir = dir
	out, err := cmd.CombinedOutput()
	took := time.Since(start).Round(time.Millisecond)

	if err != nil {
		line := firstMeaningfulLine(string(out))
		return "FAIL", fmt.Sprintf("%s — %s", ver, line)
	}
	if !strings.Contains(string(out), "ok") {
		return "FAIL", fmt.Sprintf("%s — ran but printed %q", ver, strings.TrimSpace(string(out)))
	}
	return "ok", fmt.Sprintf("%s (%s)", ver, took)
}

// progressNoise is chatter the JVM and Node toolchains emit before the real
// problem, which would otherwise be reported as the diagnosis.
var progressNoise = []string{"Downloading", "Downloaded", "Checking", "Compiling", "Fetching"}

// firstMeaningfulLine prefers a line that actually names a failure, and only
// falls back to the first non-noise line when nothing looks like an error.
func firstMeaningfulLine(s string) string {
	lines := strings.Split(s, "\n")
	for _, want := range []bool{true, false} {
		for _, ln := range lines {
			t := strings.TrimSpace(ln)
			if t == "" || hasAnyPrefix(t, progressNoise) {
				continue
			}
			if looksLikeError(t) != want {
				continue
			}
			return truncate(t, 90)
		}
	}
	return "no output"
}

func looksLikeError(s string) bool {
	l := strings.ToLower(s)
	for _, m := range []string{"error", "cannot", "not found", "failed", "no such", "unable"} {
		if strings.Contains(l, m) {
			return true
		}
	}
	return false
}

func hasAnyPrefix(s string, prefixes []string) bool {
	for _, p := range prefixes {
		if strings.HasPrefix(s, p) {
			return true
		}
	}
	return false
}

func truncate(s string, n int) string {
	if len(s) <= n {
		return s
	}
	return s[:n] + "…"
}
