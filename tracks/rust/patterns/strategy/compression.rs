// Strategy Pattern: Data Pipeline with Generic Compression
//
// Demonstrates: Static dispatch via generics (monomorphization)
//
// Scenario: A data ingestion pipeline that compresses batches of records
// before writing to storage. The compression algorithm is chosen at build
// time or initialization and never changes during the pipeline's lifetime.
// This is the ideal case for generics — the compiler generates specialized
// code for each compressor, enabling inlining and zero-cost abstraction.
//
// Run: rustc compression.rs && ./compression

use std::fmt;

// ---------------------------------------------------------------------------
// Strategy trait
// ---------------------------------------------------------------------------

trait CompressionStrategy: fmt::Display {
    /// Compress a byte slice. Returns compressed bytes.
    fn compress(&self, data: &[u8]) -> Vec<u8>;

    /// Decompress bytes previously compressed with this strategy.
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, String>;

    /// Estimated compression ratio for capacity planning (0.0 = perfect, 1.0 = no compression).
    fn estimated_ratio(&self) -> f64;

    /// Human-readable name for logging.
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Concrete strategies
// ---------------------------------------------------------------------------

/// Gzip compression with configurable level (1-9).
struct GzipCompression {
    level: u32,
}

impl GzipCompression {
    fn new(level: u32) -> Self {
        assert!((1..=9).contains(&level), "gzip level must be 1-9");
        Self { level }
    }
}

impl CompressionStrategy for GzipCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        // Simulated: In production, use flate2 crate
        // flate2::write::GzEncoder
        let mut result = Vec::with_capacity(data.len());
        result.extend_from_slice(b"GZIP");
        result.push(self.level as u8);
        // Simulate compression by removing repeated bytes
        let mut prev: Option<u8> = None;
        let mut count: u8 = 0;
        for &byte in data {
            if prev == Some(byte) && count < 255 {
                count += 1;
            } else {
                if let Some(p) = prev {
                    result.push(p);
                    if count > 0 {
                        result.push(0xFF); // escape: repeat marker
                        result.push(count);
                    }
                }
                prev = Some(byte);
                count = 0;
            }
        }
        if let Some(p) = prev {
            result.push(p);
            if count > 0 {
                result.push(0xFF);
                result.push(count);
            }
        }
        result
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() < 5 || &data[..4] != b"GZIP" {
            return Err("not gzip data".to_string());
        }
        let mut result = Vec::new();
        let payload = &data[5..]; // skip header
        let mut i = 0;
        while i < payload.len() {
            let byte = payload[i];
            result.push(byte);
            i += 1;
            if i + 1 < payload.len() && payload[i] == 0xFF {
                let count = payload[i + 1] as usize;
                for _ in 0..count {
                    result.push(byte);
                }
                i += 2;
            }
        }
        Ok(result)
    }

    fn estimated_ratio(&self) -> f64 {
        // Higher level = better compression = lower ratio
        1.0 - (self.level as f64 * 0.08)
    }

    fn name(&self) -> &str {
        "gzip"
    }
}

impl fmt::Display for GzipCompression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gzip(level={})", self.level)
    }
}

// ---

/// Zstandard compression with configurable level.
struct ZstdCompression {
    level: i32,
}

impl ZstdCompression {
    fn new(level: i32) -> Self {
        assert!((-5..=22).contains(&level), "zstd level must be -5 to 22");
        Self { level }
    }
}

impl CompressionStrategy for ZstdCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        // Simulated: In production, use zstd crate
        let mut result = Vec::with_capacity(data.len());
        result.extend_from_slice(b"ZSTD");
        result.push((self.level + 5) as u8); // offset to make positive
        // Simple RLE simulation (zstd is much more sophisticated)
        for chunk in data.chunks(64) {
            result.push(chunk.len() as u8);
            result.extend_from_slice(chunk);
        }
        result
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() < 5 || &data[..4] != b"ZSTD" {
            return Err("not zstd data".to_string());
        }
        let mut result = Vec::new();
        let mut i = 5;
        while i < data.len() {
            let chunk_len = data[i] as usize;
            i += 1;
            if i + chunk_len > data.len() {
                return Err("truncated zstd data".to_string());
            }
            result.extend_from_slice(&data[i..i + chunk_len]);
            i += chunk_len;
        }
        Ok(result)
    }

    fn estimated_ratio(&self) -> f64 {
        // Zstd generally achieves better ratios than gzip at comparable speed
        0.8 - (self.level as f64 * 0.02)
    }

    fn name(&self) -> &str {
        "zstd"
    }
}

impl fmt::Display for ZstdCompression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Zstd(level={})", self.level)
    }
}

// ---

/// No compression — passthrough. Useful for debugging or when data is already compressed.
struct NoCompression;

