# Codebase Navigation Exercise Template

## Overview

**Project:** [Name and brief description of the OSS project]

**Your Mission:** Navigate an unfamiliar codebase to answer specific questions and complete tasks without being told where to look.

**Skills Practiced:**
- Using grep/ripgrep effectively
- Following control flow across files
- Understanding architecture by exploration
- Pattern recognition in foreign code

---

## Setup

```bash
# Clone the project
git clone [repo-url]
cd [project-name]

# Checkout specific commit (for consistency)
git checkout [commit-hash]

# Optional: Set up the project (if you want to run it)
[setup commands]
```

---

## Missions

Complete these tasks by navigating the codebase. Document your process in `navigation-log.md`.

### Mission 1: [Task name]
**Question:** [Specific thing to find or understand]

**Your goal:**
- [ ] Find the relevant code
- [ ] Explain how it works
- [ ] Document the path you took to find it

**Example:** "Find where user authentication tokens are validated"

---

### Mission 2: [Task name]
**Question:** [Another specific task]

**Your goal:**
- [ ] [Sub-task 1]
- [ ] [Sub-task 2]

---

### Mission 3: [Trace a flow]
**Question:** "Trace how a [specific request/event] flows through the system"

**Your goal:**
- [ ] Identify the entry point
- [ ] List all functions/methods called in order
- [ ] Note any interesting patterns or decisions
- [ ] Draw a simple flow diagram

---

### Mission 4: [Find a pattern]
**Question:** "Find all places where [X pattern] is used"

**Your goal:**
- [ ] Use grep/ripgrep to find candidates
- [ ] Verify which ones are actual uses of the pattern
- [ ] Explain why the pattern was chosen in each case

---

## Navigation Log Template

For each mission, document:

```markdown
## Mission [N]: [Task name]

### Search Strategy
[What commands/approaches did you try?]

### Dead Ends
[What didn't work and why?]

### Discovery Path
[Step-by-step: how you found the answer]

### Answer
[The actual answer to the question]

### Insights
[What did you learn about the codebase architecture?]
```

---

## Hints (Progressive)

<details>
<summary>Hint: Useful commands</summary>

```bash
# Search for function definitions
rg "fn function_name"

# Search for struct/class definitions
rg "struct StructName"

# Find all callers of a function
rg "function_name\("

# Search in specific file types only
rg "pattern" -t rust

# Case-insensitive search
rg -i "pattern"
```

</details>

<details>
<summary>Hint: Common patterns to search for</summary>

- Entry points: `main`, `serve`, `listen`, `handle`
- Middleware: `middleware`, `handler`, `wrapper`
- Authentication: `auth`, `token`, `jwt`, `verify`
- Database: `query`, `transaction`, `pool`, `connection`
- Errors: `Error`, `Result`, `err`, `panic`

</details>

---

## Success Criteria

You've successfully completed this exercise when you can:
- [ ] Answer all missions accurately
- [ ] Explain your search strategy
- [ ] Navigate the codebase confidently without guidance
- [ ] Identify architectural patterns used in the project

---

## Notes

**Project Size:** [Small/Medium/Large] — [approximate line count]

**Difficulty:** [Easy/Medium/Hard]

**Estimated Time:** [30-60 min]

**Related Concepts:** [[project-architecture]], [[code-reading]]
