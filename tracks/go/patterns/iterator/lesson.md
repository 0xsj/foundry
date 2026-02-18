# Iterator / Generator Pattern -- Go

## The Problem Iterator Solves

You have a collection of things -- log entries, database rows, paginated API results, lines in a file, events on a stream -- and you need to process them one at a time. The naive approach is to load everything into a slice, pass it around, and loop over it. This works until:

- The data doesn't fit in memory (a 10GB log file, a table with 50 million rows)
- You only need the first N items (but you loaded all 10,000 pages)
- You want to compose processing steps (filter, then map, then take) without materializing intermediate collections
- The data source is infinite (Fibonacci numbers, a live event stream, a random number generator)
- The collection's internal structure is complex and you don't want consumers to know about it (tree traversal, graph BFS, skip list)

The Iterator pattern provides a uniform way to traverse elements one at a time, without exposing the underlying data structure, and without requiring everything to be in memory at once.

If you've used JavaScript generators (`function*`), Python's `for x in iterable`, or Rust's `Iterator` trait, you already know this pattern. The question is: how does Go express it?

### The short answer

Go has *four* idiomatic ways to express iteration, and you need to know when to reach for each one:

| Approach | Mechanism | Best For | Watch Out |
|----------|-----------|----------|-----------|
| Callback / `Walk` | `func(item T) error` passed to traversal | Tree/graph traversal, filesystem walks | Caller can't break easily (pre-Go 1.23) |
| Closure iterator | `func() (T, bool)` returned to caller | Simple pull-based sequences, parsers | Manual `for` loop, no `range` support (pre-1.23) |
| Channel generator | `chan T` returned, goroutine produces | Concurrent production, fan-out | Goroutine leaks if consumer exits early |
| Range-over-function (Go 1.23+) | `iter.Seq[T]` / `iter.Seq2[K,V]` | Everything -- the new standard | Requires Go 1.23+, new mental model |

Go 1.23 introduced range-over-function iterators, which fundamentally changed iteration in Go. Before 1.23, the standard library used callbacks and closures. After 1.23, `iter.Seq` and `iter.Seq2` are the idiomatic way to build composable iterators. We'll cover all four approaches, but range-over-function is where you should invest your mental energy.

### Real-world situations where Iterator appears

- **Log processing pipeline**: Read lines from rotated log files, filter by severity, parse structured fields, aggregate stats -- all without loading everything into memory
- **Database result scanning**: `sql.Rows` is an iterator -- call `Next()`, `Scan()`, repeat until done
- **Paginated API consumption**: Fetch page 1, yield results, fetch page 2 when needed -- the consumer sees a flat sequence
- **File system traversal**: `filepath.WalkDir` walks a directory tree, calling your function on each entry
- **Event stream processing**: Read events from Kafka/NATS, apply transformations, forward to sinks
- **Configuration loading**: Iterate over config sources (env, file, defaults) in priority order, merging values
- **Test data generation**: Produce an infinite stream of randomized test fixtures, take only what the test needs

### Your notes
<!-- User adds insights here during learning -->


---

## Approach 1: Callback-Based Iteration (The `Walk` Pattern)

### How It Works Under the Hood

The oldest and most common iteration pattern in Go's standard library is callback-based: the data source controls the loop and calls your function on each element.

```go
// filepath.WalkDir -- the data source drives the loop
filepath.WalkDir("/var/log", func(path string, d fs.DirEntry, err error) error {
    if err != nil {
        return err // stop walking
    }
    if d.IsDir() {
        return nil // skip directories
    }
    fmt.Println(path)
    return nil
})
```

The traversal function (`WalkDir`) owns the loop. Your callback receives each element. Return `nil` to continue, return an error to stop. This is push-based iteration -- the producer pushes elements to the consumer.

You've seen this shape in JavaScript with `Array.forEach()`, but with a critical difference: in Go, returning an error from the callback *stops iteration*. In JS, `forEach` always runs to completion (you can't `break` out of it).

**Standard library examples of callback iteration:**

| Function | Callback Shape | Purpose |
|----------|---------------|---------|
| `filepath.WalkDir` | `func(path string, d fs.DirEntry, err error) error` | Traverse directory tree |
| `(*http.Header).Write` | N/A (writes directly) | -- |
| `(*flag.FlagSet).Visit` | `func(*Flag)` | Visit all set flags |
| `(*sync.Map).Range` | `func(key, value any) bool` | Iterate concurrent map |

