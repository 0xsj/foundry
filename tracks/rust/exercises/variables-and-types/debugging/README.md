# Debugging Exercise: Config Parser

## Context

A teammate wrote a config parser that reads key-value pairs from a string-based
input, validates them, and produces a typed `Config` struct. The code compiles
without errors, but produces incorrect results at runtime.

Your job: find the bugs, explain why they happen, and fix them.

## How to Run

```bash
# Run the tests (6 will fail across 3 root causes)
rustc --test buggy.rs && ./buggy

# After fixing all 3 bugs, all tests should pass
```

## Symptoms

### Symptom 1: Defaults not applied

When creating a config with an empty host and port 0, the `apply_defaults`
function should fill in `"localhost"` and `8080`. But calling `apply_defaults`
returns a config that still has port 0 and an empty host.

**Failing tests:** `test_apply_defaults`, `test_build_config_with_defaults`

### Symptom 2: Port values silently corrupted

Parsing the port string `"70000"` should return an error or a safe default,
since valid ports range from 0 to 65535. Instead, `parse_port` returns `4464`
with no error.

**Failing test:** `test_port_overflow`

### Symptom 3: Debug flag always enabled

Parsing the string `"false"` as a boolean for the debug flag should return
`false`. Instead, `parse_debug("false")` returns `true`. In fact, ANY
non-empty string makes the debug flag `true`.

**Failing tests:** `test_debug_false`, `test_debug_whitespace`, `test_build_config`

## Rules

- Fix the bugs in `buggy.rs` directly
- Do not change the test assertions
- After fixing, all tests should pass
- Check `solution.md` only after you've attempted all three fixes