impl CompressionStrategy for NoCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        Ok(data.to_vec())
    }

    fn estimated_ratio(&self) -> f64 {
        1.0 // no compression
    }

    fn name(&self) -> &str {
        "none"
    }
}

impl fmt::Display for NoCompression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NoCompression")
    }
}

// ---------------------------------------------------------------------------
// Context: the data pipeline (generic over compression strategy)
// ---------------------------------------------------------------------------

/// A batch of records, compressed and ready for storage.
struct CompressedBatch {
    data: Vec<u8>,
    original_size: usize,
    record_count: usize,
    compressor_name: String,
}

impl CompressedBatch {
    fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 1.0;
        }
        self.data.len() as f64 / self.original_size as f64
    }
}

impl fmt::Display for CompressedBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Batch[records={}, original={}B, compressed={}B, ratio={:.2}, compressor={}]",
            self.record_count,
            self.original_size,
            self.data.len(),
            self.compression_ratio(),
            self.compressor_name
        )
    }
}

/// DataPipeline is generic over C: CompressionStrategy.
///
/// The compiler will generate a SEPARATE, FULLY SPECIALIZED version of
/// DataPipeline for each concrete compressor type. This means:
/// - compress() calls are inlined
/// - No vtable lookup at runtime
/// - The optimizer can see through the abstraction
///
/// The tradeoff: DataPipeline<GzipCompression> and DataPipeline<ZstdCompression>
/// are DIFFERENT TYPES. You can't put them in the same Vec or swap at runtime.
struct DataPipeline<C: CompressionStrategy> {
    compressor: C,
    batch_size: usize,
    buffer: Vec<Vec<u8>>,
    batches: Vec<CompressedBatch>,
    total_records: usize,
}

impl<C: CompressionStrategy> DataPipeline<C> {
    fn new(compressor: C, batch_size: usize) -> Self {
        println!(
            "Pipeline created: compressor={}, batch_size={}, estimated_ratio={:.2}",
            compressor,
            batch_size,
            compressor.estimated_ratio()
        );
        Self {
            compressor,
            batch_size,
            buffer: Vec::with_capacity(batch_size),
            batches: Vec::new(),
            total_records: 0,
        }
    }

    /// Ingest a record. When the buffer reaches batch_size, flush to a compressed batch.
    fn ingest(&mut self, record: &[u8]) {
        self.buffer.push(record.to_vec());
        self.total_records += 1;

        if self.buffer.len() >= self.batch_size {
            self.flush_buffer();
        }
    }

    /// Force-flush the current buffer into a compressed batch.
    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        // Concatenate records with newline separator
        let mut raw = Vec::new();
        for (i, record) in self.buffer.iter().enumerate() {
            if i > 0 {
                raw.push(b'\n');
            }
            raw.extend_from_slice(record);
        }

        let original_size = raw.len();
        let record_count = self.buffer.len();

        // This call is MONOMORPHIZED — the compiler knows the exact type of C
        // and can inline the compress() implementation directly here.
        let compressed = self.compressor.compress(&raw);

        let batch = CompressedBatch {
            data: compressed,
            original_size,
            record_count,
            compressor_name: self.compressor.name().to_string(),
        };

        println!("  Flushed: {}", batch);
        self.batches.push(batch);
        self.buffer.clear();
    }

    /// Finalize: flush remaining buffer and return all batches.
    fn finalize(mut self) -> Vec<CompressedBatch> {
        self.flush_buffer();

        let total_original: usize = self.batches.iter().map(|b| b.original_size).sum();
        let total_compressed: usize = self.batches.iter().map(|b| b.data.len()).sum();
        let overall_ratio = if total_original > 0 {
            total_compressed as f64 / total_original as f64
        } else {
            1.0
        };

        println!(
            "  Pipeline complete: {} records in {} batches, {}B -> {}B (ratio: {:.2})",
            self.total_records,
            self.batches.len(),
            total_original,
            total_compressed,
            overall_ratio
        );

        self.batches
    }

    /// Verify a batch can be round-tripped (compress -> decompress).
    fn verify_batch(&self, batch: &CompressedBatch) -> Result<Vec<u8>, String> {
        self.compressor.decompress(&batch.data)
    }
}

// ---------------------------------------------------------------------------
// Helper: create sample records
// ---------------------------------------------------------------------------