Notice `sync.Map.Range` uses a different convention: return `true` to continue, `false` to stop. This inconsistency is one reason Go introduced a standard iterator protocol in 1.23.

### When to Use Callback Iteration

- You're writing a tree or graph traversal where the producer manages a stack/queue
- The data source needs to do cleanup after iteration (close files, release locks)
- You're wrapping an existing callback-based API
- Pre-Go 1.23 code that can't use range-over-function

### The Limitation

The consumer can't pause iteration, resume later, or compose multiple callback iterators together. You're locked into the producer's control flow. This is the fundamental limitation that range-over-function solves.

### Your notes
<!-- -->


---

## Approach 2: Closure-Based Iterators (`func() (T, bool)`)

### How It Works Under the Hood

A closure iterator returns a function that produces one element at a time. The *consumer* drives the loop by calling the function repeatedly until it signals "done."

```go
// Returns a function that produces Fibonacci numbers one at a time.
func fibonacci() func() (int, bool) {
    a, b := 0, 1
    return func() (int, bool) {
        v := a
        a, b = b, a+b
        return v, true // always has more (infinite sequence)
    }
}

// Consumer pulls values
next := fibonacci()
for i := 0; i < 10; i++ {
    v, _ := next()
    fmt.Println(v)
}
```

This is pull-based iteration -- the consumer controls the pace. The closure captures state (`a`, `b`) across calls.

This is directly analogous to the `Symbol.iterator` protocol in JavaScript:

```typescript
// TypeScript: equivalent closure iterator
function fibonacci(): () => [number, boolean] {
    let a = 0, b = 1;
    return () => {
        const v = a;
        [a, b] = [b, a + b];
        return [v, true];
    };
}
```

**Key advantage over callbacks:** The consumer controls the loop. You can stop early, interleave multiple iterators, or pass the iterator function to another function.

**Key limitation:** You can't use `range` with a closure iterator (pre-Go 1.23). You write manual `for` loops with `next()` calls.

### `bufio.Scanner`: The Standard Library's Closure Iterator

`bufio.Scanner` is Go's most-used closure iterator, though it uses methods instead of a bare function:

```go
scanner := bufio.NewScanner(file)
for scanner.Scan() {       // pull next line (returns true/false)
    line := scanner.Text()  // get the value
    fmt.Println(line)
}
if err := scanner.Err(); err != nil {
    log.Fatal(err)          // CRITICAL: always check Err() after the loop
}
```

`sql.Rows` follows the same pattern:

```go
rows, err := db.Query("SELECT id, name FROM users")
if err != nil { log.Fatal(err) }
defer rows.Close()

for rows.Next() {           // pull next row
    var id int
    var name string
    rows.Scan(&id, &name)   // get values
    fmt.Println(id, name)
}
if err := rows.Err(); err != nil {
    log.Fatal(err)           // always check Err()
}
```

The pattern is: call `Next()` to advance, call a getter to read the value, check `Err()` when done. This was Go's de facto iterator protocol before 1.23.

### Your notes
<!-- -->


---

## Approach 3: Channel-Based Generators

### How It Works Under the Hood

A goroutine produces values and sends them on a channel. The consumer ranges over the channel. This is the Go equivalent of Python/JavaScript generators -- a coroutine that yields values.

```go
func fibonacci(ctx context.Context) <-chan int {
    ch := make(chan int)
    go func() {
        defer close(ch)
        a, b := 0, 1
        for {
            select {
            case <-ctx.Done():
                return // consumer cancelled -- stop producing
            case ch <- a:
                a, b = b, a+b
            }
        }
    }()
    return ch
}

// Consumer ranges over the channel
ctx, cancel := context.WithCancel(context.Background())
defer cancel() // stops the goroutine

for v := range fibonacci(ctx) {
    if v > 1000 {
        break // safe -- cancel() in defer will clean up the goroutine
    }
    fmt.Println(v)
}
```

**This looks clean, but there's a trap.** If you forget context cancellation or `break` without cancelling, the producer goroutine blocks on `ch <-` forever. This is a **goroutine leak** -- one of Go's most common resource leaks, and it's silent. No error, no panic, just a goroutine consuming memory until the process dies.

