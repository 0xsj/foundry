# Foundry System Updates — February 10, 2026

## Summary of Changes

This document summarizes the major updates made to the Foundry learning system.

---

## 1. New Exercise Types Added ✅

### Code Review Practice
- **Purpose:** Learn to read and critique code, spot bugs, security issues, and design problems
- **Location:** `tracks/<lang>/exercises/code-review/<scenario>/`
- **Template:** `vault/templates/code-review-exercise.md`
- **Example:** `tracks/go/exercises/code-review/_example-pr-auth/`

**Structure:**
```
code-review/<scenario>/
├── README.md              # PR context
├── codebase/              # Surrounding code for context
├── proposed-changes.diff  # The changes to review
├── my-review.md           # Your review (template)
├── review-guide.md        # Progressive hints
└── expert-review.md       # Reference review
```

### Codebase Navigation
- **Purpose:** Learn to explore unfamiliar codebases efficiently using grep/ripgrep
- **Location:** `vault/examples/codebase-navigation/<project>/`
- **Template:** `vault/templates/codebase-navigation-exercise.md`
- **Example:** `vault/examples/codebase-navigation/_example-project/`

**Structure:**
```
codebase-navigation/<project>/
├── README.md              # Project overview and setup
├── missions.md            # Scavenger hunt tasks
├── navigation-log.md      # Your exploration notes
└── solutions.md           # Where to find things
```

### API Design
- **Purpose:** Learn to design clean, maintainable interfaces and APIs
- **Location:** `tracks/<lang>/exercises/api-design/<scenario>/`
- **Template:** `vault/templates/api-design-exercise.md`
- **Example:** `tracks/go/exercises/api-design/_example-rate-limiter/`

**Structure:**
```
api-design/<scenario>/
├── README.md              # Requirements and constraints
├── my-design/             # Your API design
├── designs/               # Alternative approaches
└── evolution.md           # How to evolve the API
```

---

## 2. New Language Tracks Added ✅

### Zig
- **Why:** Systems programming perspective, manual memory management, comptime metaprogramming
- **Location:** `tracks/zig/`
- **README:** `tracks/zig/README.md`

### Haskell
- **Why:** Pure functional programming, lazy evaluation, strong type system
- **Location:** `tracks/haskell/`
- **README:** `tracks/haskell/README.md`

### Scala (Replaces Java)
- **Why:** Learn JVM ecosystem through modern lens, functional + OOP blend
- **Location:** `tracks/scala/`
- **README:** `tracks/scala/README.md`

**Note:** Scala provides JVM ecosystem learning with better language features. Users can learn Java through Scala since it compiles to JVM bytecode and uses Java libraries seamlessly.

---

## 3. All Documentation Updated ✅

- ✅ `CLAUDE.md` — Exercise types, language list
- ✅ `curriculum/map.yaml` — Java→Scala, added Zig/Haskell
- ✅ `curriculum/progress.yaml` — Java→Scala, added Zig/Haskell sections
- ✅ `START-HERE.md` — Updated languages and enhancements
- ✅ `README.md` — Updated language rationales
- ✅ `vault/rosetta/common-operations.md` — Added new language columns

---

## 4. Complete Directory Structures Created ✅

All necessary directories for Zig, Haskell, and Scala tracks created, including:
- fundamentals/
- patterns/
- exercises/{debugging,refactoring,code-review,api-design}/
- builds/
- vault subdirectories

---

## What's Ready Now

### For Users
1. **Clear documentation** for three new exercise types
2. **Templates** to create new exercises
3. **Example structures** showing what good exercises look like
4. **Language tracks** ready for content
5. **README files** explaining why each language and how to get started

### For Content Creation
1. **Templates exist** for all three new exercise types
2. **Example exercises** show the structure
3. **Progress tracking** configured for all languages
4. **Curriculum map** updated with language applicability

---

## Next Steps

### Immediate
1. Create first code review exercise (suggest: authentication PR review in Go)
2. Select OSS project for first codebase navigation exercise
3. Create first API design exercise (suggest: rate limiter in Go)

### Short Term
1. Generate variables-and-types modules for Scala, Zig, Haskell
2. Populate Rosetta stone with operations for new languages
3. Create 2-3 exercises for each new type

### Medium Term
1. Build out fundamental modules across all languages
2. Create language-specific pitfalls as encountered
3. Develop comparative implementation series

---

**System is production-ready for the new exercise types and language tracks!**
