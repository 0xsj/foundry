// Message Serialization Pipeline Factory -- Reference Solution
//
// Run demo: rustc solution.rs && ./solution
// Run tests: rustc --test solution_test.rs && ./solution_test

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

/// A wire-format serializer for Record data.
///
/// Object-safe: no generic methods, no Self return by value.
/// Can be used as Box<dyn Serializer>, &dyn Serializer, etc.
pub trait Serializer: fmt::Display {
    fn serialize(&self, record: &Record) -> Result<Vec<u8>, SerializerError>;
    fn deserialize(&self, data: &[u8]) -> Result<Record, SerializerError>;
    fn serialize_batch(&self, records: &[Record]) -> Result<Vec<u8>, SerializerError>;
    fn capabilities(&self) -> Capabilities;
    fn format_name(&self) -> &str;
    fn content_type(&self) -> &str;
}

// ---------------------------------------------------------------------------
// JSON Serializer
// ---------------------------------------------------------------------------

struct JsonSerializer {
    pretty: bool,
}

impl JsonSerializer {
    fn err(&self, op: &str, reason: &str) -> SerializerError {
        SerializerError {
            format: self.format_name().to_string(),
            operation: op.to_string(),
            reason: reason.to_string(),
        }
    }

    fn escape_json_string(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c < '\x20' => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    }

    fn serialize_value(value: &FieldValue) -> String {
        match value {
            FieldValue::Text(s) => format!("\"{}\"", Self::escape_json_string(s)),
            FieldValue::Integer(n) => n.to_string(),
            FieldValue::Float(n) => {
                let s = format!("{}", n);
                // Ensure it looks like a float
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            FieldValue::Boolean(b) => b.to_string(),
            FieldValue::Null => "null".to_string(),
        }
    }

    fn serialize_record(&self, record: &Record) -> String {
        if self.pretty {
            let mut out = String::from("{\n");
            for (i, (key, value)) in record.fields.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&format!(
                    "  \"{}\": {}",
                    Self::escape_json_string(key),
                    Self::serialize_value(value)
                ));
            }
            out.push_str("\n}");
            out
        } else {
            let mut out = String::from("{");
            for (i, (key, value)) in record.fields.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "\"{}\":{}",
                    Self::escape_json_string(key),
                    Self::serialize_value(value)
                ));
            }
            out.push('}');
            out
        }
    }

    fn parse_json_value<'a>(input: &'a str) -> Result<(FieldValue, &'a str), String> {
        let input = input.trim_start();
        if input.is_empty() {
            return Err("unexpected end of input".to_string());
        }

        if input.starts_with('"') {
            // String
            let (s, rest) = Self::parse_json_string(input)?;
            Ok((FieldValue::Text(s), rest))
        } else if input.starts_with("null") {
            Ok((FieldValue::Null, &input[4..]))
        } else if input.starts_with("true") {
            Ok((FieldValue::Boolean(true), &input[4..]))
        } else if input.starts_with("false") {
            Ok((FieldValue::Boolean(false), &input[5..]))
        } else if input.starts_with('-') || input.as_bytes()[0].is_ascii_digit() {
            Self::parse_json_number(input)
        } else {
            Err(format!(
                "unexpected character: '{}'",
                input.chars().next().unwrap()
            ))
        }
    }

    fn parse_json_string(input: &str) -> Result<(String, &str), String> {
        if !input.starts_with('"') {
            return Err("expected '\"'".to_string());
        }
        let input = &input[1..]; // skip opening quote
        let mut result = String::new();
        let mut chars = input.char_indices();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' => return Ok((result, &input[i + 1..])),
                '\\' => {
                    if let Some((_, escaped)) = chars.next() {
                        match escaped {
                            '"' => result.push('"'),
                            '\\' => result.push('\\'),
                            'n' => result.push('\n'),
                            'r' => result.push('\r'),
                            't' => result.push('\t'),
                            'u' => {
                                let hex: String = chars.by_ref().take(4).map(|(_, c)| c).collect();
                                if hex.len() == 4 {
                                    if let Ok(n) = u32::from_str_radix(&hex, 16) {
                                        if let Some(ch) = char::from_u32(n) {
                                            result.push(ch);
                                        }
                                    }
                                }
                            }
                            _ => {
                                result.push('\\');
                                result.push(escaped);
                            }
                        }
                    }
                }
                _ => result.push(c),
            }
        }
        Err("unterminated string".to_string())
    }

    fn parse_json_number(input: &str) -> Result<(FieldValue, &str), String> {
        let mut end = 0;
        let mut is_float = false;
        let bytes = input.as_bytes();

        if end < bytes.len() && bytes[end] == b'-' {
            end += 1;
        }
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end < bytes.len() && bytes[end] == b'.' {
            is_float = true;
            end += 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        }
        if end < bytes.len() && (bytes[end] == b'e' || bytes[end] == b'E') {
            is_float = true;
            end += 1;
            if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
                end += 1;
            }
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        }

        let num_str = &input[..end];
        let rest = &input[end..];

        if is_float {
            let n: f64 = num_str
                .parse()
                .map_err(|e| format!("invalid float '{}': {}", num_str, e))?;
            Ok((FieldValue::Float(n), rest))
        } else {
            let n: i64 = num_str
                .parse()
                .map_err(|e| format!("invalid integer '{}': {}", num_str, e))?;
            Ok((FieldValue::Integer(n), rest))
        }
    }

    fn parse_json_object(input: &str) -> Result<Record, String> {
        let input = input.trim_start();
        if !input.starts_with('{') {
            return Err("expected '{'".to_string());
        }
        let mut input = &input[1..];
        let mut record = Record::new();

        loop {
            input = input.trim_start();
            if input.starts_with('}') {
                return Ok(record);
            }

            // Parse key
            let (key, rest) = Self::parse_json_string(input)?;
            input = rest.trim_start();

            // Expect colon
            if !input.starts_with(':') {
                return Err("expected ':'".to_string());
            }
            input = &input[1..];

            // Parse value
            let (value, rest) = Self::parse_json_value(input)?;
            record.fields.push((key, value));
            input = rest.trim_start();

            // Expect comma or closing brace
            if input.starts_with(',') {
                input = &input[1..];
            } else if input.starts_with('}') {
                return Ok(record);
            } else if input.is_empty() {
                return Err("unexpected end of JSON object".to_string());
            }
        }
    }
}

