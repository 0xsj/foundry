# Solution Notes: Command-Line Task Processor

## Approach

The solution separates concerns into three layers: types (data model), parsing
(text → Command), and execution (Command → effect + message). Each layer uses
Rust control flow constructs in their most idiomatic form.

## Key Decisions

### Parser design: match on verb, then per-command helpers

The top-level `parse_command` matches on the first word, then delegates to
helper functions (`parse_add`, `parse_priority_cmd`, `parse_tag_cmd`). This
keeps each arm of the main match small and makes the individual parsers testable
in isolation.

Alternative: a single large match with all parsing inline. This works for small
command sets but becomes hard to read at scale.

### Return `Unknown` instead of `Result<Command, Error>`

Parsing failures produce `Command::Unknown(raw_line)` rather than `Err(...)`.
This means `execute()` never has to deal with parse errors separately — it just
handles `Unknown` like any other command. The caller (main loop) stays simple.

Alternative: `parse_command` returns `Result<Command, ParseError>`, and the
caller decides how to handle errors. More flexible for machines, slightly more
awkward for interactive use.

### `iter().position()` for task lookup

`position()` returns the index of the first match, which lets us both find and
mutate/remove the element in one step. No cloning required.

```rust
match tasks.iter().position(|t| t.id == id) {
    Some(idx) => { tasks[idx].done = true; ... }
    None => ...
}
```

Alternative: find with `iter_mut()`. More direct for mutation but doesn't help
with removal (you'd still need the index).

### `@` binding is not used in the final solution

The `@` binding would be natural in a place like:

```rust
match tasks.iter().position(|t| t.id == id) {
    Some(idx @ 0) => { ... } // found at index 0
    Some(idx) => { ... }
    None => ...
}
```

But no test case requires distinguishing positions. The `@` pattern is
demonstrated in `matching.rs` instead.

### Tag deduplication with `contains`

Tags are stored as `Vec<String>`. Dedup uses `contains()` which is O(n) but fine
for small tag sets. A `HashSet<String>` would be O(1) but adds complexity for
a learning exercise.

## Pattern Summary

| Concept | Where used |
|---|---|
| `match` on enum variants | `execute`, `parse_command` |
| Tuple struct destructuring | `Command::Done(id)`, `Command::Filter(p)` |
| Named field destructuring | `Command::Add { title, priority }`, `Command::Priority { id, level }` |
| `if let` for Option | `parse_add` (priority token detection), `parse_priority_cmd` |
| Chained `if let` | `parse_priority_cmd` (both id and level must succeed) |
| `match` with `None => default` | All task lookups |
| `.ok()` to convert Result → Option | All id parsing (`id_str.parse::<u32>().ok()`) |
| Guard-free match on unit variants | `Command::List`, `Command::Stats` |

## Variants Worth Exploring

### Variant A: Command-level Result

```rust
fn parse_command(line: &str) -> Result<Command, String> { ... }
```

Pros: caller can log parse errors differently from unknown commands.
Cons: execute() can no longer be the single dispatch point.

### Variant B: Trait-based dispatch

```rust
trait Executable { fn execute(self, tasks: &mut Vec<Task>) -> String; }
impl Executable for Command { ... }
```

Pros: each command is self-contained.
Cons: over-engineered for this scale. Introduce this pattern in the
patterns module (Strategy pattern).

### Variant C: `HashMap` for task storage

Using `HashMap<u32, Task>` instead of `Vec<Task>` gives O(1) lookup by id at
the cost of losing insertion order for `List`.

```rust
use std::collections::HashMap;
let mut tasks: HashMap<u32, Task> = HashMap::new();
```

Pros: faster for large task lists.
Cons: iteration order is undefined (need to sort for `list`/`filter`).
