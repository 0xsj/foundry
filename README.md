# Foundry

A progressive, multi-language learning system for programming fundamentals, design patterns, software architecture, data structures & algorithms, and system design.

## What This Is

Foundry is a personal learning environment designed to build and maintain deep programming knowledge across multiple languages. It's structured as a monorepo with a curriculum engine powered by Claude Code.

The goal isn't to collect tutorials — it's to develop muscle memory through realistic exercises, spaced repetition, and building real systems.

## Languages

- **Go** — primary
- **TypeScript** — primary
- **Rust** — expanding perspective
- **Python** — expanding perspective
- **Java** — expanding perspective, employability

## How It Works

Foundry is used through Claude Code in dedicated 30-60 minute sessions. The `claude.md` file acts as the agent's brain — it understands the curriculum, tracks progress, generates exercises, manages notes, and runs reviews.

### Session Flow

1. Open the repo in Claude Code
2. Say `next` to get a suggested module, or pick a specific topic
3. Work through exercises with realistic scenarios and acceptance criteria
4. Ask for `review` for a quick recall quiz
5. Ask for `anki` to export flashcards for long-term retention
6. Say you're done — session gets logged, progress gets updated

### Commands

| Command                   | What It Does                                                |
| ------------------------- | ----------------------------------------------------------- |
| `next`                    | Suggest the next module based on progress and prerequisites |
| `exercise [topic] [lang]` | Generate a realistic exercise                               |
| `review`                  | In-session quiz (3-5 questions)                             |
| `anki`                    | Export Anki flashcards (TSV)                                |
| `notes [topic]`           | Create or update Obsidian vault notes                       |
| `status`                  | Show progress overview                                      |
| `build [name]`            | Start a larger architecture build                           |
| `switch [lang]`           | Change active language track                                |
| `session`                 | Start or resume a session log                               |

## Structure

```
foundry/
├── claude.md              # Agent brain — curriculum engine
├── curriculum/            # Curriculum map, progress, session logs
├── vault/                 # Obsidian-compatible knowledge base
├── tracks/                # Language-specific exercises and work
├── dsa/                   # Cross-language DSA problems
├── builds/                # Standalone architecture reference builds
└── review/                # Anki exports and quiz logs
```

The structure is a living scaffold. Core categories are stable but everything within them grows organically as new topics are explored.

### Vault

The `vault/` directory is an Obsidian-compatible knowledge base. Notes are built up progressively during sessions and serve as long-term reference material. They use wiki-links for cross-referencing, frontmatter for metadata, and consistent templates for each note type.

### Tracks

Each language has its own track under `tracks/<language>/` with directories for fundamentals, patterns, exercises, and builds. This is where implementation work lives.

### Builds

Larger architecture projects live in `builds/`. These are full service implementations (HTTP servers, GraphQL APIs, Kafka microservices, CQRS systems) that tie together multiple patterns and serve as reusable reference templates.

### Review

Anki card exports (TSV format) and quiz session logs. Cards are tagged by module and language for filtered study.

## Curriculum

The curriculum follows a dependency graph defined in `curriculum/map.yaml`. Modules unlock as prerequisites are completed.

**Progression tiers:**

1. **Fundamentals** — variables, control flow, functions, error handling, concurrency, generics
2. **Patterns** — strategy, observer, factory, builder, middleware, repository, DI, pub/sub
3. **Architecture** — layered, hexagonal, CQRS, event-driven, event sourcing, microservices
4. **DSA** — arrays/hashing, linked lists, trees, graphs, sorting, dynamic programming
5. **System Design** — resilience, caching, load balancing, message queues, database patterns

The curriculum is not fixed. New modules, topics, and categories are added as they come up naturally.

## Getting Started

1. Clone the repo
2. Open in Claude Code
3. Say `next` or `status`
