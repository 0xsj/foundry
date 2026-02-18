// Message Serialization Pipeline Factory -- Starter
//
// Implement a factory that creates serializers for different wire formats.
// Run tests: rustc --test ../tests.rs && ../tests
// Or just run the demo: rustc main.rs && ./main

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub fields: Vec<(String, FieldValue)>,
}

impl Record {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field(mut self, key: &str, value: FieldValue) -> Self {
        self.fields.push((key.to_string(), value));
        self
    }

    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.fields.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct SerializerError {
    pub format: String,
    pub operation: String,
    pub reason: String,
}

impl fmt::Display for SerializerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} failed: {}",
            self.format, self.operation, self.reason
        )
    }
}

// ---------------------------------------------------------------------------
// Capabilities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Capabilities {
    pub supports_streaming: bool,
    pub supports_schema: bool,
    pub human_readable: bool,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub format: String,
    pub options: HashMap<String, String>,
}

impl FormatConfig {
    pub fn new(format: &str) -> Self {
        Self {
            format: format.to_string(),
            options: HashMap::new(),
        }
    }

    pub fn option(mut self, key: &str, value: &str) -> Self {
        self.options.insert(key.to_string(), value.to_string());
        self
    }

    pub fn get_option(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(|s| s.as_str())
    }

    pub fn get_option_or(&self, key: &str, default: &str) -> String {
        self.options
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }
}

// ---------------------------------------------------------------------------
// Serializer trait
// ---------------------------------------------------------------------------

// TODO: Define the Serializer trait
//
// Requirements:
// - Must be object-safe (no generic methods, no Self return by value)
// - Methods:
//   - fn serialize(&self, record: &Record) -> Result<Vec<u8>, SerializerError>
//   - fn deserialize(&self, data: &[u8]) -> Result<Record, SerializerError>
//   - fn serialize_batch(&self, records: &[Record]) -> Result<Vec<u8>, SerializerError>
//   - fn capabilities(&self) -> Capabilities
//   - fn format_name(&self) -> &str
//   - fn content_type(&self) -> &str
// - Must also require: fmt::Display

// ---------------------------------------------------------------------------
// JSON Serializer
// ---------------------------------------------------------------------------

// TODO: Implement JsonSerializer
//
// Fields: pretty (bool)
// Content-Type: application/json
// Capabilities: streaming=true, schema=false, human_readable=true
//
// Serialization format:
//   {"key1": "value1", "key2": 42, "key3": true, "key4": null}
// When pretty=true, indent with 2 spaces.
//
// Batch format: JSON array of objects
//
// Deserialization: Parse the JSON string back into a Record.
// Hint: You'll need a simple JSON parser. Handle: strings, integers, floats,
// booleans, null. You don't need to handle nested objects or arrays.

// ---------------------------------------------------------------------------
// Binary Protocol Serializer (Protobuf-like)
// ---------------------------------------------------------------------------

// TODO: Implement BinarySerializer
//
// Content-Type: application/x-binary-proto
// Capabilities: streaming=true, schema=true, human_readable=false
//
// Encoding format (TLV -- Tag-Length-Value):
//   Header: 4 bytes magic [0x42, 0x50, 0x01, 0x00] + 2 bytes field count (big-endian u16)
//   Per field:
//     - Key: 1 byte key length + key bytes
//     - Type tag: 1 byte (0=null, 1=text, 2=int64, 3=float64, 4=bool)
//     - Value:
//       - null: no additional bytes
//       - text: 2 bytes length (big-endian u16) + string bytes
//       - int64: 8 bytes (big-endian i64)
//       - float64: 8 bytes (big-endian f64)
//       - bool: 1 byte (0 or 1)
//
// Batch format: 2 bytes record count (big-endian u16) + concatenated records

// ---------------------------------------------------------------------------
// MessagePack-like Serializer
// ---------------------------------------------------------------------------

