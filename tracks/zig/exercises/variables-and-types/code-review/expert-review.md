# Expert Review: Feature Flag System

## Critical Issues

### 1. Non-exhaustive switch with `else => unreachable` on tagged union

**Location:** `getFlagDisplayValue`, line ~82; `isFlagEnabled`, line ~73

**The Problem:**

In `getFlagDisplayValue`, the switch on `flag.value` (a `FlagValue` union) uses `else => unreachable` as a catch-all for the `.variant_name` case:

```zig
pub fn getFlagDisplayValue(flag: FeatureFlag) []const u8 {
    return switch (flag.value) {
        .enabled => |val| if (val) "enabled" else "disabled",
        .rollout_percentage => |pct| {
            _ = pct;
            return "percentage rollout";
        },
        else => unreachable,  // <-- if someone adds a new variant, this CRASHES
    };
}
```

Similarly, `isFlagEnabled` uses `else => return true` for the variant case:

```zig
else => return true,  // catches .variant_name, but also any future variant
```

The whole point of Zig's exhaustive switch on tagged unions is **compile-time safety**. When you add a new variant to `FlagValue`, the compiler tells you every switch that needs updating. Using `else` defeats this: the code silently compiles, and then either crashes (`unreachable`) or silently does the wrong thing (`return true`) at runtime.

**The Fix:**

List all variants explicitly:

```zig
pub fn getFlagDisplayValue(flag: FeatureFlag) []const u8 {
    return switch (flag.value) {
        .enabled => |val| if (val) "enabled" else "disabled",
        .rollout_percentage => "percentage rollout",
        .variant_name => |name| name,
    };
}
```

```zig
// In isFlagEnabled:
.variant_name => return true,
```

**Concept connection:** This is the primary benefit of tagged unions over "type" string fields or separate boolean flags. If you use `else`, you're throwing away the compiler's help.

---

### 2. Unsafe optional unwrap with `.?` without null check

**Location:** `getFlagDescription`, line ~88

**The Problem:**

```zig
pub fn getFlagDescription(flag: FeatureFlag) []const u8 {
    return flag.description.?;  // panics if description is null
}
```

The `.?` operator on an optional is an unconditional unwrap. If `flag.description` is `null`, this triggers a panic at runtime (in safe mode) or undefined behavior (in release mode). This is equivalent to force-unwrapping in Swift (`!`) or calling `.unwrap()` in Rust -- it should almost never appear outside of tests.

The function signature gives no indication that it can fail. A caller would reasonably expect `getFlagDescription` to always return a valid string.

**The Fix:**

Either return an optional or provide a default:

```zig
// Option A: Return optional, let caller decide
pub fn getFlagDescription(flag: FeatureFlag) ?[]const u8 {
    return flag.description;
}

// Option B: Provide a default with orelse
pub fn getFlagDescription(flag: FeatureFlag) []const u8 {
    return flag.description orelse "No description available";
}

// Option C: Use if-capture for complex logic
pub fn getFlagDescription(flag: FeatureFlag) []const u8 {
    if (flag.description) |desc| {
        return desc;
    }
    return "No description available";
}
```

**Concept connection:** Zig's optional types exist to make null handling explicit. Using `.?` bypasses that safety and reintroduces null pointer errors. The `orelse` keyword and `if (opt) |val|` capture syntax are the idiomatic tools.

---

## Major Concerns

### 3. `@intCast` without bounds validation

**Location:** `isFlagEnabled`, lines ~70-71

**The Problem:**

```zig
const bucket: u8 = @intCast(user_id % 100);        // this is fine (0-99 fits in u8)
return bucket < @as(u8, @intCast(pct));             // BUG: pct is u64, could be > 255
```

The first `@intCast` is actually safe because `user_id % 100` is always in range 0-99, which fits in `u8`. However, the second `@intCast` converts `pct` (a `u64` from the `rollout_percentage` field) to `u8`. If someone creates a flag with `percentage = 300` (a logic error, but possible), this `@intCast` will panic in safe mode or produce undefined behavior in release mode.

