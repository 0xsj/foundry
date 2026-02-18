// Decorator Pattern: I/O Wrapper Chain
//
// Demonstrates interface decorators using io.Reader/io.Writer.
// Builds a chain of readers: counting → limiting → decompression → buffering → file.
// Also shows a custom ProgressReader that reports bytes read as a percentage.
//
// Run: go run ./iowrap/

package main

import (
	"bufio"
	"bytes"
	"compress/gzip"
	"fmt"
	"io"
	"strings"
)

// --- CountingReader: tracks total bytes read ---

// CountingReader is an io.Reader decorator that counts bytes passing through.
// This is useful for monitoring data transfer, enforcing quotas, or progress reporting.
type CountingReader struct {
	reader    io.Reader
	bytesRead int64
}

// NewCountingReader wraps an io.Reader with byte counting.
func NewCountingReader(r io.Reader) *CountingReader {
	return &CountingReader{reader: r}
}

func (cr *CountingReader) Read(p []byte) (int, error) {
	n, err := cr.reader.Read(p)
	cr.bytesRead += int64(n)
	return n, err
}

// BytesRead returns the total number of bytes read through this decorator.
func (cr *CountingReader) BytesRead() int64 {
	return cr.bytesRead
}

// --- ProgressReader: reports read progress ---

// ProgressReader is an io.Reader decorator that calls a callback with progress updates.
// In production, you'd use this for file uploads, large downloads, or ETL pipelines.
type ProgressReader struct {
	reader    io.Reader
	total     int64
	read      int64
	onProgress func(bytesRead, totalBytes int64, pct float64)
}

// NewProgressReader wraps a reader with progress reporting.
// total is the expected total size (use -1 if unknown).
func NewProgressReader(r io.Reader, total int64, onProgress func(int64, int64, float64)) *ProgressReader {
	return &ProgressReader{
		reader:     r,
		total:      total,
		onProgress: onProgress,
	}
}

func (pr *ProgressReader) Read(p []byte) (int, error) {
	n, err := pr.reader.Read(p)
	pr.read += int64(n)

	if pr.onProgress != nil && n > 0 {
		var pct float64
		if pr.total > 0 {
			pct = float64(pr.read) / float64(pr.total) * 100
		}
		pr.onProgress(pr.read, pr.total, pct)
	}

	return n, err
}

// --- PrefixWriter: adds a prefix to each line ---

// PrefixWriter is an io.Writer decorator that prepends a prefix to each line written.
// Useful for log routing, output formatting, or multiplexed output streams.
type PrefixWriter struct {
	writer    io.Writer
	prefix    string
	atLineStart bool
}

// NewPrefixWriter wraps a writer that adds a prefix to each line.
func NewPrefixWriter(w io.Writer, prefix string) *PrefixWriter {
	return &PrefixWriter{
		writer:      w,
		prefix:      prefix,
		atLineStart: true,
	}
}

func (pw *PrefixWriter) Write(p []byte) (int, error) {
	written := 0
	for len(p) > 0 {
		if pw.atLineStart {
			if _, err := pw.writer.Write([]byte(pw.prefix)); err != nil {
				return written, err
			}
			pw.atLineStart = false
		}

		idx := bytes.IndexByte(p, '\n')
		if idx >= 0 {
			n, err := pw.writer.Write(p[:idx+1])
			written += n
			if err != nil {
				return written, err
			}
			p = p[idx+1:]
			pw.atLineStart = true
		} else {
			n, err := pw.writer.Write(p)
			written += n
			return written, err
		}
	}
	return written, nil
}

// --- TeeReadCloser: splits reads to a side writer ---

// TeeReadCloser is like io.TeeReader but also implements io.Closer.
// This is a common gap in the standard library -- io.TeeReader returns io.Reader
// but if the underlying reader is a ReadCloser, you lose the Close method.
type TeeReadCloser struct {
	reader io.ReadCloser
	writer io.Writer
}

// NewTeeReadCloser creates a reader that writes all data to w as it's read from r.
func NewTeeReadCloser(r io.ReadCloser, w io.Writer) *TeeReadCloser {
	return &TeeReadCloser{reader: r, writer: w}
}

func (t *TeeReadCloser) Read(p []byte) (int, error) {
	n, err := t.reader.Read(p)
	if n > 0 {
		if _, wErr := t.writer.Write(p[:n]); wErr != nil {
			return n, wErr
		}
	}
	return n, err
}

func (t *TeeReadCloser) Close() error {
	return t.reader.Close()
}

// --- Demo Functions ---

