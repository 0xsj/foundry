# Functions and Closures — Rust

## How Functions Work Under the Hood

### Function Items Are Types

In Rust, every function definition creates a unique, zero-sized **function item type**. This is distinct from a function pointer. The distinction matters as soon as you start passing functions around.

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// The type of `add` is not `fn(i32, i32) -> i32`.
// It's a unique zero-sized type: "the function add defined at this location".
// It coerces to a function pointer when needed.
let f: fn(i32, i32) -> i32 = add;  // coercion from function item to fn pointer
```

The zero-sized nature is important: passing a function item costs nothing. There is no pointer stored, no indirection — the compiler knows at compile time exactly which function will be called and inlines accordingly.

### Expression-Based Returns

Rust is an expression-oriented language. Almost everything evaluates to a value. Functions return the value of their **last expression** — no `return` keyword needed.

```rust
fn max_latency(a: u32, b: u32) -> u32 {
    if a > b { a } else { b }  // no semicolon — this is the return value
}
```

The semicolon changes semantics: adding one turns an expression into a statement, which evaluates to `()` (unit). This is the most common beginner mistake:

```rust
fn broken(a: u32, b: u32) -> u32 {
    if a > b { a } else { b };  // semicolon! returns () — compile error
}
```

**Comparison to Go and TS:** Go always uses `return`. TypeScript arrow functions have implicit returns only for single expressions (`x => x + 1`), but braces require `return`. Rust is more consistent: braces everywhere, return optional everywhere.

### Your notes
<!-- -->


---

## Function Syntax

### Basic Definition

```rust
fn function_name(param: Type, param2: Type) -> ReturnType {
    // body
}
```

Every parameter must have an explicit type. Return type annotation is required unless the function returns `()` (unit), which can be omitted.

```rust
fn greet(name: &str) {           // implicitly returns ()
    println!("Hello, {name}");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn divide(a: f64, b: f64) -> Option<f64> {  // returns Option
    if b == 0.0 { None } else { Some(a / b) }
}
```

### Multiple Return Values

Rust returns multiple values via tuples:

```rust
fn parse_address(input: &str) -> (String, u16) {  // host + port
    // real parsing omitted
    (String::from("localhost"), 8080)
}

let (host, port) = parse_address("localhost:8080");
```

**Comparison to Go:** Go uses multiple return values directly (`func f() (string, int)`). Rust uses tuples, which are slightly more explicit about being a single value that happens to contain multiple things. Both approaches are idiomatic in their respective languages.

### Early Returns

Use `return` to exit a function before the last expression. The idiomatic pattern is to use early returns for guard clauses — invalid states, errors — and the final expression for the happy path.

```rust
fn validate_rate_limit(requests: u32, window_seconds: u32) -> Result<(), String> {
    if window_seconds == 0 {
        return Err(String::from("window_seconds must be > 0"));
    }
    if requests > 10_000 {
        return Err(format!("requests {} exceeds maximum of 10000", requests));
    }

    Ok(())  // happy path — no explicit return needed
}
```

### Your notes
<!-- -->


---

## Diverging Functions

A function that never returns has the return type `!` — called the **never type**.

```rust
fn panic_on_config_error(msg: &str) -> ! {
    eprintln!("Fatal config error: {}", msg);
    std::process::exit(1);
}
```

`!` is useful because it coerces to any type. This is why `panic!()`, `todo!()`, `unreachable!()`, and `loop {}` can appear in match arms or if expressions that need to produce a specific type:

```rust
fn get_config_value(key: &str) -> u32 {
    match key {
        "timeout" => 30,
        "retries" => 3,
        _ => panic!("unknown config key: {}", key),  // ! coerces to u32
    }
}
```

The `!` type is also the return type of `std::process::exit` and of `loop {}` without a `break` value. The compiler uses this for exhaustiveness analysis: branches returning `!` can be ignored in type unification.

### Your notes
<!-- -->


---

## Closures

### Syntax

Closures in Rust are anonymous functions that can capture their surrounding environment. The syntax uses vertical bars for parameters:

```rust
let double = |x: i32| x * 2;              // single expression, no braces
let add = |a: i32, b: i32| -> i32 {       // explicit types, braced body
    a + b
};
let greet = |name: &str| {
    println!("Hello, {name}");
};
```

Type annotations are usually optional — Rust infers them from usage. Once a closure's types are inferred from its first use, they are locked in:

```rust
let process = |x| x * 2;
let _a = process(5i32);   // inferred: x is i32
// let _b = process(5.0); // ERROR: expected i32, found f64
```

**Comparison to JS/TS:** JavaScript arrow functions (`x => x * 2`) are very similar in syntax. The key difference is in capture semantics — JavaScript closures always capture by reference (they close over the variable binding). Rust closures choose their capture mode based on what the body actually does.

**Comparison to Go:** Go function literals (`func(x int) int { return x * 2 }`) also capture from the surrounding scope. Go always captures by reference (a pointer to the outer variable). Rust gives you control.

### How Closures Capture

Rust determines how a closure captures each variable by looking at how the body uses it:

| Usage in body | Capture mode |
|---|---|
| Read only | Borrow immutably (`&T`) |
| Mutate | Borrow mutably (`&mut T`) |
| Consume or `move` keyword | Take ownership (`T`) |

```rust
let data = vec![1, 2, 3];

// Read-only — captures &data
let count = || data.len();
println!("length: {}", count());
println!("still have data: {:?}", data);  // data still accessible

// Mutating — captures &mut data
let mut data = vec![1, 2, 3];
let mut push_value = |v| data.push(v);
push_value(4);
// println!("{:?}", data);  // ERROR: can't borrow data here, closure holds &mut

drop(push_value);  // release the mutable borrow
println!("{:?}", data);  // now OK: [1, 2, 3, 4]
```

### Your notes
<!-- -->


---

## The Fn Trait Family

Closures implement one or more of three traits, depending on what they do with their captured environment:

| Trait | Can be called | Requirements | Typical use |
|---|---|---|---|
| `FnOnce` | Once | Can consume captures | One-shot callbacks, `Option::map` |
| `FnMut` | Multiple times, mutably | Can mutate captures | Iterators, accumulators |
| `Fn` | Multiple times, concurrently | Only reads captures | Shared callbacks, thread-safe |

The relationship: `Fn: FnMut: FnOnce`. Every `Fn` is also `FnMut` is also `FnOnce`. When you accept `FnMut`, you accept any closure that is `FnMut` or more capable (`Fn`).

```
FnOnce  ⊇  FnMut  ⊇  Fn
```

### FnOnce — Consumes Captured Values

A closure is `FnOnce` when it moves a captured value out of itself. It can only be called once — after that, the value is gone.

```rust
fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()  // OK: call it once
    // f()  // ERROR: use of moved value `f`
}