The deeper issue: why is `rollout_percentage` stored as `u64`? A percentage is 0-100. The type should encode this constraint.

**The Fix:**

```zig
// Option A: Validate at creation time and use a bounded type
pub const FlagValue = union(enum) {
    enabled: bool,
    rollout_percentage: u7,       // 0-127, enough for 0-100
    variant_name: []const u8,
};

// Option B: Clamp at comparison time
.rollout_percentage => |pct| {
    const clamped_pct = @min(pct, 100);
    const bucket = user_id % 100;
    return bucket < clamped_pct;
},
```

Option B also removes the need for any `@intCast` by keeping everything as `u64`.

**Concept connection:** Zig's `@intCast` is a signal that you're performing a potentially lossy conversion. When you see it, ask: "Can the source value exceed the target type's range?" If yes, validate first. Better yet, choose types that make invalid states unrepresentable.

---

### 4. Redundant `FlagType` enum duplicates the tagged union's tag

**Location:** `FlagType` enum (line ~11), `FeatureFlag.flag_type` field (line ~28)

**The Problem:**

```zig
pub const FlagType = enum {
    boolean,
    percentage,
    variant,
};

pub const FeatureFlag = struct {
    name: []const u8,
    flag_type: FlagType,       // redundant — FlagValue already carries this info
    value: FlagValue,
    // ...
};
```

`FlagValue` is a tagged union -- its active variant *is* the flag type. The separate `FlagType` enum duplicates this information, which means:

1. **Desynchronization risk:** Nothing prevents `flag_type = .boolean` paired with `value = .{ .rollout_percentage = 50 }`. The two fields can disagree, creating an invalid state.
2. **Maintenance burden:** Every new flag type requires updating both `FlagType` and `FlagValue`, and every `create*` function must manually keep them in sync.
3. **No code uses `flag_type`:** Looking at the codebase, every function switches on `flag.value` (the tagged union), not `flag.flag_type`. The enum exists but serves no purpose.

This violates the "make invalid states unrepresentable" principle. If there's only one source of truth for the flag's type (the tagged union), it's impossible for them to disagree.

**The Fix:**

Remove `FlagType` and the `flag_type` field entirely:

```zig
pub const FeatureFlag = struct {
    name: []const u8,
    value: FlagValue,          // the tag IS the type
    description: ?[]const u8,
    is_active: bool,
};
```

If you need to inspect the type without the value, use `std.meta.activeTag`:

```zig
const tag = std.meta.activeTag(flag.value);
if (tag == .rollout_percentage) { ... }
```

**Concept connection:** Tagged unions in Zig (and Rust enums, and TypeScript discriminated unions) are specifically designed to bundle "what kind" with "what data." Adding a parallel enum undermines this. The tagged union's tag is the single source of truth for the variant -- trust it.

---

## Minor Suggestions

### 5. Pointer comparison instead of content comparison for strings

**Location:** `isSameFlag`, line ~101

**The Problem:**

```zig
pub fn isSameFlag(a: FeatureFlag, b: FeatureFlag) bool {
    return a.name.ptr == b.name.ptr and a.name.len == b.name.len;
}
```

This compares the **pointer address** and **length** of the two name slices, not their **content**. Two slices with identical content but different memory addresses will compare as not equal. This happens whenever strings come from different sources (different allocations, different string literals in different compilation units, runtime-constructed strings, etc.).

For string literals in the same compilation unit, pointer comparison might happen to work because the compiler deduplicates them into the same address. This makes the bug particularly insidious -- it passes tests but fails in production when names come from different sources (e.g., user input, config files, different allocations).

Note: In modern Zig (0.12+), `==` on slices is a compile error, which is why the author manually compared `.ptr` and `.len`. But the intent was clearly to compare content, and the manual pointer comparison is the wrong approach.

**The Fix:**

