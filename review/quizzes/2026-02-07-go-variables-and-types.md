# Quiz: Go Variables and Types
**Date:** 2026-02-07
**Module:** fundamentals/variables-and-types
**Language:** Go

---

### Q1 — Output Prediction
What does this print?
```go
var x int
var y float64
fmt.Println(x, y)
```

<details>
<summary>Answer</summary>

`0 0` — Go's zero values. `int` zeros to `0`, `float64` zeros to `0` (printed without decimal by `Println`).
</details>

---

### Q2 — Conceptual
A Go string variable is 16 bytes. Where are the actual characters stored, and why is `len("café")` equal to 5?

<details>
<summary>Answer</summary>

The 16 bytes are just a header: 8-byte pointer + 8-byte length. The actual character bytes live elsewhere (read-only memory for literals, heap for constructed strings). `len()` returns bytes, not characters — `é` is 2 bytes in UTF-8, so `"café"` is 5 bytes. Use `utf8.RuneCountInString()` for character count.
</details>

---

### Q3 — Debugging
What's wrong with this code?
```go
var m map[string]int
m["port"] = 8080
```

<details>
<summary>Answer</summary>

Panic at runtime. A nil map can be read from (returns zero value) but writing to it panics. Must initialize with `make(map[string]int)` or a literal `map[string]int{}` first. Compare with nil slices, which are safe to `append` to.
</details>

---

### Q4 — Decision
You have a `User` struct with 15 fields. A function needs to update the user's email. Should the function take `User` or `*User`? Why?

<details>
<summary>Answer</summary>

`*User`. Two reasons: (1) you need to mutate the original — passing by value copies the struct, so changes wouldn't be visible to the caller. (2) With 15 fields, copying the entire struct on every call is wasteful.
</details>

---

### Q5 — Comparison
In JavaScript, `const obj = {x: 1}; function f(o) { o.x = 2; } f(obj);` mutates the original. What would the equivalent Go code do if `f` takes a `Config` by value?

<details>
<summary>Answer</summary>

Nothing — the caller's `Config` is unchanged. Go structs are value types, so `f` receives a full copy. Mutating the copy doesn't affect the original. To get the JS behavior, `f` would need to take `*Config`.
</details>

---

**Results:**
<!-- Fill in after taking the quiz: 0/5, 1/5, etc. -->
