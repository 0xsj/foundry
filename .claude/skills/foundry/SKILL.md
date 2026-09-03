---
name: foundry
description: Run a Foundry learning session end to end — pick up where the user left off, teach the next concept through read/mechanism/type-along, run any recall drills that are due, and write all progress, drills, and vault notes at the end without being asked. Use whenever the user says "go", "next", "start", "let's learn", or otherwise signals they want to work on Go, TypeScript, Python or Scala in this repo.
---

# Running a session

The user says one word. You run the whole thing. They should never type a second
command to make the session progress or to record what happened.

## 1. Orient

```
./foundry status
```

Build it first if the binary is missing: `go build -o foundry ./cmd/foundry`

If the user is switching language, `./foundry focus <go|ts|py|sc>` first — that one value
drives everything else. `./foundry guide` prints the full manual if they ask how
something works.

This tells you the focus language, the current module and its stage, and which recall
drills are due.

## 2. Recall drills first, if any are due

Due drills come before new material — this is the retention mechanic and skipping it
defeats the point.

For each due drill:

1. `./foundry drill <lang> <module> --blind`
2. Tell the user to type the example from memory into the `attempt/` path it prints.
   Do not show them the reference. Do not paste it into chat.
3. When they say they are done: `./foundry check <lang> <module>`
4. Read the diff. For every missing line, explain **why that line matters** — the
   consequence of its absence, not a restatement of it. A missing `defer` leaks; a
   missing `await` races; a missing capacity hint reallocates.
5. Record it: `--pass` if clean, `--lapse` if not.
6. Delete the `attempt/` file so the next drill starts empty.

## 3. Then the current module

Work the stages in order. One session normally covers 1-3 for a new module, or 5 if
type-along is already done.

**Stage 1 — Read.** Write `tracks/<lang>/<module>/lesson.md` if it does not exist. One
screenful. What the concept is, why the language has it, what breaks without it. Then
let the user read it and ask questions before moving on.

**Stage 2 — Mechanism.** Cover every entry in that module's `mechanism` array in
`foundry.json`. This is the part the user cares most about and the part tutorials skip.
If the example calls into the standard library, explain what that call actually does at
runtime — allocation, dispatch, syscalls, encoding.

**Stage 3 — Type along.** Write `example.go` or `example.ts` — 15-40 lines, realistic,
self-contained, every line earning its place. This file is what they will retype from
memory for months, so choose it carefully. Then `./foundry drill <lang> <module>`,
they type it with the reference visible, `./foundry check` confirms.

**Stage 5 — Apply.** Write `apply/` with a small realistic scenario and a test suite.
The user implements; tests verify. Never hand them the implementation first.

**Stage 6 — Variants.** Run `./foundry variants <lang> <module>` to see what is recorded.

**When a second credible approach comes up mid-session — in any module, not just the
ones that ship with variants — record it before moving on:**

```
./foundry variants add <lang> <module> "<approach>" "<when it wins>"
```

The first variant opens stage 6 for that module. This is the same organic-growth rule as
vault notes: capture it when it happens, because you will not come back for it. Write `variants/` with a working
implementation per approach plus a decision table in `variants/README.md`. Then have the
user write `variants/my-take.md` — which approach they would pick for a scenario you
give them, and why — **before** you share your reasoning. Review it afterwards and push
back if the argument is weak. Never frame one variant as correct and the others as
mistakes.

## 3b. Optional: leetcode or an interview question

If the user wants a change of gear, or the module work finishes early:

- `./foundry lc next` — dispenses a pattern-tagged problem. Name the pattern before
  they start coding; the pattern is what transfers, the problem is not.
- `./foundry iv next` — dispenses an interview question and creates a stub answer file.
  Have them **write** the answer out, not say it. Then `./foundry iv ref` reveals the
  reference and you review the gap: what was missing, what was vague, which follow-up
  probe they would have failed. Never accept a verbal answer in chat instead.

## 4. Close out — always, unprompted

Before the session ends:

1. `./foundry stage <lang> <module> <0-5>` — records the stage and dates. Never
   hand-edit `state/progress.json`.
2. `./foundry drill <lang> <module> --pass` — schedules the stage-4 recall for
   anything that reached stage 3.
3. `vault/` — write or extend a note if the session produced a real insight, a bug the
   user hit, or a Go/TS difference worth keeping. Skip it if nothing did; empty notes
   are worse than no notes.
4. `state/sessions/YYYY-MM-DD.md` — append what was covered and where to resume

Then give the user a three-line summary: what was covered, what is scheduled, what is
next.

## Rules

- **Generate only this session's material.** Never scaffold ahead. This is the failure
  mode that killed v1 — 858 files, zero attempts.
- **No quizzing.** Retention runs through typed recall. Do not ask trivia questions.
- **Day one means day one.** In tier 0, assume nothing. Loops are not known in
  module 00 unless module 00 taught them.
- **Four languages, no more.** Go and TypeScript are primaries; Python and Scala 3 are
  the third and fourth tracks. The tiers are parallel across all four, so compare freely.
  Scala is the useful contrast: expressions over statements, immutable by default, type
  classes instead of interfaces.
- **Realistic code only.** Config loaders, retry logic, log parsers, rate limiters.
  Never `Animal`, `Shape`, or `foo`.
- **Compare the two languages** whenever the same concept differs between them.