```zig
pub fn isSameFlag(a: FeatureFlag, b: FeatureFlag) bool {
    return std.mem.eql(u8, a.name, b.name);
}
```

`std.mem.eql` compares the actual byte content of the two slices, regardless of where they're stored in memory.

**Concept connection:** Zig strings are just `[]const u8` -- a pointer + length pair. There's no built-in string type with content equality. You must always use `std.mem.eql` (or `std.mem.order` for ordering) to compare string content. This is easy to forget, especially coming from languages where `==` on strings "just works."

**Cross-language comparison:**
| Language | String equality operator | Content comparison function |
|----------|------------------------|-----------------------------|
| Zig | N/A (compile error on slices) | `std.mem.eql(u8, a, b)` |
| Go | `==` compares content | `==` (built-in) |
| Rust | `==` compares content | `==` (via PartialEq trait) |
| Java | `==` compares reference | `.equals()` method |
| JavaScript | `===` compares content | `===` (for primitives) |

---

### 6. Magic numbers for percentage bounds

**Location:** `isFullRollout` (line ~104), `hasRollout` (line ~112), `isFlagEnabled` (line ~70)

**The Problem:**

```zig
return pct >= 100;   // what does 100 mean?
return pct > 0;      // and 0?
const bucket: u8 = @intCast(user_id % 100);  // why 100?
```

The number `100` appears in three places with the same meaning (percentage scale), and `0` appears as the minimum rollout. These are magic numbers -- they encode domain knowledge without naming it.

**The Fix:**

Define named constants:

```zig
const max_percentage: u64 = 100;
const min_percentage: u64 = 0;
const percentage_buckets: u64 = 100;

// Then:
return pct >= max_percentage;
return pct > min_percentage;
const bucket = user_id % percentage_buckets;
```

**Concept connection:** Named constants make code self-documenting and reduce the risk of inconsistent changes. If the team decides to support fractional percentages (0-10000 for 0.00%-100.00%), you'd change one constant instead of hunting through the codebase for `100`.

---

## Positive Feedback

- **Good use of tagged union for flag values.** Modeling the three flag types as a `FlagValue` union is the right design. It ensures type safety and forces handling of all cases (when switches are exhaustive).

- **Optional type for description.** Using `?[]const u8` for the description field correctly models "may or may not be present" without sentinel values or empty strings.

- **Buffer-based formatting.** The `formatFlag` function takes a caller-provided buffer instead of allocating. This is idiomatic Zig -- the caller controls memory, the function just writes into it.

- **Deterministic percentage evaluation.** Using `user_id % 100` for percentage flags gives deterministic, reproducible results for any given user. This is better than random number generation for feature flags because the same user always gets the same result.

- **Inactive flag short-circuit.** Checking `is_active` first in `isFlagEnabled` is clean and handles the global kill-switch case efficiently.

- **Clear function naming.** Function names like `createBooleanFlag`, `isFlagEnabled`, `isFullRollout` are descriptive and follow Zig's naming conventions (camelCase for functions).

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `else => unreachable` on tagged union switch | Exhaustive switches |
| 2 | Critical | `.?` unwrap without null check | Optional safety |
| 3 | Major | `@intCast` on unbounded u64 percentage | Explicit casting, bounds |
| 4 | Major | Redundant `FlagType` enum duplicates tagged union tag | Make invalid states unrepresentable |
| 5 | Minor | Pointer comparison instead of content comparison for names | Slice semantics |
| 6 | Minor | Magic numbers 100 and 0 for percentages | Named constants |

## Related Concepts

- [[fundamentals/variables-and-types]] — const/var, optionals, tagged unions, explicit casting
- [[fundamentals/zig/variables-and-types]] — Zig-specific deep dive on these topics
- [[pitfalls/zig-slice-equality]] — Slice == compares ptr+len, not content
- [[pitfalls/zig-unsafe-optional-unwrap]] — When .? is dangerous
- [[rosetta/common-operations#string-equality]] — How string comparison works across languages
