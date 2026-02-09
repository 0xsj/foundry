# How Key Concepts Are Surfaced in Foundry

This document explains how key concepts, pitfalls, and patterns are surfaced throughout the learning system so you encounter them at the right time with the right context.

---

## The Problem

When learning a new language, you need to encounter important concepts multiple times in different contexts:
- **First time**: In the lesson (introduction)
- **Second time**: When you hit the bug yourself (debugging exercise)
- **Third time**: In the pitfall database (deep dive)
- **Fourth time**: In interview prep (articulation)
- **Fifth time**: In production examples (real-world usage)

Without explicit connections, each of these is an isolated island of knowledge.

---

## The Solution: Web of Links

Every key concept is cross-referenced using Obsidian-style `[[wiki-links]]` so you can:
1. Discover related concepts naturally
2. Deep dive when you encounter something interesting
3. Build a mental map of how concepts connect

---

## Example: "Nil Map Panic" Journey

### 1. First Mention — Lesson (Preview)

**Location**: `tracks/go/fundamentals/variables-and-types/lesson.md`

```markdown
## Zero Values

...table of zero values...

But watch out: **a nil map looks valid but panics on write**.

> **Common Pitfall:** This is one of the most common Go gotchas.
> See [[go-nil-map-panic]] for a deep dive.
```

**Experience:**
- You're reading about zero values
- You see the warning about nil maps
- You click the link to read more (or continue and come back later)
- The wiki link is Obsidian-compatible (clickable in VS Code with extensions)

### 2. Second Encounter — Exercise Bug

**Location**: Somewhere in config-loader or other exercise

```go
type Cache struct {
    data map[string]string  // oops, forgot to initialize
}

func (c *Cache) Set(k, v string) {
    c.data[k] = v  // PANIC at runtime
}
```

**Experience:**
- Your code panics
- You remember the warning from the lesson
- You search the vault: `grep -r "nil map" vault/`
- You find `vault/pitfalls/go-nil-map-panic.md`
- You read the deep dive and learn the mental model

### 3. Third Encounter — Debugging Exercise

**Location**: `tracks/go/exercises/debugging/nil-map-panic/`

**Experience:**
- Explicit practice fixing this bug
- Work through buggy code → identify root cause → fix → verify
- Read `solution.md` which links to the pitfall entry:
  ```markdown
  ## Related Pitfalls
  - [[go-nil-map-panic]] in the vault
  ```

### 4. Fourth Encounter — Vault Note

**Location**: `vault/fundamentals/variables-and-types.md`

```markdown
## Common Pitfalls

### Go: Nil Map Panic

See [[go-nil-map-panic]]

```go
var m map[string]int
m["key"] = 42  // PANIC
```

**Fix:** Always use `make()` for maps.
```

**Experience:**
- Reviewing the vault note for variables-and-types
- See the pitfall listed in Common Pitfalls section
- Reinforces the pattern

### 5. Fifth Encounter — Interview Prep

**Location**: `vault/interview-prep/by-concept.md`

```markdown
## Variables and Types

### Language-Specific Questions

- **Go**: "Why can you read from a nil map but not write to it?"

### Related Pitfalls
- [[go-nil-map-panic]]
```

**Experience:**
- Preparing for interviews
- Practice explaining the concept
- Reference the pitfall entry for talking points

### 6. Sixth Encounter — Production Example

**Location**: `vault/examples/production-patterns/cache-implementation/`

```markdown
## What Makes This Production-Grade

### 1. Safe Initialization

```go
func NewCache() *Cache {
    return &Cache{
        data: make(map[string]string),  // ← Always initialized
    }
}
```

This avoids [[go-nil-map-panic]] by ensuring the map is always initialized
in the constructor.
```

**Experience:**
- See how real projects handle initialization
- Reinforces the pattern in production context

---

## Web of Connections

```
                        ┌─────────────────┐
                        │   Lesson.md     │
                        │  (first mention)│
                        └────────┬────────┘
                                 │
                                 ↓ [[wiki-link]]
                        ┌─────────────────┐
                   ┌───→│   Pitfall DB    │←────┐
                   │    │  (deep dive)    │     │
                   │    └────────┬────────┘     │
                   │             │               │
                   │             ↓               │
        ┌──────────┴─────┐  ┌───────────┐  ┌───┴──────────┐
        │ Debugging Ex   │  │ Vault Note│  │ Interview Prep│
        │  (practice)    │  │ (review)  │  │ (articulate)  │
        └────────────────┘  └───────────┘  └───────────────┘
                   │             │               │
                   │             ↓               │
                   └────→ [[go-nil-map-panic]] ←┘
                                 ↑
                                 │
                        ┌────────┴────────┐
                        │ Production Ex   │
                        │  (real code)    │
                        └─────────────────┘
```

