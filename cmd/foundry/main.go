// Command foundry drives the Foundry learning loop: it reports where you are,
// scaffolds a typing attempt, diffs that attempt against the reference, and
// dispenses leetcode problems.
package main

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
)

func main() {
	if len(os.Args) < 2 {
		usage()
		os.Exit(2)
	}

	root, err := repoRoot()
	if err != nil {
		fail(err)
	}

	var cmdErr error
	switch os.Args[1] {
	case "status":
		cmdErr = cmdStatus(root)
	case "drill":
		cmdErr = cmdDrill(root, os.Args[2:])
	case "check":
		cmdErr = cmdCheck(root, os.Args[2:])
	case "lc":
		cmdErr = cmdLC(root, os.Args[2:])
	case "iv":
		cmdErr = cmdIV(root, os.Args[2:])
	case "focus":
		cmdErr = cmdFocus(root, os.Args[2:])
	case "stage":
		cmdErr = cmdStage(root, os.Args[2:])
	case "variants":
		cmdErr = cmdVariants(root, os.Args[2:])
	case "doctor":
		cmdErr = cmdDoctor(root, os.Args[2:])
	case "guide", "manual":
		cmdErr = cmdGuide(root)
	case "help", "-h", "--help":
		usage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command %q\n\n", os.Args[1])
		usage()
		os.Exit(2)
	}
	if cmdErr != nil {
		fail(cmdErr)
	}
}

func usage() {
	fmt.Print(`foundry — Go, TypeScript, Python, Scala and Haskell, from day one

  guide                     the full manual — start here
  doctor                    run a real program in every track
  status                    where you are and what recall is due
  focus <lang>              pick the active track
  drill <lang> <module>     set up an attempt; --blind hides the reference
  check <lang> <module>     diff your attempt against the reference
  variants <lang> <module>  the competing approaches and when each wins
  stage <lang> <mod> <0-6>  record stage progress
  lc [next|done|focus|reset|list]       leetcode dispenser
  iv [next|ref|done|focus|reset|list]   interview questions

  lang is go, ts, py, sc or hs. module may be a prefix, so "00" resolves to
  00-values-and-types.
`)
}

func fail(err error) {
	fmt.Fprintln(os.Stderr, "foundry:", err)
	os.Exit(1)
}

// repoRoot walks up from the working directory until it finds foundry.json,
// so the CLI works from anywhere inside the repo.
func repoRoot() (string, error) {
	dir, err := os.Getwd()
	if err != nil {
		return "", err
	}
	for {
		if _, err := os.Stat(filepath.Join(dir, "foundry.json")); err == nil {
			return dir, nil
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return "", errors.New("not inside a foundry repo (no foundry.json found)")
		}
		dir = parent
	}
}
