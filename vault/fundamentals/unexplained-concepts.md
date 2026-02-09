---
title: Unexplained Concepts
category: fundamentals
tags: [meta, gaps, TODO]
languages: [go, typescript, rust, python, java]
status: in-progress
created: 2026-02-07
updated: 2026-02-07
related: [[variables-and-types-go]], [[variables-and-types-typescript]], [[variables-and-types-rust]]
---

# Unexplained Concepts

This file tracks concepts that appear in lessons before their formal module. It's a living staging area for preview syntax and accumulated questions.

**Purpose:** You can't teach variables without using functions, if statements, or loops. Strict linearity is pedagogically limiting. This file embraces spiral learning — we use realistic code and track what needs explanation later.

**Workflow:**
1. When teaching a concept, use realistic code even if it requires unfamiliar syntax
2. Add a preview note in the lesson: `// covered in detail in control-flow module`
3. Add an entry here with basic syntax and open questions
4. When the formal module is covered, remove from this file and ensure questions are answered in the proper vault note

**This file should shrink over time as concepts are formally covered.**

---

## 1. Print Functions / Macros

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | `fmt.Println`, `fmt.Printf`, `fmt.Sprintf` | variables.go, memory.go, config_loader.go |
| Rust       | `println!` | variables.rs, memory.rs |
| TypeScript | `console.log` | variables.ts, memory.ts |

**What's unexplained:**
- Go: Why is printing in a package (`fmt`) rather than built-in? What are format verbs (`%v`, `%d`, `%s`, `%p`, `%T`, `%q`, `%w`)? Why does `Printf` need a newline but `Println` doesn't?
- Rust: Why is `println!` a macro (`!`) and not a function? What are format specifiers (`{}`, `{:?}`, `{:p}`)? Why can't a function do what `println!` does?
- TypeScript: What is `console`? It's a global provided by the runtime (browser/Node), not a language primitive. Template literals (`` `${x}` ``) used in memory.ts without discussion.