### The Goroutine Leak Problem

```go
// DANGEROUS: no cancellation mechanism
func generate(items []string) <-chan string {
    ch := make(chan string)
    go func() {
        defer close(ch)
        for _, item := range items {
            ch <- item // blocks forever if consumer stops reading
        }
    }()
    return ch
}

// Consumer reads 3 items and walks away
ch := generate(hugeList)
for i := 0; i < 3; i++ {
    fmt.Println(<-ch)
}
// goroutine is now blocked on ch <- forever
// it will never be garbage collected
```

**Rule: Every channel-based iterator MUST accept a `context.Context` for cancellation.** No exceptions. If someone tells you "the consumer will always read everything," they're wrong. Panics happen. Early returns happen. Tests get cancelled.

### When to Use Channel Generators

| Use channels when... | Don't use channels when... |
|-----------------------|---------------------------|
| Production is genuinely concurrent (network I/O, parallel computation) | You just need sequential iteration |
| You need fan-out to multiple consumers | A closure or range-over-function would work |
| The producer and consumer run at different speeds (buffered channel) | Performance matters (channels are ~50-100ns per send/recv) |
| You're bridging goroutine boundaries | You're iterating a data structure in the same goroutine |

Channels are a concurrency primitive, not an iteration primitive. Using them for sequential iteration adds goroutine overhead, channel synchronization cost, and leak risk for no benefit. Since Go 1.23, range-over-function handles sequential iteration better in every way.

### Your notes
<!-- -->


---

## Approach 4: Range-Over-Function (Go 1.23+)

### How It Works Under the Hood

Go 1.23 introduced the ability to `range` over functions. A function with a specific signature becomes iterable:

```go
// iter.Seq[V] is defined as:
// type Seq[V any]  func(yield func(V) bool)

// iter.Seq2[K, V] is defined as:
// type Seq2[K, V any]  func(yield func(K, V) bool)
```

A range-over-function iterator is a function that receives a `yield` callback and calls it for each element. The `yield` function returns `false` when the consumer wants to stop (via `break`, `return`, etc.).

```go
// An iterator that yields numbers 0..n-1
func upTo(n int) iter.Seq[int] {
    return func(yield func(int) bool) {
        for i := 0; i < n; i++ {
            if !yield(i) {
                return // consumer said stop
            }
        }
    }
}

// Consumer uses range -- just like slices, maps, channels
for v := range upTo(10) {
    fmt.Println(v)
}
```

This is the best of all worlds:
- **`range` syntax** -- familiar, composable, works with `break`/`continue`/`return`
- **No goroutines** -- runs in the same goroutine as the consumer
- **Lazy** -- produces one element at a time, no intermediate allocations
- **Composable** -- iterators can be chained (filter, map, take, etc.)
- **Safe** -- no goroutine leaks, no channel cleanup

### Mental Model: Push That Respects Pull

Range-over-function is technically push-based (the iterator calls `yield`), but `yield` returns `false` when the consumer breaks. So the consumer has full control -- it's push-that-respects-pull.

Compare to JavaScript generators:

```typescript
// TypeScript generator -- pull-based via yield
function* upTo(n: number): Generator<number> {
    for (let i = 0; i < n; i++) {
        yield i;
    }
}

for (const v of upTo(10)) {
    console.log(v);
}
```

Same user experience, different implementation. JS generators use coroutine suspension (the generator function literally pauses). Go uses a callback (the yield function). The consumer sees the same `for...of` / `for...range` syntax.

Compare to Rust iterators:

```rust
// Rust: Iterator trait -- pull-based via next()
fn up_to(n: usize) -> impl Iterator<Item = usize> {
    (0..n).into_iter()
}

for v in up_to(10) {
    println!("{}", v);
}
```

Rust's approach is purely pull-based (call `next()` to get each element). Go 1.23's approach is push-with-stop-signal. Both compose, both are lazy, both are safe. Rust's is zero-cost (no function call overhead for `yield`). Go's has a function call per element, but the overhead is negligible in practice (~2-5ns per yield).

### `iter.Seq` vs `iter.Seq2`

Use `iter.Seq[V]` for sequences of values (like `[]V`):

