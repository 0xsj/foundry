// Package main demonstrates io.Reader/Writer composition in Go.
// Run with: go run ./readers-writers/
//
// Key concepts:
//   - io.TeeReader: read and simultaneously write to another destination
//   - io.MultiWriter: fan-out writes to multiple destinations
//   - io.MultiReader: concatenate multiple readers
//   - io.Pipe: synchronous in-memory pipe between goroutines
//   - io.LimitReader: cap bytes read from an untrusted source
//   - io.Copy: the standard idiom for streaming between reader and writer
package main

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"strings"
)

func main() {
	fmt.Println("=== Reader/Writer Composition Examples ===")
	fmt.Println()

	demoTeeReader()
	demoMultiWriter()
	demoMultiReader()
	demoPipe()
	demoLimitReader()
	demoCopy()
}

// demoTeeReader demonstrates splitting a read into two destinations.
// Real use: read an HTTP response body while simultaneously caching it.
func demoTeeReader() {
	fmt.Println("--- TeeReader ---")

	source := strings.NewReader("audit log: user alice accessed /admin at 2026-02-18T09:00:00Z")

	// Capture what's read into an audit buffer
	var auditBuf bytes.Buffer
	tee := io.TeeReader(source, &auditBuf)

	// Process the data through the tee
	data, err := io.ReadAll(tee)
	if err != nil {
		fmt.Fprintf(os.Stderr, "tee read: %v\n", err)
		return
	}

	// data has the processed content, auditBuf has an identical copy
	fmt.Printf("Processed:  %s\n", data)
	fmt.Printf("Audit copy: %s\n", auditBuf.String())
	fmt.Println()
}

// demoMultiWriter demonstrates writing to multiple destinations simultaneously.
// Real use: write logs to both a file and stdout, or stdout and a metrics sink.
func demoMultiWriter() {
	fmt.Println("--- MultiWriter ---")

	var fileSimulator bytes.Buffer // simulates writing to a log file
	var monitorBuf bytes.Buffer    // simulates a monitoring sink

	// Everything written to w goes to both destinations
	w := io.MultiWriter(&fileSimulator, &monitorBuf, os.Stdout)

	fmt.Fprintln(w, "[INFO] service started on :8080")
	fmt.Fprintln(w, "[INFO] database connection established")

	fmt.Printf("File buffer has %d bytes\n", fileSimulator.Len())
	fmt.Printf("Monitor buffer has %d bytes\n", monitorBuf.Len())
	fmt.Println()
}

// demoMultiReader demonstrates concatenating multiple readers into one stream.
// Real use: prepend headers to a body stream, concatenate log files without loading to memory.
func demoMultiReader() {
	fmt.Println("--- MultiReader ---")

	// Simulate concatenating an HTTP preamble with a body
	preamble := strings.NewReader("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n")
	body := strings.NewReader("response body here")

	// MultiReader presents them as a single continuous stream
	combined := io.MultiReader(preamble, body)

	data, err := io.ReadAll(combined)
	if err != nil {
		fmt.Fprintf(os.Stderr, "multi read: %v\n", err)
		return
	}
	fmt.Printf("Combined stream:\n%s\n", data)
}

// demoPipe demonstrates synchronous in-memory pipes for goroutine communication.
// Real use: when a function expects an io.Reader but you have an io.Writer,
// or when building streaming pipelines between goroutines.
func demoPipe() {
	fmt.Println("--- Pipe ---")

	pr, pw := io.Pipe()

	// Producer goroutine: writes JSON records
	go func() {
		defer pw.Close() // signals EOF to the reader

		records := []string{
			`{"user":"alice","action":"login"}`,
			`{"user":"bob","action":"upload"}`,
			`{"user":"carol","action":"logout"}`,
		}
		for _, record := range records {
			fmt.Fprintln(pw, record)
		}
	}()

	// Consumer: reads line by line from the pipe
	data, err := io.ReadAll(pr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "pipe read: %v\n", err)
		return
	}
	fmt.Printf("Received from pipe:\n%s\n", data)
}

// demoLimitReader demonstrates capping bytes read from an untrusted source.
// Real use: HTTP request body limits, preventing memory exhaustion from
// malicious or broken clients.
func demoLimitReader() {
	fmt.Println("--- LimitReader ---")

	// Simulate a large (potentially malicious) upload
	bigUpload := strings.NewReader(strings.Repeat("A", 1_000_000))

	const maxBodySize = 1024 // 1KB limit
	limited := io.LimitReader(bigUpload, maxBodySize)

	data, err := io.ReadAll(limited)
	if err != nil {
		fmt.Fprintf(os.Stderr, "limited read: %v\n", err)
		return
	}

	// LimitReader returns EOF at the limit — no error, just stops early
	fmt.Printf("Read %d bytes (limit: %d, actual: 1,000,000)\n", len(data), maxBodySize)
	fmt.Println()
}

// demoCopy demonstrates io.Copy for efficient streaming between reader and writer.
// Real use: stream file to HTTP response, copy S3 object to disk, pipe stdin to file.
func demoCopy() {
	fmt.Println("--- io.Copy ---")

	// Simulate streaming a log entry through a processing pipeline:
	// Source → [transform in-memory] → Destination
	source := strings.NewReader("WARN  2026-02-18 09:15:32 high memory usage: 87%\n")

	// Count bytes in transit using io.TeeReader + io.Discard trick
	var counter countingWriter
	tee := io.TeeReader(source, &counter)

	// Copy from tee (which also counts) to a destination
	var dest bytes.Buffer
	n, err := io.Copy(&dest, tee)
	if err != nil {
		fmt.Fprintf(os.Stderr, "copy: %v\n", err)
		return
	}

	fmt.Printf("io.Copy transferred: %d bytes\n", n)
	fmt.Printf("Destination received: %q\n", dest.String())
}

// countingWriter counts bytes written without storing them.
type countingWriter struct {
	total int64
}

func (cw *countingWriter) Write(p []byte) (int, error) {
	cw.total += int64(len(p))
	return len(p), nil
}
