# Vault Structure Migration Guide

This document explains the new vault organization (Option 1: Language Subdirectories) and how to migrate existing notes.

---

## What Changed

### Before (Old Structure)

```
vault/
├── fundamentals/
│   ├── variables-and-types.md           # Cross-language
│   ├── variables-and-types-go.md        # Go-specific (suffix)
│   └── variables-and-types-typescript.md # TS-specific (suffix)
```

**Problems:**
- Flat structure gets cluttered with many languages
- `-suffix` convention unclear (is it language or variant?)
- Hard to see what's language-specific vs cross-language

---

### After (New Structure)

```
vault/
├── fundamentals/
│   ├── README.md                        # Index of concepts
│   ├── variables-and-types.md           # Cross-language comparison
│   ├── go/
│   │   └── variables-and-types.md       # Go deep dive
│   ├── typescript/
│   │   └── variables-and-types.md       # TS deep dive
│   └── rust/
│       └── variables-and-types.md       # Rust deep dive
```

**Benefits:**
- Clear hierarchy (top = cross-lang, subdirs = language-specific)
- Scales cleanly as more languages/concepts are added
- Obsidian graph shows language clusters
- Easy to find all notes for a specific language

---

## Directory Structure

### Full Layout

```
vault/
├── fundamentals/
│   ├── README.md                        # Index
│   ├── <concept>.md                     # Cross-language comparisons
│   ├── go/                              # Go-specific deep dives
│   │   ├── <concept>.md
│   │   └── <go-unique-concept>.md       # e.g., goroutines.md
│   ├── typescript/
│   │   ├── <concept>.md
│   │   └── <ts-unique-concept>.md       # e.g., decorators.md
│   ├── rust/
│   ├── python/
│   ├── java/
│   └── csharp/
├── patterns/
│   ├── <pattern>.md                     # Cross-language comparison
│   ├── go/
│   │   └── <pattern>.md                 # Go implementation
│   ├── typescript/
│   └── ...
├── pitfalls/                            # Flat (always language-specific)
│   ├── go-nil-map-panic.md
│   ├── typescript-implicit-any.md
│   └── rust-borrow-checker.md
├── rosetta/
│   └── common-operations.md             # Always cross-language
├── interview-prep/
│   └── by-concept.md                    # Always cross-language
└── examples/
    └── production-patterns/             # Organized by pattern, not language
```

---

## Migration Steps

### 1. Create Directory Structure

```bash
cd vault

# Create language subdirectories
mkdir -p fundamentals/{go,typescript,rust,python,java,csharp}
mkdir -p patterns/{go,typescript,rust,python,java,csharp}
```

### 2. Move Existing Files

```bash
# Move language-specific notes to subdirectories
mv fundamentals/variables-and-types-go.md \
   fundamentals/go/variables-and-types.md

# Keep cross-language notes at top level
# (variables-and-types.md stays where it is)
```

### 3. Update Links

**Find and replace:**
```bash
# Find all references to old structure
grep -r "\[\[variables-and-types-go\]\]" .

# Replace with new structure
# [[variables-and-types-go]] → [[go/variables-and-types]]
```

**In files:**
- Old: `[[variables-and-types-go]]`
- New: `[[go/variables-and-types]]` or `[[fundamentals/go/variables-and-types]]`

### 4. Update Frontmatter

**Cross-language notes** (top level):
```yaml
---
title: Variables and Types (Cross-Language Comparison)
languages: [go, typescript, rust, python, java, csharp]
related: [[memory-and-ownership]], [[go/variables-and-types]]
---
```

**Language-specific notes** (subdirectories):
```yaml
---
title: Variables and Types (Go Deep Dive)
languages: [go]
related: [[fundamentals/variables-and-types]], [[typescript/variables-and-types]]
---

> **Cross-Language Comparison:** See [[fundamentals/variables-and-types]]
```

### 5. Update Index Files

Create `vault/fundamentals/README.md` with:
- List of all concepts
- Completion status table
- Navigation guide

---

## Linking Conventions

### Absolute Paths (Recommended)

From anywhere:
```markdown
[[fundamentals/variables-and-types]]         # Cross-language
[[fundamentals/go/variables-and-types]]      # Go deep dive
[[patterns/strategy]]                        # Pattern comparison
[[patterns/go/strategy]]                     # Go implementation
[[pitfalls/go-nil-map-panic]]                # Pitfall
```