impl Serializer for JsonSerializer {
    fn serialize(&self, record: &Record) -> Result<Vec<u8>, SerializerError> {
        Ok(self.serialize_record(record).into_bytes())
    }

    fn deserialize(&self, data: &[u8]) -> Result<Record, SerializerError> {
        let input = std::str::from_utf8(data)
            .map_err(|e| self.err("deserialize", &format!("invalid UTF-8: {}", e)))?;
        Self::parse_json_object(input)
            .map_err(|e| self.err("deserialize", &e))
    }

    fn serialize_batch(&self, records: &[Record]) -> Result<Vec<u8>, SerializerError> {
        if self.pretty {
            let mut out = String::from("[\n");
            for (i, record) in records.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                let json = self.serialize_record(record);
                for (j, line) in json.lines().enumerate() {
                    if j > 0 {
                        out.push('\n');
                    }
                    out.push_str("  ");
                    out.push_str(line);
                }
            }
            out.push_str("\n]");
            Ok(out.into_bytes())
        } else {
            let mut out = String::from("[");
            for (i, record) in records.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&self.serialize_record(record));
            }
            out.push(']');
            Ok(out.into_bytes())
        }
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            supports_streaming: true,
            supports_schema: false,
            human_readable: true,
        }
    }

    fn format_name(&self) -> &str {
        if self.pretty {
            "json-pretty"
        } else {
            "json"
        }
    }

    fn content_type(&self) -> &str {
        "application/json"
    }
}

impl fmt::Display for JsonSerializer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JsonSerializer(pretty={})",
            self.pretty
        )
    }
}

// ---------------------------------------------------------------------------
// Binary Protocol Serializer
// ---------------------------------------------------------------------------

