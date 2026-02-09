# Automatic Completion Triggers

This system ensures supporting materials are ALWAYS created when you complete a module - no manual prompting needed.

---

## The Problem

Without automatic triggers:
- ❌ You forget to create pitfall entries
- ❌ Rosetta stone has gaps
- ❌ Interview prep is incomplete
- ❌ Knowledge is lost

---

## The Solution: Automatic Prompts

When you complete a module, I will **automatically** prompt you to create supporting materials.

---

## Trigger Points

### Trigger 1: After Reading Lesson

**When:** You say "I finished reading the Go variables-and-types lesson"

**I automatically ask:**
```
Great! I notice you read about [key concepts]. Did you encounter:
- Any confusing behavior? → I can create a pitfall entry
- Operations that exist in other languages? → I can update Rosetta
- Questions you'd ask in an interview? → I can add to interview prep

Should I create these now or after you do the exercises?
```

---

### Trigger 2: After Completing Exercise

**When:** You say "I finished the config-loader exercise" or mark exercise as done

**I automatically ask:**
```
Exercise completed! Let me create supporting materials:

1. 🐛 Pitfalls - Did you hit any bugs?
   - Nil map panic?
   - Type confusion?
   → I can create vault/pitfalls/[issue].md

2. 📚 Rosetta - I can add these operations:
   - String parsing (strconv.Atoi)
   - Map initialization (make)
   → Update vault/rosetta/common-operations.md?

3. 💼 Interview - I can add questions:
   - "How do you parse strings in Go?"
   - "Why use make() for maps?"
   → Update vault/interview-prep/by-concept.md?

Proceed with all? (yes/no/pick)
```

---

### Trigger 3: After Marking Module Complete

**When:** You say "I'm done with variables-and-types for Go" or update progress.yaml

**I automatically create:**
```
Module completion detected! Creating materials:

✅ Creating:
1. Pitfall entries for [bugs you mentioned]
2. Rosetta updates for [operations used]
3. Interview questions for [concepts covered]
4. Cross-language vault note update

✅ Checking:
- vault/fundamentals/variables-and-types.md (updated Go section)
- vault/fundamentals/go/variables-and-types.md (exists? create?)
- progress.yaml (marked complete, confidence set)

Done! All materials created.
```

---

### Trigger 4: After Completing 2+ Languages

**When:** I detect you've completed the same module in multiple languages

**I automatically ask:**
```
You've now completed [module] in [Go, TypeScript]. Should I:

1. Update cross-language comparison table
2. Add side-by-side code examples
3. Highlight key differences in vault note
4. Update interview questions with comparisons

This will make the vault note much more valuable!
```

---

## Default Behavior (What I Do Automatically)

### When You Complete a Lesson:

**Without you asking, I will:**

1. **Scan lesson content** for:
   - Gotchas mentioned → pitfall candidates
   - Operations demonstrated → rosetta candidates
   - Key concepts → interview question candidates

2. **Proactively prompt:**
   ```
   I noticed these from the lesson:
   - Pitfall: nil map panic
   - Operations: map creation, string parsing
   - Key concepts: zero values, escape analysis

   Should I create:
   □ Pitfall entry: go-nil-map-panic.md
   □ Rosetta updates: 3 operations
   □ Interview questions: 5 questions

   (I recommend doing all three - takes 2 minutes)
   ```

3. **Create on confirmation** (not wait for explicit prompt)

---

## Command Shortcuts

### Quick Triggers

**After any module:**
```
done
```
→ I automatically run full completion checklist

```
done variables-and-types go confidence:4
```
→ I create all materials + mark complete

```
quick-complete
```
→ I create materials without asking (yes to all)

---

### Selective Triggers

```
pitfalls only
```
→ Just create pitfall entries

```
rosetta only
```
→ Just update Rosetta stone

```
interview only
```
→ Just add interview questions

---

## What Gets Created (Default)

### 1. Pitfall Entry (If Any Bug/Gotcha Mentioned)

**Automatic trigger:** You mention a bug, confusion, or "why does this..."

**I create:** `vault/pitfalls/<language>-<issue>.md`

**Contents:**
- The mistake (from your description)
- Why it happens (I explain mental model gap)
- The fix
- How to avoid
- Frequency rating
- Cross-links

**Example:**
```
You: "Why does my map panic when I write to it?"
Me: "That's the nil map issue. Creating pitfall entry..."
[Creates vault/pitfalls/go-nil-map-panic.md]
```

---

