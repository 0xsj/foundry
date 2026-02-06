# Architecture Builds

Standalone reference implementations that tie together multiple patterns and concepts. Each build is a working system you can study, extend, and use as a template.

## Structure

```
builds/
├── http-server/
│   ├── README.md
│   ├── go/
│   └── typescript/
├── graphql-api/
│   ├── README.md
│   ├── go/
│   └── typescript/
├── kafka-microservice/
│   ├── README.md
│   ├── go/
│   └── java/
├── cqrs-system/
│   ├── README.md
│   ├── go/
│   └── typescript/
└── event-sourced-service/
    ├── README.md
    ├── go/
    └── java/
```

## Build Philosophy

Each build:

- Implements a **realistic domain** (not todos, not blogs)
- Uses **hexagonal architecture** as the default structure unless the build is specifically demonstrating an alternative
- Includes **tests at every layer** (unit, integration)
- Has **observability built in** (structured logging, metrics hooks)
- Includes a README documenting architecture decisions, patterns used, and tradeoffs

## Builds Roadmap

| Build                 | Patterns Exercised                              | Languages |
| --------------------- | ----------------------------------------------- | --------- |
| HTTP Server           | Middleware, DI, Repository, Strategy            | Go, TS    |
| GraphQL API           | Factory, Builder, Repository, Schema validation | Go, TS    |
| Kafka Microservice    | Pub/Sub, Event-driven, Circuit breaker, Retry   | Go, Java  |
| CQRS System           | CQRS, Repository, Observer, Event bus           | Go, TS    |
| Event-Sourced Service | Event sourcing, CQRS, Projections, Snapshots    | Go, Java  |

This list grows as you progress. New builds can be added anytime.