const BINARY_MAGIC: [u8; 4] = [0x42, 0x50, 0x01, 0x00];
const TAG_NULL: u8 = 0;
const TAG_TEXT: u8 = 1;
const TAG_INT64: u8 = 2;
const TAG_FLOAT64: u8 = 3;
const TAG_BOOL: u8 = 4;

struct BinarySerializer;

impl BinarySerializer {
    fn err(&self, op: &str, reason: &str) -> SerializerError {
        SerializerError {
            format: "binary".to_string(),
            operation: op.to_string(),
            reason: reason.to_string(),
        }
    }

    fn encode_record(&self, record: &Record) -> Vec<u8> {
        let mut buf = Vec::new();

        // Magic header
        buf.extend_from_slice(&BINARY_MAGIC);

        // Field count (u16 big-endian)
        buf.extend_from_slice(&(record.fields.len() as u16).to_be_bytes());

        for (key, value) in &record.fields {
            // Key: 1 byte length + key bytes
            let key_bytes = key.as_bytes();
            buf.push(key_bytes.len() as u8);
            buf.extend_from_slice(key_bytes);

            // Value: type tag + encoded value
            match value {
                FieldValue::Null => {
                    buf.push(TAG_NULL);
                }
                FieldValue::Text(s) => {
                    buf.push(TAG_TEXT);
                    let val_bytes = s.as_bytes();
                    buf.extend_from_slice(&(val_bytes.len() as u16).to_be_bytes());
                    buf.extend_from_slice(val_bytes);
                }
                FieldValue::Integer(n) => {
                    buf.push(TAG_INT64);
                    buf.extend_from_slice(&n.to_be_bytes());
                }
                FieldValue::Float(n) => {
                    buf.push(TAG_FLOAT64);
                    buf.extend_from_slice(&n.to_be_bytes());
                }
                FieldValue::Boolean(b) => {
                    buf.push(TAG_BOOL);
                    buf.push(if *b { 1 } else { 0 });
                }
            }
        }

        buf
    }

    fn decode_record(&self, data: &[u8]) -> Result<(Record, usize), SerializerError> {
        let mut pos = 0;

        // Check magic
        if data.len() < 6 {
            return Err(self.err("deserialize", "data too short for header"));
        }
        if &data[pos..pos + 4] != &BINARY_MAGIC {
            return Err(self.err("deserialize", "invalid magic header"));
        }
        pos += 4;

        // Field count
        let field_count = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        let mut record = Record::new();

        for _ in 0..field_count {
            // Key
            if pos >= data.len() {
                return Err(self.err("deserialize", "unexpected end of data reading key length"));
            }
            let key_len = data[pos] as usize;
            pos += 1;
            if pos + key_len > data.len() {
                return Err(self.err("deserialize", "unexpected end of data reading key"));
            }
            let key = std::str::from_utf8(&data[pos..pos + key_len])
                .map_err(|e| self.err("deserialize", &format!("invalid UTF-8 key: {}", e)))?
                .to_string();
            pos += key_len;

            // Type tag
            if pos >= data.len() {
                return Err(self.err("deserialize", "unexpected end of data reading type tag"));
            }
            let tag = data[pos];
            pos += 1;

            let value = match tag {
                TAG_NULL => FieldValue::Null,
                TAG_TEXT => {
                    if pos + 2 > data.len() {
                        return Err(self.err("deserialize", "unexpected end reading text length"));
                    }
                    let text_len =
                        u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                    pos += 2;
                    if pos + text_len > data.len() {
                        return Err(self.err("deserialize", "unexpected end reading text value"));
                    }
                    let text = std::str::from_utf8(&data[pos..pos + text_len])
                        .map_err(|e| self.err("deserialize", &format!("invalid UTF-8 text: {}", e)))?
                        .to_string();
                    pos += text_len;
                    FieldValue::Text(text)
                }
                TAG_INT64 => {
                    if pos + 8 > data.len() {
                        return Err(self.err("deserialize", "unexpected end reading int64"));
                    }
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[pos..pos + 8]);
                    pos += 8;
                    FieldValue::Integer(i64::from_be_bytes(bytes))
                }
                TAG_FLOAT64 => {
                    if pos + 8 > data.len() {
                        return Err(self.err("deserialize", "unexpected end reading float64"));
                    }
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[pos..pos + 8]);
                    pos += 8;
                    FieldValue::Float(f64::from_be_bytes(bytes))
                }
                TAG_BOOL => {
                    if pos >= data.len() {
                        return Err(self.err("deserialize", "unexpected end reading bool"));
                    }
                    let b = data[pos] != 0;
                    pos += 1;
                    FieldValue::Boolean(b)
                }
                _ => {
                    return Err(self.err(
                        "deserialize",
                        &format!("unknown type tag: {}", tag),
                    ));
                }
            };

