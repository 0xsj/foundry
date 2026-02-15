# Control Flow in Go

Control flow determines how your program makes decisions and repeats operations. Go's approach is deliberately minimal—it has fewer control structures than most languages, but what it has is powerful and composable.

## Conditionals: If/Else

Go's `if` statement is straightforward but has one powerful feature: you can execute a statement before the condition.

```go
// Standard if
if temperature > 30 {
    fmt.Println("It's hot")
}

// If with initialization statement
if temp := getTemperature(); temp > 30 {
    fmt.Println("It's hot:", temp)
}
// temp is scoped to the if block
```

**Why the initialization statement matters:** This pattern keeps variables scoped tightly. You'll see this everywhere in Go, especially with error handling:

```go
if err := validateConfig(cfg); err != nil {
    return fmt.Errorf("invalid config: %w", err)
}
```

The `err` variable exists only within the if block, preventing it from polluting the surrounding scope.

### Comparison to JavaScript/TypeScript

In JS/TS, you'd write:
```typescript
const err = validateConfig(cfg)
if (err !== null) {
    throw new Error(`invalid config: ${err}`)
}
// err is still in scope here (not ideal)
```

Go's scoped initialization is cleaner—the variable disappears when you're done with it.

### No Parentheses, But Braces Required

Go doesn't require parentheses around conditions but **always requires braces**, even for single-line blocks:

```go
// ✅ Correct
if x > 10 {
    fmt.Println("x is large")
}

// ❌ Won't compile
if x > 10
    fmt.Println("x is large")

// ❌ Won't compile (no one-liners without braces)
if x > 10 fmt.Println("x is large")
```

This prevents bugs from omitted braces and enforces consistency.

## Loops: For (and Only For)

Go has exactly one loop construct: `for`. It's versatile enough to handle all looping scenarios.

### Traditional Three-Component Loop

```go
for i := 0; i < 10; i++ {
    fmt.Println(i)
}
```

Just like C or Java. The initialization, condition, and post statement are all optional:

### While-Style Loop

```go
i := 0
for i < 10 {
    fmt.Println(i)
    i++
}
```

Omit the init and post statements, and you have a while loop.

### Infinite Loop

```go
for {
    // runs forever (or until break/return)
    if shouldStop() {
        break
    }
}
```

Omit everything, and you get an infinite loop. Use `break` to exit.

### Range Loops

The `range` keyword lets you iterate over slices, arrays, maps, strings, and channels:

```go
numbers := []int{10, 20, 30}

// Index and value
for i, num := range numbers {
    fmt.Printf("numbers[%d] = %d\n", i, num)
}

// Just values (ignore index with _)
for _, num := range numbers {
    fmt.Println(num)
}

// Just indices
for i := range numbers {
    fmt.Println(i)
}
```

**Important:** `range` creates **copies** of values. If you need to modify elements, use the index:

```go
// ❌ This won't modify the slice
for _, num := range numbers {
    num = num * 2  // modifies the copy, not the slice
}

// ✅ This will
for i := range numbers {
    numbers[i] = numbers[i] * 2
}
```

### Continue and Break

- `continue` skips to the next iteration
- `break` exits the loop entirely

```go
for i := 0; i < 10; i++ {
    if i%2 == 0 {
        continue  // skip even numbers
    }
    if i > 7 {
        break  // stop at 7
    }
    fmt.Println(i)  // prints 1, 3, 5, 7
}
```

### Labeled Breaks (Breaking Out of Nested Loops)

```go
outer:
    for i := 0; i < 5; i++ {
        for j := 0; j < 5; j++ {
            if i*j > 6 {
                break outer  // breaks out of both loops
            }
            fmt.Printf("(%d,%d) ", i, j)
        }
    }
```

Without the label, `break` would only exit the inner loop.

## Switch Statements

Go's `switch` is more powerful than in C-family languages:

### Basic Switch

```go
switch day {
case "Monday":
    fmt.Println("Start of the week")
case "Friday":
    fmt.Println("Almost weekend")
case "Saturday", "Sunday":
    fmt.Println("Weekend!")
default:
    fmt.Println("Midweek")
}
```

**No fallthrough by default.** In C/Java, cases fall through unless you `break`. In Go, each case automatically breaks. If you want fallthrough, you explicitly say so:

```go
switch num {
case 1:
    fmt.Println("One")
    fallthrough
case 2:
    fmt.Println("One or Two")  // runs if num is 1 or 2
}
```

### Switch with Initialization

Just like `if`, you can run a statement first:

```go
switch err := doSomething(); err {
case nil:
    fmt.Println("Success")
case ErrNotFound:
    fmt.Println("Not found")
default:
    fmt.Println("Error:", err)
}
```

### Expression Switch (Condition-less)

Omit the expression to use `switch` as a cleaner if-else chain:

```go
switch {
case temperature < 0:
    fmt.Println("Freezing")
case temperature < 20:
    fmt.Println("Cold")
case temperature < 30:
    fmt.Println("Warm")
default:
    fmt.Println("Hot")
}
```

