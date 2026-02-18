# Standard Exercise: Command-Line Task Processor

## Scenario

You're building the core engine for a CLI task management tool used internally
at your company. The tool reads text commands from stdin (or a script file),
parses each line into a typed `Command` enum, and executes the command against
an in-memory task list. The team uses it for scripting deployment checklists,
on-call runbooks, and CI pipeline steps.

## Brief

Implement a `Command` enum with variants for each supported operation, a `Task`
struct to hold task data, and the parsing and execution logic that ties them
together. Heavy use of `match`, `if let`, guards, and `@` bindings.

## Acceptance Criteria

### 1. `Priority` enum with variants `Low`, `Medium`, `High`, `Critical`
- Derives `Debug`, `Clone`, `PartialEq`
- Implement `Priority::from_str(s: &str) -> Option<Priority>`
  - Case-insensitive: "low", "LOW", "Low" all work
  - Returns `None` for unrecognized strings

### 2. `Task` struct with fields:
- `id: u32`
- `title: String`
- `done: bool`
- `priority: Priority`
- `tags: Vec<String>`

### 3. `Command` enum with variants:
- `Add { title: String, priority: Priority }` — add a new task
- `Done(u32)` — mark task with given id as complete
- `Delete(u32)` — remove task with given id
- `List` — print all tasks
- `Priority { id: u32, level: Priority }` — change a task's priority
- `Tag { id: u32, tag: String }` — add a tag to a task
- `Filter(Priority)` — list only tasks with a given priority
- `Stats` — print summary statistics
- `Unknown(String)` — unrecognized command (stores the raw line)

### 4. `parse_command(line: &str) -> Command`

Parse a text line into a `Command`. Input format:

| Input | Command |
|-------|---------|
| `add <title> [priority:<level>]` | `Add { title, priority }` |
| `done <id>` | `Done(id)` |
| `delete <id>` | `Delete(id)` |
| `list` | `List` |
| `priority <id> <level>` | `Priority { id, level }` |
| `tag <id> <tag>` | `Tag { id, tag }` |
| `filter <level>` | `Filter(level)` |
| `stats` | `Stats` |
| anything else | `Unknown(raw_line)` |

- `add` examples: `"add Deploy to staging"` → priority defaults to `Medium`
- `add` with priority: `"add Deploy to staging priority:high"` → parses trailing `priority:X`
- Whitespace-only lines and empty lines → `Unknown` (or skip in execute)
- If an id cannot be parsed as `u32` → `Unknown`

### 5. `execute(command: Command, tasks: &mut Vec<Task>) -> String`

Execute a command against the task list. Returns a human-readable result string.

- `Add`: appends a new task; id is `tasks.len() as u32 + 1`; returns `"added task #N: <title>"`
- `Done(id)`: finds the task, marks it done; returns `"task #N marked done"` or `"task #N not found"`
- `Delete(id)`: removes the task; returns `"deleted task #N"` or `"task #N not found"`
- `List`: returns a formatted list of all tasks (see format below)
- `Priority { id, level }`: updates priority; returns `"task #N priority set to <level>"` or not-found message
- `Tag { id, tag }`: appends tag (skip if already present); returns `"tagged task #N with <tag>"`
- `Filter(level)`: returns formatted list of tasks matching that priority
- `Stats`: returns `"total: N, done: N, pending: N, critical: N"`
- `Unknown(line)`: returns `"unknown command: <line>"`

### 6. Task list format (for `List` and `Filter`)

```
[x] #1 [HIGH] Deploy to staging (tags: deploy, prod)
[ ] #2 [LOW] Update docs
[x] #3 [CRITICAL] Fix auth bypass (tags: security)
```

- `[x]` for done, `[ ]` for pending
- `[HIGH]` etc. in uppercase
- Tags section omitted if no tags

## Constraints

- No external crates — stdlib only
- All tests must pass against your implementation
- `parse_command` must handle malformed input gracefully (return `Unknown`, not panic)
- Do not use `.unwrap()` on user-supplied id strings — use `parse::<u32>().ok()` and return `Unknown` if it fails

## Hints

<details>
<summary>Hint 1: Parsing the add command</summary>

Split on whitespace, check if the last token starts with `"priority:"`. If so,
strip the prefix and parse the priority, then join the remaining tokens as the
title. If not, the entire rest of the line is the title with default priority.

```rust
let parts: Vec<&str> = rest.split_whitespace().collect();
let (title_parts, priority) = if let Some(last) = parts.last() {
    if let Some(level_str) = last.strip_prefix("priority:") {
        if let Some(p) = Priority::from_str(level_str) {
            (&parts[..parts.len()-1], p)
        } else {
            (parts.as_slice(), Priority::Medium)
        }
    } else {
        (parts.as_slice(), Priority::Medium)
    }
} else {
    return Command::Unknown(line.to_string());
};
```

</details>

<details>
<summary>Hint 2: Finding a task by id</summary>

Use `iter().position()` to find the index, then use that index to mutate or remove:

```rust
match tasks.iter().position(|t| t.id == id) {
    Some(idx) => {
        tasks[idx].done = true;
        format!("task #{} marked done", id)
    }
    None => format!("task #{} not found", id),
}
```

</details>

<details>
<summary>Hint 3: Stats with match arms</summary>

Use a for loop with a mutable tuple counter and a match inside:

```rust
let mut total = 0u32;
let mut done = 0u32;
// ...
for task in tasks.iter() {
    total += 1;
    if task.done { done += 1; }
    if task.priority == Priority::Critical { critical += 1; }
}
```

</details>

<details>
<summary>Hint 4: Formatting the task list</summary>

Use `format!` with a conditional for done, uppercase debug format for priority,
and `join` for tags:

```rust
let done_marker = if task.done { "x" } else { " " };
let tag_str = if task.tags.is_empty() {
    String::new()
} else {
    format!(" (tags: {})", task.tags.join(", "))
};
format!("[{}] #{} [{:?}] {}{}", done_marker, task.id, task.priority, task.title, tag_str)
```

</details>