let config_json = String::from(r#"{"timeout": 30}"#);

// This closure moves config_json out when called — FnOnce only
let emit = || {
    // "consume" the string — we give it away
    send_to_audit_log(config_json)  // moves config_json
};

run_once(emit);
// emit();  // ERROR: emit was FnOnce, already consumed
```

### FnMut — Mutates Captured Values

A closure is `FnMut` (but not necessarily `Fn`) when it mutates a captured value. It can be called multiple times, but not concurrently — the mutable access requires exclusive ownership.

```rust
fn apply_n_times<F: FnMut(i32) -> i32>(mut f: F, value: i32, n: u32) -> i32 {
    let mut result = value;
    for _ in 0..n {
        result = f(result);
    }
    result
}

let mut call_count = 0u32;

// Mutates call_count — FnMut
let mut counted_double = |x: i32| {
    call_count += 1;
    x * 2
};

let result = apply_n_times(&mut counted_double, 1, 4);
println!("result: {}, calls: {}", result, call_count);  // result: 16, calls: 4
```

### Fn — Read-Only Captures

A closure is `Fn` when it only reads its captured values. It can be called any number of times, including concurrently from multiple threads.

```rust
fn apply_to_each<F: Fn(i32) -> i32>(items: &[i32], f: F) -> Vec<i32> {
    items.iter().map(|&x| f(x)).collect()
}

let multiplier = 3;

// Only reads multiplier — Fn
let triple = |x: i32| x * multiplier;

let result = apply_to_each(&[1, 2, 3, 4], triple);
println!("{:?}", result);  // [3, 6, 9, 12]
```

### What Determines Which Trait a Closure Implements

| Closure body does... | Implements |
|---|---|
| Nothing with captures | `Fn + FnMut + FnOnce` |
| Reads captures | `Fn + FnMut + FnOnce` |
| Mutates captures | `FnMut + FnOnce` (not `Fn`) |
| Moves captures out | `FnOnce` only |

You cannot manually choose. The compiler determines it from your code. If you need `Fn` but your closure accidentally mutates something, the compiler tells you.

### Your notes
<!-- -->


---

## Move Closures

The `move` keyword forces a closure to take ownership of all captured variables, regardless of whether the body needs it.

```rust
let config = String::from("production");

// Without move: captures &config
let describe = || println!("config: {}", config);
describe();
println!("still have: {}", config);  // OK

// With move: takes ownership of config
let describe_owned = move || println!("config: {}", config);
// println!("still have: {}", config);  // ERROR: config moved into closure
describe_owned();
```

### Why `move` Closures Exist

The primary use case is **thread safety**. When spawning a thread, the new thread might outlive the current scope. References would dangle. `move` forces the closure to own its data, making it safe to send across thread boundaries:

```rust
// We'll cover threads in detail later — for now, just notice the pattern
let batch_id = String::from("batch-2026-001");

let handle = std::thread::spawn(move || {
    // batch_id is owned by this closure, not borrowed from the outer scope
    println!("Processing batch: {}", batch_id);
});

handle.join().unwrap();
// println!("{}", batch_id);  // ERROR: batch_id was moved into the thread
```

`move` closures are also used when returning closures from functions, as the closure must own its captured data to outlive the function's scope.

### Your notes
<!-- -->


---

## Function Pointers vs Closure Traits

There are two distinct categories of callable in Rust:

### `fn` — Function Pointer Type

`fn(Type, Type) -> ReturnType` is a concrete type. It's a pointer to a function's machine code. It has a fixed size (one pointer), carries no captured state, and is `Copy`. All function items coerce to `fn` pointers. Non-capturing closures also coerce to `fn` pointers.

```rust
fn double(x: i32) -> i32 { x * 2 }

// Function pointer: concrete, copyable, zero capture
let f: fn(i32) -> i32 = double;
let g = f;  // copied — f and g both work
println!("{}", f(5));  // 10
println!("{}", g(5));  // 10
```

### `Fn`, `FnMut`, `FnOnce` — Trait Objects

These are traits, not concrete types. To use them in certain positions, you need either generics or trait objects:

```rust
// As a generic bound (monomorphized — zero cost at runtime)
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

// As a trait object (dynamic dispatch — virtual call at runtime)
fn apply_dyn(f: &dyn Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}
```

### When to Use Which

| Situation | Use |
|---|---|
| Function with no captures | `fn` pointer or `impl Fn` |
| Passing closure as argument | `impl Fn` / `impl FnMut` / `impl FnOnce` |
| Storing closure in struct | `Box<dyn Fn>` or generic parameter |
| Multiple different closures in same collection | `Vec<Box<dyn Fn>>` |
| Performance-critical hot path | Generic (`impl Fn`) — avoids vtable |
| Flexibility (plugin system) | `Box<dyn Fn>` — runtime dispatch |

### Your notes
<!-- -->


---

## Higher-Order Functions

Functions that accept closures as parameters or return closures.

### Accepting Closures — `impl Trait` in Argument Position

`impl Trait` in argument position is syntactic sugar for a generic parameter. It's the most ergonomic way to accept closures:

```rust
// These two are equivalent:
fn apply(f: impl Fn(i32) -> i32, x: i32) -> i32 { f(x) }
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }
```

Prefer `impl Trait` when you only have one function parameter of that type and no reason to name the generic.

```rust
fn transform_records(
    records: &[&str],
    filter: impl Fn(&str) -> bool,
    transform: impl Fn(&str) -> String,
) -> Vec<String> {
    records
        .iter()
        .filter(|&&r| filter(r))
        .map(|&r| transform(r))
        .collect()
}

