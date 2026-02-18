# Functions and Closures — Go

## How Functions Work Under the Hood

### The Call Stack

When you call a function, the runtime sets up a **stack frame** — a contiguous block of memory on the goroutine's stack that holds the function's local variables, arguments, and return values. When the function returns, that frame is popped and the memory is available for reuse.

```go
func add(a, b int) int {
    result := a + b  // result lives in add's stack frame
    return result    // result's value is copied to caller's frame
}

func main() {
    x := add(3, 4)  // x lives in main's stack frame
    fmt.Println(x)
}
```

Under the hood:
1. `main` pushes `3` and `4` onto the stack (or into registers on modern CPUs with calling conventions)
2. A new frame is created for `add` — `a`, `b`, and `result` are allocated there
3. `add` computes `7`, writes it to the return slot, and the frame is popped
4. `main` reads `7` into `x`

The key insight: **return by value = copy.** The value crosses a frame boundary. The original stack allocation in `add` is gone after the call.

Go's goroutine stacks start small (2-8KB) and grow dynamically. This is different from OS threads, which have large fixed stacks. It's what makes spawning thousands of goroutines cheap.

### Your notes
<!-- -->

---

## Function Declarations

The basic syntax is `func name(params) returnType { body }`. Everything is explicit — no implicit return types, no default arguments.

```go
func greet(name string) string {
    return "Hello, " + name
}

// Multiple parameters of the same type can share the type declaration
func add(a, b int) int {
    return a + b
}

// No parameters, no return value
func logStartup() {
    fmt.Println("service starting...")
}
```

### Compared to JS/TS

```typescript
// TypeScript: multiple syntaxes for the same thing
function greet(name: string): string { return "Hello, " + name; }
const greet = (name: string): string => "Hello, " + name;
const greet = (name: string) => "Hello, " + name;  // inferred return type
```

```go
// Go: one syntax for named functions
func greet(name string) string {
    return "Hello, " + name
}
```

Go is more rigid (one way), which makes codebases more consistent. The tradeoff: less flexibility for one-liner style.

### Your notes
<!-- -->

---

## Multiple Return Values

This is one of Go's most distinctive features. Functions can return more than one value, and this is idiomatic — not a workaround.

```go
func divide(a, b float64) (float64, error) {
    if b == 0 {
        return 0, fmt.Errorf("division by zero")
    }
    return a / b, nil
}

result, err := divide(10, 3)
if err != nil {
    log.Fatal(err)
}
fmt.Println(result)  // 3.3333...
```

The canonical use is the `(value, error)` pattern — it forces callers to acknowledge errors at the call site. Contrast this with exceptions (Java, Python, JS) where errors can be silently propagated up the call stack without the intermediate code knowing.

### Named Return Values

You can name the return values in the signature. This documents intent and enables **naked returns** (return without arguments):

```go
func parseRange(s string) (min, max int, err error) {
    parts := strings.SplitN(s, "-", 2)
    if len(parts) != 2 {
        err = fmt.Errorf("invalid range: %q", s)
        return  // naked return — returns current values of min, max, err
    }
    min, err = strconv.Atoi(parts[0])
    if err != nil {
        return
    }
    max, err = strconv.Atoi(parts[1])
    return
}
```

Named returns are useful for documentation and for `defer` interactions (covered below). But naked returns in long functions obscure what's being returned — prefer them only in short functions or `defer`-heavy code.

```go
// Named return purely for documentation — no naked return
func httpHandler(r *http.Request) (statusCode int, responseBody []byte, err error) {
    // ...
    return 200, body, nil  // explicit, clear
}
```

### Ignoring Return Values

The blank identifier `_` discards values you don't need:

```go
result, _ := divide(10, 3)  // ignoring the error (usually bad practice)

for _, v := range items {  // ignoring the index
    process(v)
}
```

> **Warning:** Ignoring errors with `_` is sometimes appropriate (e.g., `fmt.Println`) but often a bug waiting to happen. Go's linters (`errcheck`, `staticcheck`) can catch this.

### Your notes
<!-- -->

---

## Variadic Functions

A variadic function accepts a variable number of arguments. The `...` syntax makes the final parameter a slice inside the function.

```go
func sum(nums ...int) int {
    total := 0
    for _, n := range nums {
        total += n
    }
    return total
}

sum(1, 2, 3)         // 6
sum(1, 2, 3, 4, 5)   // 15
sum()                 // 0 — nums is an empty slice (not nil, well actually nil — check it)
```

### Spreading a Slice

To pass a slice as variadic arguments, use the spread operator `...`:

```go
nums := []int{1, 2, 3, 4}
sum(nums...)   // equivalent to sum(1, 2, 3, 4)
```