**De-mystify:** Print functions need to accept any number of arguments of any type. Go solves this with `interface{}` + reflection. Rust solves it with a macro that expands at compile time (macros can do things functions can't, like accept variable argument counts with type checking). JS just has dynamic typing.

**Own module?** No dedicated module. Covered naturally in [[modules-and-packages]] (Go/Rust import system) and when Rust macros come up in [[interfaces-and-traits]] or a future macros module.

---

## 2. Import / Module System

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | `import "fmt"`, `import "unsafe"`, `import "strconv"` | all Go files |
| Rust       | `use std::mem`, `use std::collections::HashMap` | memory.rs, config_loader.rs |
| TypeScript | `export { Config, defaults, loadConfig }` | config_loader.ts |

**What's unexplained:**
- Go: How does Go resolve `"fmt"`? What's the difference between standard library imports and third-party? What is `go.mod`?
- Rust: What is `std`? What does `use` actually do (path shortening, not loading)? How is `use` different from `mod`? What's a crate?
- TypeScript: ESM (`import`/`export`) vs CJS (`require`/`module.exports`). We used `export` in config_loader.ts without explaining module resolution.

**De-mystify:** Every language needs a way to split code into files and pull in dependencies. The mechanisms differ but the goal is the same: namespacing, visibility control, and dependency management.

**Own module?** Yes: [[modules-and-packages]] (tier 1, prereq: variables-and-types).

---

## 3. Standard Library Functions

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | `unsafe.Sizeof()`, `strconv.Atoi()`, `strconv.ParseFloat()`, `strconv.ParseBool()`, `fmt.Errorf()` | memory.go, config_loader.go |
| Rust       | `std::mem::size_of_val()`, `.parse()`, `.to_string()`, `.as_ptr()` | memory.rs, variables.rs, config_loader.rs |
| TypeScript | `typeof`, `parseInt()`, `parseFloat()`, `Number.isNaN()`, `JSON.stringify()`, `Object.freeze()`, `structuredClone()` | memory.ts, config_loader.ts |

**What's unexplained:**
- Go: `unsafe` package — why is it called "unsafe"? `strconv` — why is string conversion its own package? `fmt.Errorf` with `%w` for error wrapping.
- Rust: `.parse()` returns a `Result` — we mentioned this in the exercise hints but never taught Result. `.as_ptr()` gives raw pointer access.
- TypeScript: `typeof` is an operator, not a function. `parseInt` takes a radix parameter (we used `10`). `Number.isNaN` vs global `isNaN` (different behavior).

**De-mystify:** Standard libraries are curated collections of commonly-needed functionality. Each language organizes them differently — Go has small focused packages, Rust has `std` with nested modules, TypeScript inherits globals from JavaScript plus browser/Node APIs.

**Own module?** No single module — these get covered as they come up. `unsafe` gets a mention in [[memory-and-ownership]]. `strconv` / `parse` covered in exercises naturally. Error wrapping in [[error-handling]].

---

## 4. String Methods / Operations

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | `string()` conversion, `[]byte()` conversion | mentioned conceptually in memory.go |
| Rust       | `String::from()`, `.to_string()`, `.push_str()`, `.as_ptr()`, `"hello".to_string()` | variables.rs, memory.rs |
| TypeScript | Template literals (`` `${x}` ``), `JSON.stringify()` | memory.ts |

**What's unexplained:**
- Rust: Why two string types (`String` vs `&str`)? We covered this in variables.rs section 6 but didn't explain the method syntax — what is `String::from()` (associated function) vs `.to_string()` (method on a trait)?
- Go: String immutability — `[]byte()` conversion creates a copy. Why?
- TypeScript: Template literals are syntactic sugar but we never said for what.

**De-mystify:** Strings are surprisingly complex because they involve ownership (who frees the memory?), encoding (UTF-8 vs UTF-16), and the stack/heap boundary. Rust makes all of this explicit, Go hides it behind a simple immutable type, and JS abstracts it away entirely.

**Own module?** Covered incrementally. String types are part of [[variables-and-types]]. String methods come up in exercises. Rust's `String` vs `&str` is deeply connected to [[memory-and-ownership]].

---

## 5. Structs / Interfaces / Type Definitions

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | `type Config struct { ... }`, `type Point struct { ... }` | config_loader.go, memory.go |
| Rust       | `pub struct Config { ... }` | config_loader.rs |
| TypeScript | `interface Config { ... }`, `interface User { ... }` | config_loader.ts, memory.ts |

**What's unexplained:**
- Go: `type` keyword for defining custom types. Struct literal syntax (`Config{Host: "...", Port: 8080}`). Why exported fields are capitalized.
- Rust: `pub` visibility modifier. Field-level `pub`. Struct instantiation syntax.
- TypeScript: `interface` vs `type` — we used `interface` but never discussed when you'd pick one over the other. `Record<string, string>` utility type.

**De-mystify:** All three languages need a way to group related data together. Go and Rust use structs (value types with named fields). TypeScript uses interfaces (structural type descriptions that are erased at runtime). The syntax differs but the concept is the same: define a shape.

**Own module?** Partially covered in [[variables-and-types]] (composite types). Deeper treatment in [[interfaces-and-traits]].

---

## 6. Derive Macros / Attributes (Rust)

| What we used | Where |
| ------------ | ----- |
| `#[derive(Debug, PartialEq)]` | config_loader.rs |

**What's unexplained:**
- What is `#[derive(...)]`? It's an attribute that auto-generates trait implementations at compile time.
- What is `Debug`? It lets you print a struct with `{:?}`.
- What is `PartialEq`? It lets you compare structs with `==`.
- Why do you have to opt in? Rust doesn't give you anything for free — you must explicitly request capabilities.

**De-mystify:** `derive` is Rust's way of saying "auto-implement these common traits for this type." It's code generation at compile time. Go gives you some of these for free (structs are comparable if all fields are). TypeScript/JS objects have no structural equality built in.

**Own module?** Covered in [[interfaces-and-traits]] (traits, trait implementations, derive).

---

## 7. Error Handling Patterns (Preview)

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | Multiple return `(Config, error)`, `fmt.Errorf("...: %w", err)`, `if err != nil` | config_loader.go |
| Rust       | `Result<Config, String>`, `todo!()`, `.parse()` returns Result | config_loader.rs |
| TypeScript | `throw new Error(...)`, explicit validation checks | config_loader.ts |

**What's unexplained:**
- Go: Why does Go return errors as values instead of throwing? What is the `error` interface? What does `%w` do in `fmt.Errorf`?
- Rust: What is `Result<T, E>`? What is `todo!()` (a macro that panics with "not yet implemented")? How do you handle a Result without `.unwrap()`?
- TypeScript: `throw` vs returning errors. No compile-time indication that a function can throw.

**De-mystify:** Languages handle failure in fundamentally different ways. Go and Rust make errors explicit in the return type — you can't ignore them without trying. TypeScript/Java use exceptions that propagate invisibly up the stack. Each has tradeoffs for readability, safety, and composability.

**Own module?** Yes: [[error-handling]] (tier 2, prereq: functions-and-closures).

---

## 8. Testing Conventions (Preview)

| Language   | What we mentioned | Where |
| ---------- | ----------------- | ----- |
| Go         | `_test.go` files, `testing.T` | config_loader exercise acceptance criteria |
| Rust       | `#[cfg(test)]`, `#[test]`, `assert_eq!` | config_loader exercise acceptance criteria |
| TypeScript | jest/vitest, `describe`/`it`/`expect` | config_loader exercise acceptance criteria |

**What's unexplained:**
- Go: Why `_test.go` naming convention? What is `testing.T`? What are table-driven tests?
- Rust: What is `#[cfg(test)]` (conditional compilation)? What is `#[test]` (test attribute)? `assert_eq!` is a macro — why?
- TypeScript: How do test runners discover and execute tests? What is the `describe`/`it`/`expect` pattern (BDD-style)?

**De-mystify:** Every language has conventions for organizing and running tests. Go bakes testing into the toolchain (`go test`). Rust bakes it into the language (`#[test]`). TypeScript relies on third-party runners (Jest, Vitest) since JS has no built-in test framework.

**Own module?** Yes: [[testing-fundamentals]] (tier 2, prereq: functions-and-closures, error-handling).

---

## 9. Trait Implementations (Rust)

| What we used | Where |
| ------------ | ----- |
| `impl Default for Config { fn default() -> Self { ... } }` | config_loader.rs |

**What's unexplained:**
- What is a trait? (A contract that types can implement — like Go interfaces but explicit.)
- What is `impl`? (The block where you write the actual implementation.)
- What is `Self`? (Refers to the type being implemented — `Config` in this case.)
- What is the `Default` trait? (A standard trait for providing default values.)

**De-mystify:** `impl Trait for Type` is Rust's way of saying "this type satisfies this contract." It's explicit opt-in (unlike Go's implicit satisfaction). `Default` is one of many standard traits — `Debug`, `Clone`, `PartialEq`, `Display`, etc.

**Own module?** Yes: [[interfaces-and-traits]] (tier 2, prereq: functions-and-closures).

---

## 10. Pointers and Address-of Operator

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | `&x` (address-of), `*p` (dereference), `%p` format verb | memory.go |
| Rust       | `&x` (borrow/reference), `{:p}` format specifier | memory.rs |

**What's unexplained:**
- We used pointers extensively in memory.go and memory.rs but they were part of the lesson, not just tooling. However, Go's pointer syntax (`*int` as a type, `&x` to take address, `*p` to dereference) and Rust's reference/borrow system (`&T`, `&mut T`) were shown by example without covering the full mental model.
- Rust's borrowing rules (one mutable XOR many immutable) were not discussed.

**De-mystify:** A pointer/reference is a value that holds a memory address. `&x` says "give me the address of x." `*p` says "give me the value at this address." Rust adds compile-time rules about who can hold references and when, which prevents data races and use-after-free at compile time.

**Own module?** Partially covered in [[variables-and-types]] (memory lessons). Full treatment in [[memory-and-ownership]].

---

## 11. HashMap / Collections

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | `map[string]string`, `for key, val := range raw` | config_loader.go |
| Rust       | `HashMap<String, String>`, `use std::collections::HashMap` | config_loader.rs |
| TypeScript | `Record<string, string>` | config_loader.ts |

**What's unexplained:**
- Go: Map literal syntax, `range` iteration, how maps are reference types.
- Rust: `HashMap` is not in the prelude — you must import it. Difference between `HashMap` and `BTreeMap`.
- TypeScript: `Record<K, V>` is a utility type — what does it actually mean? How does it differ from `{ [key: string]: string }`?

**De-mystify:** Hash maps are the workhorse data structure for key-value lookups. Every language has them, but they live in different places: Go has them as a built-in type (`map`), Rust has them in `std::collections`, TypeScript describes them with type aliases over plain objects.

**Own module?** Covered in [[arrays-and-hashing]] (DSA) and touched on in [[variables-and-types]] exercises.

---

## 12. Control Flow Used Without Teaching

| Language   | What we used | Where |
| ---------- | ------------ | ----- |
| Go         | `for key, val := range raw`, `switch key { case "..." }`, `if err != nil` | config_loader.go |
| Rust       | (not yet — config_loader.rs is still `todo!()`) | — |
| TypeScript | `if (raw.host !== undefined)`, strict equality `===` | config_loader.ts |

**What's unexplained:**
- Go: `range` keyword for iteration. `switch` without an expression (Go's "clean if-else"). Multiple return value destructuring.
- TypeScript: `!==` vs `!=`. Truthy/falsy values and why we check `!== undefined` explicitly.

**De-mystify:** Control flow is how you direct program execution. We used it out of necessity in the config_loader exercises before formally teaching it. The patterns are intuitive if you've programmed before, but the language-specific idioms matter.

**Own module?** Yes: [[control-flow]] (tier 1, prereq: variables-and-types). This is the next module in sequence.

---

## Checklist

- [ ] Print functions / macros
- [ ] Import / module system
- [ ] Standard library functions
- [ ] String methods / operations
- [ ] Structs / interfaces / type definitions
- [ ] Derive macros (Rust)
- [ ] Error handling patterns
- [ ] Testing conventions
- [ ] Trait implementations (Rust)
- [ ] Pointers and address-of
- [ ] HashMap / collections
- [ ] Control flow constructs

Mark items off as they get formally covered in their respective modules.
