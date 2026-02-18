# Exercises: Factory Pattern (Rust)

| Type | Status | Description |
|------|--------|-------------|
| Standard | Available | Build a message serialization pipeline factory with multiple wire formats |
| Debugging | Available | Four factory-related bugs: object safety, lifetimes, Send+Sync, From recursion |
| Code Review | Available | Review a database driver factory PR with design and performance issues |
| Refactoring | N/A | Better suited after completing more pattern modules |
| API Design | N/A | Covered implicitly by the standard exercise's trait design |
| Codebase Navigation | N/A | No external codebase exercise for this topic |

## Prerequisites

- [[interfaces-and-traits]] -- Trait objects, dynamic dispatch, object safety
- [[generics]] -- Generic parameters, monomorphization
- [[error-handling]] -- Result types, custom errors
- [[structs-methods-and-enums]] -- Enum variants, associated functions

## Related Concepts

- [[patterns/strategy]] -- Strategy uses factory-like creation for runtime dispatch
- [[patterns/builder]] -- Builder as a specialized factory for complex construction
- [[fundamentals/generics]] -- Associated types vs generic parameters