            record.fields.push((key, value));
        }

        Ok((record, pos))
    }
}

impl Serializer for BinarySerializer {
    fn serialize(&self, record: &Record) -> Result<Vec<u8>, SerializerError> {
        Ok(self.encode_record(record))
    }

    fn deserialize(&self, data: &[u8]) -> Result<Record, SerializerError> {
        let (record, _) = self.decode_record(data)?;
        Ok(record)
    }

    fn serialize_batch(&self, records: &[Record]) -> Result<Vec<u8>, SerializerError> {
        let mut buf = Vec::new();
        // Record count header
        buf.extend_from_slice(&(records.len() as u16).to_be_bytes());
        for record in records {
            buf.extend_from_slice(&self.encode_record(record));
        }
        Ok(buf)
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            supports_streaming: true,
            supports_schema: true,
            human_readable: false,
        }
    }

    fn format_name(&self) -> &str {
        "binary"
    }

    fn content_type(&self) -> &str {
        "application/x-binary-proto"
    }
}

impl fmt::Display for BinarySerializer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BinarySerializer(TLV)")
    }
}

// ---------------------------------------------------------------------------
// MsgPack Serializer
// ---------------------------------------------------------------------------

struct MsgPackSerializer;

impl MsgPackSerializer {
    fn err(&self, op: &str, reason: &str) -> SerializerError {
        SerializerError {
            format: "msgpack".to_string(),
            operation: op.to_string(),
            reason: reason.to_string(),
        }
    }

    fn encode_string(buf: &mut Vec<u8>, s: &str) {
        let bytes = s.as_bytes();
        if bytes.len() < 32 {
            buf.push(0xa0 | (bytes.len() as u8));
        } else if bytes.len() < 256 {
            buf.push(0xd9);
            buf.push(bytes.len() as u8);
        } else {
            buf.push(0xda);
            buf.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        }
        buf.extend_from_slice(bytes);
    }

    fn encode_record(&self, record: &Record) -> Vec<u8> {
        let mut buf = Vec::new();
        let count = record.fields.len();

        // fixmap
        if count < 16 {
            buf.push(0x80 | (count as u8));
        } else {
            buf.push(0xde);
            buf.extend_from_slice(&(count as u16).to_be_bytes());
        }

        for (key, value) in &record.fields {
            Self::encode_string(&mut buf, key);

            match value {
                FieldValue::Null => buf.push(0xc0),
                FieldValue::Boolean(true) => buf.push(0xc3),
                FieldValue::Boolean(false) => buf.push(0xc2),
                FieldValue::Text(s) => Self::encode_string(&mut buf, s),
                FieldValue::Integer(n) => {
                    if *n >= 0 && *n < 128 {
                        buf.push(*n as u8);
                    } else if *n >= -32 && *n < 0 {
                        buf.push(*n as u8); // negative fixint
                    } else {
                        buf.push(0xd3); // int64
                        buf.extend_from_slice(&n.to_be_bytes());
                    }
                }
                FieldValue::Float(n) => {
                    buf.push(0xcb); // float64
                    buf.extend_from_slice(&n.to_be_bytes());
                }
            }
        }

        buf
    }

