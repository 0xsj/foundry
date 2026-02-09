# 🚀 Start Here — Foundry Quick Start

Welcome to Foundry! This is your roadmap to the learning system.

---

## First Time Here?

**Read these in order:**

1. **This file** (you are here) — 2 min
2. **[CLAUDE.md](./CLAUDE.md)** — How the system works (10 min)
3. **[docs/learning-workflow.md](./docs/learning-workflow.md)** — Step-by-step guide (5 min to skim, reference as needed)

---

## Quick Start: Your First Module

### Option A: Go (If you prefer Go)

```bash
# 1. Read the lesson
open tracks/go/fundamentals/variables-and-types/lesson.md

# 2. Do the exercise
cd tracks/go/exercises/variables-and-types/01-config-loader/starter
go test  # watch tests fail
# implement config.go
go test  # verify your solution

# 3. Compare with solution
cd ../solutions
cat README.md
cat config.go

# 4. Try the debugging exercise
cd ../../debugging/nil-map-panic
go test  # watch it panic
# fix buggy.go
go test  # verify fix
```

### Option B: TypeScript (If you prefer TypeScript)

```bash
# 1. Read the lesson
open tracks/typescript/fundamentals/variables-and-types/lesson.md

# 2. Do the exercise
cd tracks/typescript/exercises/variables-and-types/01-config-loader/starter
npm test  # watch tests fail
# implement config.ts
npm test  # verify your solution

# 3. Compare with solution
cd ../solutions
cat config.ts
```

### Option C: C# (To test the new track)

```bash
# Read the lesson
open tracks/csharp/fundamentals/variables-and-types/lesson.md

# Exercises coming soon — lesson is ready to read
```

---

## The Learning Loop

For every module, follow this pattern:

```
1. Read lesson.md          (30-45 min) — Tutorial + mental models
2. Skim reference.md       (5-10 min)  — Spec for later lookup
3. Do exercise             (30-60 min) — Practice in starter/
4. Compare with solutions  (10-15 min) — Learn patterns
5. Write review notes      (5 min)     — Reflect on what you learned
6. (Optional) Debug/Refactor (20-45 min) — Extra practice
7. Update progress.yaml    (2 min)     — Track confidence
```

**Full guide:** [docs/learning-workflow.md](./docs/learning-workflow.md)

---

## Repository Structure

```
foundry/
├── START-HERE.md          ← You are here
├── CLAUDE.md              ← System overview (read this!)
├── docs/
│   ├── learning-workflow.md    ← Step-by-step guide
│   ├── concept-surfacing.md    ← How concepts are linked
│   └── enhancements.md         ← What's been built
├── tracks/
│   ├── go/                ← Go learning path
│   ├── typescript/        ← TypeScript learning path
│   ├── rust/              ← Rust learning path
│   ├── python/            ← Python learning path
│   ├── java/              ← Java learning path
│   └── csharp/            ← C# learning path
├── vault/                 ← Knowledge base (Obsidian compatible)
│   ├── fundamentals/      ← Cross-language concept notes
│   ├── pitfalls/          ← Common mistakes database
│   ├── rosetta/           ← Cross-language quick reference
│   ├── interview-prep/    ← Interview question mapping
│   └── examples/          ← Production code examples
└── curriculum/
    ├── progress.yaml      ← Your progress tracking
    └── map.yaml           ← Dependency graph
```

---

## Key Resources

### For Learning
- **Lessons**: `tracks/<lang>/fundamentals/<module>/lesson.md`
- **Exercises**: `tracks/<lang>/exercises/<module>/`
- **References**: `tracks/<lang>/fundamentals/<module>/reference.md`

### For Review
- **Vault notes**: `vault/<category>/<module>.md` (all languages compared)
- **Pitfalls**: `vault/pitfalls/` (common mistakes)
- **Progress**: `curriculum/progress.yaml` (your journey)

### For Practice
- **Debugging**: `tracks/<lang>/exercises/debugging/`
- **Refactoring**: `tracks/<lang>/exercises/refactoring/`
- **Interview prep**: `vault/interview-prep/`

### For Reference
- **Rosetta stone**: `vault/rosetta/common-operations.md` (5 languages side-by-side)
- **Production examples**: `vault/examples/` (real-world code)

---

## Current Status

### Languages
- ✅ **Go** — variables-and-types complete with exercises
- ✅ **TypeScript** — variables-and-types complete with exercises
- ✅ **Rust** — variables-and-types lesson + reference complete
- ✅ **Python** — variables-and-types lesson + reference complete
- ✅ **Java** — variables-and-types lesson + reference complete
- ✅ **C#** — variables-and-types complete (NEW!)

### Enhancements Live
- ✅ Standard exercises with solutions + variants
- ✅ Debugging exercises (Go)
- ✅ Refactoring exercises (Go)
- ✅ Pitfalls database (2 entries)
- ✅ Production examples (Kubernetes config)
- ✅ Interview prep mapping
- ✅ Rosetta stone (all 5 languages)
- ✅ Cross-language vault notes
- ✅ Spaced repetition schema

See [ENHANCEMENTS_DEMO.md](./ENHANCEMENTS_DEMO.md) for full tour.

---

## Next Steps

1. **Choose your primary language** (Go or TypeScript recommended)
2. **Read [docs/learning-workflow.md](./docs/learning-workflow.md)** (your instruction manual)
3. **Start with variables-and-types**
   - Read lesson.md
   - Do the config-loader exercise
   - Compare with solutions
4. **Check progress.yaml** to see what's next
5. **Follow the spaced repetition** schedule for reviews

---

## Tips

- 📖 **Read, don't skip** — The lessons build mental models
- 💪 **Practice first** — Don't look at solutions immediately
- 📝 **Take notes** — Use "Your notes" sections liberally
- 🔗 **Follow links** — `[[wiki-links]]` connect concepts
- 🎯 **One module at a time** — Depth > breadth
- ⏱️ **Schedule reviews** — Trust the spaced repetition algorithm

---

## Getting Help

- **Command reference**: Read [CLAUDE.md](./CLAUDE.md) for system commands
- **Workflow confused?**: Check [docs/learning-workflow.md](./docs/learning-workflow.md)
- **Can't find something?**: Use grep: `grep -r "concept name" vault/`
- **Lost?**: Check `curriculum/progress.yaml` for what's completed

---

## Demo: Try This Now

**5-minute taste of the system:**

```bash
# 1. Read a short section
head -50 tracks/go/fundamentals/variables-and-types/lesson.md

# 2. See the cross-language comparison
cat vault/fundamentals/variables-and-types.md | head -100

# 3. Check out a pitfall
cat vault/pitfalls/go-nil-map-panic.md

# 4. See the Rosetta stone
cat vault/rosetta/common-operations.md | head -80
```

---

**Ready?** → Start with [docs/learning-workflow.md](./docs/learning-workflow.md)

**Questions?** → Check [CLAUDE.md](./CLAUDE.md)

**Let's learn!** 🚀
