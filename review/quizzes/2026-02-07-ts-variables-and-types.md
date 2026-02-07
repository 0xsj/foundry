# Quiz: TypeScript Variables and Types
**Date:** 2026-02-07
**Module:** fundamentals/variables-and-types
**Language:** TypeScript

---

### Q1 — Output Prediction
What type does TypeScript infer for each?
```typescript
const a = "hello";
let b = "hello";
const c = 8080;
let d = 8080;
```

<details>
<summary>Answer</summary>

- `a`: `"hello"` (string literal type — const narrows)
- `b`: `string` (let widens because value can change)
- `c`: `8080` (number literal type)
- `d`: `number` (widened)
</details>

---

### Q2 — Conceptual
TypeScript types are "erased." What does that mean, and what's the practical consequence?

<details>
<summary>Answer</summary>

Types exist only at compile time and are completely removed before runtime. The JS engine never sees them. Consequence: types provide zero runtime safety. If you lie to the compiler (`as any`), nothing stops invalid values at runtime. Your parsing/validation logic is your actual runtime type safety.
</details>

---

### Q3 — Debugging
What's wrong with this code?
```typescript
function getPort(config: unknown): number {
  return config.port;
}
```

<details>
<summary>Answer</summary>

`unknown` can't be accessed without narrowing first. TypeScript will error on `config.port`. You need a type guard or assertion:
```typescript
function getPort(config: unknown): number {
  if (typeof config === "object" && config !== null && "port" in config) {
    return (config as { port: number }).port;
  }
  throw new Error("invalid config");
}
```
`unknown` is the type-safe alternative to `any` — it forces you to check before using.
</details>

---

### Q4 — Comparison
How does error handling in the TS config loader differ from the Go version?

<details>
<summary>Answer</summary>

Go returns `(Config, error)` — the caller checks `if err != nil`. TS throws an `Error` — the caller needs `try/catch`. Both are conventions that neither language enforces at the type level. Go at least makes the error visible in the return signature. TS throws are invisible in the function signature — nothing tells the caller `loadConfig` can throw.
</details>

---

### Q5 — Decision
You're writing a function that accepts a status parameter. The only valid values are `"active"`, `"inactive"`, and `"pending"`. Would you type it as `string` or something else?

<details>
<summary>Answer</summary>

Use a union of literal types: `type Status = "active" | "inactive" | "pending"`. This is narrower than `string` — the compiler rejects any other value at compile time. In Go, you'd use `const` + `iota`, but nothing stops you from passing an arbitrary `int`. TS literal unions are stricter.
</details>

---

**Results:**
<!-- Fill in after taking the quiz: 0/5, 1/5, etc. -->
