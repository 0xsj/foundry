// Factory Pattern: Serialization Format Factory
//
// Demonstrates: Enum dispatch as a factory alternative, From/Into conversions
//
// Scenario: A data pipeline needs to serialize records into different wire
// formats (JSON, CSV, MessagePack-like binary). All formats are known at
// compile time, so enum dispatch is the right tool -- no heap allocation,
// no vtable overhead, exhaustive matching.
//
// Run: rustc serializer.rs && ./serializer

use std::fmt;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A key-value record to be serialized. In production this would be a
/// domain struct; here we use a simple representation.
#[derive(Debug, Clone)]
struct Record {
    fields: Vec<(String, FieldValue)>,
}

#[derive(Debug, Clone)]
enum FieldValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::Text(s) => write!(f, "{}", s),
            FieldValue::Integer(n) => write!(f, "{}", n),
            FieldValue::Float(n) => write!(f, "{}", n),
            FieldValue::Boolean(b) => write!(f, "{}", b),
            FieldValue::Null => write!(f, "null"),
        }
    }
}

impl Record {
    fn new() -> Self {
        Self { fields: Vec::new() }
    }

    fn field(mut self, key: &str, value: FieldValue) -> Self {
        self.fields.push((key.to_string(), value));
        self
    }
}

// ---------------------------------------------------------------------------
// From/Into conversions -- built-in factory protocol
// ---------------------------------------------------------------------------

/// Convert a tuple of (&str, &str) pairs into a Record.
/// This is a factory hiding in plain sight.
impl From<Vec<(&str, &str)>> for Record {
    fn from(pairs: Vec<(&str, &str)>) -> Self {
        let fields = pairs
            .into_iter()
            .map(|(k, v)| (k.to_string(), FieldValue::Text(v.to_string())))
            .collect();
        Record { fields }
    }
}

/// Convert from a JSON-like string representation (simplified parser).
/// Uses TryFrom because parsing can fail.
impl std::convert::TryFrom<&str> for Record {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        // Very simplified: expects "key1=value1,key2=value2" format
        let mut record = Record::new();
        if input.is_empty() {
            return Ok(record);
        }
        for pair in input.split(',') {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("invalid field: '{}' (expected key=value)", pair));
            }
            let key = parts[0].trim();
            let value = parts[1].trim();

            // Try to parse as different types
            let field_value = if value == "null" {
                FieldValue::Null
            } else if value == "true" || value == "false" {
                FieldValue::Boolean(value == "true")
            } else if let Ok(n) = value.parse::<i64>() {
                FieldValue::Integer(n)
            } else if let Ok(n) = value.parse::<f64>() {
                FieldValue::Float(n)
            } else {
                FieldValue::Text(value.to_string())
            };

            record.fields.push((key.to_string(), field_value));
        }
        Ok(record)
    }
}

// ---------------------------------------------------------------------------
// Serializer enum -- closed set of formats
// ---------------------------------------------------------------------------

/// Serialization format configuration.
///
/// All variants are known at compile time. Adding a new format requires
/// modifying this enum and all match arms -- the compiler enforces exhaustiveness.
enum Serializer {
    Json {
        pretty: bool,
        sort_keys: bool,
    },
    Csv {
        delimiter: char,
        include_header: bool,
    },
    MsgPack,
}

/// Serialization output with metadata.
struct SerializedOutput {
    data: Vec<u8>,
    content_type: String,
    format_name: String,
}

impl fmt::Display for SerializedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} bytes, content-type: {}",
            self.format_name,
            self.data.len(),
            self.content_type
        )
    }
}

// ---------------------------------------------------------------------------
// Serializer factory (enum constructor) and implementation
// ---------------------------------------------------------------------------

impl Serializer {
    /// Factory method: create a serializer from a format name string.
    /// This is the "factory function" -- it maps a runtime string to a
    /// compile-time-known enum variant.
    fn from_format(format: &str) -> Result<Self, String> {
        match format {
            "json" => Ok(Serializer::Json {
                pretty: false,
                sort_keys: false,
            }),
            "json-pretty" => Ok(Serializer::Json {
                pretty: true,
                sort_keys: true,
            }),
            "csv" => Ok(Serializer::Csv {
                delimiter: ',',
                include_header: true,
            }),
            "tsv" => Ok(Serializer::Csv {
                delimiter: '\t',
                include_header: true,
            }),
            "msgpack" => Ok(Serializer::MsgPack),
            other => Err(format!(
                "unsupported format: '{}'. Available: json, json-pretty, csv, tsv, msgpack",
                other
            )),
        }
    }

