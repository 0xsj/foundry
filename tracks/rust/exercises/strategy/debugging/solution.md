# Solution — Strategy Pattern Bugs

## Bug 1: Object Safety Violation

### The Problem

The `ExecutionStrategy` trait has a generic method:

```rust
fn execute_typed<T: fmt::Debug>(&self, task: T) -> String {
    format!("executed: {:?}", task)
}
```

A trait with generic methods is **not object safe** — you cannot create `dyn ExecutionStrategy`. The compiler error is:

```
error[E0038]: the trait `ExecutionStrategy` cannot be made into an object
  --> buggy.rs
   |
   |     strategy: &'a mut dyn ExecutionStrategy,
   |                       ^^^^^^^^^^^^^^^^^^^^ `ExecutionStrategy` cannot be made into an object
   |
   = help: consider moving `execute_typed` to another trait
note: for a trait to be "object safe" it needs to allow building a vtable...
      method `execute_typed` has generic type parameters
```

**Why?** The vtable for a trait object has a fixed number of slots. A generic method like `execute_typed<T>` would need a different slot for every possible `T` — that's infinite. The compiler can't build a finite vtable.

### The Fix

Either remove the method or add `where Self: Sized` to exclude it from the vtable:

```rust
trait ExecutionStrategy {
    fn execute(&mut self, tasks: &[Task]) -> Vec<String>;
    fn name(&self) -> &str;

    // Option A: Add Self: Sized bound — method won't be available through dyn
    fn execute_typed<T: fmt::Debug>(&self, task: T) -> String
    where
        Self: Sized,
    {
        format!("executed: {:?}", task)
    }
}
```

Or simply remove the method if it's not needed (it was added "just in case"):

```rust
trait ExecutionStrategy {
    fn execute(&mut self, tasks: &[Task]) -> Vec<String>;
    fn name(&self) -> &str;
    // execute_typed removed — wasn't being used
}
```

**The `where Self: Sized` approach** is the most common pattern. It keeps the method available on concrete types while allowing the trait to be used as a trait object. Any call to `execute_typed` through `&dyn ExecutionStrategy` will be a compile error, which is exactly the right behavior.

### Go/TS Comparison

In Go, interfaces cannot have generic methods at all, so this problem doesn't exist. In TypeScript, generic methods on interfaces work fine because there's no vtable — everything is dynamic. Rust sits in between: you CAN have generic methods on traits, but only when using the trait with concrete types (generics), not trait objects.

---

## Bug 2: Lifetime Issue — Borrowed Strategy

### The Problem

In `main()`, the `SequentialExecutor` is created as a temporary value:

```rust
let mut runner = TaskRunner::new(&mut SequentialExecutor);
```

The issue is that `SequentialExecutor` (the value) is a temporary created inline. The `TaskRunner` tries to store a mutable reference to it, but the temporary is dropped at the end of the statement. The reference becomes dangling.

This actually compiles in modern Rust due to temporary lifetime extension rules — the temporary's lifetime is extended to match the reference's usage. **However**, the pattern is fragile and the intent is to demonstrate the general problem.

The more realistic form of this bug appears when the strategy is created inside a function and returned:

```rust
// This would NOT compile:
fn make_runner() -> TaskRunner<'_> {
    let mut executor = SequentialExecutor;
    TaskRunner::new(&mut executor) // ERROR: executor dropped here
}
```

### The Fix

Bind the strategy to a variable so it has a clear, named lifetime:

```rust
let mut executor = SequentialExecutor;
let mut runner = TaskRunner::new(&mut executor);
runner.run(&tasks);
println!("History: {:?}\n", runner.history());
```

Or better yet, change `TaskRunner` to **own** the strategy instead of borrowing it:

```rust
struct TaskRunner {
    strategy: Box<dyn ExecutionStrategy>,
    history: Vec<String>,
}

impl TaskRunner {
    fn new(strategy: Box<dyn ExecutionStrategy>) -> Self {
        Self {
            strategy,
            history: Vec::new(),
        }
    }
}
```

Ownership is almost always better than borrowing for strategy holders in Rust. The lifetime parameter on `TaskRunner<'a>` infects everything that touches it.

### Go/TS Comparison

Go and TypeScript don't have this problem because they use garbage collection. In Go, interfaces always hold a pointer to the value, and the GC keeps it alive. In TypeScript, objects are garbage collected. Rust makes the ownership explicit — you must decide who owns the strategy.

---

## Bug 3: Missing `Send` Bound

### The Problem

`AsyncTaskQueue` tries to move a `Box<dyn ExecutionStrategy>` into a new thread:

```rust
std::thread::spawn(move || {
    strategy.execute(&tasks)  // strategy is Box<dyn ExecutionStrategy>
})
```

The compiler error:

```
error[E0277]: `dyn ExecutionStrategy` cannot be sent between threads safely
  --> buggy.rs
   |
   | std::thread::spawn(move || {
   |                    ^^^^^^^ `dyn ExecutionStrategy` cannot be sent between threads safely
   |
   = help: the trait `Send` is not implemented for `dyn ExecutionStrategy`
```

**Why?** `dyn ExecutionStrategy` erases the concrete type. The compiler doesn't know if the concrete type behind the trait object is safe to send across threads. It could contain `Rc`, raw pointers, or other non-Send types. You must explicitly promise thread safety.

### The Fix

Add `Send` to the trait object bound:

```rust
struct AsyncTaskQueue {
    strategy: Box<dyn ExecutionStrategy + Send>,
    pending: Vec<Task>,
}

impl AsyncTaskQueue {
    fn new(strategy: Box<dyn ExecutionStrategy + Send>) -> Self {
        // ...
    }

    fn process_in_thread(&mut self) -> std::thread::JoinHandle<Vec<String>> {
        let mut strategy = std::mem::replace(
            &mut self.strategy,
            Box::new(SequentialExecutor) as Box<dyn ExecutionStrategy + Send>,
        );
        // ...
    }
}
```

All three concrete strategies (`SequentialExecutor`, `PriorityExecutor`, `BatchExecutor`) automatically implement `Send` because they only contain `Send` types (`usize`, no `Rc`, no raw pointers). The bound on the trait object makes this requirement explicit.

### Go/TS Comparison

Go's goroutines share memory freely — there's no compile-time check for thread safety. The `-race` detector catches data races at runtime. TypeScript's Web Workers use message passing (serialization), so thread safety is handled by copying data. Rust's `Send`/`Sync` system catches these issues at compile time, which is more work upfront but eliminates an entire class of bugs.

---

## Bug 4: Wrong Dispatch — Static When Dynamic Is Needed

### The Problem

`run_with_strategy` is generic:

```rust
fn run_with_strategy<S: ExecutionStrategy>(strategy: &mut S, tasks: &[Task]) -> Vec<String> {
```

In `main()`, it's called in a loop over `Vec<Box<dyn ExecutionStrategy>>`:

```rust
for strategy in &mut strategies {
    let results = run_with_strategy(strategy.as_mut(), &tasks);
}
```

With `strategy.as_mut()` returning `&mut dyn ExecutionStrategy`, and the function being generic over `S: ExecutionStrategy`, Rust needs to resolve `S` to a concrete type at compile time. Here `S` would be `dyn ExecutionStrategy`, but `dyn ExecutionStrategy` is unsized (`!Sized`), so it can't be used as a generic parameter (which defaults to `S: Sized`).

### The Fix

Change the function to accept a trait object instead of a generic:

```rust
fn run_with_strategy(strategy: &mut dyn ExecutionStrategy, tasks: &[Task]) -> Vec<String> {
    println!("\n--- Running with {} ---", strategy.name());
    strategy.execute(tasks)
}
```

Now the function uses dynamic dispatch and can accept any `&mut dyn ExecutionStrategy` from the heterogeneous collection.

Alternatively, you could relax the `Sized` bound:

```rust
fn run_with_strategy<S: ExecutionStrategy + ?Sized>(strategy: &mut S, tasks: &[Task]) -> Vec<String> {
```

But this is less idiomatic — if you're always going to call it with trait objects, just accept a trait object directly.

### Go/TS Comparison

Go doesn't have this distinction. Interface parameters are always dynamic dispatch. TypeScript's type erasure means there's no static/dynamic choice. In Rust, you must consciously choose between `fn foo<S: Trait>(s: &S)` (static, monomorphized) and `fn foo(s: &dyn Trait)` (dynamic, single code path). The generic version is faster (inlining) but can't handle heterogeneous collections. The trait object version is flexible but has a vtable cost.

---

## Summary of Fixes

| Bug | Root Cause | Fix | Lesson |
|-----|-----------|-----|--------|
| 1 | Generic method on trait | Add `where Self: Sized` or remove | Object safety rules prevent generic methods in vtables |
| 2 | Temporary value doesn't live long enough | Bind to variable or use `Box` ownership | Prefer ownership over borrowing for strategy holders |
| 3 | `dyn Trait` missing `Send` | Add `+ Send` to trait object type | Thread safety must be explicitly declared on trait objects |
| 4 | Generic fn can't accept `dyn Trait` | Change to `&mut dyn Trait` parameter | Choose dynamic dispatch when strategies are heterogeneous |

## Related Pitfalls

- [[rust-object-safety]] — Full rules for object safe traits
- [[rust-send-sync]] — When and why Send/Sync bounds are needed
- [[rust-lifetime-temporary]] — Temporary value lifetime rules
