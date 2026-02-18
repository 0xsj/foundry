// Package main demonstrates file operations, directory traversal, and
// environment variable access using the os and path/filepath packages.
// Run with: go run ./files/
//
// Key concepts:
//   - os.Open, os.Create, os.OpenFile for file access
//   - defer f.Close() placement and the write-close error gotcha
//   - os.ReadFile / os.WriteFile for small files
//   - os.Stat, os.Mkdir, os.ReadDir, os.Rename, os.Remove
//   - os.CreateTemp for atomic write patterns
//   - filepath.Join, Ext, Base, Dir, WalkDir
//   - os.Getenv vs os.LookupEnv
package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

func main() {
	// Use a temp directory for all file operations — no test artifacts left behind
	dir, err := os.MkdirTemp("", "foundry-io-*")
	if err != nil {
		fmt.Fprintf(os.Stderr, "create temp dir: %v\n", err)
		os.Exit(1)
	}
	defer os.RemoveAll(dir) // cleanup everything when main exits

	fmt.Printf("Working in temp dir: %s\n\n", dir)

	demoBasicFileRW(dir)
	demoOpenFile(dir)
	demoAtomicWrite(dir)
	demoDirOps(dir)
	demoFilepathPackage(dir)
	demoWalk(dir)
	demoEnvVars()
}

// demoBasicFileRW demonstrates the most common file read/write patterns.
func demoBasicFileRW(dir string) {
	fmt.Println("=== Basic File Read/Write ===")

	path := filepath.Join(dir, "config.yaml")

	// os.WriteFile — writes entire content, creates or truncates
	content := `service:
  name: api-gateway
  port: 8080
  timeout: 30s
`
	if err := os.WriteFile(path, []byte(content), 0644); err != nil {
		fmt.Fprintf(os.Stderr, "write file: %v\n", err)
		return
	}
	fmt.Printf("Wrote %d bytes to %s\n", len(content), filepath.Base(path))

	// os.ReadFile — reads entire content into memory
	data, err := os.ReadFile(path)
	if err != nil {
		fmt.Fprintf(os.Stderr, "read file: %v\n", err)
		return
	}
	fmt.Printf("Read %d bytes:\n%s\n", len(data), data)

	// os.Stat — inspect file metadata
	info, err := os.Stat(path)
	if err != nil {
		fmt.Fprintf(os.Stderr, "stat: %v\n", err)
		return
	}
	fmt.Printf("File info: name=%s, size=%d, mode=%s\n\n", info.Name(), info.Size(), info.Mode())
}

// demoOpenFile demonstrates os.OpenFile with various flag combinations.
func demoOpenFile(dir string) {
	fmt.Println("=== os.OpenFile ===")

	logPath := filepath.Join(dir, "app.log")

	// Append-mode log file — creates if not exists, appends if exists
	writeLog := func(msg string) error {
		f, err := os.OpenFile(logPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
		if err != nil {
			return fmt.Errorf("open log: %w", err)
		}
		defer f.Close() // for read-only or append, ignoring Close error is acceptable

		_, err = fmt.Fprintln(f, msg)
		return err
	}

	entries := []string{
		"2026-02-18 09:00:00 INFO  service started",
		"2026-02-18 09:00:01 INFO  connected to database",
		"2026-02-18 09:15:32 WARN  high memory usage: 87%",
		"2026-02-18 09:16:00 ERROR disk write failed: no space left",
	}
	for _, entry := range entries {
		if err := writeLog(entry); err != nil {
			fmt.Fprintf(os.Stderr, "write log: %v\n", err)
			return
		}
	}

	// Read it back to verify
	data, _ := os.ReadFile(logPath)
	lineCount := len(strings.Split(strings.TrimRight(string(data), "\n"), "\n"))
	fmt.Printf("Log file has %d entries\n\n", lineCount)
}

// demoAtomicWrite demonstrates the temp-file-then-rename pattern for safe writes.
// Writing directly to a file is non-atomic — a crash mid-write leaves a corrupt file.
// With this pattern, the old file is intact until the rename succeeds.
func demoAtomicWrite(dir string) {
	fmt.Println("=== Atomic Write (temp + rename) ===")

	target := filepath.Join(dir, "state.json")

	// Simulate initial state
	if err := os.WriteFile(target, []byte(`{"version":1}`), 0644); err != nil {
		fmt.Fprintf(os.Stderr, "initial write: %v\n", err)
		return
	}

	// Atomic update
	if err := writeAtomic(target, []byte(`{"version":2,"updated":true}`)); err != nil {
		fmt.Fprintf(os.Stderr, "atomic write: %v\n", err)
		return
	}

	data, _ := os.ReadFile(target)
	fmt.Printf("State after atomic write: %s\n\n", data)
}

// writeAtomic writes data to path atomically using a temp file and rename.
// Safe for concurrent access (rename is atomic on most POSIX systems).
func writeAtomic(path string, data []byte) (err error) {
	dir := filepath.Dir(path)

	// Temp file must be on the same filesystem as target for rename to be atomic
	tmp, err := os.CreateTemp(dir, ".tmp-*")
	if err != nil {
		return fmt.Errorf("create temp file: %w", err)
	}
	tmpName := tmp.Name()

	// Cleanup the temp file if anything fails
	defer func() {
		if err != nil {
			os.Remove(tmpName)
		}
	}()

	// Write data, capturing Close error (writes flush on Close)
	if _, err = tmp.Write(data); err != nil {
		tmp.Close()
		return fmt.Errorf("write temp: %w", err)
	}
	if err = tmp.Close(); err != nil {
		return fmt.Errorf("close temp: %w", err)
	}

	// Atomic rename — replaces target if it exists
	if err = os.Rename(tmpName, path); err != nil {
		return fmt.Errorf("rename: %w", err)
	}
	return nil
}

// demoDirOps demonstrates directory creation, listing, and removal.
func demoDirOps(dir string) {
	fmt.Println("=== Directory Operations ===")

	// MkdirAll — like mkdir -p
	subDir := filepath.Join(dir, "reports", "2026", "02")
	if err := os.MkdirAll(subDir, 0755); err != nil {
		fmt.Fprintf(os.Stderr, "mkdirall: %v\n", err)
		return
	}
	fmt.Printf("Created: %s\n", subDir)

	// Create some files in the subdirectory
	for _, name := range []string{"daily.csv", "weekly.csv", "summary.txt"} {
		path := filepath.Join(subDir, name)
		os.WriteFile(path, []byte("placeholder"), 0644)
	}

	// ReadDir — list contents
	entries, err := os.ReadDir(subDir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "readdir: %v\n", err)
		return
	}
	fmt.Printf("Contents of reports/2026/02:\n")
	for _, e := range entries {
		info, _ := e.Info()
		fmt.Printf("  %s (%d bytes)\n", e.Name(), info.Size())
	}

	// Check existence pattern
	missingPath := filepath.Join(dir, "does-not-exist.txt")
	if _, err := os.Stat(missingPath); os.IsNotExist(err) {
		fmt.Printf("Correctly detected: %s does not exist\n", filepath.Base(missingPath))
	}
	fmt.Println()
}

