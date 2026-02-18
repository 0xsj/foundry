# Solution: Delivery Channel Manager

## Bug 1: Value Receiver on `Disable` (update doesn't persist)

**Location:** `Disable` method, line `func (m ChannelManager) Disable(...)`

**What:** `Disable` has a **value receiver** (`m ChannelManager`), not a pointer receiver (`m *ChannelManager`). In Go, a value receiver receives a copy of the struct. Any mutations to `m` inside the method — including `m.channels[name] = cfg` and `m.lastUpdated = name` — happen on that copy. When the method returns, the copy is discarded. The original `ChannelManager` is unchanged.

This is subtle: `m.channels[name] = cfg` actually does modify the map (maps are reference types — the map header is copied but it points to the same underlying data). So `cfg.Enabled = false` *does* persist. However, `m.lastUpdated = name` is a field on the struct itself — that change is lost.

The `TestLastUpdatedTracksCorrectChannel` test fails because `m.lastUpdated` is never updated. The `TestDisablePersists` test actually passes (map modification persists through value receivers), but the tracking state is still broken.

**Fix:** Change the receiver to a pointer:

```go
// Before
func (m ChannelManager) Disable(name string) error {

// After
func (m *ChannelManager) Disable(name string) error {
```

**Why it happens:** Value receivers are a conscious design in Go — methods that only read benefit from not needing a pointer (can call them on non-addressable values, thread-safe if struct is immutable). But any method that mutates the struct's own fields must use a pointer receiver.

**How to avoid:** If any method on a type uses a pointer receiver, make all methods use pointer receivers. This is idiomatic Go (see Effective Go). Mixing receivers leads to confusing behavior like this.

**Related:** [[go-value-receiver-mutation-pitfall]]

---

## Bug 2: Nil Embedded Pointer — `*PingClient` never initialized (panic)

**Location:** `NewChannelManager()` — the `*PingClient` embedded field is never set.

**What:** `ChannelManager` embeds `*PingClient` (a pointer). The zero value of any pointer is `nil`. `NewChannelManager` initializes `channels` with `make()` but leaves `PingClient` as nil:

```go
return &ChannelManager{
    // PingClient is nil — omitted from the literal
    channels: make(map[string]ChannelConfig),
}
```

When `PingChannel` is called, it calls `m.Ping(name)`. The promoted `Ping` method has receiver `*PingClient`, so Go dereferences `m.PingClient` — which is nil. Nil pointer dereference → panic.

**Fix:** Initialize `PingClient` in the constructor:

```go
return &ChannelManager{
    PingClient: &PingClient{},    // explicitly initialize
    channels:   make(map[string]ChannelConfig),
}
```

**Why it happens:** Embedding by pointer (`*T`) vs by value (`T`) is a meaningful choice. Embedding by value gives you the zero value of the embedded struct — always valid. Embedding by pointer gives you `nil` by default — you must initialize it before calling any method through it. Beginners often choose pointer embedding without thinking about the nil risk.

**How to avoid:** If the embedded type's zero value is valid and the embedded struct should always exist, embed by value. If initialization is optional or deferred, embed by pointer but document that it must be set before use — and check for nil where appropriate.

**Related:** [[go-nil-embedded-pointer-panic]]

---

## Bug 3: Write-back to Wrong Map Key in `UpdateTimeout` (wrong channel modified)

**Location:** `UpdateTimeout`, the line `m.channels[m.lastUpdated] = cfg`

**What:** The method reads the config for `name`, modifies `cfg.TimeoutMs`, then writes it back. But the write-back uses `m.lastUpdated` as the key instead of `name`:

```go
cfg, ok := m.channels[name]      // read from "sms"
cfg.TimeoutMs = timeoutMs        // modify the copy
m.channels[m.lastUpdated] = cfg  // BUG: write to "" or whatever m.lastUpdated is
m.lastUpdated = name             // update tracking (too late)
```

On the first call, `m.lastUpdated` is `""` — an empty string. The modified config is written to `m.channels[""]` (creating a new map entry with an empty key). The "sms" entry in the map is untouched.

This bug is compounded by Bug 1: if `Disable` worked correctly, `m.lastUpdated` would be set. But since `Disable`'s value-receiver update to `m.lastUpdated` is lost, `m.lastUpdated` is always stale — so `UpdateTimeout` consistently overwrites the wrong entry.

**Fix:** Write back to `name`:

```go
cfg, ok := m.channels[name]
if !ok {
    return fmt.Errorf("channel %q not found", name)
}
cfg.TimeoutMs = timeoutMs
m.channels[name] = cfg        // correct: write back to the key we read from
m.lastUpdated = name
return nil
```

**Why it happens:** This is a classic read-modify-writeback pattern in Go. Because map values are not addressable, you must copy the value out, modify the copy, and write it back. The write-back key must match the read key. Using any other key creates a different map entry.

**How to avoid:** When doing the read-modify-writeback pattern, use the same key variable for both the read and the write. Avoid "clever" indirection through a tracking field — it creates a dependency between the write and the state of other operations.

---

## Summary

| Bug | Symptom | Root Cause | Fix |
|-----|---------|-----------|-----|
| Value receiver on `Disable` | `lastUpdated` not updated | `m ChannelManager` receiver copies the struct | Change to `m *ChannelManager` |
| `*PingClient` not initialized | Panic on `PingChannel` | Nil embedded pointer — zero value of `*T` is nil | Initialize `PingClient: &PingClient{}` in constructor |
| Write-back to wrong map key | Wrong channel's timeout updated | `m.channels[m.lastUpdated]` instead of `m.channels[name]` | Write back to `name` |

All three bugs stem from structs-methods-and-enums fundamentals:
- Bug 1: pointer vs value receivers
- Bug 2: zero value of pointer types in embedding
- Bug 3: map value semantics and the read-modify-writeback pattern