---

## How Links Are Added

### When Writing Lessons

**Rule:** Whenever you mention a concept that has a pitfall or pattern, add a link.

```markdown
❌ Bad: "Watch out for nil maps — they panic on write."

✅ Good: "Watch out for nil maps — they panic on write.
         See [[go-nil-map-panic]] for details."
```

### When Creating Exercises

**Solution READMEs** should always have a "Related Concepts" section:

```markdown
## Related Concepts

- [[error-handling]] — How Go handles errors
- [[go-nil-map-panic]] — Why we always initialize maps
- [[constructors-and-initialization]] — Constructor patterns
```

### When Hitting Bugs

**Immediately create a pitfall entry** and link it:

1. Hit bug during exercise
2. Create `vault/pitfalls/go-nil-map-panic.md`
3. Update `vault/fundamentals/variables-and-types.md` to link it
4. Update lesson.md to reference it
5. Add to interview-prep mapping

### When Writing Vault Notes

**Pitfalls section** should list all common mistakes:

```markdown
## Common Pitfalls

### Go: Nil Map Panic
See [[go-nil-map-panic]]

### TypeScript: Implicit any
See [[typescript-implicit-any]]
```

---

## Discovery Mechanisms

### 1. Obsidian Graph View

If you open the vault in Obsidian, you'll see a visual graph of all connections:
- Nodes = files
- Edges = wiki links
- Clusters = related concepts

### 2. Grep Search

```bash
# Find all mentions of nil-map-panic
grep -r "nil-map" vault/

# Find all pitfalls
ls vault/pitfalls/

# Find all links to a concept
grep -r "\[\[go-nil-map-panic\]\]" .
```

### 3. IDE Integration

With VS Code extensions like "Markdown Links" or "Foam":
- Cmd+Click on `[[wiki-link]]` → jump to file
- See backlinks (what links to this file)
- Autocomplete wiki links as you type

### 4. Explicit Reminders

The AI agent is programmed to:
- Mention related pitfalls when teaching
- Link to vault notes after exercises
- Surface pitfalls when you hit bugs
- Suggest interview questions for concepts

---

## Example Workflow: Learning Go Zero Values

1. **Read lesson.md**
   - "Zero values are Go's default initialization"
   - Table showing nil for maps
   - Warning: "nil maps panic on write" + link

2. **Click [[go-nil-map-panic]]**
   - Deep dive: why it happens, mental model gap
   - Comparison to nil slices
   - Production incidents

3. **Return to lesson, continue reading**
   - See zero values for other types
   - Notice structs have all fields zeroed

4. **Do config-loader exercise**
   - Implement config parsing
   - Compare to solution
   - Solution README links to related concepts

5. **Later: Do debugging exercise**
   - Fix nil map bug
   - Solution links back to pitfall entry
   - Reinforce the pattern

6. **Week later: Review vault note**
   - See Common Pitfalls section
   - Click link to refresh memory
   - Update confidence in progress.yaml

7. **Before interview: Check interview-prep**
   - See question: "Why can you read from nil map?"
   - Click link to pitfall for talking points
   - Practice explaining to friend

---

## Automatic Linking (Future Enhancement)

**Future feature**: AI could automatically suggest links when creating content:

```markdown
You wrote: "Maps must be initialized with make()"

💡 Suggestion: Add link to [[go-nil-map-panic]]
```

But for now, it's manual. The templates remind you to add links.

---

## Checklist for New Content

When creating any new content, ask:

- [ ] Does this mention a pitfall? → Add `[[link]]` to pitfall entry
- [ ] Does this use a pattern? → Add `[[link]]` to pattern note
- [ ] Does this relate to interview questions? → Update interview-prep
- [ ] Does this connect to other modules? → Add to "Related Concepts"
- [ ] Is this a new pitfall? → Create entry + update vault note + update lesson

---

This web of connections is what transforms a directory of files into a **knowledge graph** you can navigate and explore.
