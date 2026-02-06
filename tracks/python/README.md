# Python Track

Expanding perspective. Python's dynamic typing and expressiveness make it the fastest language for prototyping and scripting. The goal is writing Python that's clean, typed (via type hints), and not just "it works."

## Key Language Characteristics

- Dynamic typing with optional type hints
- Duck typing (protocols over interfaces)
- First-class functions, decorators, generators
- GIL limits true parallelism (asyncio for I/O concurrency)
- Rich standard library
- Multiple paradigms (OOP, functional, procedural)

## What To Pay Attention To

- **Type hints everywhere.** Use `mypy` with strict mode. Python without type hints is a maintenance liability.
- **Protocols over ABCs.** Structural typing via `typing.Protocol` is more Pythonic than abstract base classes.
- **Generators are underused.** Lazy evaluation, memory efficiency, pipeline composition.
- **Pydantic for boundaries.** Validate external data at the edges, trust internals.
- **The GIL matters.** Know when to use asyncio vs threading vs multiprocessing.

## Directory Structure

```
python/
├── fundamentals/    # Language basics exercises
├── patterns/        # Design pattern implementations
├── exercises/       # Generated exercises
└── builds/          # Larger architecture projects
```

## Tools & Setup

- `pyproject.toml` at the track root
- `uv` or `poetry` for dependency management
- `pytest` for testing
- `mypy --strict` for type checking
- `ruff` for linting and formatting

## Idiomatic Conventions

- Use dataclasses or Pydantic models over raw dicts
- Prefer composition over inheritance
- Use context managers (`with`) for resource management
- List/dict/generator comprehensions where they improve readability
- `__slots__` for performance-critical classes
