# Java Track

Expanding perspective and employability. Java's verbosity is real but modern Java (17+) is significantly better. The goal is understanding the enterprise ecosystem (Spring, Maven/Gradle) while writing clean, modern Java.

## Key Language Characteristics

- Static, nominal typing
- Class-based OOP with interfaces
- Generics with type erasure
- Virtual threads (Project Loom) for concurrency
- JVM provides garbage collection, JIT compilation
- Massive ecosystem (Spring, Jakarta EE)

## What To Pay Attention To

- **Modern Java features.** Records, sealed classes, pattern matching, text blocks, var. Don't write Java 8 in 2026.
- **Spring is everywhere.** Understand DI containers, annotations, Spring Boot auto-configuration. Even if you prefer manual wiring, you need to know Spring.
- **Checked vs unchecked exceptions.** Know the debate. Know when to use each. Know why most modern Java avoids checked exceptions.
- **Streams API.** Functional-style collection processing. Parallel streams and their pitfalls.
- **Virtual threads change everything.** The old thread-per-request model is back, but better. Understand structured concurrency.

## Directory Structure

```
java/
├── fundamentals/    # Language basics exercises
├── patterns/        # Design pattern implementations
├── exercises/       # Generated exercises
└── builds/          # Larger architecture projects
```

## Tools & Setup

- Gradle with Kotlin DSL at the track root
- Java 21+ (LTS with virtual threads)
- JUnit 5 for testing
- Use `./gradlew test` to run all tests
- Checkstyle or SpotBugs for static analysis

## Idiomatic Conventions

- Use records for immutable data carriers
- Prefer composition over inheritance
- Use sealed interfaces for closed type hierarchies
- Favor `Optional` over null returns
- Use `var` for local variables where the type is obvious
