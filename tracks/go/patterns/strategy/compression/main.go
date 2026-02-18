// Strategy Pattern: Compression Pipeline
//
// Demonstrates runtime strategy selection. A data pipeline applies
// different compression strategies (gzip, zstd-simulated, none) to
// payloads before storage. The strategy is selected at runtime based
// on payload characteristics — small payloads skip compression,
// large payloads use heavy compression, and the default is fast compression.
//
// Also demonstrates the adapter pattern: CompressorFunc allows a plain
// function to satisfy the Compressor interface.
//
// Run: go run ./compression/

package main

import (
	"bytes"
	"compress/gzip"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"strings"
)

// --- Strategy Interface ---

// Compressor defines a compression strategy.
// Implementations decide how (and whether) to compress data.
type Compressor interface {
	Compress(data []byte) ([]byte, error)
	Name() string
}

// CompressorFunc adapts a plain function into a Compressor.
// This is the http.HandlerFunc pattern applied to compression.
// Simple strategies can be inline functions; complex ones use structs.
type CompressorFunc struct {
	CompressFn func(data []byte) ([]byte, error)
	NameStr    string
}

func (cf CompressorFunc) Compress(data []byte) ([]byte, error) {
	return cf.CompressFn(data)
}

func (cf CompressorFunc) Name() string {
	return cf.NameStr
}

// --- Concrete Strategies ---

// GzipCompressor compresses data using gzip at the specified level.
type GzipCompressor struct {
	Level int // gzip.BestSpeed (1) to gzip.BestCompression (9)
}

var _ Compressor = (*GzipCompressor)(nil)

func (g *GzipCompressor) Compress(data []byte) ([]byte, error) {
	var buf bytes.Buffer
	w, err := gzip.NewWriterLevel(&buf, g.Level)
	if err != nil {
		return nil, fmt.Errorf("creating gzip writer: %w", err)
	}
	if _, err := w.Write(data); err != nil {
		return nil, fmt.Errorf("writing gzip data: %w", err)
	}
	if err := w.Close(); err != nil {
		return nil, fmt.Errorf("closing gzip writer: %w", err)
	}
	return buf.Bytes(), nil
}

func (g *GzipCompressor) Name() string {
	return fmt.Sprintf("gzip(level=%d)", g.Level)
}

// ZstdSimulatedCompressor simulates zstd compression.
// In production, you'd use github.com/klauspost/compress/zstd.
// Here we simulate it to avoid external dependencies.
type ZstdSimulatedCompressor struct {
	Level int
}

var _ Compressor = (*ZstdSimulatedCompressor)(nil)

func (z *ZstdSimulatedCompressor) Compress(data []byte) ([]byte, error) {
	// Simulate zstd by using gzip with best compression + a header marker.
	// In production, replace with real zstd.
	var buf bytes.Buffer
	buf.Write([]byte("ZSTD"))       // Simulated magic bytes
	buf.WriteByte(byte(z.Level))    // Level indicator
	w, err := gzip.NewWriterLevel(&buf, gzip.BestCompression)
	if err != nil {
		return nil, fmt.Errorf("creating zstd-simulated writer: %w", err)
	}
	if _, err := w.Write(data); err != nil {
		return nil, fmt.Errorf("writing zstd-simulated data: %w", err)
	}
	if err := w.Close(); err != nil {
		return nil, fmt.Errorf("closing zstd-simulated writer: %w", err)
	}
	return buf.Bytes(), nil
}

func (z *ZstdSimulatedCompressor) Name() string {
	return fmt.Sprintf("zstd(level=%d)", z.Level)
}

// NoopCompressor returns data unchanged. Used for small payloads
// where compression overhead exceeds the space savings.
var NoopCompressor = CompressorFunc{
	CompressFn: func(data []byte) ([]byte, error) {
		return data, nil
	},
	NameStr: "none",
}

// --- Runtime Strategy Selector ---

// CompressionPolicy selects a compression strategy based on payload characteristics.
// This is the "context" in classic Strategy pattern terminology.
type CompressionPolicy struct {
	// Threshold below which compression is skipped (overhead > savings)
	SmallPayloadThreshold int
	// Threshold above which heavy compression is used (worth the CPU cost)
	LargePayloadThreshold int

	Fast  Compressor // For medium payloads
	Heavy Compressor // For large payloads
}

// Select chooses the appropriate compressor for the given data.
// This is runtime strategy selection — the decision is made per-payload.
func (cp *CompressionPolicy) Select(data []byte) Compressor {
	size := len(data)
	switch {
	case size < cp.SmallPayloadThreshold:
		return NoopCompressor
	case size > cp.LargePayloadThreshold:
		return cp.Heavy
	default:
		return cp.Fast
	}
}

// --- Data Pipeline ---

// StorageRecord represents a compressed payload ready for storage.
type StorageRecord struct {
	Key              string
	OriginalSize     int
	CompressedSize   int
	CompressionRatio float64
	Compressor       string
	Checksum         string
	Data             []byte
}

// Pipeline processes data through compression and prepares it for storage.
type Pipeline struct {
	policy *CompressionPolicy
}

// NewPipeline creates a pipeline with the given compression policy.
func NewPipeline(policy *CompressionPolicy) *Pipeline {
	return &Pipeline{policy: policy}
}