This is similar to JavaScript's spread: `sum(...nums)`. The difference: in Go the `...` goes on the argument, not inside it.

### How It Works Under the Hood

Inside the function, `nums` is just a `[]int`. The compiler builds the slice from the arguments at the call site. If you call `sum(1, 2, 3)`, the compiler emits code roughly equivalent to:

```go
tmp := []int{1, 2, 3}
sum(tmp)
```

This means there's a slice allocation per variadic call (unless the compiler can optimize it away). For hot paths, `append` or explicit slices are more predictable.

### Your notes
<!-- -->

---

## defer

`defer` registers a function call to execute when the surrounding function returns — regardless of how it returns (normal, error, or panic). Multiple defers execute in **LIFO order** (last in, first out).

```go
func processFile(path string) error {
    f, err := os.Open(path)
    if err != nil {
        return err
    }
    defer f.Close()  // will run when processFile returns, no matter what

    // ... process f ...
    return nil
}
```

This is Go's resource cleanup idiom. The benefit: the cleanup is written immediately after the acquisition, close to the relevant code. No need to remember to close at every return path.

### How defer Works Under the Hood

The deferred calls are stored in a **defer chain** on the goroutine's stack. When the function returns:
1. The runtime walks the defer chain in LIFO order
2. Each deferred function is called with its arguments **evaluated at the defer statement**, not at the call time

```go
func example() {
    x := 10
    defer fmt.Println(x)  // x is captured as 10 here — defer args are evaluated immediately
    x = 20
    fmt.Println(x)        // prints 20
}
// Output:
// 20
// 10  (not 20! x was 10 when defer ran)
```

This is a common gotcha. Defer arguments are evaluated **immediately**, but the function call itself happens later.

### Defer Execution Order

```go
func counting() {
    fmt.Println("start")
    defer fmt.Println("first defer")
    defer fmt.Println("second defer")
    defer fmt.Println("third defer")
    fmt.Println("end")
}
// Output:
// start
// end
// third defer  ← LIFO: last registered, first called
// second defer
// first defer
```

### defer with Named Return Values

This is where `defer` gets powerful — and where it can surprise you. A deferred function can read and **modify** named return values because it runs in the same scope:

```go
func divide(a, b float64) (result float64, err error) {
    defer func() {
        if err != nil {
            // Can rewrite err — the return value hasn't been sent yet
            err = fmt.Errorf("divide: %w", err)
        }
    }()

    if b == 0 {
        err = fmt.Errorf("division by zero")
        return  // naked return — deferred func runs here and wraps the error
    }
    result = a / b
    return
}
```

The deferred function runs *after* the return values are set but *before* the caller receives them. This lets you add consistent error wrapping, logging, or cleanup based on the actual return value.

### defer in Loops — The Classic Trap

**Don't do this:**

```go
func processFiles(paths []string) error {
    for _, path := range paths {
        f, err := os.Open(path)
        if err != nil {
            return err
        }
        defer f.Close()  // WRONG: defers don't run until the function returns!
        // ... process f ...
    }
    // All files are still open here. f.Close() runs after the loop.
    return nil
}
```

Every `defer f.Close()` in the loop registers a new deferred call. They all fire when `processFiles` returns — not when each iteration ends. If you're processing 10,000 files, you hold 10,000 open file descriptors until the function returns.

**Fix: extract to a function or use explicit close:**

```go
// Option 1: Extract to a helper function (defers run when helper returns)
func processFile(path string) error {
    f, err := os.Open(path)
    if err != nil {
        return err
    }
    defer f.Close()  // correct now — runs when THIS function returns
    // ... process f ...
    return nil
}

func processFiles(paths []string) error {
    for _, path := range paths {
        if err := processFile(path); err != nil {
            return err
        }
    }
    return nil
}

// Option 2: Explicit close in the loop (no defer)
for _, path := range paths {
    f, err := os.Open(path)
    if err != nil {
        return err
    }
    err = processFile(f)
    f.Close()  // explicit, runs each iteration
    if err != nil {
        return err
    }
}
```

### Your notes
<!-- -->

---

## Anonymous Functions and Function Literals

Functions are values in Go. You can declare a function without a name (an **anonymous function** or **function literal**) and assign it to a variable, pass it as an argument, or call it immediately.

```go
// Assign to a variable
greet := func(name string) string {
    return "Hello, " + name
}
fmt.Println(greet("Alice"))  // Hello, Alice

// Call immediately (IIFE — immediately invoked function expression)
result := func(a, b int) int {
    return a + b
}(3, 4)
fmt.Println(result)  // 7
```

### Compared to JS/TS