    fn decode_string<'a>(&self, data: &'a [u8], pos: &mut usize) -> Result<String, SerializerError> {
        if *pos >= data.len() {
            return Err(self.err("deserialize", "unexpected end reading string marker"));
        }
        let marker = data[*pos];
        *pos += 1;

        let len = if marker & 0xe0 == 0xa0 {
            (marker & 0x1f) as usize
        } else if marker == 0xd9 {
            if *pos >= data.len() {
                return Err(self.err("deserialize", "unexpected end reading str8 length"));
            }
            let l = data[*pos] as usize;
            *pos += 1;
            l
        } else if marker == 0xda {
            if *pos + 2 > data.len() {
                return Err(self.err("deserialize", "unexpected end reading str16 length"));
            }
            let l = u16::from_be_bytes([data[*pos], data[*pos + 1]]) as usize;
            *pos += 2;
            l
        } else {
            return Err(self.err(
                "deserialize",
                &format!("expected string marker, got 0x{:02x}", marker),
            ));
        };

        if *pos + len > data.len() {
            return Err(self.err("deserialize", "unexpected end reading string data"));
        }
        let s = std::str::from_utf8(&data[*pos..*pos + len])
            .map_err(|e| self.err("deserialize", &format!("invalid UTF-8: {}", e)))?
            .to_string();
        *pos += len;
        Ok(s)
    }

    fn decode_value(&self, data: &[u8], pos: &mut usize) -> Result<FieldValue, SerializerError> {
        if *pos >= data.len() {
            return Err(self.err("deserialize", "unexpected end reading value marker"));
        }
        let marker = data[*pos];

        // Check string markers first (fixstr, str8, str16)
        if marker & 0xe0 == 0xa0 || marker == 0xd9 || marker == 0xda {
            let s = self.decode_string(data, pos)?;
            return Ok(FieldValue::Text(s));
        }

        *pos += 1;

        match marker {
            0xc0 => Ok(FieldValue::Null),
            0xc2 => Ok(FieldValue::Boolean(false)),
            0xc3 => Ok(FieldValue::Boolean(true)),
            0xcb => {
                // float64
                if *pos + 8 > data.len() {
                    return Err(self.err("deserialize", "unexpected end reading float64"));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[*pos..*pos + 8]);
                *pos += 8;
                Ok(FieldValue::Float(f64::from_be_bytes(bytes)))
            }
            0xd3 => {
                // int64
                if *pos + 8 > data.len() {
                    return Err(self.err("deserialize", "unexpected end reading int64"));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[*pos..*pos + 8]);
                *pos += 8;
                Ok(FieldValue::Integer(i64::from_be_bytes(bytes)))
            }
            b if b < 0x80 => {
                // positive fixint
                Ok(FieldValue::Integer(b as i64))
            }
            b if b >= 0xe0 => {
                // negative fixint
                Ok(FieldValue::Integer(b as i8 as i64))
            }
            _ => Err(self.err(
                "deserialize",
                &format!("unsupported msgpack marker: 0x{:02x}", marker),
            )),
        }
    }

    fn decode_record(&self, data: &[u8], pos: &mut usize) -> Result<Record, SerializerError> {
        if *pos >= data.len() {
            return Err(self.err("deserialize", "unexpected end reading map marker"));
        }
        let marker = data[*pos];
        *pos += 1;

        let count = if marker & 0xf0 == 0x80 {
            (marker & 0x0f) as usize
        } else if marker == 0xde {
            if *pos + 2 > data.len() {
                return Err(self.err("deserialize", "unexpected end reading map16 count"));
            }
            let c = u16::from_be_bytes([data[*pos], data[*pos + 1]]) as usize;
            *pos += 2;
            c
        } else {
            return Err(self.err(
                "deserialize",
                &format!("expected map marker, got 0x{:02x}", marker),
            ));
        };

        let mut record = Record::new();
        for _ in 0..count {
            let key = self.decode_string(data, pos)?;
            let value = self.decode_value(data, pos)?;
            record.fields.push((key, value));
        }

        Ok(record)
    }
}

impl Serializer for MsgPackSerializer {
    fn serialize(&self, record: &Record) -> Result<Vec<u8>, SerializerError> {
        Ok(self.encode_record(record))
    }

    fn deserialize(&self, data: &[u8]) -> Result<Record, SerializerError> {
        let mut pos = 0;
        self.decode_record(data, &mut pos)
    }