// Process compresses data using the policy-selected strategy and produces a storage record.
func (p *Pipeline) Process(key string, data []byte) (*StorageRecord, error) {
	// Runtime strategy selection
	compressor := p.policy.Select(data)

	compressed, err := compressor.Compress(data)
	if err != nil {
		return nil, fmt.Errorf("compressing %q with %s: %w", key, compressor.Name(), err)
	}

	hash := sha256.Sum256(compressed)

	ratio := float64(len(compressed)) / float64(len(data))
	if len(data) == 0 {
		ratio = 1.0
	}

	return &StorageRecord{
		Key:              key,
		OriginalSize:     len(data),
		CompressedSize:   len(compressed),
		CompressionRatio: ratio,
		Compressor:       compressor.Name(),
		Checksum:         hex.EncodeToString(hash[:8]),
		Data:             compressed,
	}, nil
}

// --- Demonstration ---

func generatePayload(sizeBytes int) []byte {
	// Generate semi-compressible data (repeated JSON-like structures)
	var buf bytes.Buffer
	template := `{"id":%d,"name":"item-%d","value":%.2f,"tags":["alpha","beta","gamma"],"active":true}`
	for buf.Len() < sizeBytes {
		fmt.Fprintf(&buf, template, buf.Len(), buf.Len(), float64(buf.Len())/100.0)
		buf.WriteByte('\n')
	}
	return buf.Bytes()[:sizeBytes]
}

func decompressGzip(data []byte) ([]byte, error) {
	r, err := gzip.NewReader(bytes.NewReader(data))
	if err != nil {
		return nil, err
	}
	defer r.Close()
	return io.ReadAll(r)
}

func main() {
	// Configure the compression policy
	policy := &CompressionPolicy{
		SmallPayloadThreshold: 256,    // < 256 bytes: skip compression
		LargePayloadThreshold: 10240,  // > 10KB: use heavy compression
		Fast:  &GzipCompressor{Level: gzip.BestSpeed},        // medium: fast gzip
		Heavy: &ZstdSimulatedCompressor{Level: 5},             // large: heavy compression
	}

	pipeline := NewPipeline(policy)

	// Process payloads of various sizes to see strategy selection in action
	payloads := []struct {
		key  string
		size int
	}{
		{"tiny-config", 64},
		{"small-config", 200},
		{"api-response", 512},
		{"user-list", 2048},
		{"analytics-batch", 5120},
		{"full-export", 20480},
		{"database-dump", 51200},
	}

	fmt.Println("=== Compression Pipeline with Runtime Strategy Selection ===")
	fmt.Println()
	fmt.Printf("Policy: skip < %d bytes | fast gzip < %d bytes | heavy zstd above\n",
		policy.SmallPayloadThreshold, policy.LargePayloadThreshold)
	fmt.Println()
	fmt.Printf("%-20s %10s %10s %8s %20s %s\n",
		"Key", "Original", "Compressed", "Ratio", "Strategy", "Checksum")
	fmt.Println(strings.Repeat("-", 85))

	for _, p := range payloads {
		data := generatePayload(p.size)
		record, err := pipeline.Process(p.key, data)
		if err != nil {
			fmt.Printf("ERROR processing %s: %v\n", p.key, err)
			continue
		}

		fmt.Printf("%-20s %8d B %8d B %6.1f%% %20s %s\n",
			record.Key,
			record.OriginalSize,
			record.CompressedSize,
			record.CompressionRatio*100,
			record.Compressor,
			record.Checksum,
		)
	}

	// Verify round-trip for gzip-compressed data
	fmt.Println("\n--- Round-trip Verification ---")
	original := generatePayload(1024)
	record, err := pipeline.Process("roundtrip-test", original)
	if err != nil {
		fmt.Printf("ERROR: %v\n", err)
		return
	}

	if record.Compressor == "none" {
		fmt.Println("Skipped (no compression applied)")
	} else if strings.HasPrefix(record.Compressor, "gzip") {
		decompressed, err := decompressGzip(record.Data)
		if err != nil {
			fmt.Printf("Decompression failed: %v\n", err)
		} else if bytes.Equal(original, decompressed) {
			fmt.Printf("Round-trip verified: %d bytes -> %d bytes -> %d bytes (match)\n",
				len(original), record.CompressedSize, len(decompressed))
		} else {
			fmt.Println("Round-trip FAILED: decompressed data does not match original")
		}
	}

	// Demonstrate strategy composability with custom inline strategy
	fmt.Println("\n--- Custom Strategy via CompressorFunc ---")
	xorCompressor := CompressorFunc{
		CompressFn: func(data []byte) ([]byte, error) {
			// XOR "encryption" (not real — just demonstrates inline strategy)
			result := make([]byte, len(data))
			for i, b := range data {
				result[i] = b ^ 0x42
			}
			return result, nil
		},
		NameStr: "xor-scramble",
	}

	customPolicy := &CompressionPolicy{
		SmallPayloadThreshold: 0,
		LargePayloadThreshold: 999999,
		Fast:                  xorCompressor,
		Heavy:                 xorCompressor,
	}

	customPipeline := NewPipeline(customPolicy)
	record, err = customPipeline.Process("custom-test", []byte("hello, strategy pattern!"))
	if err != nil {
		fmt.Printf("ERROR: %v\n", err)
	} else {
		fmt.Printf("Strategy: %s | Original: %d B | Output: %d B\n",
			record.Compressor, record.OriginalSize, record.CompressedSize)
	}

	fmt.Println("\n" + strings.Repeat("=", 60))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- CompressionPolicy.Select() chooses strategy at runtime per-payload")
	fmt.Println("- GzipCompressor and ZstdSimulatedCompressor are struct strategies (stateful)")
	fmt.Println("- NoopCompressor uses CompressorFunc (adapter pattern for inline functions)")
	fmt.Println("- The pipeline doesn't know which compressor it's using — just calls Compress()")
	fmt.Println("- Adding a new compressor (LZ4, Snappy, Brotli) requires zero changes to Pipeline")
}