```typescript
// JavaScript arrow function
const greet = (name: string): string => `Hello, ${name}`;

// JavaScript function expression
const greet = function(name: string): string { return `Hello, ${name}`; };
```

```go
// Go function literal (Go has no arrow syntax)
greet := func(name string) string { return "Hello, " + name }
```

Key differences:
- Go has no arrow syntax (`=>`) — there's exactly one function literal syntax
- Go function literals can't use `this` (methods work differently)
- Go doesn't have implicit returns — the `return` keyword is always required

### Your notes
<!-- -->

---

## Closures

A **closure** is a function that captures variables from its surrounding scope. The function and its captured variables form a bundle — the closure "closes over" the outer variables.

```go
func makeCounter() func() int {
    count := 0  // this variable is captured by the returned function
    return func() int {
        count++
        return count
    }
}

counter := makeCounter()
fmt.Println(counter())  // 1
fmt.Println(counter())  // 2
fmt.Println(counter())  // 3

counter2 := makeCounter()
fmt.Println(counter2())  // 1 — fresh count, independent closure
```

### How Closures Work Under the Hood

When a function literal captures a variable from an outer scope, the variable **escapes to the heap**. The compiler detects this with escape analysis and allocates the variable on the heap instead of the stack. The closure holds a pointer to that heap allocation.

```
makeCounter() call:
  Stack frame for makeCounter:
    count → [heap address 0x1234]

  Heap:
    0x1234: [0]   ← count lives here

  The returned func() int holds:
    env pointer → 0x1234   ← points to count on heap

After makeCounter returns, its stack frame is gone.
But count on the heap is kept alive by the closure.
```

This is why closures work in Go even after the outer function has returned — the captured variable stays alive as long as the closure is reachable.

### Capture by Reference

Go closures capture variables **by reference** (via pointer), not by value. This is where the loop variable gotcha comes from:

```go
// WRONG: all closures capture the same loop variable
funcs := make([]func(), 3)
for i := 0; i < 3; i++ {
    funcs[i] = func() {
        fmt.Println(i)  // i is captured by reference — points to same variable
    }
}
for _, f := range funcs {
    f()
}
// Output:
// 3
// 3
// 3
```

All three closures point to the same `i` variable. By the time they execute, the loop has finished and `i` is 3.

**Fix 1: Create a new variable per iteration (Go < 1.22)**

```go
for i := 0; i < 3; i++ {
    i := i  // shadow i with a new variable — each iteration gets its own
    funcs[i] = func() {
        fmt.Println(i)  // now captures the loop-local i
    }
}
```

**Fix 2: Pass as argument**

```go
for i := 0; i < 3; i++ {
    funcs[i] = func(n int) func() {
        return func() { fmt.Println(n) }
    }(i)  // i is evaluated and passed by value at loop time
}
```

**Fix 3: Go 1.22+ loop variable semantics (the language fixed this)**

As of Go 1.22, loop variables in `for` loops have **per-iteration scope**. Each iteration gets its own copy of the loop variable. The gotcha is gone for code compiled with Go 1.22+, but you'll encounter it in older code and in other languages (JS has the same issue without `let`).

> **See also:** [[go-closure-loop-gotcha]] in the vault for a full deep dive with Go version history.

### Your notes
<!-- -->

---

## Function Types and First-Class Functions

In Go, functions are first-class values. A function type describes the parameter and return types:

```go
type Predicate func(int) bool
type Transform func(string) string
type ErrorHandler func(error) bool
```

These can be used anywhere a type can be used: variable declarations, struct fields, function parameters, return types.

```go
// Function type as parameter — higher-order function
func filter(nums []int, keep func(int) bool) []int {
    var result []int
    for _, n := range nums {
        if keep(n) {
            result = append(result, n)
        }
    }
    return result
}

evens := filter([]int{1, 2, 3, 4, 5}, func(n int) bool {
    return n%2 == 0
})
// evens: [2, 4]
```

### Function Factories (Functions That Return Functions)

```go
// makeMultiplier returns a function that multiplies by n
func makeMultiplier(n int) func(int) int {
    return func(x int) int {
        return x * n
    }
}

double := makeMultiplier(2)
triple := makeMultiplier(3)

fmt.Println(double(5))  // 10
fmt.Println(triple(5))  // 15
```

This is the **factory pattern** using closures. Each returned function has its own captured `n`. It's equivalent to a class with a single method where the "state" is captured in the closure rather than a struct field.

### Compared to JS/TS

```typescript
// JS factory
const makeMultiplier = (n: number) => (x: number) => x * n;
const double = makeMultiplier(2);
```

The patterns are identical; Go just requires explicit types and no arrow syntax.

### Your notes
<!-- -->

---

## Higher-Order Functions

