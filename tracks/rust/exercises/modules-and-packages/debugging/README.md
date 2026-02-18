# Debugging Exercise: Broken Module Wiring

## Scenario

You pulled a feature branch from a colleague who refactored a configuration loading service into multiple modules. When you try to build it, you get a cascade of compiler errors. The errors mention "module not found", "is private", and "unresolved import". Your colleague says "it compiled on my machine" — you know the code just needs some visibility and import fixes.

## Symptoms

```
error[E0432]: unresolved import `crate::config`
error[E0603]: function `load_defaults` is private
error[E0603]: struct `RawConfig` is private
error[E0433]: failed to resolve: use of undeclared crate or module `util`
error[E0308]: mismatched types
```

## Your Task

1. Build the buggy crate: `cd buggy_crate && cargo build`
2. Read each error message carefully
3. Fix the bugs without changing any function signatures or logic
4. All `cargo test` tests should pass when you're done

## Hints

<details>
<summary>Hint 1: Start with the "module not found" errors</summary>

If a module isn't in the module tree, none of its items are accessible. Fix module declaration errors first — they cause a cascade of secondary errors.

</details>

<details>
<summary>Hint 2: Look at what's pub and what isn't</summary>

Items are private by default. If `main.rs` needs to use something from another module, that thing needs an appropriate visibility modifier. Think about who legitimately needs access before just adding `pub` everywhere.

</details>

<details>
<summary>Hint 3: Check the use paths</summary>

A `use` path must match the actual module path in the crate. If you moved something to a different module, the `use` path needs to update.

</details>
