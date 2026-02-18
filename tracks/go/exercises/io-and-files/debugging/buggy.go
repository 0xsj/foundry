// Package reports provides utilities for writing and reading daily reports.
// There are 4 bugs in this file. Find them.
package reports

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"
)

// ============================================================================
// BUG 1 LIVES HERE
// writeReport writes a structured report to path.
// The file should contain the report content when the function returns.
// ============================================================================

// Report holds data for a daily operational report.
type Report struct {
	Date     time.Time
	Service  string
	Requests int
	Errors   int
	P99Ms    float64
}

// writeReport serializes r and writes it to the file at path.
// Creates or truncates the file. Returns an error if writing fails.
func writeReport(path string, r Report) error {
	f, err := os.Create(path)
	if err != nil {
		return fmt.Errorf("create %s: %w", path, err)
	}

	bw := bufio.NewWriter(f)
	// BUG 1: defer f.Close() is registered AFTER defer bw.Flush().
	// Defers run LIFO — so f.Close() runs FIRST, then bw.Flush() tries to
	// write to an already-closed file. The data sits in the buffer and never
	// reaches disk. The file is empty and no error is surfaced.
	defer bw.Flush() // registered first — runs LAST (after Close!)
	defer f.Close()  // registered second — runs FIRST (closes file before Flush)

	fmt.Fprintf(bw, "date:     %s\n", r.Date.Format("2006-01-02"))
	fmt.Fprintf(bw, "service:  %s\n", r.Service)
	fmt.Fprintf(bw, "requests: %d\n", r.Requests)
	fmt.Fprintf(bw, "errors:   %d\n", r.Errors)
	fmt.Fprintf(bw, "p99_ms:   %.2f\n", r.P99Ms)

	return nil
}

// ============================================================================
// BUG 2 LIVES HERE
// processLogLines reads lines from r, counts them, and returns any error.
// It should surface I/O errors that occur while reading.
// ============================================================================

// processLogLines counts lines in r, returning count and any error.
func processLogLines(r io.Reader) (int, error) {
	count := 0
	scanner := bufio.NewScanner(r)
	for scanner.Scan() {
		count++
		_ = scanner.Text()
	}
	// BUG 2: something is missing here.
	// When the reader errors mid-stream, this function returns nil instead of the error.
	return count, nil
}

// ============================================================================
// BUG 3 LIVES HERE
// buildReportPath constructs the file path for a report.
// Should work correctly on all operating systems, including Windows.
// ============================================================================

// buildReportPath returns the file path for a report given the base directory,
// service name, and date. The path is: base/service/YYYY-MM-DD.report
func buildReportPath(base, service string, date time.Time) string {
	// BUG 3: this concatenation uses a hardcoded separator.
	// On Unix it works. On Windows it produces paths the OS can't open.
	return base + "/" + service + "/" + date.Format("2006-01-02") + ".report"
}

// ============================================================================
// BUG 4 LIVES HERE
// processAll processes a list of report files, reading each in turn.
// Each file should be closed after it is processed, not deferred to the end.
// ============================================================================

// ProcessResult holds the outcome of processing one file.
type ProcessResult struct {
	Path  string
	Lines int
	Err   error
}

// processAll opens and reads each file in paths, returning results.
// Files must be closed after each is processed (not deferred to function return).
func processAll(paths []string) []ProcessResult {
	results := make([]ProcessResult, 0, len(paths))

	for _, path := range paths {
		f, err := os.Open(path)
		if err != nil {
			results = append(results, ProcessResult{Path: path, Err: err})
			continue
		}
		// BUG 4: resource leak. This defer runs when processAll returns,
		// not at the end of each loop iteration. With 1000 files, you'd
		// hold 1000 open file descriptors.
		defer f.Close()

		n, err := processLogLines(f)
		results = append(results, ProcessResult{Path: path, Lines: n, Err: err})
	}

	return results
}

// ============================================================================
// Helper: not buggy, used by tests
// ============================================================================

// readLines reads all lines from path, returning them as a slice of strings.
func readLines(path string) ([]string, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("open: %w", err)
	}
	defer f.Close()

	var lines []string
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		lines = append(lines, scanner.Text())
	}
	return lines, scanner.Err()
}

// validPath makes buildReportPath result cross-platform comparable in tests.
func validPath(base, service string, date time.Time) string {
	return filepath.Join(base, service, date.Format("2006-01-02")+".report")
}
