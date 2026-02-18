# Exercises: I/O & Files

| Type | Status | Description |
|------|--------|-------------|
| Standard | Available | Build a streaming log processor — reads log files line by line, filters, transforms, and writes results using Reader/Writer composition |
| Debugging | Available | Four bugs: defer Close on writer, missing Scanner.Err(), reading entire file instead of streaming, filepath.Join vs string concatenation |
| Code Review | Available | PR adding file-based caching — missing cleanup, unbuffered writes in loop, hardcoded paths, no atomic write |
| Refactoring | N/A | Not enough patterns introduced yet at this tier to apply a meaningful before/after refactor |
| API Design | N/A | I/O interfaces are defined by the standard library; designing wrappers is covered in interfaces-and-traits |
| Codebase Navigation | N/A | Not yet applicable — codebase navigation is introduced after multiple modules are complete |
