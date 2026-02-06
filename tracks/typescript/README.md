# TypeScript Track

Primary language. TypeScript's structural type system and functional capabilities mean many classic OOP patterns simplify or disappear. The goal is leveraging the type system to catch bugs at compile time while writing clean, modern TS.

## Key Language Characteristics

- Structural typing (duck typing with static checks)
- Union and intersection types
- Generics with conditional and mapped types
- Async/await on a single-threaded event loop
- First-class functions and closures
- Runtime is JavaScript — TS is erased at compile time

## What To Pay Attention To

- **Discriminated unions over class hierarchies.** Model state machines and variants with tagged unions.
- **Type narrowing is powerful.** Use it instead of casting. `in`, `typeof`, `instanceof`, custom type guards.
- **Generics with constraints.** Don't reach for `any`. Constrain generics to express what you actually need.
- **Runtime validation.** Types disappear at runtime. Zod or similar for boundaries (API inputs, config, external data).
- **async/await error handling.** Understand the event loop. Know when `.catch()` vs try/catch matters.

## Directory Structure

```
typescript/
├── fundamentals/    # Language basics exercises
├── patterns/        # Design pattern implementations
├── exercises/       # Generated exercises
└── builds/          # Larger architecture projects
```

## Tools & Setup

- `package.json` and `tsconfig.json` at the track root
- Use `vitest` for testing
- `strict: true` in tsconfig — always
- ESM modules preferred
- `tsx` for running TS directly during exercises

## Idiomatic Conventions

- Prefer `type` over `interface` unless you need declaration merging
- Use `readonly` liberally
- Avoid `enum` — use `as const` objects or union types
- Prefer function composition over class inheritance
- Use `satisfies` for type checking without widening