```go
func lines(r io.Reader) iter.Seq[string] {
    return func(yield func(string) bool) {
        scanner := bufio.NewScanner(r)
        for scanner.Scan() {
            if !yield(scanner.Text()) {
                return
            }
        }
    }
}
```

Use `iter.Seq2[K, V]` for key-value pairs (like `map[K]V`) or sequences that carry an error:

```go
// Yielding index + value (like enumerate in Python)
func enumerate[T any](seq iter.Seq[T]) iter.Seq2[int, T] {
    return func(yield func(int, T) bool) {
        i := 0
        for v := range seq {
            if !yield(i, v) {
                return
            }
            i++
        }
    }
}

// Yielding value + error (like Rust's Iterator<Item = Result<T, E>>)
func linesWithErr(r io.Reader) iter.Seq2[string, error] {
    return func(yield func(string, error) bool) {
        scanner := bufio.NewScanner(r)
        for scanner.Scan() {
            if !yield(scanner.Text(), nil) {
                return
            }
        }
        if err := scanner.Err(); err != nil {
            yield("", err)
        }
    }
}
```

### `iter.Pull`: Converting Push to Pull

Sometimes you need pull-based iteration (call `next()` to get each value). The `iter.Pull` function converts a push-based `iter.Seq` into a pull-based `(next, stop)` pair:

```go
seq := slices.Values([]string{"a", "b", "c"})
next, stop := iter.Pull(seq)
defer stop() // MUST call stop to release resources

v, ok := next()  // "a", true
v, ok = next()   // "b", true
v, ok = next()   // "c", true
v, ok = next()   // "", false -- done
```

`iter.Pull` spawns a goroutine internally to bridge push and pull. Always call `stop()` when done -- otherwise you leak the goroutine. This is the same trap as channel-based iterators, so use `defer stop()` immediately after calling `Pull`.

### Your notes
<!-- -->


---

## Building Iterator Pipelines

### The Composability Advantage

The real power of iterators is composition. Instead of writing one monolithic function that reads, filters, transforms, and aggregates data, you chain small, reusable iterator transformers.

```go
// Pipeline: read lines -> filter errors -> extract message -> take first 100
result := take(100,
    mapIter(extractMessage,
        filter(isError,
            lines(file))))

for msg := range result {
    fmt.Println(msg)
}
```

No intermediate slices. Each element flows through the pipeline one at a time, all in the same goroutine.

### Standard Combinators

These are the building blocks you'll write once and reuse everywhere:

```go
// Filter keeps elements where pred returns true
func filter[T any](pred func(T) bool, seq iter.Seq[T]) iter.Seq[T] {
    return func(yield func(T) bool) {
        for v := range seq {
            if pred(v) {
                if !yield(v) {
                    return
                }
            }
        }
    }
}

// Map transforms each element
func mapIter[T, U any](f func(T) U, seq iter.Seq[T]) iter.Seq[U] {
    return func(yield func(U) bool) {
        for v := range seq {
            if !yield(f(v)) {
                return
            }
        }
    }
}

// Take yields at most n elements
func take[T any](n int, seq iter.Seq[T]) iter.Seq[T] {
    return func(yield func(T) bool) {
        i := 0
        for v := range seq {
            if i >= n {
                return
            }
            if !yield(v) {
                return
            }
            i++
        }
    }
}

// Skip discards the first n elements
func skip[T any](n int, seq iter.Seq[T]) iter.Seq[T] {
    return func(yield func(T) bool) {
        i := 0
        for v := range seq {
            if i < n {
                i++
                continue
            }
            if !yield(v) {
                return
            }
        }
    }
}
```

### Collecting Results

When you're done composing, collect the results into a concrete type:

```go
// Collect gathers all elements from an iterator into a slice
func collect[T any](seq iter.Seq[T]) []T {
    var result []T
    for v := range seq {
        result = append(result, v)
    }
    return result
}

// Reduce folds all elements into a single value
func reduce[T, U any](seq iter.Seq[T], initial U, f func(U, T) U) U {
    acc := initial
    for v := range seq {
        acc = f(acc, v)
    }
    return acc
}
```

### The `slices` and `maps` Packages (Go 1.23+)

Go 1.23 also added iterator support to the standard library:

```go
import "slices"

// slices.Values converts []T to iter.Seq[T]
for v := range slices.Values([]string{"a", "b", "c"}) {
    fmt.Println(v)
}

// slices.All converts []T to iter.Seq2[int, T] (index, value)
for i, v := range slices.All([]string{"a", "b", "c"}) {
    fmt.Printf("%d: %s\n", i, v)
}

// slices.Collect converts iter.Seq[T] to []T
result := slices.Collect(take(5, fibonacci()))

// slices.Sorted collects and sorts
sorted := slices.Sorted(slices.Values([]int{3, 1, 4, 1, 5}))
```

### Your notes
<!-- -->


---

## Lazy Evaluation and Infinite Sequences

### The Core Insight

An iterator that never terminates is perfectly valid. The consumer decides how much to take. This is impossible with slices -- you can't have an infinite slice.

```go
// Infinite sequence of natural numbers
func naturals() iter.Seq[int] {
    return func(yield func(int) bool) {
        n := 0
        for {
            if !yield(n) {
                return
            }
            n++
        }
    }
}

// Take the first 5 even numbers
evens := filter(func(n int) bool { return n%2 == 0 }, naturals())
first5 := take(5, evens)
for v := range first5 {
    fmt.Println(v) // 0, 2, 4, 6, 8
}
```

This is the same concept as JavaScript generators or Haskell's lazy lists:

```typescript
// TypeScript: infinite generator
function* naturals() {
    let n = 0;
    while (true) yield n++;
}
```

```haskell
-- Haskell: infinite lazy list
take 5 (filter even [0..])  -- [0, 2, 4, 6, 8]
```

### Paginated API Results

One of the most practical uses of lazy iteration: transparently paginating through an API.

```go
func allUsers(client *APIClient) iter.Seq2[User, error] {
    return func(yield func(User, error) bool) {
        cursor := ""
        for {
            page, nextCursor, err := client.ListUsers(cursor, 100)
            if err != nil {
                yield(User{}, err)
                return
            }
            for _, user := range page {
                if !yield(user, nil) {
                    return // consumer stopped early -- no wasted API calls
                }
            }
            if nextCursor == "" {
                return // no more pages
            }
            cursor = nextCursor
        }
    }
}

// Consumer sees a flat stream of users, doesn't know about pagination
for user, err := range allUsers(client) {
    if err != nil {
        log.Fatal(err)
    }
    processUser(user)
}
```

The consumer doesn't know or care that data arrives in pages. The iterator fetches the next page only when the current one is exhausted. If the consumer breaks after 50 users, only one page was fetched.

### Your notes
<!-- -->


---

## Anti-Patterns and Gotchas

### 1. Goroutine Leak from Channel Iterators

The most dangerous iterator bug in Go. If a consumer stops reading from a channel-based iterator without signaling cancellation, the producer goroutine blocks forever.

```go
// DANGEROUS
func generate() <-chan int {
    ch := make(chan int)
    go func() {
        for i := 0; ; i++ {
            ch <- i // blocks forever if nobody reads
        }
    }()
    return ch
}
```

**Fix:** Always use `context.Context` and `select`:

```go
func generate(ctx context.Context) <-chan int {
    ch := make(chan int)
    go func() {
        defer close(ch)
        for i := 0; ; i++ {
            select {
            case <-ctx.Done():
                return
            case ch <- i:
            }
        }
    }()
    return ch
}
```

### 2. Materializing When Lazy Would Work

```go
// BAD: loads everything into memory, then iterates
func processLogs(path string) {
    data, _ := os.ReadFile(path) // entire 5GB file in memory
    lines := strings.Split(string(data), "\n")
    for _, line := range lines {
        // ...
    }
}

// GOOD: lazy line-by-line iteration
func processLogs(path string) {
    f, _ := os.Open(path)
    defer f.Close()
    for line := range lines(f) { // processes one line at a time
        // ...
    }
}
```

### 3. Calling Yield After Iterator Returns

With range-over-function, the `yield` function becomes invalid after the iterator function returns. Calling it later (e.g., from a goroutine) is a runtime panic.

```go
// DANGEROUS: yield escapes the iterator
func broken() iter.Seq[int] {
    return func(yield func(int) bool) {
        go func() {
            yield(42) // PANIC: yield called after iterator returned
        }()
    }
}
```

### 4. Not Checking `Err()` After Scanner/Rows