fn sample_log_records() -> Vec<Vec<u8>> {
    vec![
        b"2026-02-18T10:00:00Z INFO  server started on :8080".to_vec(),
        b"2026-02-18T10:00:01Z INFO  connected to database postgres://localhost:5432/app".to_vec(),
        b"2026-02-18T10:00:01Z INFO  migrations complete (12 applied)".to_vec(),
        b"2026-02-18T10:00:02Z INFO  health check passed".to_vec(),
        b"2026-02-18T10:00:03Z WARN  slow query detected (1.2s): SELECT * FROM users".to_vec(),
        b"2026-02-18T10:00:04Z INFO  request GET /api/users 200 12ms".to_vec(),
        b"2026-02-18T10:00:04Z INFO  request POST /api/orders 201 45ms".to_vec(),
        b"2026-02-18T10:00:05Z ERROR connection pool exhausted, waiting...".to_vec(),
        b"2026-02-18T10:00:06Z INFO  connection pool recovered (32/50 active)".to_vec(),
        b"2026-02-18T10:00:07Z INFO  request GET /api/products 200 8ms".to_vec(),
        b"2026-02-18T10:00:08Z INFO  request GET /api/products 200 8ms".to_vec(),
        b"2026-02-18T10:00:09Z INFO  request GET /api/products 200 8ms".to_vec(),
    ]
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Data Pipeline (Generic Strategy / Static Dispatch) ===\n");

    let records = sample_log_records();

    // --- Pipeline with Gzip compression ---
    println!("--- Gzip Pipeline (level 6, batch size 4) ---");
    let mut gzip_pipeline = DataPipeline::new(GzipCompression::new(6), 4);
    for record in &records {
        gzip_pipeline.ingest(record);
    }
    let gzip_batches = gzip_pipeline.finalize();
    println!();

    // --- Pipeline with Zstd compression ---
    println!("--- Zstd Pipeline (level 3, batch size 4) ---");
    let mut zstd_pipeline = DataPipeline::new(ZstdCompression::new(3), 4);
    for record in &records {
        zstd_pipeline.ingest(record);
    }
    let zstd_batches = zstd_pipeline.finalize();
    println!();

    // --- Pipeline with no compression (baseline) ---
    println!("--- No Compression Pipeline (batch size 4) ---");
    let mut raw_pipeline = DataPipeline::new(NoCompression, 4);
    for record in &records {
        raw_pipeline.ingest(record);
    }
    let raw_batches = raw_pipeline.finalize();
    println!();

    // --- Round-trip verification ---
    println!("--- Round-trip Verification ---");

    // Verify gzip
    let gzip_verifier = DataPipeline::new(GzipCompression::new(6), 1);
    for (i, batch) in gzip_batches.iter().enumerate() {
        match gzip_verifier.verify_batch(batch) {
            Ok(decompressed) => println!(
                "  Gzip batch {}: OK (decompressed {}B)",
                i,
                decompressed.len()
            ),
            Err(e) => println!("  Gzip batch {}: FAILED ({})", i, e),
        }
    }

    // Verify zstd
    let zstd_verifier = DataPipeline::new(ZstdCompression::new(3), 1);
    for (i, batch) in zstd_batches.iter().enumerate() {
        match zstd_verifier.verify_batch(batch) {
            Ok(decompressed) => println!(
                "  Zstd batch {}: OK (decompressed {}B)",
                i,
                decompressed.len()
            ),
            Err(e) => println!("  Zstd batch {}: FAILED ({})", i, e),
        }
    }

    // Verify no-compression
    let raw_verifier = DataPipeline::new(NoCompression, 1);
    for (i, batch) in raw_batches.iter().enumerate() {
        match raw_verifier.verify_batch(batch) {
            Ok(decompressed) => println!(
                "  Raw batch {}: OK (decompressed {}B)",
                i,
                decompressed.len()
            ),
            Err(e) => println!("  Raw batch {}: FAILED ({})", i, e),
        }
    }

    println!();

    // --- Comparison summary ---
    println!("--- Compression Comparison ---");
    println!(
        "  {:10} | {:>10} | {:>12} | {:>8}",
        "Strategy", "Batches", "Total Bytes", "Ratio"
    );
    println!("  {:-<10}-+-{:-<10}-+-{:-<12}-+-{:-<8}", "", "", "", "");

    for (name, batches) in [
        ("gzip-6", &gzip_batches),
        ("zstd-3", &zstd_batches),
        ("none", &raw_batches),
    ] {
        let total: usize = batches.iter().map(|b| b.data.len()).sum();
        let original: usize = batches.iter().map(|b| b.original_size).sum();
        let ratio = if original > 0 {
            total as f64 / original as f64
        } else {
            1.0
        };
        println!(
            "  {:10} | {:>10} | {:>12} | {:>8.2}",
            name,
            batches.len(),
            total,
            ratio
        );
    }

    // --- Key observation about types ---
    println!();
    println!("--- Type System Observation ---");
    println!("  DataPipeline<GzipCompression> and DataPipeline<ZstdCompression>");
    println!("  are DIFFERENT TYPES. The compiler generated separate, optimized");
    println!("  code for each. You cannot store them in the same Vec or swap");
    println!("  them at runtime. For that, you'd need trait objects:");
    println!("  Vec<Box<dyn CompressionStrategy>> or Box<dyn CompressionStrategy>.");
}
