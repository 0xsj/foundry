// Tests for Message Serialization Pipeline Factory
//
// Run: rustc --test tests.rs && ./tests
//
// These tests import from your solution. Copy your implementation into this
// file or use `mod` and `include!` to bring it in.
//
// For simplicity, paste your complete implementation above the test module,
// or replace this file's contents with your solution + these tests appended.

// TODO: Paste your implementation here (or use include!)
// include!("main.rs");

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper ---

    fn sample_record() -> Record {
        Record::new()
            .field("event", FieldValue::Text("deployment".to_string()))
            .field("service", FieldValue::Text("api-gateway".to_string()))
            .field("duration_ms", FieldValue::Integer(1234))
            .field("success", FieldValue::Boolean(true))
            .field("rollback", FieldValue::Null)
    }

    fn numeric_record() -> Record {
        Record::new()
            .field("count", FieldValue::Integer(42))
            .field("ratio", FieldValue::Float(3.14))
            .field("enabled", FieldValue::Boolean(false))
    }

    fn records_batch() -> Vec<Record> {
        vec![
            Record::new()
                .field("id", FieldValue::Integer(1))
                .field("name", FieldValue::Text("alice".to_string())),
            Record::new()
                .field("id", FieldValue::Integer(2))
                .field("name", FieldValue::Text("bob".to_string())),
            Record::new()
                .field("id", FieldValue::Integer(3))
                .field("name", FieldValue::Text("charlie".to_string())),
        ]
    }

    // -----------------------------------------------------------------------
    // Factory creation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_factory_creates_json() {
        let config = FormatConfig::new("json");
        let s = create_serializer(&config).expect("should create json serializer");
        assert_eq!(s.format_name(), "json");
        assert_eq!(s.content_type(), "application/json");
    }

    #[test]
    fn test_factory_creates_json_pretty() {
        let config = FormatConfig::new("json-pretty");
        let s = create_serializer(&config).expect("should create json-pretty serializer");
        assert_eq!(s.format_name(), "json-pretty");
    }

    #[test]
    fn test_factory_creates_binary() {
        let config = FormatConfig::new("binary");
        let s = create_serializer(&config).expect("should create binary serializer");
        assert_eq!(s.format_name(), "binary");
        assert_eq!(s.content_type(), "application/x-binary-proto");
    }

    #[test]
    fn test_factory_creates_msgpack() {
        let config = FormatConfig::new("msgpack");
        let s = create_serializer(&config).expect("should create msgpack serializer");
        assert_eq!(s.format_name(), "msgpack");
    }

    #[test]
    fn test_factory_creates_csv() {
        let config = FormatConfig::new("csv");
        let s = create_serializer(&config).expect("should create csv serializer");
        assert_eq!(s.format_name(), "csv");
        assert_eq!(s.content_type(), "text/csv");
    }

    #[test]
    fn test_factory_creates_tsv() {
        let config = FormatConfig::new("tsv");
        let s = create_serializer(&config).expect("should create tsv serializer");
        assert_eq!(s.format_name(), "tsv");
    }

    #[test]
    fn test_factory_rejects_unknown_format() {
        let config = FormatConfig::new("xml");
        let result = create_serializer(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_proto_alias() {
        let config = FormatConfig::new("proto");
        let s = create_serializer(&config).expect("'proto' should alias to binary");
        assert_eq!(s.format_name(), "binary");
    }

    // -----------------------------------------------------------------------
    // Capabilities tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_json_capabilities() {
        let config = FormatConfig::new("json");
        let s = create_serializer(&config).unwrap();
        let caps = s.capabilities();
        assert!(caps.supports_streaming);
        assert!(!caps.supports_schema);
        assert!(caps.human_readable);
    }

    #[test]
    fn test_binary_capabilities() {
        let config = FormatConfig::new("binary");
        let s = create_serializer(&config).unwrap();
        let caps = s.capabilities();
        assert!(caps.supports_streaming);
        assert!(caps.supports_schema);
        assert!(!caps.human_readable);
    }

    #[test]
    fn test_csv_capabilities() {
        let config = FormatConfig::new("csv");
        let s = create_serializer(&config).unwrap();
        let caps = s.capabilities();
        assert!(!caps.supports_streaming);
        assert!(!caps.supports_schema);
        assert!(caps.human_readable);
    }

    // -----------------------------------------------------------------------
    // JSON serialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_json_serialize_basic() {
        let config = FormatConfig::new("json");
        let s = create_serializer(&config).unwrap();
        let record = sample_record();
        let bytes = s.serialize(&record).expect("should serialize");
        let output = String::from_utf8(bytes).expect("should be valid utf-8");
        assert!(output.contains("\"event\""));
        assert!(output.contains("\"deployment\""));
        assert!(output.contains("1234"));
        assert!(output.contains("true"));
        assert!(output.contains("null"));
    }

    #[test]
    fn test_json_pretty_has_newlines() {
        let config = FormatConfig::new("json-pretty");
        let s = create_serializer(&config).unwrap();
        let record = sample_record();
        let bytes = s.serialize(&record).expect("should serialize");
        let output = String::from_utf8(bytes).expect("should be valid utf-8");
        assert!(output.contains('\n'), "pretty JSON should contain newlines");
    }

    #[test]
    fn test_json_roundtrip() {
        let config = FormatConfig::new("json");
        let s = create_serializer(&config).unwrap();
        let record = sample_record();
        let bytes = s.serialize(&record).expect("should serialize");
        let decoded = s.deserialize(&bytes).expect("should deserialize");

        assert_eq!(decoded.fields.len(), record.fields.len());
        assert_eq!(
            decoded.get("event"),
            Some(&FieldValue::Text("deployment".to_string()))
        );
        assert_eq!(
            decoded.get("service"),
            Some(&FieldValue::Text("api-gateway".to_string()))
        );
        assert_eq!(
            decoded.get("duration_ms"),
            Some(&FieldValue::Integer(1234))
        );
        assert_eq!(decoded.get("success"), Some(&FieldValue::Boolean(true)));
        assert_eq!(decoded.get("rollback"), Some(&FieldValue::Null));
    }

    #[test]
    fn test_json_roundtrip_numeric() {
        let config = FormatConfig::new("json");
        let s = create_serializer(&config).unwrap();
        let record = numeric_record();
        let bytes = s.serialize(&record).expect("should serialize");
        let decoded = s.deserialize(&bytes).expect("should deserialize");

        assert_eq!(decoded.get("count"), Some(&FieldValue::Integer(42)));
        assert_eq!(decoded.get("enabled"), Some(&FieldValue::Boolean(false)));
        // Float comparison -- check it round-trips approximately
        if let Some(FieldValue::Float(f)) = decoded.get("ratio") {
            assert!(
                (*f - 3.14).abs() < 0.001,
                "expected ~3.14, got {}",
                f
            );
        } else {
            panic!("expected Float for 'ratio'");
        }
    }

    // -----------------------------------------------------------------------
    // Binary serialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_binary_serialize_has_magic_header() {
        let config = FormatConfig::new("binary");
        let s = create_serializer(&config).unwrap();
        let record = sample_record();
        let bytes = s.serialize(&record).expect("should serialize");
        assert!(bytes.len() >= 6, "binary output should have at least 6 bytes header");
        assert_eq!(&bytes[0..4], &[0x42, 0x50, 0x01, 0x00], "magic header mismatch");
    }

    #[test]
    fn test_binary_field_count() {
        let config = FormatConfig::new("binary");
        let s = create_serializer(&config).unwrap();
        let record = sample_record(); // 5 fields
        let bytes = s.serialize(&record).expect("should serialize");
        let count = u16::from_be_bytes([bytes[4], bytes[5]]);
        assert_eq!(count, 5, "field count should be 5");
    }

    #[test]
    fn test_binary_roundtrip() {
        let config = FormatConfig::new("binary");
        let s = create_serializer(&config).unwrap();
        let record = sample_record();
        let bytes = s.serialize(&record).expect("should serialize");
        let decoded = s.deserialize(&bytes).expect("should deserialize");

        assert_eq!(decoded.fields.len(), record.fields.len());
        assert_eq!(
            decoded.get("event"),
            Some(&FieldValue::Text("deployment".to_string()))
        );
        assert_eq!(
            decoded.get("duration_ms"),
            Some(&FieldValue::Integer(1234))
        );
        assert_eq!(decoded.get("success"), Some(&FieldValue::Boolean(true)));
        assert_eq!(decoded.get("rollback"), Some(&FieldValue::Null));
    }

    #[test]
    fn test_binary_roundtrip_numeric() {
        let config = FormatConfig::new("binary");
        let s = create_serializer(&config).unwrap();
        let record = numeric_record();
        let bytes = s.serialize(&record).expect("should serialize");
        let decoded = s.deserialize(&bytes).expect("should deserialize");

        assert_eq!(decoded.get("count"), Some(&FieldValue::Integer(42)));
        assert_eq!(decoded.get("enabled"), Some(&FieldValue::Boolean(false)));
        if let Some(FieldValue::Float(f)) = decoded.get("ratio") {
            assert!((*f - 3.14).abs() < 0.001);
        } else {
            panic!("expected Float for 'ratio'");
        }
    }

    // -----------------------------------------------------------------------
    // MsgPack serialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_msgpack_starts_with_fixmap() {
        let config = FormatConfig::new("msgpack");
        let s = create_serializer(&config).unwrap();
        let record = sample_record(); // 5 fields
        let bytes = s.serialize(&record).expect("should serialize");
        assert_eq!(
            bytes[0] & 0xf0, 0x80,
            "first byte should be fixmap marker"
        );
        assert_eq!(
            bytes[0] & 0x0f, 5,
            "fixmap should encode 5 fields"
        );
    }

    #[test]
    fn test_msgpack_roundtrip() {
        let config = FormatConfig::new("msgpack");
        let s = create_serializer(&config).unwrap();
        let record = sample_record();
        let bytes = s.serialize(&record).expect("should serialize");
        let decoded = s.deserialize(&bytes).expect("should deserialize");

        assert_eq!(decoded.fields.len(), record.fields.len());
        assert_eq!(
            decoded.get("event"),
            Some(&FieldValue::Text("deployment".to_string()))
        );
        assert_eq!(
            decoded.get("duration_ms"),
            Some(&FieldValue::Integer(1234))
        );
    }

    // -----------------------------------------------------------------------
    // CSV serialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_csv_has_header() {
        let config = FormatConfig::new("csv");
        let s = create_serializer(&config).unwrap();
        let record = sample_record();
        let bytes = s.serialize(&record).expect("should serialize");
        let output = String::from_utf8(bytes).expect("should be valid utf-8");
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 2, "CSV should have header + data row");
        assert!(
            lines[0].contains("event"),
            "header should contain field names"
        );
    }

    #[test]
    fn test_csv_escapes_delimiter() {
        let config = FormatConfig::new("csv");
        let s = create_serializer(&config).unwrap();
        let record = Record::new()
            .field("name", FieldValue::Text("Smith, John".to_string()))
            .field("age", FieldValue::Integer(30));
        let bytes = s.serialize(&record).expect("should serialize");
        let output = String::from_utf8(bytes).expect("should be valid utf-8");
        assert!(
            output.contains("\"Smith, John\""),
            "values with commas should be quoted: {}",
            output
        );
    }

    #[test]
    fn test_tsv_uses_tab_delimiter() {
        let config = FormatConfig::new("tsv");
        let s = create_serializer(&config).unwrap();
        let record = Record::new()
            .field("a", FieldValue::Text("hello".to_string()))
            .field("b", FieldValue::Text("world".to_string()));
        let bytes = s.serialize(&record).expect("should serialize");
        let output = String::from_utf8(bytes).expect("should be valid utf-8");
        let lines: Vec<&str> = output.lines().collect();
        assert!(
            lines[0].contains('\t'),
            "TSV header should use tabs: {:?}",
            lines[0]
        );
    }

    #[test]
    fn test_csv_roundtrip() {
        let config = FormatConfig::new("csv");
        let s = create_serializer(&config).unwrap();
        let record = Record::new()
            .field("name", FieldValue::Text("alice".to_string()))
            .field("score", FieldValue::Text("100".to_string()));
        let bytes = s.serialize(&record).expect("should serialize");
        let decoded = s.deserialize(&bytes).expect("should deserialize");

        // CSV loses type info -- everything comes back as Text
        assert_eq!(decoded.fields.len(), 2);
        assert_eq!(
            decoded.get("name"),
            Some(&FieldValue::Text("alice".to_string()))
        );
    }

    // -----------------------------------------------------------------------
    // Batch serialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_json_batch() {
        let config = FormatConfig::new("json");
        let s = create_serializer(&config).unwrap();
        let records = records_batch();
        let bytes = s.serialize_batch(&records).expect("should serialize batch");
        let output = String::from_utf8(bytes).expect("should be valid utf-8");
        assert!(output.starts_with('['), "JSON batch should be an array");
        assert!(output.ends_with(']'), "JSON batch should end with ]");
        assert!(output.contains("alice"));
        assert!(output.contains("bob"));
        assert!(output.contains("charlie"));
    }

    #[test]
    fn test_binary_batch() {
        let config = FormatConfig::new("binary");
        let s = create_serializer(&config).unwrap();
        let records = records_batch();
        let bytes = s.serialize_batch(&records).expect("should serialize batch");
        // First 2 bytes should be record count
        let count = u16::from_be_bytes([bytes[0], bytes[1]]);
        assert_eq!(count, 3, "batch should contain 3 records");
    }

    #[test]
    fn test_csv_batch_single_header() {
        let config = FormatConfig::new("csv");
        let s = create_serializer(&config).unwrap();
        let records = records_batch();
        let bytes = s.serialize_batch(&records).expect("should serialize batch");
        let output = String::from_utf8(bytes).expect("should be valid utf-8");
        let lines: Vec<&str> = output.lines().collect();
        // Should be: 1 header + 3 data rows = 4 lines
        assert_eq!(lines.len(), 4, "CSV batch should have 1 header + 3 rows, got: {:?}", lines);
    }

    // -----------------------------------------------------------------------
    // Trait object tests (factory returns Box<dyn Serializer>)
    // -----------------------------------------------------------------------

    #[test]
    fn test_serializers_in_vec() {
        let formats = vec!["json", "binary", "msgpack", "csv"];
        let serializers: Vec<Box<dyn Serializer>> = formats
            .iter()
            .map(|f| create_serializer(&FormatConfig::new(f)).unwrap())
            .collect();

        let record = sample_record();
        for s in &serializers {
            let result = s.serialize(&record);
            assert!(result.is_ok(), "{} serialization failed", s.format_name());
        }
    }

    #[test]
    fn test_display_impl() {
        let formats = vec!["json", "binary", "msgpack", "csv"];
        for f in &formats {
            let s = create_serializer(&FormatConfig::new(f)).unwrap();
            let display = format!("{}", s);
            assert!(!display.is_empty(), "{} Display should not be empty", f);
        }
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_record() {
        let formats = vec!["json", "binary", "msgpack", "csv"];
        let empty = Record::new();
        for f in &formats {
            let s = create_serializer(&FormatConfig::new(f)).unwrap();
            let result = s.serialize(&empty);
            assert!(
                result.is_ok(),
                "{} should handle empty record",
                f
            );
        }
    }

    #[test]
    fn test_empty_batch() {
        let formats = vec!["json", "binary", "msgpack", "csv"];
        let empty: Vec<Record> = vec![];
        for f in &formats {
            let s = create_serializer(&FormatConfig::new(f)).unwrap();
            let result = s.serialize_batch(&empty);
            assert!(
                result.is_ok(),
                "{} should handle empty batch",
                f
            );
        }
    }

    #[test]
    fn test_json_special_characters() {
        let config = FormatConfig::new("json");
        let s = create_serializer(&config).unwrap();
        let record = Record::new()
            .field("message", FieldValue::Text(r#"He said "hello""#.to_string()));
        let bytes = s.serialize(&record).expect("should handle quotes");
        let output = String::from_utf8(bytes).expect("should be valid utf-8");
        // The output should have escaped quotes
        assert!(
            output.contains(r#"\""#) || output.contains(r#"\u0022"#),
            "JSON should escape quotes in strings: {}",
            output
        );
    }
}