A **higher-order function** either takes a function as an argument or returns a function (or both). Go supports them natively — no special syntax required.

```go
// Map: transform each element
func mapInts(nums []int, fn func(int) int) []int {
    result := make([]int, len(nums))
    for i, n := range nums {
        result[i] = fn(n)
    }
    return result
}

// Filter: keep elements matching predicate
func filterInts(nums []int, fn func(int) bool) []int {
    var result []int
    for _, n := range nums {
        if fn(n) {
            result = append(result, n)
        }
    }
    return result
}

// Reduce: collapse to a single value
func reduceInts(nums []int, initial int, fn func(int, int) int) int {
    acc := initial
    for _, n := range nums {
        acc = fn(acc, n)
    }
    return acc
}
```

Note: Go 1.18+ has generics, so you'd normally write these with `func Map[T, U any](...)`. But the pattern is identical — generics just remove the type-specificity.

### Middleware Pattern

The most common real-world use of higher-order functions in Go is middleware: a function that wraps another function to add behavior.

```go
type HandlerFunc func(r Request) Response

// Logger wraps a handler and adds request logging
func Logger(next HandlerFunc) HandlerFunc {
    return func(r Request) Response {
        start := time.Now()
        resp := next(r)
        log.Printf("%s %s → %d (%s)", r.Method, r.Path, resp.Status, time.Since(start))
        return resp
    }
}

// Auth wraps a handler and checks authorization
func Auth(next HandlerFunc) HandlerFunc {
    return func(r Request) Response {
        if r.Header("Authorization") == "" {
            return Response{Status: 401}
        }
        return next(r)
    }
}
```

This is exactly what `net/http` middleware does — and what you'll implement in the exercises.

### Your notes
<!-- -->

---

## Method Expressions and Method Values

Go has **methods** — functions with a receiver. But you can also extract a method as a plain function value in two ways:

```go
type Formatter struct {
    prefix string
}

func (f Formatter) Format(msg string) string {
    return f.prefix + ": " + msg
}

f := Formatter{prefix: "INFO"}

// Method value: the receiver is bound — f is baked in
methodVal := f.Format
fmt.Println(methodVal("hello"))  // INFO: hello

// Method expression: receiver is the first argument
methodExpr := Formatter.Format
fmt.Println(methodExpr(f, "hello"))  // INFO: hello
```

**Method values** are closures that capture the receiver. They're useful for passing methods to higher-order functions:

```go
formatters := []func(string) string{f1.Format, f2.Format, f3.Format}
for _, fmt := range formatters {
    output := fmt("message")
    // ...
}
```

**Method expressions** treat the method as a function where the receiver is the first parameter — useful for dynamic dispatch or adapters.

### Your notes
<!-- -->

---

## init() Functions

Every package can define one or more `init()` functions. They run automatically before `main()`, after all package-level variables are initialized.

```go
package config

var defaultTimeout time.Duration

func init() {
    // Runs once, before any other code in this package
    defaultTimeout = 30 * time.Second
    if v := os.Getenv("TIMEOUT"); v != "" {
        // parse and override
    }
}
```

Rules:
- `init()` cannot be called directly — only the runtime calls it
- A package can have multiple `init()` functions (even in the same file)
- They run in the order they appear, in a single goroutine
- All `init()` in imported packages run before the importing package's `init()`

> **Cross-reference:** `init()` is covered in depth in the `modules-and-packages` module. For now, know it exists and what it's used for: one-time setup that can't be done at package-level variable initialization (e.g., needs I/O, needs other packages to be ready).

### Your notes
<!-- -->

---

## Putting It Together: The Composition Pattern

The real power of functions-as-values is **composition** — building complex behavior from simple, focused functions.

```go
// A pipeline: each step transforms the data
type Pipeline[T any] struct {
    stages []func(T) T
}

func (p *Pipeline[T]) Add(stage func(T) T) *Pipeline[T] {
    p.stages = append(p.stages, stage)
    return p  // method chaining
}

func (p *Pipeline[T]) Run(input T) T {
    result := input
    for _, stage := range p.stages {
        result = stage(result)
    }
    return result
}
```

```go
// Practical example: request processing pipeline
normalize := func(s string) string { return strings.TrimSpace(strings.ToLower(s)) }
sanitize  := func(s string) string { return strings.ReplaceAll(s, "<script>", "") }
truncate  := func(s string) string {
    if len(s) > 100 { return s[:100] }
    return s
}

p := &Pipeline[string]{}
p.Add(normalize).Add(sanitize).Add(truncate)

clean := p.Run("  Hello WORLD <script>  ")
// "hello world "
```

This pattern directly precedes the middleware exercise. You're building the same idea: a chain of functions, each transforming the input.

### Your notes
<!-- -->