    /// Serialize a single record.
    fn serialize(&self, record: &Record) -> SerializedOutput {
        match self {
            Serializer::Json { pretty, sort_keys } => {
                self.serialize_json(record, *pretty, *sort_keys)
            }
            Serializer::Csv {
                delimiter,
                include_header,
            } => self.serialize_csv(record, *delimiter, *include_header),
            Serializer::MsgPack => self.serialize_msgpack(record),
        }
    }

    /// Serialize multiple records (batch).
    fn serialize_batch(&self, records: &[Record]) -> SerializedOutput {
        match self {
            Serializer::Json { pretty, sort_keys } => {
                let mut out = String::from("[");
                for (i, record) in records.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    if *pretty {
                        out.push('\n');
                    }
                    let single = self.serialize_json(record, *pretty, *sort_keys);
                    let json_str = String::from_utf8_lossy(&single.data);
                    if *pretty {
                        // Indent each line
                        for (j, line) in json_str.lines().enumerate() {
                            if j > 0 {
                                out.push('\n');
                            }
                            out.push_str("  ");
                            out.push_str(line);
                        }
                    } else {
                        out.push_str(&json_str);
                    }
                }
                if *pretty {
                    out.push('\n');
                }
                out.push(']');
                SerializedOutput {
                    data: out.into_bytes(),
                    content_type: "application/json".to_string(),
                    format_name: "json-array".to_string(),
                }
            }
            Serializer::Csv {
                delimiter,
                include_header,
            } => {
                let mut out = String::new();
                for (i, record) in records.iter().enumerate() {
                    let single = self.serialize_csv(record, *delimiter, i > 0 || !include_header);
                    out.push_str(&String::from_utf8_lossy(&single.data));
                    out.push('\n');
                }
                SerializedOutput {
                    data: out.into_bytes(),
                    content_type: "text/csv".to_string(),
                    format_name: "csv-batch".to_string(),
                }
            }
            Serializer::MsgPack => {
                // fixarray header + concatenated records
                let count = records.len();
                let mut bytes = vec![0x90 | (count as u8 & 0x0f)];
                for record in records {
                    let single = self.serialize_msgpack(record);
                    bytes.extend_from_slice(&single.data);
                }
                SerializedOutput {
                    data: bytes,
                    content_type: "application/x-msgpack".to_string(),
                    format_name: "msgpack-array".to_string(),
                }
            }
        }
    }

    fn content_type(&self) -> &str {
        match self {
            Serializer::Json { .. } => "application/json",
            Serializer::Csv { .. } => "text/csv",
            Serializer::MsgPack => "application/x-msgpack",
        }
    }

    fn format_name(&self) -> &str {
        match self {
            Serializer::Json { pretty: true, .. } => "json-pretty",
            Serializer::Json { .. } => "json",
            Serializer::Csv { delimiter: '\t', .. } => "tsv",
            Serializer::Csv { .. } => "csv",
            Serializer::MsgPack => "msgpack",
        }
    }

    // --- Private serialization methods ---

    fn serialize_json(&self, record: &Record, pretty: bool, sort_keys: bool) -> SerializedOutput {
        let mut fields = record.fields.clone();
        if sort_keys {
            fields.sort_by(|a, b| a.0.cmp(&b.0));
        }

        let indent = if pretty { "  " } else { "" };
        let newline = if pretty { "\n" } else { "" };
        let sep = if pretty { ",\n" } else { "," };

        let mut out = format!("{{{}", newline);
        for (i, (key, value)) in fields.iter().enumerate() {
            if i > 0 {
                out.push_str(sep);
            }
            let json_value = match value {
                FieldValue::Text(s) => format!(r#""{}""#, s.replace('"', r#"\""#)),
                FieldValue::Integer(n) => n.to_string(),
                FieldValue::Float(n) => format!("{}", n),
                FieldValue::Boolean(b) => b.to_string(),
                FieldValue::Null => "null".to_string(),
            };
            out.push_str(&format!(r#"{}"{}": {}"#, indent, key, json_value));
        }
        out.push_str(&format!("{}}}", newline));

        SerializedOutput {
            data: out.into_bytes(),
            content_type: "application/json".to_string(),
            format_name: self.format_name().to_string(),
        }
    }

    fn serialize_csv(
        &self,
        record: &Record,
        delimiter: char,
        skip_header: bool,
    ) -> SerializedOutput {
        let mut out = String::new();
        let delim = delimiter.to_string();

        if !skip_header {
            let header: Vec<&str> = record.fields.iter().map(|(k, _)| k.as_str()).collect();
            out.push_str(&header.join(&delim));
            out.push('\n');
        }

        let values: Vec<String> = record
            .fields
            .iter()
            .map(|(_, v)| {
                let s = v.to_string();
                if s.contains(delimiter) || s.contains('"') || s.contains('\n') {
                    format!(r#""{}""#, s.replace('"', r#""""#))
                } else {
                    s
                }
            })
            .collect();
        out.push_str(&values.join(&delim));

        SerializedOutput {
            data: out.into_bytes(),
            content_type: "text/csv".to_string(),
            format_name: self.format_name().to_string(),
        }
    }

    fn serialize_msgpack(&self, record: &Record) -> SerializedOutput {
        let count = record.fields.len();
        // fixmap: 0x80 | count (for count < 16)
        let mut bytes = vec![0x80 | (count as u8 & 0x0f)];

        for (key, value) in &record.fields {
            // fixstr for key
            let key_bytes = key.as_bytes();
            bytes.push(0xa0 | (key_bytes.len() as u8 & 0x1f));
            bytes.extend_from_slice(key_bytes);

            // value encoding
            match value {
                FieldValue::Text(s) => {
                    let val_bytes = s.as_bytes();
                    if val_bytes.len() < 32 {
                        bytes.push(0xa0 | (val_bytes.len() as u8 & 0x1f));
                    } else {
                        bytes.push(0xd9); // str 8
                        bytes.push(val_bytes.len() as u8);
                    }
                    bytes.extend_from_slice(val_bytes);
                }
                FieldValue::Integer(n) => {
                    if *n >= 0 && *n < 128 {
                        bytes.push(*n as u8); // positive fixint
                    } else {
                        bytes.push(0xd1); // int16
                        bytes.extend_from_slice(&(*n as i16).to_be_bytes());
                    }
                }
                FieldValue::Float(n) => {
                    bytes.push(0xcb); // float64
                    bytes.extend_from_slice(&n.to_be_bytes());
                }
                FieldValue::Boolean(b) => {
                    bytes.push(if *b { 0xc3 } else { 0xc2 });
                }
                FieldValue::Null => {
                    bytes.push(0xc0);
                }
            }
        }

        SerializedOutput {
            data: bytes,
            content_type: "application/x-msgpack".to_string(),
            format_name: "msgpack".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Implement FromStr so we can use .parse()
// ---------------------------------------------------------------------------

impl std::str::FromStr for Serializer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_format(s)
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Serializer Factory (Enum Dispatch + From/Into) ===\n");

    // --- Create records using From/Into conversions ---
    println!("--- Creating records via From/Into ---");

    // From Vec<(&str, &str)>
    let record1: Record = vec![("host", "db.internal"), ("port", "5432"), ("ssl", "true")].into();
    println!("  From Vec<(&str, &str)>: {:?}", record1.fields.len());

    // TryFrom &str
    let record2 = Record::try_from("name=my-service,version=2.3.1,replicas=3,healthy=true");
    match &record2 {
        Ok(r) => println!("  TryFrom &str: {} fields parsed", r.fields.len()),
        Err(e) => println!("  TryFrom &str failed: {}", e),
    }
    let record2 = record2.unwrap();

    // Using builder-style
    let record3 = Record::new()
        .field("event", FieldValue::Text("deployment".to_string()))
        .field("service", FieldValue::Text("api-gateway".to_string()))
        .field("duration_ms", FieldValue::Integer(1234))
        .field("success", FieldValue::Boolean(true))
        .field("rollback_target", FieldValue::Null);

    println!("  Builder-style: {} fields\n", record3.fields.len());

    // --- Factory: create serializers from format strings ---
    let formats = ["json", "json-pretty", "csv", "tsv", "msgpack", "xml"];

    for format in &formats {
        print!("--- Format: {} --- ", format);
        match Serializer::from_format(format) {
            Ok(serializer) => {
                let output = serializer.serialize(&record3);
                println!("{}", output);
                let text = String::from_utf8_lossy(&output.data);
                if output.data.len() < 200 {
                    println!("{}\n", text);
                } else {
                    println!("  (output truncated, {} bytes)\n", output.data.len());
                }
            }
            Err(e) => println!("Error: {}\n", e),
        }
    }

    // --- Using .parse() via FromStr ---
    println!("--- Using .parse() ---");
    let serializer: Result<Serializer, _> = "json-pretty".parse();
    if let Ok(s) = serializer {
        println!("  Parsed format: {}", s.format_name());
        println!("  Content-Type: {}\n", s.content_type());
    }

    // --- Batch serialization ---
    println!("--- Batch serialization (json-pretty) ---");
    let serializer = Serializer::from_format("json-pretty").unwrap();
    let records = vec![record1, record2, record3];
    let batch_output = serializer.serialize_batch(&records);
    println!("{}", batch_output);
    println!("{}", String::from_utf8_lossy(&batch_output.data));
}
