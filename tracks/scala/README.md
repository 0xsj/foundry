# Scala Track

## Why Scala (Instead of Java)?

Scala runs on the JVM and provides:
- **Java interop** — use Java libraries seamlessly, learn JVM ecosystem
- **Functional + OOP** — blend both paradigms naturally
- **Powerful type system** — type inference, algebraic data types, for-comprehensions
- **Expressive syntax** — less boilerplate than Java
- **Better language features** — pattern matching, traits, case classes, implicits

**You can learn Java through Scala** because:
- Scala compiles to JVM bytecode
- You'll use Java libraries and frameworks
- You'll understand Java concepts (interfaces, classes, exceptions)
- But with modern language features (no verbose getters/setters, better collections)

## Learning Focus

**JVM ecosystem through modern lens:**
- Object-oriented programming (classes, traits, inheritance)
- Functional programming (immutable data, higher-order functions)
- Type system (generics, variance, type bounds)
- Concurrency (Futures, Akka actors)
- Ecosystem (sbt, Maven, Java libraries)

**Key advantages over Java:**
- Type inference (less type noise)
- Case classes (automatic equals/hashCode/toString)
- Pattern matching (vs verbose if-else chains)
- For-comprehensions (vs nested loops)
- Traits (vs interfaces + abstract classes)
- No null by default (Option type)

## Getting Started

### Installation

```bash
# Install Scala via Coursier
curl -fL https://github.com/coursier/coursier/releases/latest/download/cs-x86_64-apple-darwin.gz | gzip -d > cs
chmod +x cs
./cs setup

# Verify installation
scala --version
sbt --version  # Scala Build Tool
```

### Hello World

```scala
object Main extends App {
  println("Hello, World!")
}
```

### Running Code

```bash
# Run with Scala interpreter
scala Main.scala

# Compile and run
scalac Main.scala
scala Main

# Interactive REPL
scala
> println("Hello")

# Using sbt (recommended for projects)
sbt
> run
> test
```

## Module Structure

```
scala/
├── fundamentals/
│   ├── variables-and-types/
│   ├── control-flow/
│   ├── functions-and-closures/
│   ├── error-handling/
│   ├── interfaces-and-traits/       # Scala traits
│   ├── case-classes/                # Core Scala concept
│   ├── pattern-matching/            # Core Scala concept
│   ├── collections/                 # Scala collections library
│   ├── for-comprehensions/          # Core Scala concept
│   └── testing-fundamentals/
├── patterns/
│   ├── strategy/
│   ├── factory/
│   ├── builder/
│   └── ...
├── architecture/
│   ├── layered/
│   ├── hexagonal/
│   └── ...
├── dsa/
│   └── ...
├── system-design/
│   └── ...
└── exercises/
    ├── debugging/
    ├── refactoring/
    ├── code-review/
    └── api-design/
```

## Resources

- [Scala Documentation](https://docs.scala-lang.org/)
- [Scala Book](https://docs.scala-lang.org/overviews/scala-book/introduction.html)
- [Scala Exercises](https://www.scala-exercises.org/)
- [Scaladex](https://index.scala-lang.org/) — Package index
- [Functional Programming in Scala (Red Book)](https://www.manning.com/books/functional-programming-in-scala)

## Java Interop Examples

```scala
// Using Java libraries
import java.util.{ArrayList, HashMap}
import java.time.LocalDate

// Java collections
val javaList = new ArrayList[String]()
javaList.add("item")

// Convert between Scala and Java collections
import scala.jdk.CollectionConverters._
val scalaList = javaList.asScala.toList
val backToJava = scalaList.asJava

// Java classes
val today = LocalDate.now()
```

## Notes

**Not covering (beyond scope):**
- Advanced type system features (F-bounded polymorphism, path-dependent types)
- Implicits in depth (focusing on basics)
- Akka in depth (basics of actors only)
- Cats/ZIO libraries (focusing on standard library)

**Focus:**
- Practical Scala for everyday use
- Understanding JVM ecosystem
- Blending OOP and FP effectively
- Writing idiomatic Scala (not "Java in Scala")
