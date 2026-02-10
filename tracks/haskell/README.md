# Haskell Track

## Why Haskell?

Haskell is a pure functional programming language that provides:
- **Pure functions** — no side effects, referential transparency
- **Lazy evaluation** — values computed only when needed
- **Strong type system** — type inference, algebraic data types, type classes
- **Immutability by default** — all values are immutable
- **Principled abstractions** — Functor, Applicative, Monad, etc.

Haskell complements Go (imperative vs pure functional), TypeScript (dynamic vs static functional), and provides deep functional programming perspective.

## Learning Focus

**Pure functional programming perspective:**
- Thinking in terms of transformations, not steps
- Leveraging immutability and purity
- Type-driven development
- Composability and abstraction
- Lazy evaluation strategies

**Key differences from other languages:**
- Everything is immutable by default
- Functions are pure (no side effects)
- Lazy evaluation (non-strict semantics)
- Type classes instead of interfaces
- Monads for effect management

## Getting Started

### Installation

```bash
# Install GHCup (Haskell toolchain installer)
curl --proto '=https' --tlsv1.2 -sSf https://get-ghcup.haskell.org | sh

# Install GHC (compiler) and Cabal (build tool)
ghcup install ghc
ghcup install cabal

# Verify installation
ghc --version
cabal --version
```

### Hello World

```haskell
main :: IO ()
main = putStrLn "Hello, World!"
```

### Running Code

```bash
# Run with GHC interpreter
runhaskell Main.hs

# Compile and run
ghc Main.hs
./Main

# Interactive REPL
ghci
> :load Main.hs
> main
```

## Module Structure

```
haskell/
├── fundamentals/
│   ├── variables-and-types/
│   ├── functions-and-closures/
│   ├── algebraic-data-types/     # Core Haskell concept
│   ├── type-classes/              # Core Haskell concept
│   ├── error-handling/            # Maybe, Either
│   ├── lazy-evaluation/           # Core Haskell concept
│   └── testing-fundamentals/
├── patterns/
│   ├── functor-applicative-monad/ # Core abstractions
│   ├── parser-combinators/        # Classic Haskell pattern
│   └── ...
└── exercises/
    ├── debugging/
    ├── refactoring/
    ├── code-review/
    └── api-design/
```

## Resources

- [Haskell Language](https://www.haskell.org/)
- [Learn You a Haskell](http://learnyouahaskell.com/)
- [Haskell Wiki](https://wiki.haskell.org/)
- [Hackage](https://hackage.haskell.org/) — Package repository
- [Hoogle](https://hoogle.haskell.org/) — Type-based search

## Notes

**Not covering:**
- Advanced type system features (GADTs, type families, dependent types) — outside foundational scope
- Category theory deep dives — focus on practical patterns
- Lens library — advanced, comes later

**Focus:**
- Core functional programming concepts
- Type-driven development
- Practical Haskell patterns
- Understanding monads and other abstractions