let output = transform_records(
    &["user:alice", "admin:bob", "user:carol"],
    |r| r.starts_with("user:"),
    |r| r.trim_start_matches("user:").to_uppercase(),
);

println!("{:?}", output);  // ["ALICE", "CAROL"]
```

### Returning Closures — `impl Trait` in Return Position

Closures have unnameable types. To return a closure, use `impl Trait` (when the concrete type is always the same) or `Box<dyn Trait>` (when you might return different closure types):

```rust
// impl Trait — concrete return type, statically dispatched
fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |x| x * factor  // move is required: factor must outlive the function
}

let triple = make_multiplier(3);
let quadruple = make_multiplier(4);
println!("{}", triple(5));    // 15
println!("{}", quadruple(5)); // 20
```

```rust
// Box<dyn Fn> — when the returned closure type varies at runtime
fn make_formatter(format: &str) -> Box<dyn Fn(&str) -> String> {
    match format {
        "upper" => Box::new(|s: &str| s.to_uppercase()),
        "lower" => Box::new(|s: &str| s.to_lowercase()),
        _       => Box::new(|s: &str| s.to_string()),
    }
}

let fmt = make_formatter("upper");
println!("{}", fmt("hello")); // HELLO
```

**Why `impl Trait` can't be used here for multiple types:** `impl Trait` is monomorphized — the caller gets a single concrete type. If two branches of a `match` return closures with different types, the compiler cannot unify them. `Box<dyn Trait>` erases the type behind a pointer, enabling runtime polymorphism.

### Function Composition

```rust
fn compose<A, B, C>(
    f: impl Fn(A) -> B,
    g: impl Fn(B) -> C,
) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

