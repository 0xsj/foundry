# Exercises: Dependency Injection (Rust)

| Type | Status | Description |
|------|--------|-------------|
| [Standard](./standard/) | Available | Build a metrics collection service with injectable MetricsSource, Processor, and Exporter dependencies |
| [Debugging](./debugging/) | Available | Fix 4 bugs related to trait objects, Send+Sync, lifetimes, and trait bounds in a DI-based service |
| [Code Review](./code-review/) | Available | Review a notification dispatch service using Box\<dyn Any\>, excessive Arc\<Mutex\>, and missing abstractions |
| Refactoring | N/A | Covered implicitly by code-review exercise (refactoring from concrete to trait-based DI) |
| API Design | N/A | Better suited for the interfaces-and-traits module |
| Codebase Navigation | N/A | Better suited after multiple pattern modules completed |
