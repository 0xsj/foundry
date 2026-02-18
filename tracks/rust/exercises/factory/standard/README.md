# Standard Exercise: Message Serialization Pipeline Factory

## Scenario

Your team is building a message broker that accepts events from internal services and forwards them to external consumers. Each consumer has negotiated a specific wire format (JSON, Protocol Buffers-like binary, MessagePack-like binary, CSV). The broker must serialize each event into the consumer's preferred format before delivery. Some formats support schema validation, some support streaming output, and the set of formats may grow over time.

## Brief

Implement a serialization pipeline factory that:
1. Defines a `Serializer` trait with serialize/deserialize capabilities
2. Provides a factory function that creates the appropriate serializer from a format configuration
3. Supports at least four formats: JSON, a binary protocol (Protobuf-like), a compact binary (MessagePack-like), and CSV
4. Tracks serializer capabilities (streaming support, schema validation) via a `Capabilities` struct
5. Handles batch serialization for multiple records

## Acceptance Criteria

- [ ] `Serializer` trait is object-safe and can be used as `Box<dyn Serializer>`
- [ ] `create_serializer(config: &FormatConfig) -> Result<Box<dyn Serializer>, SerializerError>` factory function works
- [ ] JSON serializer handles nested-like data (key-value pairs with typed values)
- [ ] Binary serializer produces compact output with a simple encoding scheme
- [ ] CSV serializer handles quoting/escaping for values containing delimiters
- [ ] Each serializer reports its capabilities via `fn capabilities(&self) -> Capabilities`
- [ ] `Capabilities` struct has `supports_streaming: bool` and `supports_schema: bool` fields
- [ ] Batch serialization works: `fn serialize_batch(&self, records: &[Record]) -> Result<Vec<u8>, SerializerError>`
- [ ] Round-trip works: `deserialize(serialize(record))` produces equivalent data
- [ ] All tests pass: `rustc --test tests.rs && ./tests`

## Constraints

- Do not use any external crates -- pure `std` only
- The `Serializer` trait must remain object-safe (no generic methods)
- Each serializer must implement `Display` for logging
- Error types must provide context about which format failed and why
- CSV must handle the edge case of values containing the delimiter character

## Files

- `starter/main.rs` -- Scaffold with traits, types, and TODOs
- `starter/tests.rs` -- Full test suite (run with `rustc --test tests.rs && ./tests`)

## Getting Started

```bash
cd starter
# Review the scaffold
cat main.rs

# Run tests (will fail initially)
rustc --test tests.rs && ./tests
```

## Hints

<details>
<summary>Hint 1: Trait design</summary>

Start with the `Serializer` trait. Make sure every method uses `&self` or `&mut self` as receiver. For the data parameter, use `&[u8]` or a concrete `Record` type -- avoid generics on the trait methods if you want object safety.

</details>

<details>
<summary>Hint 2: Factory function structure</summary>

The factory function should match on the format config's `format` field and construct the appropriate variant. Return `Box::new(ConcreteSerializer { ... })` for each case. Use `Result` to handle unknown formats.

</details>

<details>
<summary>Hint 3: Binary encoding</summary>

For the binary formats, use a simple tag-length-value encoding:
- 1 byte for the field type tag (0=null, 1=text, 2=int, 3=float, 4=bool)
- 2 bytes for value length (big-endian u16)
- N bytes for the value

This keeps the implementation simple without external crates.

</details>

<details>
<summary>Hint 4: Round-trip testing</summary>

For deserialization, you need to parse the exact format your serializer produces. The JSON serializer should parse its own JSON output. The binary serializer should decode its own TLV encoding. Keep both sides consistent.

</details>

## Solution

After completing your implementation, compare with `solutions/` directory.
