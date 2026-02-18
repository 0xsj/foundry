# Solution Notes: Webhook Payload Handler

## Approach

The solution models the problem in three layers:

1. **Per-provider types** (`GitHubEvent`, `StripeEvent`, `SlackEvent`) — the typed representation of each provider's JSON shape. These are what serde's `#[derive(Deserialize)]` would produce automatically.

2. **`ProviderPayload` enum** — a tagged sum of all provider types. The `parse_provider_payload` function routes to the right parser and wraps the result in the right variant.

3. **`InternalEvent`** — the normalized format. `into_internal` consumes the typed payload and converts it to a single uniform shape for downstream code.

## Key Decisions

**Required vs optional fields are explicit.** Each required field returns early with a descriptive error via `?`. Optional fields are `Option<T>` — absence is fine. Default fields use `.unwrap_or_else(...)`. This maps directly to serde's `#[serde(default)]` attribute.

**Semantic validation happens immediately.** The Stripe ID prefix check (`starts_with("evt_")`) runs right after the field is extracted, before the struct is constructed. This is the "parse, don't validate" principle — the validity constraint is checked at the boundary, not scattered through business logic.

**`into_internal` is a method on `ProviderPayload`, not a free function.** This lets each variant's conversion logic live in one place (the match arm) rather than spread across helper functions. Adding a new provider means adding a new variant and a new match arm — you can't forget one without the compiler complaining.

**`metadata` is `Vec<(String, String)>`** rather than a `HashMap`. Order is preserved (important for debugging), and the structure stays simple. In real code you'd likely use `HashMap<String, String>` or `serde_json::Value` for truly dynamic metadata.

## Comparison to Real serde

| Manual (this solution) | With serde |
|------------------------|------------|
| `get_str(json, "id")?` | `#[derive(Deserialize)]` + `serde_json::from_str(json)?` |
| `get_str(json, "action").unwrap_or_else(...)` | `#[serde(default = "fn_name")]` |
| `get_u64(json, "number")` → `Option<u64>` | `number: Option<u64>` — serde handles None automatically |
| `get_str(json, "type")` with `rename` | `#[serde(rename = "type")] event_type: String` |
| Manual prefix check on `id` | Still manual — serde handles structure, not semantics |

## Variants

See `variants/` for:

- **`functional.rs`** — uses `and_then` chaining for required field extraction

## Performance Notes

- This solution does O(n) string scanning for each field — not production-grade
- Real serde parses the JSON once into a token stream, visiting each field in one pass
- For high-throughput webhook handlers: use serde + zero-copy deserialization for path/id fields
- Validation is still synchronous and cheap — no concern there

## Concepts Demonstrated

- [[fundamentals/serialization]] — the parse → validate two-step pattern
- [[fundamentals/rust/structs-methods-and-enums]] — enum dispatch, struct variants
- [[patterns/repository]] — the internal event type as a domain boundary