let parse_and_double = compose(
    |s: &str| s.parse::<i32>().unwrap_or(0),
    |n: i32| n * 2,
);

println!("{}", parse_and_double("21")); // 42
```

### Your notes
<!-- -->


---

## Closures in the Standard Library

The standard library is saturated with higher-order functions. These are the patterns you'll use constantly:

```rust
let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8];

// Iterator adapters — all lazy, evaluated when collected/consumed
let result: Vec<i32> = numbers.iter()
    .filter(|&&x| x % 2 == 0)   // FnMut — closure called for each element
    .map(|&x| x * x)             // FnMut — transforms each element
    .collect();

println!("{:?}", result);  // [4, 16, 36, 64]

// fold — accumulator pattern (FnMut)
let sum: i32 = numbers.iter().fold(0, |acc, &x| acc + x);
println!("sum: {}", sum);  // 36

// sort_by — custom ordering (FnMut)
let mut words = vec!["banana", "apple", "cherry", "date"];
words.sort_by(|a, b| a.len().cmp(&b.len()));
println!("{:?}", words);  // ["date", "apple", "banana", "cherry"]
```

### Vec::retain and HashMap::retain

```rust
let mut events: Vec<String> = vec![
    String::from("login"),
    String::from("debug-trace"),
    String::from("purchase"),
    String::from("debug-dump"),
];

// Retain only non-debug events — FnMut closure modifies the vec in place
events.retain(|e| !e.starts_with("debug"));
println!("{:?}", events);  // ["login", "purchase"]
```

### Your notes
<!-- -->


---

## Generic Functions

A brief introduction — generics get their own deep module.

```rust
// T is a generic type parameter. The compiler generates a concrete version
// for each type T is used with (monomorphization).
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

println!("{}", largest(&[34, 50, 25, 100]));  // 100
println!("{}", largest(&["pear", "apple", "mango"]));  // pear (lex order)
```

The constraint `T: PartialOrd` says: "T must implement `PartialOrd` so we can use `>`." Without it, the compiler rejects the comparison.

We'll cover generic functions, trait bounds, where clauses, and lifetime parameters in the generics module. For now: any time you see `<T>` or `impl Trait`, the compiler is doing monomorphization — generating specialized code per concrete type with no runtime overhead.

### Your notes
<!-- -->


---

## Cross-Language Comparison

### Function Definitions

| Concept | Rust | Go | TypeScript |
|---|---|---|---|
| Basic function | `fn f(x: i32) -> i32 {}` | `func f(x int) int {}` | `function f(x: number): number {}` |
| No return | `fn f() {}` | `func f() {}` | `function f(): void {}` |
| Multiple returns | `(T, U)` tuple | `(T, U)` multi-return | `[T, U]` tuple |
| Expression return | Last expression (no `;`) | Always `return` | Arrow: implicit; block: `return` |
| Diverging | `fn f() -> ! { loop {} }` | No equivalent (no `!` type) | `function f(): never {}` |

### Closures

| Concept | Rust | Go | TypeScript |
|---|---|---|---|
| Syntax | `\|x\| x * 2` | `func(x int) int { return x * 2 }` | `x => x * 2` |
| Capture mode | By borrow (default) or `move` | Always by reference | Always by reference |
| Type inference | Full inference from usage | Explicit types in literal | Full inference |
| Trait system | `Fn`/`FnMut`/`FnOnce` | `func(...)` type only | `(x: T) => U` structural typing |
| Returning closure | `impl Fn` or `Box<dyn Fn>` | `func(...) ...` (simple) | `(x: T) => U` |

### The Key Difference: Capture Semantics

This is where Rust diverges meaningfully from Go and JS:

```rust
// Rust: compiler chooses capture mode; you can force ownership with move
let data = vec![1, 2, 3];
let read = || data.len();    // borrows &data
let own = move || data;      // moves data — can't use data after this

// Go equivalent: always reference (pointer to outer variable)
// data := []int{1, 2, 3}
// read := func() int { return len(data) }  // reference to data

// JS/TS equivalent: always reference (closure over binding)
// const data = [1, 2, 3]
// const read = () => data.length  // reference to binding
```

The practical consequence: Rust closures require you to think about ownership when capturing heap data. Go and JS/TS closures "just work" — but this means data can be mutated unexpectedly by any closure that captured it, and lifetimes are managed by the GC.

### Your notes
<!-- -->
