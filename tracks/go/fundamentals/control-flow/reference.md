# Go Control Flow Reference

Extracted from the [Go Language Specification](https://go.dev/ref/spec) and [Effective Go](https://go.dev/doc/effective_go).

## If Statements

### Syntax

```
IfStmt = "if" [ SimpleStmt ";" ] Expression Block [ "else" ( IfStmt | Block ) ] .
```

### Specification

"If" statements specify the conditional execution of two branches according to the value of a boolean expression. If the expression evaluates to true, the "if" branch is executed, otherwise, if present, the "else" branch is executed.

```go
if x > max {
    x = max
}
```

The expression may be preceded by a simple statement, which executes before the expression is evaluated.

```go
if x := f(); x < y {
    return x
} else if x > z {
    return z
} else {
    return y
}
```

### Scoping Rules

Variables declared in the init statement are scoped to the if statement (including any else clauses).

```go
if err := file.Close(); err != nil {
    return err
}
// err is not accessible here
```

## For Statements

### Syntax

```
ForStmt = "for" [ Condition | ForClause | RangeClause ] Block .
Condition = Expression .
ForClause = [ InitStmt ] ";" [ Condition ] ";" [ PostStmt ] .
RangeClause = [ ExpressionList "=" | IdentifierList ":=" ] "range" Expression .
```

### Three Forms

#### 1. For with Single Condition (While Loop)

A "for" statement with a condition executes as long as the boolean condition evaluates to true.

```go
for a < b {
    a *= 2
}
```

#### 2. For with Clause (Traditional Loop)

A "for" statement with a ForClause is controlled by its condition, but may also have init and post statements.

```go
for i := 0; i < 10; i++ {
    f(i)
}
```

Each iteration:
1. If the init statement exists, it is executed once (before the first iteration)
2. The condition is evaluated; if false, iteration terminates
3. The block is executed
4. The post statement is executed

#### 3. For without Condition (Infinite Loop)

A "for" statement without a condition executes repeatedly until a break or return statement terminates it.

```go
for {
    // infinite loop
}
```

### Range Clause

A "for" statement with a "range" clause iterates through all entries of an array, slice, string, map, or values received on a channel.

```go
for i, x := range a {
    // i is the index, x is a[i]
}
```

| Range expression | 1st value | 2nd value |
|------------------|-----------|-----------|
| array or slice `a [n]E` | index `i int` | `a[i] E` |
| string `s string` | index `i int` | rune `rune` |
| map `m map[K]V` | key `k K` | `m[k] V` |
| channel `c chan E` | element `e E` | none |

**Important:** The iteration values are assigned to the respective iteration variables as in an assignment statement. The iteration variables may be declared by the "range" clause using the `:=` form.

```go
var a [10]string
for i, s := range a {
    // type of i is int
    // type of s is string
}

// Blank identifier to ignore values
for _, s := range a {
    // only use s, ignore index
}

for i := range a {
    // only use index, ignore value
}
```

**Range creates copies:** The range expression is evaluated once before beginning the loop. The iteration variables receive copies of the iterated values.

```go
// This modifies the slice
for i := range numbers {
    numbers[i] = numbers[i] * 2
}

// This does NOT modify the slice (modifies copies)
for _, num := range numbers {
    num = num * 2
}
```

## Switch Statements

### Syntax

```
SwitchStmt = ExprSwitchStmt | TypeSwitchStmt .
ExprSwitchStmt = "switch" [ SimpleStmt ";" ] [ Expression ] "{" { ExprCaseClause } "}" .
ExprCaseClause = ExprSwitchCase ":" StatementList .
ExprSwitchCase = "case" ExpressionList | "default" .
```

### Expression Switch

In an expression switch, the cases contain expressions that are compared against the value of the switch expression.

```go
switch tag {
default: s3()
case 0, 1, 2, 3: s1()
case 4, 5, 6, 7: s2()
}
```

The switch expression is evaluated exactly once. Case expressions are evaluated left-to-right and top-to-bottom; the first one that equals the switch expression triggers execution of the statements of the associated case.

**No automatic fallthrough:** Execution of the statements in a case automatically terminates when the next case clause is reached, unless the statements end with a "fallthrough" statement.

```go
switch i {
case 0:
    fmt.Println("zero")
    fallthrough
case 1:
    fmt.Println("one or zero")  // executes if i == 0 or i == 1
}
```

### Switch with Initialization

Like if statements, switch statements may include a simple statement before the expression.

```go
switch err := file.Chmod(0664); err {
case nil:
    fmt.Println("success")
case ErrPermission:
    fmt.Println("permission denied")
default:
    fmt.Println("error:", err)
}
```

### Expression-less Switch

A missing switch expression is equivalent to the boolean value true.

```go
switch {
case x < 0:
    return -x
case x == 0:
    return 0
default:
    return x
}
```

This form is equivalent to an if-else chain but more readable when there are many conditions.

### Type Switch

A type switch compares types rather than values. It uses the keyword `type` in a type assertion.

```
TypeSwitchStmt  = "switch" [ SimpleStmt ";" ] TypeSwitchGuard "{" { TypeCaseClause } "}" .
TypeSwitchGuard = [ identifier ":=" ] PrimaryExpr "." "(" "type" ")" .
TypeCaseClause  = TypeSwitchCase ":" StatementList .
TypeSwitchCase  = "case" TypeList | "default" .
```

```go
switch v := x.(type) {
case nil:
    fmt.Println("x is nil")
case int:
    fmt.Printf("x is int %d\n", v)  // v has type int
case string:
    fmt.Printf("x is string %q\n", v)  // v has type string
default:
    fmt.Printf("unknown type %T\n", v)  // v has same type as x
}
```

## Break, Continue, Goto, Fallthrough

### Break

A "break" statement terminates execution of the innermost "for", "switch", or "select" statement.

```go
for i := 0; i < 10; i++ {
    if i == 5 {
        break  // exits the loop
    }
}
```

With a label, break terminates the labeled statement:

```go
OuterLoop:
    for i := 0; i < 10; i++ {
        for j := 0; j < 10; j++ {
            if condition {
                break OuterLoop  // exits both loops
            }
        }
    }
```

### Continue

A "continue" statement begins the next iteration of the innermost "for" loop.

```go
for i := 0; i < 10; i++ {
    if i%2 == 0 {
        continue  // skip even numbers
    }
    fmt.Println(i)
}
```

### Fallthrough

A "fallthrough" statement transfers control to the first statement of the next case clause in an expression switch.

```go
switch n {
case 1:
    fmt.Println("one")
    fallthrough
case 2:
    fmt.Println("also executed if n == 1")
}
```

**Restrictions:** "fallthrough" may be used only as the final statement in a case or default clause.

### Goto

A "goto" statement transfers control to the statement with the corresponding label within the same function.

```go
goto Error

Error:
    fmt.Println("error occurred")
    return
```

**Restrictions:** Executing a "goto" statement must not cause any variables to come into scope that were not already in scope at the point of the goto.

## Control Flow Best Practices (from Effective Go)

### Prefer Early Returns

```go
// Good
func (f *File) Read(buf []byte) (n int, err error) {
    if f == nil {
        return 0, errors.New("file is nil")
    }
    if len(buf) == 0 {
        return 0, nil
    }
    // main logic here
    return len(buf), nil
}
```

### Avoid Deep Nesting

```go
// Bad
if condition1 {
    if condition2 {
        if condition3 {
            // deeply nested
        }
    }
}

// Good
if !condition1 {
    return
}
if !condition2 {
    return
}
if !condition3 {
    return
}
// main logic at top level
```

### Use Switch for Clarity

When you have multiple conditions, prefer a switch statement over if-else chains:

```go
// Prefer this
switch {
case n < 0:
    return errors.New("negative")
case n == 0:
    return errors.New("zero")
case n > 100:
    return errors.New("too large")
default:
    return nil
}

// Over this
if n < 0 {
    return errors.New("negative")
} else if n == 0 {
    return errors.New("zero")
} else if n > 100 {
    return errors.New("too large")
} else {
    return nil
}
```

## Performance Notes

### Range vs Index Loop

For slices, both forms have similar performance:

```go
// Equivalent performance
for i := 0; i < len(slice); i++ {
    process(slice[i])
}

for _, v := range slice {
    process(v)
}
```

For maps, range is the only way to iterate. For channels, range automatically handles closure.

### Switch vs If-Else Chain

For small numbers of cases (< 5), switch and if-else chains have similar performance. For larger numbers, switch can be optimized by the compiler into jump tables or binary searches, making it faster.

## Official References

- [Go Language Specification - Statements](https://go.dev/ref/spec#Statements)
- [Effective Go - Control Structures](https://go.dev/doc/effective_go#control-structures)
- [Go Tour - Flow Control](https://go.dev/tour/flowcontrol/1)