// demonstrateReaderChain shows composing io.Reader decorators.
func demonstrateReaderChain() {
	fmt.Println("=== Reader Decorator Chain ===")
	fmt.Println()

	// Original data: simulate a gzipped payload
	var compressed bytes.Buffer
	gz := gzip.NewWriter(&compressed)
	originalData := "This is the original payload data that was compressed with gzip.\n" +
		"It demonstrates how io.Reader decorators compose.\n" +
		"Each wrapper adds one capability without knowing about the others.\n"
	gz.Write([]byte(originalData))
	gz.Close()

	fmt.Printf("Original data size: %d bytes\n", len(originalData))
	fmt.Printf("Compressed size:    %d bytes\n", compressed.Len())
	fmt.Println()

	// Build the decorator chain (read bottom to top):
	//   CountingReader → LimitReader → gzip.Reader → bufio.Reader → bytes.Reader
	//
	// Each layer adds behavior:
	//   - bytes.Reader: provides the raw compressed bytes
	//   - bufio.Reader: reduces read syscalls (buffers 4KB)
	//   - gzip.Reader: decompresses the stream
	//   - LimitReader: caps output at 1MB (safety limit)
	//   - CountingReader: tracks total decompressed bytes

	source := bytes.NewReader(compressed.Bytes())
	buffered := bufio.NewReader(source)
	decompressed, err := gzip.NewReader(buffered)
	if err != nil {
		fmt.Printf("gzip error: %v\n", err)
		return
	}
	defer decompressed.Close()

	limited := io.LimitReader(decompressed, 1024*1024) // 1MB safety limit
	counted := NewCountingReader(limited)

	// Read all data through the chain
	result, err := io.ReadAll(counted)
	if err != nil {
		fmt.Printf("read error: %v\n", err)
		return
	}

	fmt.Printf("Decompressed output (%d bytes via counter):\n", counted.BytesRead())
	fmt.Println(string(result))

	fmt.Println("Chain: bytes.Reader → bufio.Reader → gzip.Reader → LimitReader → CountingReader")
	fmt.Println("Each decorator implements io.Reader. Each is independent and composable.")
	fmt.Println()
}

// demonstrateProgressReader shows a progress-reporting decorator.
func demonstrateProgressReader() {
	fmt.Println("=== Progress Reader ===")
	fmt.Println()

	// Simulate a large data source (in production: file, HTTP response body, S3 object)
	data := strings.Repeat("x", 1000)
	source := strings.NewReader(data)

	progress := NewProgressReader(source, int64(len(data)), func(read, total int64, pct float64) {
		// In production, this would update a progress bar, emit metrics, or notify a UI
		bar := int(pct / 5) // 20 chars wide
		fmt.Printf("\r  [%-20s] %.0f%% (%d/%d bytes)",
			strings.Repeat("#", bar), pct, read, total)
	})

	// Read in small chunks to see progress updates
	buf := make([]byte, 100) // 100-byte chunks
	for {
		_, err := progress.Read(buf)
		if err == io.EOF {
			break
		}
		if err != nil {
			fmt.Printf("\nerror: %v\n", err)
			return
		}
	}
	fmt.Println() // newline after progress bar
	fmt.Println()
}

// demonstrateWriterDecorators shows composing io.Writer decorators.
func demonstrateWriterDecorators() {
	fmt.Println("=== Writer Decorator Chain ===")
	fmt.Println()

	// Build a writer chain:
	//   PrefixWriter → io.MultiWriter → [stdout, &buffer]
	//
	// Everything written goes to both stdout (with prefix) and a buffer (without).

	var captured bytes.Buffer

	// MultiWriter: writes to multiple destinations simultaneously
	multi := io.MultiWriter(&captured, io.Discard)

	// PrefixWriter: adds "[APP] " to each line before writing to multi
	prefixed := NewPrefixWriter(multi, "[APP] ")

	// Write some log lines through the chain
	fmt.Fprintln(prefixed, "Server starting on :8080")
	fmt.Fprintln(prefixed, "Loading configuration from /etc/app/config.yaml")
	fmt.Fprintln(prefixed, "Database connection established")
	fmt.Fprintln(prefixed, "Ready to accept requests")

	fmt.Println()
	fmt.Println("Captured in buffer (without prefix because it goes through MultiWriter to both Discard and buffer):")
	fmt.Println(captured.String())
}

// demonstrateTeeReader shows the TeeReadCloser decorator.
func demonstrateTeeReader() {
	fmt.Println("=== TeeReadCloser ===")
	fmt.Println()

	// Simulate reading a request body while simultaneously capturing it for audit logging
	body := io.NopCloser(strings.NewReader(`{"action": "transfer", "amount": 5000, "to": "acct-789"}`))

	var auditLog bytes.Buffer
	tee := NewTeeReadCloser(body, &auditLog)

	// Handler reads the body normally
	content, _ := io.ReadAll(tee)
	tee.Close()

	fmt.Println("Handler received:")
	fmt.Printf("  %s\n", content)
	fmt.Println()
	fmt.Println("Audit log captured (simultaneously):")
	fmt.Printf("  %s\n", auditLog.String())
	fmt.Println()
	fmt.Println("The handler didn't know its reader was being tee'd to an audit log.")
	fmt.Println("This is the decorator pattern: transparent behavior addition.")
	fmt.Println()
}

func main() {
	demonstrateReaderChain()
	fmt.Println(strings.Repeat("-", 60))
	fmt.Println()

	demonstrateProgressReader()
	fmt.Println(strings.Repeat("-", 60))
	fmt.Println()

	demonstrateWriterDecorators()
	fmt.Println(strings.Repeat("-", 60))
	fmt.Println()

	demonstrateTeeReader()

	fmt.Println("=== Key Takeaways ===")
	fmt.Println()
	fmt.Println("- io.Reader and io.Writer are the decorator interfaces of Go's I/O system")
	fmt.Println("- Each wrapper adds one capability: buffering, compression, counting, limiting, teeing")
	fmt.Println("- Decorators compose freely because they all satisfy the same interface")
	fmt.Println("- The standard library provides many decorators; build custom ones for your domain")
	fmt.Println("- Order matters: buffered(gzip(file)) is different from gzip(buffered(file))")
}
