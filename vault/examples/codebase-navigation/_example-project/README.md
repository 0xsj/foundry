# Example: Codebase Navigation Exercise

This is an **example structure** for codebase navigation exercises. Use `vault/templates/codebase-navigation-exercise.md` as a template.

## Overview

**Project:** [Name of OSS project — e.g., "miniserve" (Rust HTTP server)]

**Your Mission:** Navigate this unfamiliar codebase to answer specific questions without being told where to look.

**Skills Practiced:**
- Using grep/ripgrep effectively
- Following control flow across files
- Understanding architecture by exploration
- Pattern recognition in foreign code

---

## Files in this directory

```
codebase-navigation/_example-project/
├── README.md              # This file — project overview and setup
├── missions.md            # Scavenger hunt tasks
├── navigation-log.md      # Template for your exploration notes
└── solutions.md           # Where to find things + reasoning
```

---

## Setup Instructions

```bash
# Clone the project
git clone https://github.com/user/project
cd project

# Checkout specific commit (for consistency across exercises)
git checkout abc123def

# Optional: build and run if you want to experiment
cargo build  # or npm install, go build, etc.
```

---

## Example Missions

### Mission 1: Find the Entry Point
**Question:** Where does the application start? What are the first functions called?

### Mission 2: Trace Request Flow
**Question:** Trace how an HTTP request flows from receiving the connection to sending the response.

### Mission 3: Find All Auth
**Question:** Find all places where authentication/authorization is checked.

### Mission 4: Understand Error Handling
**Question:** How are errors represented and handled throughout the codebase?

---

## Creating a New Navigation Exercise

1. **Choose an OSS project** — small (5-10k lines) for beginners, larger for advanced
2. **Create missions** that require actual exploration (not answerable by reading README)
3. **Write solutions** with:
   - File paths and line numbers
   - Search commands that would find it
   - Architectural insights
4. **Include navigation tips** — common patterns, where to start looking

---

## Good Projects to Use

### Beginner (5-10k lines)
- **miniserve** (Rust) — Simple HTTP file server
- **gitmoji-cli** (TypeScript) — Git emoji tool
- **httpie** (Python) — HTTP client

### Intermediate (20-50k lines)
- **caddy** (Go) — Web server
- **prettier** (TypeScript) — Code formatter
- **ripgrep** (Rust) — Fast grep

### Advanced (100k+ lines)
- **Kubernetes** (Go) — Container orchestration
- **VS Code** (TypeScript) — Code editor
- **Tokio** (Rust) — Async runtime

---

## Navigation Techniques to Teach

```bash
# Find function/method definitions
rg "fn function_name"
rg "func functionName"
rg "class ClassName"

# Find callers of a function
rg "functionName\("

# Search in specific file types
rg "pattern" -t rust
rg "pattern" -t go

# Case-insensitive
rg -i "pattern"

# Find imports/uses
rg "^use " -t rust
rg "^import " -t go

# Find TODO comments
rg "TODO|FIXME"
```

---

## Mission Types

1. **Find and explain** — "Where is X implemented?"
2. **Trace a flow** — "How does Y flow through the system?"
3. **Find all uses** — "Find all places where Z is used"
4. **Understand patterns** — "How is dependency injection done here?"
5. **Identify architecture** — "What architectural pattern does this follow?"
6. **Debug by reading** — "Why would this cause bug X?" (without running code)
