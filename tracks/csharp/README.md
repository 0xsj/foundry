# C# Track

C# is a modern, statically-typed, object-oriented language from Microsoft. It runs on .NET (formerly .NET Core), a cross-platform runtime.

## Why C#

- **Enterprise adoption**: Widely used in enterprise software, game development (Unity), and cloud (Azure)
- **Modern features**: Pattern matching, async/await, LINQ, nullable reference types
- **Strong ecosystem**: Rich standard library, mature tooling, extensive package ecosystem (NuGet)
- **Cross-platform**: Runs on Windows, macOS, Linux via .NET
- **Career value**: High demand in enterprise and gaming sectors

## Language Characteristics

- **Static typing** with strong type inference (`var` keyword)
- **Managed memory** (garbage collected)
- **Object-oriented** with first-class support for interfaces, inheritance, and polymorphism
- **Functional features**: LINQ, lambda expressions, pattern matching
- **Modern safety**: Nullable reference types, pattern matching, records

## Comparison to Other Languages

| Feature | C# | Java | TypeScript | Go | Rust |
|---|---|---|---|---|---|
| Type System | Static, nominal | Static, nominal | Static, structural | Static, structural (interfaces) | Static, nominal |
| Memory Management | GC | GC | GC (JS runtime) | GC | Manual (ownership) |
| Null Safety | Nullable types (opt-in) | No | Strict mode (opt-in) | No (but clear nil) | Yes (Option) |
| Async | async/await | async/await, CompletableFuture | async/await | goroutines | async/await (tokio) |
| Generics | Yes, with constraints | Yes, with erasure | Yes, structural | Yes, simple | Yes, with traits |

## Setup

### Install .NET SDK

```bash
# macOS (Homebrew)
brew install --cask dotnet-sdk

# Verify installation
dotnet --version
```

### Create a new project

```bash
# Console application
dotnet new console -n MyApp
cd MyApp
dotnet run

# Library
dotnet new classlib -n MyLib
```

### Run code files directly

```bash
# C# supports top-level statements (C# 9+)
dotnet run Program.cs
```

## Resources

- **Official Docs**: [Microsoft C# Documentation](https://learn.microsoft.com/en-us/dotnet/csharp/)
- **Language Spec**: [C# Language Specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/)
- **API Reference**: [.NET API Browser](https://learn.microsoft.com/en-us/dotnet/api/)
- **Interactive Tutorial**: [C# 101](https://learn.microsoft.com/en-us/shows/csharp-101/)

## Track Progress

See `curriculum/progress.yaml` for your progress in this track.