```go
scanner := bufio.NewScanner(file)
for scanner.Scan() {
    process(scanner.Text())
}
// If scanner hit a read error, scanner.Scan() returns false
// and scanner.Err() returns the error.
// If you don't check Err(), you silently lose data.
if err := scanner.Err(); err != nil {
    log.Fatal(err)
}
```

### 5. Forgetting `stop()` After `iter.Pull`

```go
next, stop := iter.Pull(seq)
// If you forget defer stop(), the internal goroutine leaks.
defer stop() // ALWAYS do this immediately
```

### Your notes
<!-- -->


---

## Cross-Language Comparison

| Feature | Go (1.23+) | TypeScript | Rust | Python |
|---------|-----------|------------|------|--------|
| Protocol | `iter.Seq[T]` (push w/ stop) | `Symbol.iterator` / `function*` | `Iterator` trait (pull) | `__iter__` / `__next__` (pull) |
| Syntax | `for v := range seq` | `for (const v of iter)` | `for v in iter` | `for v in iter:` |
| Lazy by default | Yes (iterators) | Yes (generators) | Yes (Iterator adaptors) | Yes (generators) |
| Infinite sequences | Yes | Yes | Yes | Yes |
| Composition | Manual functions | Manual / libraries | `.filter().map().take()` (built-in) | `itertools` / comprehensions |
| Error handling | `iter.Seq2[T, error]` | Throw from generator | `Iterator<Item=Result<T,E>>` | Raise from generator |
| Coroutine-based | No (callback) | Yes (generator suspension) | No (trait methods) | Yes (generator suspension) |
| Conversion stdlib | `slices.Values`, `maps.Keys` | `Array.from()` | `.collect::<Vec<_>>()` | `list()`, `tuple()` |

### Key Insight

Go's range-over-function iterators are push-based with a stop signal -- the iterator calls `yield(v)` and checks the return value. This is different from Rust's pull-based `next() -> Option<T>` and JavaScript's coroutine-based `yield`. The user-facing syntax (`for ... range`) is identical to ranging over slices, maps, and channels, which makes adoption painless.

Rust has the richest composition story -- `.filter().map().take().collect()` chains are built into the standard library. Go requires writing the combinator functions yourself (or using a library). This is a deliberate trade-off: Go favors explicit code over method chains.

### Your notes
<!-- -->


---

## Standard Library Iterators Worth Knowing

### `bufio.Scanner`

Line-by-line file reading. The most common iterator in real Go code.

```go
scanner := bufio.NewScanner(reader)
scanner.Split(bufio.ScanWords) // custom split function -- scan words instead of lines
for scanner.Scan() {
    fmt.Println(scanner.Text())
}
```

### `sql.Rows`

Database result iteration. Always `defer rows.Close()`, always check `rows.Err()`.

### `filepath.WalkDir`

Directory tree traversal. Callback-based. Use `fs.SkipDir` to prune subtrees. Replaced the older `filepath.Walk` (which `os.Stat`s every file -- `WalkDir` avoids this).

### `encoding/json.Decoder`

Streaming JSON parsing. Useful for large JSON arrays:

```go
dec := json.NewDecoder(reader)
dec.Token() // read opening [
for dec.More() {
    var item MyStruct
    dec.Decode(&item)
    process(item)
}
```

### `slices.Values` / `slices.All` / `maps.Keys` / `maps.Values` (Go 1.23+)

Convert standard collections to `iter.Seq` / `iter.Seq2`.

### Your notes
<!-- -->


---

## When to Use Which Approach

| Situation | Recommended Approach |
|-----------|---------------------|
| New code, Go 1.23+ | Range-over-function (`iter.Seq`) |
| Composable pipeline (filter/map/take) | Range-over-function |
| Wrapping `bufio.Scanner` or `sql.Rows` | Range-over-function wrapping the scanner loop |
| Tree/graph traversal with cleanup | Range-over-function (replaces Walk callbacks) |
| Need pull-based from push iterator | `iter.Pull` |
| Fan-out to multiple consumers | Channels (true concurrency) |
| Producer involves network I/O in a separate goroutine | Channels with context cancellation |
| Pre-Go 1.23 codebase | Closure iterator `func() (T, bool)` or callback |

The rule of thumb: **start with `iter.Seq`. Reach for channels only when you need actual concurrency.** Channels are for communication between goroutines, not for sequential iteration.

### Your notes
<!-- -->
