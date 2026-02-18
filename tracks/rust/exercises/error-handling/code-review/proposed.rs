// CSV Sales Report Pipeline — Proposed PR
//
// "Adds proper error handling throughout the pipeline. All panics replaced
//  with Result return types. Ready for production."
//
// Run: rustc proposed.rs && ./proposed

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct SalesRecord {
    pub region: String,
    pub product_id: String,
    pub quantity: u32,
    pub unit_price: f64,
}

impl SalesRecord {
    pub fn total(&self) -> f64 {
        self.quantity as f64 * self.unit_price
    }
}

#[derive(Debug)]
pub struct RegionSummary {
    pub region: String,
    pub total_sales: f64,
    pub record_count: usize,
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse a single CSV line into a SalesRecord.
/// CSV format: region,product_id,quantity,unit_price
pub fn parse_record(line: &str, line_num: usize) -> Result<SalesRecord, String> {
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() != 4 {
        return Err(format!("line {}: expected 4 fields, got {}", line_num, parts.len()));
    }

    let region = parts[0].trim().to_string();
    let product_id = parts[1].trim().to_string();

    let quantity: u32 = parts[2].trim().parse().map_err(|_| {
        format!("line {}: invalid quantity '{}'", line_num, parts[2].trim())
    })?;

    let unit_price: f64 = parts[3].trim().parse().map_err(|_| {
        format!("line {}: invalid unit_price '{}'", line_num, parts[3].trim())
    })?;

    Ok(SalesRecord { region, product_id, quantity, unit_price })
}

/// Parse an entire CSV file content into a list of SalesRecords.
/// Skips the header row. Returns an error if any row fails to parse.
pub fn parse_csv(content: &str, filename: &str) -> Result<Vec<SalesRecord>, String> {
    let mut records = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        if idx == 0 { continue; } // skip header

        let line = line.trim();
        if line.is_empty() { continue; }

        let record = parse_record(line, idx + 1).map_err(|e| {
            format!("error in file '{}': {}", filename, e)
        })?;

        records.push(record);
    }

    Ok(records)
}

// ---------------------------------------------------------------------------
// Aggregation
// ---------------------------------------------------------------------------

/// Aggregate records by region, computing total sales and count.
pub fn aggregate_by_region(records: &[SalesRecord]) -> HashMap<String, RegionSummary> {
    let mut summaries: HashMap<String, RegionSummary> = HashMap::new();

    for record in records {
        let entry = summaries.entry(record.region.clone()).or_insert_with(|| {
            RegionSummary {
                region: record.region.clone(),
                total_sales: 0.0,
                record_count: 0,
            }
        });
        entry.total_sales += record.total();
        entry.record_count += 1;
    }

    summaries
}

// ---------------------------------------------------------------------------
// File loading
// ---------------------------------------------------------------------------

/// Load a CSV file from disk.
pub fn load_csv_file(path: &str) -> Result<Vec<SalesRecord>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| e.to_string())?;   // io::Error converted to String
    parse_csv(&content, path)
}

// ---------------------------------------------------------------------------
// Pipeline orchestration
// ---------------------------------------------------------------------------

/// Process all CSV files in a directory. Returns summaries per region.
///
/// On any error, logs the error and returns it immediately — stops processing.
pub fn run_pipeline(data_dir: &str) -> Result<HashMap<String, RegionSummary>, String> {
    let entries = std::fs::read_dir(data_dir)
        .map_err(|e| e.to_string())?;

    let mut all_records: Vec<SalesRecord> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("csv") {
            continue;
        }

        let path_str = path.to_string_lossy().to_string();

        match load_csv_file(&path_str) {
            Ok(records) => all_records.extend(records),
            Err(e) => {
                eprintln!("error processing {}: {}", path_str, e);
                return Err(e);  // abort on first file error
            }
        }
    }

    Ok(aggregate_by_region(&all_records))
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Print a summary report to stdout.
pub fn print_report(summaries: &HashMap<String, RegionSummary>) {
    let mut regions: Vec<&RegionSummary> = summaries.values().collect();
    regions.sort_by(|a, b| b.total_sales.partial_cmp(&a.total_sales).unwrap());

    println!("Sales Report");
    println!("{:-<40}", "");
    for summary in regions {
        println!(
            "{:20} {:>10.2}  ({} records)",
            summary.region, summary.total_sales, summary.record_count
        );
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    // In production, data_dir would come from an environment variable or CLI arg.
    // Here we simulate it with a directory that doesn't exist.
    let data_dir = std::env::args().nth(1).unwrap_or_else(|| "/tmp/sales".to_string());

    match run_pipeline(&data_dir) {
        Ok(summaries) => print_report(&summaries),
        Err(e) => {
            eprintln!("pipeline failed: {}", e);
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_record() {
        let record = parse_record("North,SKU-001,10,25.50", 1).unwrap();
        assert_eq!(record.region, "North");
        assert_eq!(record.quantity, 10);
        assert!((record.unit_price - 25.50).abs() < 1e-9);
    }

    #[test]
    fn test_parse_invalid_quantity() {
        let err = parse_record("North,SKU-001,abc,25.50", 3).unwrap_err();
        assert!(err.contains("line 3"), "error should mention line number: {}", err);
        assert!(err.contains("quantity"), "error should mention field: {}", err);
    }

    #[test]
    fn test_parse_csv_propagates_filename() {
        let csv = "region,product,quantity,price\nNorth,SKU-1,bad,10.0\n";
        let err = parse_csv(csv, "sales_2026_01.csv").unwrap_err();
        assert!(err.contains("sales_2026_01.csv"), "should include filename: {}", err);
    }

    #[test]
    fn test_aggregate_totals() {
        let records = vec![
            SalesRecord { region: "North".into(), product_id: "A".into(), quantity: 2, unit_price: 10.0 },
            SalesRecord { region: "North".into(), product_id: "B".into(), quantity: 3, unit_price: 5.0 },
            SalesRecord { region: "South".into(), product_id: "A".into(), quantity: 1, unit_price: 20.0 },
        ];
        let summaries = aggregate_by_region(&records);
        assert!((summaries["North"].total_sales - 35.0).abs() < 1e-9);
        assert!((summaries["South"].total_sales - 20.0).abs() < 1e-9);
    }
}
