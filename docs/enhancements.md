# Foundry Enhancements — February 2026

This document tracks major enhancements to the Foundry learning system.

## February 8, 2026 — Comprehensive Structure Expansion

### Module Structure Standardization

**Documentation added to `CLAUDE.md`:**
- Defined standard file structure for all modules
- Three-tier approach: lesson.md (tutorial), reference.md (spec), code files (examples)
- Cross-language vault notes for concept synthesis
- Clear workflow for when to create each file type

**Templates created:**
- `vault/templates/module-lesson.md` — Tutorial-style lesson template
- `vault/templates/module-reference.md` — Spec extract template

### Exercise System Expansion

**New exercise types:**

1. **Standard Exercises** (`tracks/<lang>/exercises/<topic>/`)
   - `starter/` — Scaffold with TODOs
   - `solutions/` — Reference implementations + variants
   - `my-solution/` — User's work + review notes
   - Progressive hints system
   - Benchmarking for performance-sensitive exercises

2. **Debugging Exercises** (`tracks/<lang>/exercises/debugging/<scenario>/`)
   - Buggy code with intentional bugs
   - Symptom-based descriptions (not solutions)
   - Builds debugging intuition distinct from writing code

3. **Refactoring Exercises** (`tracks/<lang>/exercises/refactoring/<scenario>/`)
   - Working but poorly structured code
   - Code smell identification
   - Pattern application practice
   - Before/after comparison with trade-off analysis

**Templates created:**
- `vault/templates/exercise-readme.md` — Exercise description template
- `vault/templates/solution-readme.md` — Solution explanation template
- `vault/templates/review.md` — Self-review template
- `vault/templates/debugging-exercise.md` — Debugging exercise template
- `vault/templates/refactoring-exercise.md` — Refactoring exercise template

### Knowledge Base Enhancements

**New vault sections:**

1. **Pitfalls Database** (`vault/pitfalls/`)
   - Common mistakes by language/concept
   - Mental model gap explanations
   - Prevention strategies
   - Template: `vault/templates/pitfall.md`

2. **Real-World Examples** (`vault/examples/production-patterns/`)
   - Annotated code from production OSS projects
   - What makes code production-grade
   - Evolution and lessons learned
   - Template: `vault/templates/production-example.md`

3. **Rosetta Stone** (`vault/rosetta/common-operations.md`)
   - Cross-language quick reference
   - Common operations (file I/O, HTTP, JSON, strings, collections, etc.)
   - Side-by-side comparison table format
   - All 5 languages (Go, TypeScript, Rust, Python, Java)

4. **Interview Prep** (`vault/interview-prep/`)
   - `by-concept.md` — Module to interview question mapping
   - `by-company.md` — Company-specific patterns
   - `leetcode-mapping.md` — DSA problems by concept

### Spaced Repetition System

**Enhanced `curriculum/progress.yaml` schema:**
- `completed_date` — When module was finished
- `review_count` — Number of reviews
- `next_review` — Auto-calculated review date
- `confidence` — User self-assessment (1-5 scale)
- `time_spent_minutes` — Optional time tracking

**Algorithm:**
```
1st review: +1 day
2nd review: +3 days
3rd review: +7 days
4th review: +14 days
5th+ review: +30 days

Adjustments:
- confidence < 3: halve interval
- confidence = 5: double interval
```

**Automatic prompts:**
- Reviews due today
- Low confidence modules to revisit
- Modules not reviewed in 30+ days

### Concept Map Visualization

**Enhanced `curriculum/map.yaml` support:**
- Dependency tracking
- Unlock requirements
- Estimated time per module
- Language applicability

**New command: `graph`**
- Generates Mermaid diagram
- Visual status (completed/in-progress/locked/available)
- Dependency arrows
- Integration with Obsidian graph view

### Performance Analysis

**Benchmarking support:**
- `*_bench_test.go` files for performance-critical exercises
- `performance-notes.md` template
- Time/space complexity analysis
- When-to-use guidance for variants

### Documentation Updates

**Updated `CLAUDE.md` sections:**
- Repository Structure (enhanced)
- Module Structure (new)
- Exercise Structure (new, comprehensive)
- Supporting Resources (new)
- Knowledge Base Enhancements (new)
- Spaced Repetition System (new)
- Concept Map and Visualization (new)
- Behavioral Rules (added rule #8)

## Impact

**For the learner:**
- Clear structure for every module
- Multiple exercise types matching real work
- Built-in review system prevents forgetting
- Cross-language comparisons build transferable understanding
- Production code examples show real-world application
- Interview prep integrated into curriculum

**For the AI agent:**
- Consistent templates reduce ambiguity
- Clear workflows for module creation
- Automated review prompts
- Visual progress tracking
- Structured knowledge base

## Next Steps

**Phase 2 (Future):**
- Weekly/monthly retrospective templates
- Build artifacts and post-mortems
- ADR (Architecture Decision Records) for larger builds
- Session goal setting and achievement tracking
- Community-contributed exercises
- Spaced repetition algorithm tuning based on actual retention data
