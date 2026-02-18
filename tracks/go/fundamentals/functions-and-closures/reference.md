# Go Specification Reference — Functions and Closures

> Extracted from [The Go Programming Language Specification](https://go.dev/ref/spec)
> for the `functions-and-closures` module. Covers: function types, function declarations,
> function literals, closures, method declarations, defer statements, and call expressions.

---

## Function Types

Source: [spec#Function_types](https://go.dev/ref/spec#Function_types)

A function type denotes the set of all functions with the same parameter and result types. The value of an uninitialized variable of function type is `nil`.

### EBNF

```
FunctionType   = "func" Signature .
Signature      = Parameters [ Result ] .
Result         = Parameters | Type .
Parameters     = "(" [ ParameterList [ "," ] ] ")" .
ParameterList  = ParameterDecl { "," ParameterDecl } .
ParameterDecl  = [ IdentifierList ] [ "..." ] Type .
```

### Examples

```go
func()
func(x int) int
func(a, _ int, z float32) bool
func(a, b int, z float32) (bool)
func(prefix string, values ...int)
func(a, b int, z float64, opt ...interface{}) (success bool)
func(int, int, float64) (float64, *[]int)
func(n int) func(p *T)
```

### Variadic Parameters

A parameter section with a `...` prefix is **variadic**. The function may be invoked with zero or more arguments for that parameter. If the function is invocable, the variadic parameter is equivalent to a parameter of type `[]T`:

```go
func(a, b int, z float64, opt ...interface{}) (success bool)
// opt has type []interface{}
```

Only the **final** parameter may be variadic.

### Two Identical Function Type Examples

```go
func(int, int, float64) (float64, *[]int)
func(x int, y int, z float64) (result float64, results *[]int)
// These are identical — parameter/result names are not part of the type
```

---

## Function Declarations

Source: [spec#Function_declarations](https://go.dev/ref/spec#Function_declarations)

A function declaration binds a function name in the file block.

### EBNF

```
FunctionDecl = "func" FunctionName [ TypeParameters ] Signature [ FunctionBody ] .
FunctionName = identifier .
FunctionBody = Block .
```

### Forms

```go
func min(x int, y int) int {
    if x < y {
        return x
    }
    return y
}

// Parameters of same type share the type declaration
func min(x, y int) int { ... }

// Generic function (Go 1.18+)
func min[T constraints.Ordered](x, y T) T { ... }

// Function declaration without body — implemented externally
func flushICache(begin, end uintptr)  // implemented in assembly
```

### Named Result Parameters

Result parameters may be named. Each name declares a variable that is initialized to the zero value for its type. If the function executes a `return` with no arguments, the current values of those variables are returned:

```go
func split(sum int) (x, y int) {
    x = sum * 4 / 9
    y = sum - x
    return  // naked return
}
```

---

## Method Declarations

Source: [spec#Method_declarations](https://go.dev/ref/spec#Method_declarations)

A method is a function with a **receiver**. A method declaration binds a method name to a base type.

### EBNF

```
MethodDecl = "func" Receiver MethodName Signature [ FunctionBody ] .
Receiver   = Parameters .
```

### Receiver Constraints

- The receiver's base type must be a **defined type** `T` or pointer to defined type `*T`
- `T` must be defined in the same package as the method
- `T` must not be a pointer or interface type
- The receiver's type must be of the form `T` or `*T`

```go
type Point struct{ x, y float64 }

// Value receiver — receives a copy of the Point
func (p Point) Length() float64 {
    return math.Sqrt(p.x * p.x + p.y * p.y)
}

// Pointer receiver — receives a pointer, can mutate the Point
func (p *Point) Scale(factor float64) {
    p.x *= factor
    p.y *= factor
}
```

### Method Expressions and Method Values

**Method value** — calling `t.M` returns a function value with the receiver bound:

```go
p := Point{1, 2}
f := p.Scale    // f is func(float64), receiver bound to p
f(3)            // equivalent to p.Scale(3)
```

**Method expression** — `T.M` or `(*T).M` returns a function with receiver as first argument:

```go
f := Point.Length      // f is func(Point) float64
f(p)                   // equivalent to p.Length()
```

---

## Function Literals

Source: [spec#Function_literals](https://go.dev/ref/spec#Function_literals)

A function literal represents an anonymous function. Function literals **cannot** declare type parameters (no generic function literals).

### EBNF

```
FunctionLit = "func" Signature FunctionBody .
```

### Examples

```go
func(a, b int, z float64) bool { return a*b < int(z) }
```

A function literal can be assigned to a variable or invoked directly:

```go
f := func(x, y int) int { return x + y }

func(ch chan int) { ch <- ACK }(replyCh)  // invoked immediately
```

---

## Closures

Source: [spec#Function_literals](https://go.dev/ref/spec#Function_literals)

Function literals are **closures**: they may refer to variables defined in the surrounding function. Those variables are shared between the surrounding function and the function literal, and they survive as long as they are accessible.

```go
func adder() func(int) int {
    sum := 0
    return func(x int) int {
        sum += x
        return sum
    }
}
```

The returned function literal and `adder` both have access to `sum`. The variable `sum` persists on the heap as long as the returned function is reachable.

### Variable Capture Semantics

Closures capture **variables** (by reference), not values. This means all closures created in the same scope share the same variable:

```go
// All f functions close over the same i variable
var funcs [3]func()
for i := 0; i < 3; i++ {
    funcs[i] = func() { fmt.Print(i) }
}
funcs[0]()  // prints 3 (loop completed before any call)
funcs[1]()  // prints 3
funcs[2]()  // prints 3
```

**Fix:** Create a new variable in each iteration:

```go
for i := 0; i < 3; i++ {
    i := i  // new variable per iteration
    funcs[i] = func() { fmt.Print(i) }
}
```

> **Go 1.22 change:** Starting in Go 1.22, each iteration of a `for` loop creates new variables for the loop variables, fixing this gotcha. Code compiled with `go 1.22` or later in `go.mod` gets the new semantics automatically.

---

## Call Expressions

Source: [spec#Calls](https://go.dev/ref/spec#Calls)

### EBNF

```
Call      = Expression "(" [ ( ExpressionList | Type [ "," ExpressionList ] ) [ "..." ] ] ")" .
```

### Variadic Call with Spread

If `f` is variadic with final parameter `...T`, then within `f` that parameter has type `[]T`. If `f` is invoked with no actual arguments for `p`, the value passed to `p` is `nil`. Otherwise, the value passed is a new slice of type `[]T` with a new underlying array:

```go
s := []string{"James", "Jasmine"}
Greeting("goodbye:", s...)
// Greeting receives same slice as s, no new slice allocated
```

When spreading a slice (`s...`), no new slice is allocated — the slice header is passed directly.

### Passing Arguments to Variadic Parameters

| Call form | What function receives |
|-----------|----------------------|
| `f(a, b, c)` | New `[]T{a, b, c}` slice |
| `f(slice...)` | The slice directly (same backing array) |
| `f()` | `nil` slice (`[]T(nil)`) |

---

## Defer Statements

Source: [spec#Defer_statements](https://go.dev/ref/spec#Defer_statements)

A `defer` statement pushes a function call onto a list. The list of saved calls is executed in **LIFO order** (last in, first out) when the surrounding function returns.

### EBNF

```
DeferStmt = "defer" Expression .
```

The expression must be a function or method call. **The function value and parameters are evaluated immediately** when the defer statement executes. The actual call is deferred.

### Execution Mechanics

Each time a `defer` statement executes, the function value and parameters to the call are evaluated as usual and saved anew but the actual function is not invoked.

Deferred functions are executed in LIFO order immediately before the surrounding function returns, in the following cases:

1. The surrounding function executes a `return` statement
2. The function body executes to its end
3. The goroutine executing the function panics

```go
lock(l)
defer unlock(l)

// prints 3 2 1 0 before the surrounding function returns
for i := 0; i <= 3; i++ {
    defer fmt.Print(i)
}

// f returns 1
func f() (result int) {
    defer func() {
        result++  // deferred func modifies named return value
    }()
    return 0  // named return sets result=0, then defer runs, setting result=1
}
```

### Interaction with Named Returns

When a deferred function modifies named return parameters, those modifications are visible to the caller. This is because the deferred function executes **after** the return value has been set but **before** control returns to the caller.

```go
func double(x int) (result int) {
    defer func() { result *= 2 }()
    return x  // sets result = x, then defer doubles it
}
// double(4) returns 8
```

### panic and recover

`defer` is also how `recover()` works — `recover` may only be called directly from a deferred function:

```go
func safeDiv(a, b int) (result int, err error) {
    defer func() {
        if r := recover(); r != nil {
            err = fmt.Errorf("recovered: %v", r)
        }
    }()
    return a / b, nil  // panics if b == 0
}
```

---

## Return Statements

Source: [spec#Return_statements](https://go.dev/ref/spec#Return_statements)

A `return` statement terminates execution of the innermost containing function and optionally provides result values.

### Naked Returns

If the function's result parameters are named, a `return` statement without expressions returns the current values of those named variables:

```go
func ReadFull(r Reader, buf []byte) (n int, err error) {
    for len(buf) > 0 && err == nil {
        var nr int
        nr, err = r.Read(buf)
        n += nr
        buf = buf[nr:]
    }
    return  // returns current n and err
}
```

### Rules for Return Values

1. If the function has no result type, `return` takes no values
2. If the function has named result parameters, `return` with no values returns current parameter values
3. Otherwise, `return` must provide exactly the right number and types of values

---

## init Functions

Source: [spec#Package_initialization](https://go.dev/ref/spec#Package_initialization)

A package may contain one or more `init` functions (multiple per file, multiple files per package):

```go
func init() {
    // initialization logic
}
```

### Properties

- `init` functions have no arguments and no return values
- A package with no imports is initialized by assigning initial values to package-level variables, then calling all `init` functions in the order they appear in source (as presented to the compiler)
- `init` functions cannot be referred to from anywhere in a program — they cannot be called nor can a pointer to `init` be assigned to a function variable
- Multiple `init` functions may be defined per file; all run in order

### Init Order

1. Package-level variables are initialized in declaration order (respecting dependencies)
2. All `init()` functions of imported packages run first
3. Then the current package's `init()` functions run in source order

```go
var x = computeX()  // runs before init()

func init() {
    // x is already initialized here
}
```

---

## Blank Identifier

Source: [spec#Blank_identifier](https://go.dev/ref/spec#Blank_identifier)

The blank identifier `_` may be used like any other identifier in a declaration, but it does not introduce a binding and thus is not declared.

```go
// Discard a return value
_, err := os.Open("file.txt")

// Discard loop index
for _, v := range items {
    process(v)
}

// Discard entire function return
_ = f()
```

---

## Operator Summary: Functions

| Operation | Syntax | Notes |
|-----------|--------|-------|
| Declare a function | `func name(params) returnType { }` | |
| Declare with multiple returns | `func name(params) (T1, T2) { }` | |
| Named returns | `func name(params) (x T1, y T2) { }` | Enables naked return |
| Variadic parameter | `func name(args ...T)` | `args` is `[]T` inside function |
| Spread slice to variadic | `name(slice...)` | No new allocation |
| Function literal | `func(params) returnType { }` | Anonymous, assignable |
| Defer | `defer expr()` | Args evaluated now, call runs on return |
| Method value | `receiver.Method` | Bound function value |
| Method expression | `Type.Method` | Unbound; receiver is first arg |
| Blank identifier | `_` | Discard any value |
