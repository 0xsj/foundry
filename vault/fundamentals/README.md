# Fundamentals — Index

This directory contains cross-language comparisons of fundamental programming concepts.

---

## How to Navigate

### Cross-Language Comparisons (This Directory)

These files compare concepts across all 6 languages:

- [[variables-and-types]] — Type systems, memory models, primitives
- (More to come: control-flow, functions-and-closures, error-handling, etc.)

### Language-Specific Deep Dives (Subdirectories)

For in-depth, language-specific explanations:

- **Go**: `go/` — Escape analysis, goroutines, interfaces
- **TypeScript**: `typescript/` — Type erasure, narrowing, decorators
- **Rust**: `rust/` — Ownership, lifetimes, traits
- **Python**: `python/` — Duck typing, metaclasses, descriptors
- **Java**: `java/` — JVM internals, reflection, generics erasure
- **C#**: `csharp/` — CLR, LINQ, async/await

---

## Structure

```
fundamentals/
├── README.md (this file)
├── variables-and-types.md          ← Start here for cross-language view
├── go/
│   └── variables-and-types.md      ← Go deep dive
├── typescript/
│   └── variables-and-types.md      ← TypeScript deep dive
├── rust/
│   └── variables-and-types.md      ← Rust deep dive
├── python/
├── java/
└── csharp/
```

---

## Concepts Covered

### ✅ Completed

| Concept | Cross-Language | Go | TypeScript | Rust | Python | Java | C# |
|---------|----------------|----|-----------:|------|--------|------|----|
| Variables and Types | ✅ | ✅ | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |

### 🚧 Coming Soon

- Control Flow (if/switch/loops)
- Functions and Closures
- Error Handling
- Interfaces and Traits
- Generics
- Concurrency
- Memory and Ownership
- Serialization
- Testing Fundamentals
- Modules and Packages

---

## How to Use

1. **Start with the cross-language note** (e.g., [[variables-and-types]])
   - Get the high-level comparison
   - See how all languages approach the concept
   - Identify key differences

2. **Deep dive into specific languages** (e.g., [[go/variables-and-types]])
   - Understand language-specific semantics
   - See detailed examples
   - Learn unique concepts

3. **Cross-reference as you learn**
   - Notes link to each other
   - Build a web of understanding
   - See connections between languages

---

## Obsidian Tips

### Graph View
Open Obsidian graph view to see:
- Cross-language notes as central hubs
- Language-specific notes clustered by language
- Connections between concepts

### Search
- Search across all fundamentals: Click this directory in Obsidian
- Filter by language: Use tag `#go`, `#typescript`, etc.
- Find related concepts: Check `related:` in frontmatter

### Links
- Use `Cmd+Click` (Mac) or `Ctrl+Click` (Windows) on `[[wiki-links]]`
- See backlinks: Open backlinks panel (right sidebar)
- Navigate breadcrumbs: Use forward/back buttons

---

## Contributing

When adding new concepts:

1. **Create cross-language note first**
   - Location: `vault/fundamentals/<concept>.md`
   - Template: `vault/templates/concept-note.md`
   - Compare all 6 languages

2. **Add language-specific notes as needed**
   - Location: `vault/fundamentals/<language>/<concept>.md`
   - Template: `vault/templates/module-lesson.md` (adapted)
   - Focus on language-unique aspects

3. **Link bidirectionally**
   - Cross-lang → language-specific: `[[go/concept]]`
   - Language-specific → cross-lang: `[[fundamentals/concept]]`

4. **Update this README**
   - Add to table of concepts
   - Mark completion status

---

## Related

- [[../patterns/README]] — Design patterns
- [[../pitfalls/]] — Common mistakes
- [[../rosetta/common-operations]] — Quick reference
- [[../interview-prep/by-concept]] — Interview questions
