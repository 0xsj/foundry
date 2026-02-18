# Exercises: Decorator / Middleware (Rust)

| Type | Status | Description |
|------|--------|-------------|
| [Standard](./standard/) | Available | Build a composable middleware pipeline for a service handler: logging, auth, rate limiting, timeout, metrics |
| [Debugging](./debugging/) | Available | Fix 4 bugs in a decorator stack: ownership, lifetimes, infinite recursion, blocking in async context |
| [Code Review](./code-review/) | Available | Review a middleware system with dynamic dispatch overhead, unnecessary trait bounds, missing Debug/Send+Sync |
| Refactoring | N/A | Covered by code-review exercise (refactoring from Box<dyn> to generics) |
| API Design | Not yet generated | Design a Layer/Service middleware API -- ask to generate |
| Codebase Navigation | N/A | Better suited after multiple pattern modules completed |
