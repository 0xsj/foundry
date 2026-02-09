# Learning Workflow — How to Use Foundry

This guide shows the recommended order for working through any module. Follow these steps to get the most out of the system.

---

## Quick Reference

**For a new module (first time learning):**
1. Read `lesson.md` (30-45 min)
2. Skim `reference.md` (5-10 min, bookmark for later)
3. Do exercises in `starter/` (30-60 min per exercise)
4. Compare with `solutions/` (10-15 min)
5. Read variants + solution README (5-10 min)
6. Write review notes in `my-solution/review.md` (5 min)
7. (Optional) Do debugging exercise (20-30 min)
8. (Optional) Do refactoring exercise (30-45 min)
9. Update progress.yaml with confidence level

**For review (revisiting a completed module):**
1. Skim vault note `vault/<category>/<module>.md` (5-10 min)
2. Review pitfalls section (2-3 min)
3. Try to explain concepts out loud (5 min)
4. Update confidence in progress.yaml

**For interview prep:**
1. Check `vault/interview-prep/by-concept.md` for the module
2. Review pitfalls (common gotchas to discuss)
3. Try LeetCode problems (if applicable)
4. Practice explaining tradeoffs

---

## Full Workflow for Learning a New Module

### Phase 1: Understanding (Foundation)

#### Step 1: Read the Lesson (30-45 minutes)

**File**: `tracks/<language>/fundamentals/<module>/lesson.md`

**What it is:**
- Tutorial-style teaching
- Mental models and "how it works under the hood"
- Comparisons to other languages (especially JS/TS for you)
- "Your notes" sections for personal insights

**How to use it:**
```
1. Read through completely (don't skip sections)
2. Run code examples in your editor/terminal
3. When you see [[wiki-links]], click to explore (or bookmark for later)
4. Fill in "Your notes" sections with:
   - Aha moments
   - Connections to other languages
   - Questions you still have
   - Things that confused you initially
```

**Example notes:**
```markdown
### Your notes
- Zero values remind me of JS default parameters, but more pervasive
- The nil map panic is similar to undefined property access in JS but caught differently
- Question: When does escape analysis actually matter for performance?
```

**Time**: 30-45 minutes for focused reading + note-taking

---

#### Step 2: Skim the Reference (5-10 minutes)

**File**: `tracks/<language>/fundamentals/<module>/reference.md`

**What it is:**
- Official language spec extracts
- Tables, syntax rules, edge cases
- Authoritative source for exact behavior

**How to use it:**
```
1. Skim headings to see what's covered
2. Read tables (type sizes, ranges, defaults)
3. Bookmark for later reference
4. Don't try to memorize — this is a lookup resource
```

**When to return to it:**
- During exercises when you need exact syntax
- When debugging weird behavior
- When you wonder "what's the official rule?"
- Before interviews (to review specs)

**Time**: 5-10 minutes for first pass, come back as needed

---

### Phase 2: Practice (Application)

#### Step 3: Do the Main Exercise (30-60 minutes)

**Location**: `tracks/<language>/exercises/<module>/01-<name>/`

**Order:**
1. **Read README.md** (5 min)
   - Understand the scenario (realistic context)
   - Read acceptance criteria
   - Note constraints

2. **Look at starter code** (5 min)
   - See the scaffold
   - Read test file to understand expected behavior
   - Run tests to see them fail: `go test` or `npm test`

3. **Implement solution** (30-45 min)
   - Work in `starter/` directory
   - Start with the TODOs
   - Run tests frequently
   - Use hints if stuck (but try first!)

4. **Verify tests pass** (1 min)
   ```bash
   cd starter
   go test -v
   ```