// demoFilepathPackage demonstrates path manipulation functions.
func demoFilepathPackage(dir string) {
	fmt.Println("=== filepath package ===")

	path := filepath.Join(dir, "reports", "2026", "02", "daily.csv")

	fmt.Printf("Full path: %s\n", path)
	fmt.Printf("Dir:       %s\n", filepath.Dir(path))
	fmt.Printf("Base:      %s\n", filepath.Base(path))
	fmt.Printf("Ext:       %s\n", filepath.Ext(path))

	// Stem (filename without extension)
	base := filepath.Base(path)
	ext := filepath.Ext(path)
	stem := strings.TrimSuffix(base, ext)
	fmt.Printf("Stem:      %s\n", stem)

	// Rel — relative path from one location to another
	rel, err := filepath.Rel(dir, path)
	if err == nil {
		fmt.Printf("Rel from base: %s\n", rel)
	}

	// Clean — resolve dots and redundant separators
	messy := filepath.Join(dir, "reports", "..", "reports", "2026", ".", "02", "daily.csv")
	fmt.Printf("Clean path: %s\n", filepath.Clean(messy))
	fmt.Println()
}

// demoWalk demonstrates recursive directory traversal with filepath.WalkDir.
func demoWalk(dir string) {
	fmt.Println("=== filepath.WalkDir ===")

	// Count files by extension
	counts := make(map[string]int)

	err := filepath.WalkDir(dir, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			// Log and skip paths we can't access
			fmt.Fprintf(os.Stderr, "walk error at %s: %v\n", path, err)
			return nil
		}

		if d.IsDir() {
			// Demonstrate SkipDir by skipping temp dirs
			if strings.HasPrefix(d.Name(), ".tmp") {
				return filepath.SkipDir
			}
			return nil
		}

		ext := filepath.Ext(d.Name())
		if ext == "" {
			ext = "(no ext)"
		}
		counts[ext]++
		return nil
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "walk: %v\n", err)
		return
	}

	fmt.Println("Files by extension:")
	for ext, count := range counts {
		fmt.Printf("  %s: %d\n", ext, count)
	}
	fmt.Println()
}

// demoEnvVars demonstrates os.Getenv vs os.LookupEnv.
func demoEnvVars() {
	fmt.Println("=== Environment Variables ===")

	// Getenv — returns "" for both "unset" and "set to empty"
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		fmt.Println("DATABASE_URL not set (using default)")
	}

	// LookupEnv — distinguishes "not set" from "set to empty string"
	if val, ok := os.LookupEnv("DATABASE_URL"); ok {
		fmt.Printf("DATABASE_URL = %q\n", val)
	} else {
		fmt.Println("DATABASE_URL is not present in environment")
	}

	// Set an env var for demonstration (useful in tests, not usually in prod)
	os.Setenv("APP_LOG_LEVEL", "debug")
	level, _ := os.LookupEnv("APP_LOG_LEVEL")
	fmt.Printf("APP_LOG_LEVEL = %q\n", level)
	os.Unsetenv("APP_LOG_LEVEL")

	// PATH is always set — useful to show a real value
	pathVar := os.Getenv("PATH")
	dirs := strings.Split(pathVar, string(os.PathListSeparator))
	fmt.Printf("PATH has %d entries\n", len(dirs))
}
