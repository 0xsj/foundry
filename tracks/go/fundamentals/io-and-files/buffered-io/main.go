// Package main demonstrates buffered I/O with the bufio package.
// Run with: go run ./buffered-io/
//
// Key concepts:
//   - bufio.Scanner for line-by-line reading (and why it beats ReadString)
//   - Scanner.Err() — the trap of not checking it after the loop
//   - Custom split functions for tokenizing non-line data
//   - bufio.Writer and the critical Flush()
//   - Buffer sizing: when defaults are wrong
//   - Why buffering matters: syscall cost
package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"
	"unicode"
)

func main() {
	dir, err := os.MkdirTemp("", "foundry-bufio-*")
	if err != nil {
		fmt.Fprintf(os.Stderr, "temp dir: %v\n", err)
		os.Exit(1)
	}
	defer os.RemoveAll(dir)

	fmt.Println("=== Buffered I/O Examples ===")
	fmt.Println()

	demoScanner(dir)
	demoScannerErrTrap(dir)
	demoCustomSplitFn()
	demoBufferedWriter(dir)
	demoLargeLine(dir)
	demoBufferingPerformance(dir)
}

// demoScanner demonstrates the idiomatic way to read a file line by line.
// bufio.Scanner handles partial reads, buffer management, and newline stripping.
func demoScanner(dir string) {
	fmt.Println("--- bufio.Scanner: line-by-line ---")

	// Write a sample log file
	logPath := filepath.Join(dir, "access.log")
	logContent := `192.168.1.1 GET /api/users 200 45ms
10.0.0.5 POST /api/orders 201 120ms
192.168.1.1 GET /api/users/42 404 12ms
10.0.0.7 DELETE /api/sessions 204 8ms
10.0.0.5 GET /api/orders 500 2341ms
`
	os.WriteFile(logPath, []byte(logContent), 0644)

	f, err := os.Open(logPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "open: %v\n", err)
		return
	}
	defer f.Close()

	// Count lines and find errors
	var totalLines, errorLines int
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := scanner.Text() // string, newline stripped
		totalLines++
		if strings.Contains(line, " 5") { // 5xx status
			fmt.Printf("  ERROR: %s\n", line)
			errorLines++
		}
	}
	// ALWAYS check Err() after the loop
	// scanner.Scan() returns false for BOTH EOF and error — you can't tell the difference without this
	if err := scanner.Err(); err != nil {
		fmt.Fprintf(os.Stderr, "scan: %v\n", err)
		return
	}

	fmt.Printf("Scanned %d lines, found %d errors\n\n", totalLines, errorLines)
}

// demoScannerErrTrap demonstrates what happens when you skip scanner.Err().
// This is the most common bufio mistake.
func demoScannerErrTrap(dir string) {
	fmt.Println("--- Scanner.Err() trap ---")

	// Simulate a reader that errors mid-stream
	failReader := &failAfterN{
		data:     []byte("line one\nline two\nline three\n"),
		failAt:   20, // fails after 20 bytes (middle of "line three")
		failWith: fmt.Errorf("disk I/O error"),
	}

	scanner := bufio.NewScanner(failReader)
	var lines []string
	for scanner.Scan() {
		lines = append(lines, scanner.Text())
	}

	// Without err check — would silently have incomplete data:
	fmt.Printf("Lines collected: %v\n", lines)

	// With err check — reveals the problem:
	if err := scanner.Err(); err != nil {
		fmt.Printf("Scanner stopped early due to: %v\n", err)
		fmt.Println("  => Without checking Err(), we'd silently process incomplete data")
	}
	fmt.Println()
}

// failAfterN is an io.Reader that fails after reading n bytes.
type failAfterN struct {
	data     []byte
	pos      int
	failAt   int
	failWith error
}

func (r *failAfterN) Read(p []byte) (int, error) {
	if r.pos >= r.failAt {
		return 0, r.failWith
	}
	available := r.failAt - r.pos
	if available > len(p) {
		available = len(p)
	}
	if r.pos+available > len(r.data) {
		available = len(r.data) - r.pos
	}
	if available == 0 {
		return 0, io.EOF
	}
	n := copy(p, r.data[r.pos:r.pos+available])
	r.pos += n
	return n, nil
}

// demoCustomSplitFn demonstrates writing a custom Scanner split function.
// Here we parse pipe-delimited event records without loading the whole file.
func demoCustomSplitFn() {
	fmt.Println("--- Custom SplitFunc: pipe-delimited records ---")

	// Imagine a stream of pipe-delimited events from a message queue
	stream := strings.NewReader("login|alice|2026-02-18T09:00:00Z|logout|bob|2026-02-18T09:01:00Z|error|alice|2026-02-18T09:02:00Z")

	scanner := bufio.NewScanner(stream)
	scanner.Split(splitOnPipe)

	var events []string
	for scanner.Scan() {
		events = append(events, scanner.Text())
	}
	if err := scanner.Err(); err != nil {
		fmt.Fprintf(os.Stderr, "scan: %v\n", err)
		return
	}

	fmt.Printf("Parsed %d tokens from pipe-delimited stream:\n", len(events))
	// Group into records of 3 (event type, user, timestamp)
	for i := 0; i+2 < len(events); i += 3 {
		fmt.Printf("  event=%-8s user=%-8s time=%s\n", events[i], events[i+1], events[i+2])
	}
	fmt.Println()
}