5. **Copy your work to my-solution/** (1 min)
   ```bash
   cp starter/*.go ../my-solution/
   ```

**Tips:**
- Don't look at solutions yet!
- Get tests passing first, optimize later
- If stuck for >20 minutes, peek at a hint
- If still stuck, peek at solution approach (not full code)

---

#### Step 4: Compare with Solutions (10-15 minutes)

**Location**: `solutions/`

**Order:**
1. **Read solutions/config.go** (5 min)
   - How does it differ from yours?
   - Is it simpler? More complex?
   - Note any patterns you didn't use

2. **Read solutions/README.md** (5 min)
   - Understand the approach
   - Read key decisions section
   - Check complexity analysis
   - See trade-offs discussed

3. **Look at variants/** (3-5 min)
   - Alternative approaches
   - When each is appropriate
   - Pros/cons comparison

**Key questions to answer:**
- [ ] Did I overcomplicate anything?
- [ ] Did I miss an edge case?
- [ ] Is my error handling clear?
- [ ] Would I do anything differently now?

---

#### Step 5: Write Review Notes (5 minutes)

**File**: `my-solution/review.md`

**Template provided at**: `vault/templates/review.md`

**Fill in:**
```markdown
## My Approach
[What strategy did you use?]

## What Worked Well
- Pattern X made error handling clean
- Helper function Y reduced duplication

## Challenges
- Parsing edge cases took trial-and-error
- Initially forgot to check for empty string

## Comparison to Reference
Similarities:
- Both used early returns for errors
Differences:
- I used a single parse function, reference used separate helpers
- Reference had better error messages

## Lessons Learned
- Always validate after parsing, not during
- Helper functions are worth it even for 2 uses
- Error messages should include the invalid value

## Next Time
- Write tests for edge cases first
- Use helper functions earlier
```

**Why this matters:**
- Solidifies learning through reflection
- Creates a reference for future you
- Identifies patterns to apply elsewhere

---

### Phase 3: Reinforcement (Deepening)

#### Step 6: Debugging Exercise (Optional, 20-30 minutes)

**Location**: `tracks/<language>/exercises/debugging/<scenario>/`

**When to do it:**
- After completing main exercise
- When you've encountered this bug before
- When preparing for interviews (debugging questions)

**Order:**
1. **Read README.md** (3 min)
   - See symptoms (not the solution!)
   - Understand context

2. **Read buggy code** (5 min)
   - Form hypotheses
   - Document in README "Your Debugging Process"

3. **Try to fix it** (10-15 min)
   - Test hypothesis
   - Run tests to verify
   - Document what you tried

4. **Compare with solution.md** (5 min)
   - Was your diagnosis correct?
   - Learn the root cause
   - Understand prevention strategies

5. **Read related pitfall entry** (5 min)
   - Click the [[wiki-link]] in solution.md
   - See production impact
   - Learn how to avoid in future

**Value:**
- Builds debugging intuition
- Teaches common pitfalls
- Simpler than building from scratch

---

#### Step 7: Refactoring Exercise (Optional, 30-45 minutes)

**Location**: `tracks/<language>/exercises/refactoring/<scenario>/`

**When to do it:**
- After understanding the basics
- When learning design patterns
- When preparing for senior-level interviews

**Order:**
1. **Read README.md** (5 min)
   - Understand context
   - Review code smells checklist

2. **Read before.go and tests** (10 min)
   - Identify smells (check them off)
   - Run tests to verify they pass
   - Document what smells you found

3. **Plan refactoring** (5 min)
   - What patterns could help?
   - What order to refactor?
   - Document your plan

4. **Refactor incrementally** (20-30 min)
   - One change at a time
   - Run tests after each change
   - Commit after each successful refactor

5. **Compare with after.go** (5 min)
   - What's different?
   - What patterns were applied?
   - Read analysis.md for explanation

**Value:**
- Teaches code smell identification
- Practices design patterns in context
- Closer to real-world work than toy examples

---

### Phase 4: Integration (Connecting)

#### Step 8: Explore Cross-References (10-15 minutes)

**Locations to visit:**

1. **Vault note** (5 min)
   - `vault/<category>/<module>.md`
   - See all languages compared
   - Review Common Pitfalls section
   - Click [[wiki-links]] to explore

2. **Pitfalls** (3-5 min per pitfall)
   - `vault/pitfalls/<language-concept>.md`
   - Read any that were mentioned in lesson
   - Bookmark for future reference

3. **Interview prep** (3 min)
   - `vault/interview-prep/by-concept.md`
   - See what questions map to this module
   - Bookmark LeetCode problems if any

4. **Production examples** (5-10 min)
   - `vault/examples/production-patterns/<pattern>/`
   - See how real projects use these concepts
   - Note production-grade practices

**Why this matters:**
- Connects isolated knowledge
- Shows real-world application
- Prepares for interviews
- Builds cross-language understanding

---

#### Step 9: Update Progress (2 minutes)

**File**: `curriculum/progress.yaml`

```yaml
go:
  fundamentals:
    variables-and-types:
      status: completed
      completed_date: 2026-02-08
      last_reviewed: 2026-02-08
      review_count: 1
      next_review: 2026-02-15        # +7 days (algorithm)
      confidence: 4                   # 1=shaky, 5=solid
      exercises_done: [config_loader, nil_map_debug]
      time_spent_minutes: 90
```

**Set confidence level:**
- **1 (Shaky)**: "I don't really get it yet"
- **2 (Uncertain)**: "I understand basics but confused on details"
- **3 (Okay)**: "I can use it but not explain it well"
- **4 (Solid)**: "I can explain it and use it confidently"
- **5 (Expert)**: "I could teach this to someone else"

**This affects:**
- When your next review is scheduled
- What the system suggests next
- Your overall progress tracking

---

## Review Workflow (Completed Modules)

When `next_review` date arrives or you want to refresh:

### Quick Review (15-20 minutes)

1. **Read vault note** (5-10 min)
   - `vault/<category>/<module>.md`
   - Skim language comparison
   - Review Common Pitfalls

2. **Review your notes** (3-5 min)
   - Re-read "Your notes" sections in lesson.md
   - Look at my-solution/review.md

3. **Explain out loud** (5 min)
   - Pretend you're teaching someone
   - Can you explain:
     - What the concept is?
     - Why it matters?
     - Common pitfalls?
     - When to use it?

4. **Update progress.yaml** (1 min)
   - Increment review_count
   - Update last_reviewed date
   - Adjust confidence if needed

### Deep Review (If Confidence < 3)

1. **Re-read lesson.md** (20 min)
2. **Redo exercises from scratch** (30 min)
3. **Read pitfalls again** (5 min)
4. **Compare to other languages** in vault note (10 min)

---

## Interview Prep Workflow

Before interviews focusing on this topic:

1. **Read vault note** (5 min)
   - All language comparison
   - Common Pitfalls section

2. **Check interview-prep/by-concept.md** (5 min)
   - What questions map to this module?
   - What talking points should you hit?

3. **Review pitfalls** (10 min)
   - Can you explain why each happens?
   - Do you know the fix?
   - Can you discuss production impact?

4. **Try LeetCode problems** (if applicable)
   - Do the problems listed
   - Time yourself
   - Explain your approach out loud

5. **Practice explaining** (10 min)
   - Record yourself or talk to a friend
   - Cover: what, why, when, tradeoffs
   - Practice saying "I don't know but here's how I'd find out"

---

## Time Budgets

### Minimal Path (Core Concepts Only)
- Lesson: 30 min
- Reference: 5 min (skim)
- Main exercise: 30 min
- Solution comparison: 10 min
- **Total: ~75 minutes**

### Standard Path (Recommended)
- Lesson + notes: 45 min
- Reference: 10 min
- Main exercise: 45 min
- Solution + variants: 15 min
- Review notes: 5 min
- Debugging exercise: 25 min
- Cross-references: 10 min
- Progress update: 2 min
- **Total: ~2.5 hours**

### Deep Dive Path (Mastery)
- Standard path: 2.5 hours
- Refactoring exercise: 45 min
- Production examples: 15 min
- Interview prep review: 10 min
- Practice explaining: 10 min
- **Total: ~4 hours**

**Recommendation:** Do Standard Path for primary languages (Go, TypeScript), Minimal Path for others.

---

## Tips for Success

### Don't Skip Steps
- ❌ Don't look at solutions before trying
- ❌ Don't skip writing review notes
- ❌ Don't ignore the reference.md completely
- ✅ Do follow the order
- ✅ Do take time to understand, not just memorize
- ✅ Do write notes in your own words

### When Stuck
1. **First 10 min**: Try on your own
2. **Next 10 min**: Look at a hint
3. **Next 10 min**: Google or search the vault
4. **After 30 min**: Look at solution approach (not code)
5. **After 45 min**: Read solution code, understand it, implement yourself

### Pacing
- **One module per language per day** = sustainable
- **One module across all 5 languages per week** = thorough
- **Don't rush** — depth > speed

### Note-Taking
- Use "Your notes" sections liberally
- Write in your own words (not copy/paste)
- Draw diagrams if helpful
- Compare to languages you know well

### Spaced Repetition
- Trust the review schedule
- Don't cram — spread out over weeks
- Revisit when notified
- Adjust confidence honestly

---

## Visual Workflow

```
START
  ↓
┌─────────────────┐
│  Read lesson.md │  30-45 min
│  + take notes   │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Skim reference  │  5-10 min
│  (bookmark it)  │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Do main exercise│  30-60 min
│  in starter/    │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Compare with    │  10-15 min
│  solutions/     │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Write review    │  5 min
│  notes          │
└────────┬────────┘
         ↓
    ┌────┴────┐
    │ Optional │
    └────┬────┘
         ↓
┌─────────────────┐
│  Do debugging   │  20-30 min
│   exercise      │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Do refactoring  │  30-45 min
│   exercise      │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Explore vault,  │  10-15 min
│ pitfalls, etc.  │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Update progress │  2 min
│  + confidence   │
└────────┬────────┘
         ↓
       DONE
         ↓
  (Schedule next review)
```

---

## Quick Checklist

Print this or keep it visible:

**For Each New Module:**
- [ ] Read lesson.md with notes
- [ ] Skim reference.md
- [ ] Do main exercise from starter
- [ ] Compare with solutions
- [ ] Write review notes
- [ ] (Optional) Do debugging exercise
- [ ] (Optional) Do refactoring exercise
- [ ] Explore cross-references
- [ ] Update progress.yaml with confidence

**For Review:**
- [ ] Read vault note
- [ ] Review pitfalls
- [ ] Explain out loud
- [ ] Update progress.yaml

**Before Interviews:**
- [ ] Review vault note + pitfalls
- [ ] Check interview-prep mapping
- [ ] Try LeetCode problems
- [ ] Practice explaining

---

## FAQs

**Q: Can I skip the debugging/refactoring exercises?**
A: Yes, they're optional. Do them if you want extra practice or are preparing for interviews.

**Q: Should I do all 5 languages at once?**
A: No. Complete one language fully, then move to the next. Otherwise, you'll get confused.

**Q: How long should a module take?**
A: 2.5 hours (Standard Path) per language. So variables-and-types across 5 languages = ~12 hours total, spread over a week or two.

**Q: What if I already know the concept?**
A: Do the Minimal Path (75 min) to fill gaps and see language-specific gotchas.

**Q: When do I move to the next module?**
A: When confidence >= 3 and you've completed at least one exercise.

**Q: What if the reference.md is overwhelming?**
A: Don't read it all at once. Skim it, bookmark it, return when you need specifics.

---

Save this file and refer back whenever you start a new module!