This is **idiomatic Go** for complex conditionals. It's clearer than nested if-else statements.

### Type Switch

You can switch on a value's type (we'll cover this more in interfaces):

```go
switch v := value.(type) {
case int:
    fmt.Println("Integer:", v)
case string:
    fmt.Println("String:", v)
default:
    fmt.Println("Unknown type")
}
```

## Early Returns and Guard Clauses

**Idiomatic Go prefers early returns over deep nesting.** Handle errors and edge cases first, then proceed with the happy path.

### ❌ Nested Style (Not Idiomatic)

```go
func processFile(path string) error {
    if file, err := os.Open(path); err == nil {
        defer file.Close()
        if data, err := io.ReadAll(file); err == nil {
            if err := validate(data); err == nil {
                return process(data)
            } else {
                return err
            }
        } else {
            return err
        }
    } else {
        return err
    }
}
```

Deeply nested, hard to read.

### ✅ Guard Clause Style (Idiomatic)

```go
func processFile(path string) error {
    file, err := os.Open(path)
    if err != nil {
        return err
    }
    defer file.Close()

    data, err := io.ReadAll(file)
    if err != nil {
        return err
    }

    if err := validate(data); err != nil {
        return err
    }

    return process(data)
}
```

**Guard clauses** exit early on failure. The happy path stays at the top level, making the logic linear and clear.

### The Go If-Err Pattern

This is the most common pattern in Go code:

```go
result, err := doSomething()
if err != nil {
    return err  // or handle it
}
// use result
```

You'll write this hundreds of times. It's explicit, predictable, and makes error handling visible.

**Coming from JavaScript:** This feels verbose compared to try-catch, but it has advantages:
- Errors are values, not exceptions
- You can't accidentally ignore errors (the compiler checks)
- Error handling is local and visible

## How It Works Under the Hood

### Conditionals Compile to Jumps

At the assembly level, `if` statements become conditional jump instructions. The CPU evaluates the condition and jumps to different addresses based on the result.

For example:
```go
if x > 10 {
    doA()
} else {
    doB()
}
```

Compiles roughly to:
```
COMPARE x, 10
JUMP_IF_LESS_EQUAL else_block
CALL doA
JUMP end
else_block:
CALL doB
end:
```

### Loops Compile to Labels and Jumps

A `for` loop becomes a label and a conditional jump back to the start:

```go
for i := 0; i < 10; i++ {
    doSomething(i)
}
```

Compiles roughly to:
```
MOVE i, 0
loop_start:
COMPARE i, 10
JUMP_IF_GREATER_EQUAL loop_end
CALL doSomething(i)
INCREMENT i
JUMP loop_start
loop_end:
```

### Switch Statements Can Optimize

Go's compiler can optimize `switch` statements in different ways depending on the cases:

- **Jump table:** For dense integer cases, the compiler may generate a lookup table
- **Binary search:** For sparse but ordered cases
- **If-else chain:** For complex conditions

You don't need to think about this—the compiler chooses the best strategy.

## Key Differences from Other Languages

| Feature | Go | JavaScript/TypeScript | Rust | Python |
|---------|----|-----------------------|------|--------|
| If without braces | ❌ Always required | ✅ Optional (bad idea) | ❌ Not applicable (no braces) | ❌ Uses indentation |
| Parentheses in conditions | ❌ Not allowed | ✅ Required | ❌ Not allowed | ❌ Not allowed |
| Loop constructs | `for` only | `for`, `while`, `do-while`, `for-of`, `for-in` | `loop`, `while`, `for` | `for`, `while` |
| Switch fallthrough | ❌ Explicit `fallthrough` needed | ✅ Default (must `break`) | ❌ Uses `match` (no fallthrough) | ❌ No switch (uses if-elif) |
| Type switch | ✅ Built-in | ❌ Not built-in | ✅ Via `match` | ❌ Use isinstance() |

## Common Patterns

### Retry Loop with Backoff

```go
for attempt := 0; attempt < maxRetries; attempt++ {
    if err := tryConnect(); err == nil {
        return nil  // success
    }
    time.Sleep(time.Second * time.Duration(1<<attempt))  // exponential backoff
}
return fmt.Errorf("failed after %d attempts", maxRetries)
```

### State Machine with Switch

```go
state := "start"
for {
    switch state {
    case "start":
        state = initialize()
    case "processing":
        state = process()
    case "done":
        return nil
    case "error":
        return fmt.Errorf("state machine failed")
    }
}
```

### Validation Guard Clauses

```go
func createUser(name, email string, age int) error {
    if name == "" {
        return errors.New("name required")
    }
    if email == "" {
        return errors.New("email required")
    }
    if age < 18 {
        return errors.New("must be 18+")
    }
    // happy path: create user
    return nil
}
```

## Your Notes

**Questions that came up:**

**Patterns I noticed:**

**Comparisons to languages I know:**

**Things I want to try:**