// TODO: Implement MsgPackSerializer
//
// Content-Type: application/x-msgpack
// Capabilities: streaming=true, schema=false, human_readable=false
//
// Simplified MessagePack encoding:
//   fixmap (0x80 | count) for record
//   fixstr (0xa0 | len) for short strings (<32 bytes)
//   str8 (0xd9 + 1-byte len) for longer strings
//   positive fixint (0x00-0x7f) for small positive integers
//   int64 (0xd3 + 8 bytes big-endian) for larger integers
//   float64 (0xcb + 8 bytes big-endian) for floats
//   true (0xc3), false (0xc2) for booleans
//   nil (0xc0) for null
//
// Batch format: fixarray (0x90 | count) + concatenated records

// ---------------------------------------------------------------------------
// CSV Serializer
// ---------------------------------------------------------------------------

// TODO: Implement CsvSerializer
//
// Fields: delimiter (char), include_header (bool)
// Content-Type: text/csv
// Capabilities: streaming=false, schema=false, human_readable=true
//
// Rules:
//   - First row is header (field names) if include_header is true
//   - Values containing the delimiter, quotes, or newlines must be quoted
//   - Quotes within values are escaped by doubling them
//   - All field types are converted to their string representation
//
// Batch format: header (once) + one row per record
//
// Deserialization: Parse CSV back into a Record.
// Hint: All values will be FieldValue::Text since CSV loses type info.

// ---------------------------------------------------------------------------
// Factory function
// ---------------------------------------------------------------------------

// TODO: Implement create_serializer
//
// pub fn create_serializer(config: &FormatConfig) -> Result<Box<dyn Serializer>, SerializerError>
//
// Match on config.format:
//   "json" -> JsonSerializer { pretty: false }
//   "json-pretty" -> JsonSerializer { pretty: true }
//   "binary" | "proto" -> BinarySerializer
//   "msgpack" -> MsgPackSerializer
//   "csv" -> CsvSerializer { delimiter: ',', include_header: true }
//   "tsv" -> CsvSerializer { delimiter: '\t', include_header: true }
//   _ -> Err(SerializerError { ... })
//
// Read options from config:
//   "pretty" -> override pretty for JSON
//   "delimiter" -> override delimiter for CSV
//   "header" -> override include_header for CSV

// ---------------------------------------------------------------------------
// main (demo)
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Message Serialization Pipeline Factory ===\n");

    let record = Record::new()
        .field("event", FieldValue::Text("deployment".to_string()))
        .field("service", FieldValue::Text("api-gateway".to_string()))
        .field("duration_ms", FieldValue::Integer(1234))
        .field("success", FieldValue::Boolean(true))
        .field("rollback", FieldValue::Null);

    let formats = ["json", "json-pretty", "binary", "msgpack", "csv", "tsv"];

    for format in &formats {
        let config = FormatConfig::new(format);
        println!("--- {} ---", format);

        // TODO: Uncomment after implementing the factory
        // match create_serializer(&config) {
        //     Ok(serializer) => {
        //         println!("  Format: {} ({})", serializer.format_name(), serializer);
        //         println!("  Content-Type: {}", serializer.content_type());
        //         println!("  Capabilities: {:?}", serializer.capabilities());
        //
        //         match serializer.serialize(&record) {
        //             Ok(bytes) => {
        //                 println!("  Serialized: {} bytes", bytes.len());
        //                 if serializer.capabilities().human_readable {
        //                     println!("  Output:\n{}", String::from_utf8_lossy(&bytes));
        //                 }
        //
        //                 // Round-trip test
        //                 match serializer.deserialize(&bytes) {
        //                     Ok(decoded) => println!("  Round-trip: {} fields decoded", decoded.fields.len()),
        //                     Err(e) => println!("  Deserialize error: {}", e),
        //                 }
        //             }
        //             Err(e) => println!("  Serialize error: {}", e),
        //         }
        //     }
        //     Err(e) => println!("  Factory error: {}", e),
        // }
        println!();
    }
}