    fn serialize_batch(&self, records: &[Record]) -> Result<Vec<u8>, SerializerError> {
        let mut buf = Vec::new();
        let count = records.len();
        if count < 16 {
            buf.push(0x90 | (count as u8));
        } else {
            buf.push(0xdc);
            buf.extend_from_slice(&(count as u16).to_be_bytes());
        }
        for record in records {
            buf.extend_from_slice(&self.encode_record(record));
        }
        Ok(buf)
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            supports_streaming: true,
            supports_schema: false,
            human_readable: false,
        }
    }

    fn format_name(&self) -> &str {
        "msgpack"
    }

    fn content_type(&self) -> &str {
        "application/x-msgpack"
    }
}

impl fmt::Display for MsgPackSerializer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MsgPackSerializer")
    }
}

// ---------------------------------------------------------------------------
// CSV Serializer
// ---------------------------------------------------------------------------

struct CsvSerializer {
    delimiter: char,
    include_header: bool,
}

impl CsvSerializer {
    fn err(&self, op: &str, reason: &str) -> SerializerError {
        SerializerError {
            format: self.format_name().to_string(),
            operation: op.to_string(),
            reason: reason.to_string(),
        }
    }

    fn escape_field(&self, value: &str) -> String {
        if value.contains(self.delimiter) || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }

    fn serialize_row(&self, record: &Record) -> String {
        let delim = self.delimiter.to_string();
        record
            .fields
            .iter()
            .map(|(_, v)| self.escape_field(&v.to_string()))
            .collect::<Vec<_>>()
            .join(&delim)
    }

    fn serialize_header(&self, record: &Record) -> String {
        let delim = self.delimiter.to_string();
        record
            .fields
            .iter()
            .map(|(k, _)| self.escape_field(k))
            .collect::<Vec<_>>()
            .join(&delim)
    }

    fn parse_csv_line<'a>(&self, line: &'a str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            if in_quotes {
                if c == '"' {
                    if chars.peek() == Some(&'"') {
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    current.push(c);
                }
            } else if c == '"' {
                in_quotes = true;
            } else if c == self.delimiter {
                fields.push(current.clone());
                current.clear();
            } else {
                current.push(c);
            }
        }
        fields.push(current);
        fields
    }
}

impl Serializer for CsvSerializer {
    fn serialize(&self, record: &Record) -> Result<Vec<u8>, SerializerError> {
        let mut out = String::new();
        if self.include_header {
            out.push_str(&self.serialize_header(record));
            out.push('\n');
        }
        out.push_str(&self.serialize_row(record));
        Ok(out.into_bytes())
    }

    fn deserialize(&self, data: &[u8]) -> Result<Record, SerializerError> {
        let input = std::str::from_utf8(data)
            .map_err(|e| self.err("deserialize", &format!("invalid UTF-8: {}", e)))?;

        let lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return Ok(Record::new());
        }

        if lines.len() < 2 && self.include_header {
            return Err(self.err("deserialize", "CSV has header but no data row"));
        }

        let headers = self.parse_csv_line(lines[0]);
        let values = if lines.len() > 1 {
            self.parse_csv_line(lines[1])
        } else {
            self.parse_csv_line(lines[0])
        };

        let mut record = Record::new();
        for (i, header) in headers.iter().enumerate() {
            let value = values
                .get(i)
                .cloned()
                .unwrap_or_default();
            record
                .fields
                .push((header.clone(), FieldValue::Text(value)));
        }

        Ok(record)
    }

    fn serialize_batch(&self, records: &[Record]) -> Result<Vec<u8>, SerializerError> {
        let mut out = String::new();
        for (i, record) in records.iter().enumerate() {
            if i == 0 && self.include_header {
                out.push_str(&self.serialize_header(record));
                out.push('\n');
            }
            out.push_str(&self.serialize_row(record));
            out.push('\n');
        }
        // Remove trailing newline for consistency
        if out.ends_with('\n') {
            out.pop();
        }
        Ok(out.into_bytes())
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            supports_streaming: false,
            supports_schema: false,
            human_readable: true,
        }
    }

    fn format_name(&self) -> &str {
        if self.delimiter == '\t' {
            "tsv"
        } else {
            "csv"
        }
    }

    fn content_type(&self) -> &str {
        "text/csv"
    }
}

