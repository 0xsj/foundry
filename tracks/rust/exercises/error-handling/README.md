# Exercises: Error Handling

| Type | Status | Description |
|------|--------|-------------|
| Standard | Available | Build a TOML-like config file parser with structured error types, thiserror, and full ? propagation |
| Debugging | Available | Four bugs: unwrap() on None, missing From impl for ?, map_err discarding context, ? in fn without correct return type |
| Code Review | Available | PR adding error handling to a CSV data pipeline — issues with .unwrap() everywhere, String errors, missing context |
| Refactoring | N/A | Module is foundational — not enough patterns established for meaningful refactoring exercise yet |
| API Design | N/A | Covered implicitly in the standard exercise (designing the error enum itself) |
| Codebase Navigation | N/A | Premature at this stage of the curriculum |