// splitOnPipe is a bufio.SplitFunc that splits on '|' characters.
// The scanner framework calls this repeatedly with increasing data until
// a token is returned or atEOF is true.
func splitOnPipe(data []byte, atEOF bool) (advance int, token []byte, err error) {
	// Scan for a pipe delimiter
	for i, b := range data {
		if b == '|' {
			return i + 1, trimSpace(data[:i]), nil // advance past '|', return token before it
		}
	}
	// No pipe found: if at EOF and there's data, return remaining as final token
	if atEOF && len(data) > 0 {
		return len(data), trimSpace(data), nil
	}
	// Request more data
	return 0, nil, nil
}

func trimSpace(b []byte) []byte {
	return []byte(strings.TrimFunc(string(b), unicode.IsSpace))
}

// demoBufferedWriter demonstrates bufio.Writer and the critical Flush().
// Without Flush, data sits in memory and never reaches the underlying writer.
func demoBufferedWriter(dir string) {
	fmt.Println("--- bufio.Writer: always Flush ---")

	outPath := filepath.Join(dir, "output.csv")
	f, err := os.Create(outPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "create: %v\n", err)
		return
	}
	// Note: we close f after the flush to capture write errors
	defer f.Close()

	bw := bufio.NewWriter(f)
	// CRITICAL: flush before closing. If you only defer f.Close(),
	// the buffer is never flushed and the file will be empty (or partial).
	defer func() {
		if ferr := bw.Flush(); ferr != nil {
			fmt.Fprintf(os.Stderr, "flush: %v\n", ferr)
		}
	}()

	// Write CSV header and rows
	fmt.Fprintln(bw, "user_id,event,timestamp")
	events := [][3]string{
		{"u001", "login", "2026-02-18T09:00:00Z"},
		{"u002", "purchase", "2026-02-18T09:01:30Z"},
		{"u001", "logout", "2026-02-18T09:45:00Z"},
	}
	for _, ev := range events {
		fmt.Fprintf(bw, "%s,%s,%s\n", ev[0], ev[1], ev[2])
	}

	// Flush is called by defer above.
	// We can also call it explicitly here to check the error immediately.
	if err := bw.Flush(); err != nil {
		fmt.Fprintf(os.Stderr, "flush: %v\n", err)
		return
	}

	// Verify
	data, _ := os.ReadFile(outPath)
	lines := strings.Split(strings.TrimRight(string(data), "\n"), "\n")
	fmt.Printf("Wrote %d-line CSV (%d bytes)\n\n", len(lines), len(data))
}

// demoLargeLine demonstrates configuring scanner buffer for long lines.
// Default max token size is 64KB. Real-world logs can exceed this.
func demoLargeLine(dir string) {
	fmt.Println("--- Scanner: large line handling ---")

	// Generate a file with one very long line (JSON blob, minified)
	longLinePath := filepath.Join(dir, "large-json.log")
	longLine := `{"type":"audit","payload":"` + strings.Repeat("x", 100_000) + `"}`
	os.WriteFile(longLinePath, []byte(longLine+"\nnormal line\n"), 0644)

	f, _ := os.Open(longLinePath)
	defer f.Close()

	scanner := bufio.NewScanner(f)

	// Increase buffer to handle lines up to 256KB
	const maxLine = 256 * 1024
	buf := make([]byte, bufio.MaxScanTokenSize)
	scanner.Buffer(buf, maxLine)

	var lineCount int
	for scanner.Scan() {
		lineCount++
		line := scanner.Text()
		if len(line) > 80 {
			fmt.Printf("  Line %d: %s... (%d chars)\n", lineCount, line[:60], len(line))
		} else {
			fmt.Printf("  Line %d: %s\n", lineCount, line)
		}
	}
	if err := scanner.Err(); err != nil {
		fmt.Printf("  Scanner error (would happen without Buffer call): %v\n", err)
	} else {
		fmt.Printf("Scanned %d lines with custom buffer\n", lineCount)
	}
	fmt.Println()
}

// demoBufferingPerformance shows the concrete cost of unbuffered vs buffered writes.
// On most systems, the difference is 10-100x for many small writes.
func demoBufferingPerformance(dir string) {
	fmt.Println("--- Buffering Performance ---")

	const nLines = 10_000
	line := "2026-02-18 09:00:00 INFO request processed latency=45ms endpoint=/api/v1/users\n"

	// Unbuffered writes: each Fprintln is a syscall
	start := time.Now()
	unbufferedPath := filepath.Join(dir, "unbuffered.log")
	f1, _ := os.Create(unbufferedPath)
	for i := 0; i < nLines; i++ {
		fmt.Fprint(f1, line)
	}
	f1.Close()
	unbufferedTime := time.Since(start)

	// Buffered writes: batches into 4KB chunks, far fewer syscalls
	start = time.Now()
	bufferedPath := filepath.Join(dir, "buffered.log")
	f2, _ := os.Create(bufferedPath)
	bw := bufio.NewWriter(f2)
	for i := 0; i < nLines; i++ {
		fmt.Fprint(bw, line)
	}
	bw.Flush()
	f2.Close()
	bufferedTime := time.Since(start)

	// Verify both files have the same content
	s1, _ := os.Stat(unbufferedPath)
	s2, _ := os.Stat(bufferedPath)

	fmt.Printf("Lines written: %d\n", nLines)
	fmt.Printf("Unbuffered: %v (%d bytes)\n", unbufferedTime, s1.Size())
	fmt.Printf("Buffered:   %v (%d bytes)\n", bufferedTime, s2.Size())
	if unbufferedTime > bufferedTime {
		speedup := float64(unbufferedTime) / float64(bufferedTime)
		fmt.Printf("Buffered was %.1fx faster\n", speedup)
	}
}
