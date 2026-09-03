# Values and Types

**Tier 0 · syntax** — Type the language without thinking about it.

## Read

Every value in Go has a type, fixed when the program is compiled. Not when it runs.
The compiler knows the exact size and shape of every variable before your program
exists, and it will refuse to build if anything is ambiguous.

Two consequences you feel immediately:

**Nothing starts empty.** Declare a variable without assigning it and Go fills it with
its type's *zero value* — `0`, `""`, `false`, `nil`. There is no `undefined`. There is
no uninitialized memory. `var port int` is `0` and you can use it right now.

**Nothing converts silently.** An `int` and a `float64` will not mix. You write the
conversion yourself, every time, even when it is obviously safe. This is tedious for
about a week and then it stops costing you anything, and in exchange you never lose
precision by accident.

```go
var port int          // 0
var host string       // ""
timeout := 2.5        // inferred float64
ratio := timeout / float64(port)   // float64(...) is required
```

Coming from TypeScript, the shift is: `let x` gives you `undefined` and a type that
exists only until compile time. `var x int` gives you `0` and a type that governs how
many bytes get allocated. TS types describe your intent; Go types describe memory.

## Mechanism

### Zero values are memory, not a convention

When Go allocates a variable it zeroes the bytes. That is the whole mechanism — a
`0`-filled region reinterpreted as whatever type sits there. `0` bytes read as an `int`
is `0`; as a `bool` is `false`; as a `string` is a header with a nil pointer and length
zero, which prints as `""`.

This is why zero values cost nothing and why they are consistent across every type.
It is also why a zero-value `map` is `nil` and panics on write — the header is there,
the backing table is not. You will meet that one in module 04.

### Static sizing

`int` is not "a number". On your machine it is exactly 64 bits, and the compiler bakes
that into every instruction touching it. `int8`, `int16`, `int32`, `int64` are separate
types, not hints — you cannot assign one to another without a conversion, because they
occupy different amounts of memory and the compiler will not guess what you meant.

This is why the conversion rule exists. `float64(maxConns)` is not ceremony; it is you
telling the compiler to emit an actual instruction that reinterprets 8 bytes of integer
as 8 bytes of IEEE-754 float.

### Type identity versus assignability

Two types are *identical* if they have the same name, or the same structure for unnamed
types. A value is *assignable* to a variable if the types are identical, or if one side
is an unnamed type with the same underlying structure.

```go
type Port int
var p Port = 8080   // ok: 8080 is an untyped constant
var n int = 9000
p = n               // compile error: Port and int are different types
p = Port(n)         // ok
```

`defaultPort` in the example is an **untyped constant**. It has no type until it is
used, at which point it takes the type of its context. That is why `port = defaultPort`
works with `port` being `int`, and would equally work if `port` were `int64` or
`float64`. Untyped constants are the one place Go relaxes, and they exist so that
writing `8080` does not force you to pick a width.

### What `fmt.Printf` actually does

It is not "print with formatting". `Printf` is variadic over `...any`, so every
argument you pass gets boxed into an interface value — a pair of pointers, one to the
type and one to the data. That boxing usually allocates.

It then walks the format string, and for each verb uses **reflection** to inspect the
boxed type at runtime and decide how to render it. `%v` means "ask the value what it
looks like". Finally it writes to `os.Stdout`, which is an unbuffered `*os.File`, so
each call is a `write` syscall.

Three things follow. It is genuinely slow, so it does not belong in a hot loop. `%d`
against a string is caught at runtime, not compile time — `go vet` is what saves you.
And the verbs are worth memorizing: `%v` default, `%+v` with field names, `%q` quoted,
`%T` the type itself, `%d` `%t` `%s` `%f` for the obvious ones.

## Type it

`example.go` is the file you will retype from memory. Twenty-six lines, and every one
carries something: zero values, an untyped constant, short declaration, an explicit
conversion, and four `Printf` verbs.

```
foundry drill go 00
foundry check go 00
```

## Related

- [[go-zero-values]]
- [[go-vs-ts-type-systems]]