### 2. Rosetta Stone Update (Always for New Operations)

**Automatic trigger:** Lesson demonstrates operations

**I update:** `vault/rosetta/common-operations.md`

**I add rows for:**
- Operations you learned
- All 6 languages side-by-side
- Links to documentation

**Example:**
```
You: "Finished Go variables lesson"
Me: "Adding to Rosetta:
- String to int conversion
- Type declarations
- Map initialization"
[Updates vault/rosetta/common-operations.md with 3 rows]
```

---

### 3. Interview Questions (Always)

**Automatic trigger:** Module completion

**I update:** `vault/interview-prep/by-concept.md`

**I add:**
- Conceptual questions
- Language-specific questions
- Debugging scenarios
- LeetCode mappings (if applicable)

**Example:**
```
Me: "Adding interview questions:

Conceptual:
- Explain zero values and why Go uses them
- Compare stack vs heap allocation

Language-specific:
- Why can you read from nil map but not write?
- What is escape analysis?

Debugging:
- Why does this code panic? [nil map example]"
```

---

### 4. Cross-Language Vault Update (When 2+ Languages Done)

**Automatic trigger:** Same module completed in multiple languages

**I update:** `vault/fundamentals/<module>.md`

**I ensure:**
- Comparison table is complete
- All languages have examples
- Pitfalls section lists all languages
- Links to language-specific deep dives exist

---

## Session Flow with Auto-Triggers

### Example: Learning Go Variables-and-Types

**You:** "Let's start Go variables-and-types"

**Me:** [Creates lesson.md and reference.md if needed]

---

**You:** "I finished reading the lesson"

**Me (automatically):**
```
Great! I noticed from the lesson:
- Pitfall: nil map panic (mentioned on line 138)
- Operations: type declarations, map creation, string parsing
- Concepts: zero values, escape analysis, pointers

Should I create:
□ vault/pitfalls/go-nil-map-panic.md
□ Update vault/rosetta/common-operations.md (3 operations)
□ Update vault/interview-prep/by-concept.md (5 questions)

Yes to all? (or pick individual ones)
```

**You:** "yes"

**Me:** [Creates all three]

---

**You:** "I finished the config-loader exercise"

**Me (automatically):**
```
Exercise complete! Did you encounter:
- The nil map panic we discussed? → Already have pitfall entry ✓
- Any other bugs?

I'll add to Rosetta:
- Error handling with multiple returns
- Parsing with strconv package

And interview questions:
- "How do you parse environment variables safely?"
- "Why return (Config, error) instead of throwing?"

Creating now...
```

---

**You:** "Mark as complete"

**Me (automatically):**
```
Marking variables-and-types (Go) as complete.

Final checklist:
✅ Pitfall: go-nil-map-panic.md (created)
✅ Rosetta: 5 operations added
✅ Interview: 7 questions added
✅ Cross-lang vault: Updated Go section
✅ Progress: marked complete

What's your confidence level? (1-5)
```

**You:** "4"

**Me:**
```
Set to confidence: 4
Next review scheduled: 2026-02-15 (7 days, based on confidence:4)

Next up: Would you like to:
1. Do TypeScript variables-and-types (for comparison)
2. Do Go control-flow (next in sequence)
3. Do debugging exercise for nil map panic
```

---

## Behavioral Rules (Added to CLAUDE.md)

**I will automatically:**

1. **After lesson completion** → Prompt for pitfall/rosetta/interview materials
2. **After exercise completion** → Prompt for any bugs encountered + rosetta updates
3. **After marking complete** → Run full checklist, ensure nothing is missing
4. **After 2+ language completion** → Update cross-language comparison

**I will NOT wait for you to remember to ask.**

---

## Opt-Out

If you don't want auto-prompts for a session:

```
/no-auto
```

I'll only create materials when explicitly asked.

To re-enable:
```
/auto
```

Default is AUTO-ENABLED.

---

## What You Don't Need to Remember

❌ "Did I create a pitfall entry?"
❌ "Should I update Rosetta?"
❌ "What interview questions relate?"
❌ "Is my vault note complete?"

✅ **I will ask you automatically at the right times.**

---

## Summary

**Old way:**
- You finish module
- You forget to create materials
- Knowledge is lost

**New way (automatic):**
- You finish module
- I automatically prompt: "Create supporting materials?"
- You say yes
- Everything is created
- Nothing is lost

**The only thing you need to remember:**
```
Say "yes" when I prompt you.
```

(Or just say "done" and I'll create everything)
