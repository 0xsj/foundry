# Foundry

A learning repo for Go, TypeScript, Python, Scala 3 and Haskell, run from day one, built
around retyping code from memory rather than reading about it.

## Why it works this way

Reading a lesson feels like learning and mostly is not. The retention here comes from
one mechanic: you type the canonical example, then a session or two later you type it
again with the reference hidden, and a diff tells you exactly what you lost.

Everything else — the lessons, the mechanism deep-dives, the notes — exists to make
that example worth having in muscle memory.

## The loop

| Stage | You | Checked by |
|-------|-----|------------|
| Read | one screenful on what it is and why | — |
| Mechanism | what actually happens underneath | — |
| Type along | retype `example.go` with it visible | `foundry check` |
| Recall | retype it later, from memory | `foundry check` |
| Apply | build something small with it | tests |

## Starting a track

```
foundry focus go     # or ts, py, sc
```

Then open the repo in Claude Code and say `go`. That is the entire setup — there is no
per-module scaffolding step, and nothing to create by hand. The lesson, the example, the
drill schedule and the notes are all written during the session.

`foundry guide` prints the full manual, and it is offered automatically on a fresh track.

## Using it

Open the repo in Claude Code. A hook prints where you are:

```
foundry — go · tier 0 syntax
  now   00-values-and-types           read → mechanism → type-along
  due   2 recall drills               01-operators, 02-control-flow
  lc    two-pointers · 3 unpulled
```

Say `go`. That is the whole interface. Progress, drills, and notes are written for you
at the end of a session — there are no bookkeeping commands to remember.

## CLI

```
foundry guide               the full manual
foundry doctor              verify every track can actually run code
foundry status              where you are, what is due
foundry focus <lang>        pick the active track (go, ts, py)
foundry drill <lang> <mod>  set up an attempt (add --blind to hide the reference)
foundry check <lang> <mod>  diff your attempt against the reference
foundry lc                  dispense a leetcode problem
foundry iv                  dispense an interview question
foundry lc reset            wipe leetcode progress only
foundry iv reset            wipe interview progress only
```

`<lang>` is `go`, `ts`, `py` or `sc`. `<mod>` accepts a prefix, so `00` resolves to
`00-values-and-types`.

Build it once: `go build -o foundry ./cmd/foundry`

## Interview questions

`foundry iv` works the same way the drills do: it hands you a question and a blank file,
you write the answer out in full, and only then does `foundry iv ref` reveal the model
answer. It refuses to reveal if you have not written anything.

This is deliberate. Answering in your head feels like knowing; composing three
paragraphs under a question you have not seen is what an interview actually asks for,
and the gap between the two is the whole point.

## Toolchain

Pinned via `.nvmrc` (Node 24), `.python-version` (Python 3.14.7), `.sdkmanrc`
(Scala 3.8.4, JDK 25) and `.ghc-version` (GHC 9.14.1, installed with Homebrew). Go is
1.27. TypeScript runs under `bun`, Scala under `scala-cli` and Haskell under `runghc`,
so none of the three needs a build step.

`foundry doctor` compiles and runs a real program in all five:

```
go          ok   go version go1.27.1 darwin/arm64 (407ms)
typescript  ok   1.3.11 (89ms)
python      ok   Python 3.14.7 (149ms)
scala       ok   1.16.0 (881ms)
haskell     ok   9.14.1 (307ms)
```

GHC ships `text`, `bytestring`, `containers`, `mtl`, `transformers`, `stm`, `parsec`
and `deepseq` in its global package database, which covers tiers 0-4. The tier 5 builds
are the only modules that need `cabal` installed.

## Layout

```
foundry.json    curriculum spine, 25 modules per language across 6 tiers
tracks/         go/, typescript/, python/, scala/ — created as you reach them
leetcode/       97 problems, 15 patterns, standalone and resettable
interview/      31 questions, written answers, standalone and resettable
vault/          Obsidian notes, written as a side effect
state/          progress, drill schedule, session logs
```

## History

v1 of this repo generated 858 files across seven languages and recorded zero attempted
exercises. It is preserved at the `foundry-v1` tag. The lesson taken from it: content is
cheap to generate and expensive to consume, so nothing here is generated before you
reach it.