### Relative Paths (Alternative)

From `vault/fundamentals/go/variables-and-types.md`:
```markdown
[[../variables-and-types]]                   # Parent (cross-lang)
[[../typescript/variables-and-types]]        # Sibling language
```

From `vault/fundamentals/variables-and-types.md`:
```markdown
[[go/variables-and-types]]                   # Child (language-specific)
```

**Recommendation:** Use absolute paths for clarity.

---

## When to Create Each Type

### Cross-Language Note (Top Level)

**Create at:** `vault/<category>/<concept>.md`

**When:**
- Concept exists in 2+ languages
- Comparing implementations is valuable
- Need a hub that links to all language-specific notes

**Example:** `vault/fundamentals/variables-and-types.md`

**Contains:**
- Overview
- Language comparison table
- Brief idioms per language
- Links to language-specific deep dives

---

### Language-Specific Note (Subdirectory)

**Create at:** `vault/<category>/<language>/<concept>.md`

**When:**
- Need detailed explanation of language-specific behavior
- Concept is unique to that language (e.g., goroutines in Go)
- Have extensive "Your notes" that don't fit in cross-language comparison

**Example:** `vault/fundamentals/go/variables-and-types.md`

**Contains:**
- Deep dive on language semantics
- Extensive code examples
- Language-specific insights
- "Your notes" from learning
- Link back to cross-language comparison

---

## Examples

### Complete Structure for Variables-and-Types

```
vault/fundamentals/
├── variables-and-types.md              # Hub: all 6 languages compared
│   └── Links to: [[go/variables-and-types]], [[typescript/variables-and-types]], etc.
├── go/
│   └── variables-and-types.md          # Deep dive
│       ├── Escape analysis (Go-specific)
│       ├── String header internals
│       ├── Zero value philosophy
│       └── Links back to [[fundamentals/variables-and-types]]
├── typescript/
│   └── variables-and-types.md          # Deep dive
│       ├── Type erasure at runtime
│       ├── var/let/const in depth
│       └── Links back to [[fundamentals/variables-and-types]]
└── rust/
    └── variables-and-types.md          # Deep dive
        ├── Ownership details
        ├── Borrowing rules
        └── Links back to [[fundamentals/variables-and-types]]
```

---

## Obsidian Setup

### Graph View

With this structure, the graph will show:
- **Central hubs**: Cross-language comparison notes
- **Clusters**: Language-specific notes grouped by language
- **Connections**: Links between related concepts

### File Explorer

```
vault/
├── fundamentals/
│   ├── 📄 variables-and-types.md       # Cross-language
│   ├── 📁 go/
│   │   └── 📄 variables-and-types.md
│   ├── 📁 typescript/
│   │   └── 📄 variables-and-types.md
│   └── ...
```

### Search

- **All fundamentals**: Click `fundamentals/` folder
- **All Go notes**: Click `fundamentals/go/` folder
- **Specific concept**: Search "variables-and-types"

---

## Checklist

After migration, verify:

- [ ] All language-specific notes moved to subdirectories
- [ ] Cross-language notes remain at top level
- [ ] Links updated to new paths
- [ ] Frontmatter includes `related:` field with both cross-lang and language-specific
- [ ] Cross-language notes link to language-specific deep dives
- [ ] Language-specific notes link back to cross-language comparison
- [ ] README.md exists in each category
- [ ] Obsidian graph shows clean clusters

---

## Future Additions

When creating new content:

1. **Start with cross-language note**
   ```bash
   # Create comparison hub
   touch vault/fundamentals/control-flow.md
   ```

2. **Add language-specific notes as you learn each language**
   ```bash
   # After learning Go control-flow
   touch vault/fundamentals/go/control-flow.md

   # After learning TypeScript control-flow
   touch vault/fundamentals/typescript/control-flow.md
   ```

3. **Link bidirectionally**
   - Cross-lang → language-specific
   - Language-specific → cross-lang
   - Update both frontmatter `related:` fields

4. **Update README.md**
   - Add concept to table
   - Mark completion status per language

---

This structure will scale cleanly as you add more languages and concepts!