impl fmt::Display for CsvSerializer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let delim_name = match self.delimiter {
            ',' => "comma",
            '\t' => "tab",
            c => return write!(f, "CsvSerializer(delim='{}')", c),
        };
        write!(
            f,
            "CsvSerializer(delim={}, header={})",
            delim_name, self.include_header
        )
    }
}

// ---------------------------------------------------------------------------
// Factory function
// ---------------------------------------------------------------------------

pub fn create_serializer(config: &FormatConfig) -> Result<Box<dyn Serializer>, SerializerError> {
    match config.format.as_str() {
        "json" => {
            let pretty = config.get_option("pretty").map(|v| v == "true").unwrap_or(false);
            Ok(Box::new(JsonSerializer { pretty }))
        }
        "json-pretty" => Ok(Box::new(JsonSerializer { pretty: true })),
        "binary" | "proto" => Ok(Box::new(BinarySerializer)),
        "msgpack" => Ok(Box::new(MsgPackSerializer)),
        "csv" => {
            let delimiter = config
                .get_option("delimiter")
                .and_then(|d| d.chars().next())
                .unwrap_or(',');
            let include_header = config
                .get_option("header")
                .map(|v| v != "false")
                .unwrap_or(true);
            Ok(Box::new(CsvSerializer {
                delimiter,
                include_header,
            }))
        }
        "tsv" => Ok(Box::new(CsvSerializer {
            delimiter: '\t',
            include_header: true,
        })),
        other => Err(SerializerError {
            format: other.to_string(),
            operation: "create".to_string(),
            reason: format!(
                "unknown format '{}'. Available: json, json-pretty, binary, proto, msgpack, csv, tsv",
                other
            ),
        }),
    }
}

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

        match create_serializer(&config) {
            Ok(serializer) => {
                println!("  Format: {} ({})", serializer.format_name(), serializer);
                println!("  Content-Type: {}", serializer.content_type());
                println!("  Capabilities: {:?}", serializer.capabilities());

                match serializer.serialize(&record) {
                    Ok(bytes) => {
                        println!("  Serialized: {} bytes", bytes.len());
                        if serializer.capabilities().human_readable {
                            println!("  Output:\n{}", String::from_utf8_lossy(&bytes));
                        } else {
                            println!(
                                "  Hex: {}",
                                bytes
                                    .iter()
                                    .take(40)
                                    .map(|b| format!("{:02x}", b))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            );
                        }

                        // Round-trip
                        match serializer.deserialize(&bytes) {
                            Ok(decoded) => {
                                println!("  Round-trip: {} fields decoded", decoded.fields.len());
                                for (k, v) in &decoded.fields {
                                    println!("    {} = {:?}", k, v);
                                }
                            }
                            Err(e) => println!("  Deserialize error: {}", e),
                        }
                    }
                    Err(e) => println!("  Serialize error: {}", e),
                }
            }
            Err(e) => println!("  Factory error: {}", e),
        }
        println!();
    }

    // Batch demo
    println!("=== Batch Serialization ===\n");
    let batch = vec![
        Record::new()
            .field("id", FieldValue::Integer(1))
            .field("name", FieldValue::Text("alice".to_string())),
        Record::new()
            .field("id", FieldValue::Integer(2))
            .field("name", FieldValue::Text("bob".to_string())),
        Record::new()
            .field("id", FieldValue::Integer(3))
            .field("name", FieldValue::Text("charlie".to_string())),
    ];

    for format in &["json-pretty", "csv", "binary"] {
        let s = create_serializer(&FormatConfig::new(format)).unwrap();
        match s.serialize_batch(&batch) {
            Ok(bytes) => {
                println!("--- {} batch ({} bytes) ---", format, bytes.len());
                if s.capabilities().human_readable {
                    println!("{}", String::from_utf8_lossy(&bytes));
                }
            }
            Err(e) => println!("Batch error: {}", e),
        }
        println!();
    }
}
