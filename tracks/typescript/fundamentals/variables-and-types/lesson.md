# Variables and Types — TypeScript

## How Variables Work Under the Hood

### The Two-Layer System

TypeScript is unusual: it has a **compile-time type system** that is **completely erased** before execution. At runtime, you're running JavaScript on the V8 engine (or equivalent). This means:

- Types exist only in your editor and at compile time
- There is zero runtime cost to TypeScript types
- There is also zero runtime safety from TypeScript types
- If you lie to the compiler (`as any`, `as unknown`), nothing stops you at runtime

```typescript
const port: number = 8080;
// At compile time: TypeScript knows this is a number
// At runtime: V8 just sees `const port = 8080` — the `: number` is gone
```

### V8's Memory Model

JavaScript (and therefore TypeScript at runtime) uses V8's memory layout:

- **Small integers (Smis)**: stored inline as tagged values — no heap allocation
- **Heap numbers**: floats and large ints are boxed objects on the heap
- **Strings**: complex internal representation — can be flat, concatenated (cons strings), or sliced
- **Objects**: hidden classes (shapes) + property storage

```typescript
const x = 42;        // Smi — stored inline, very fast
const y = 3.14;      // HeapNumber — boxed on the heap
const z = 2 ** 53;   // HeapNumber — too large for Smi
```

You don't control any of this directly. V8 makes these decisions. But understanding it helps explain why `number` is "just number" — V8 handles the int/float distinction internally.

### Your notes
<!-- -->


---

## Type System

### Static, Structural, Erased

TypeScript's type system is:
- **Static**: checked at compile time
- **Structural**: types are compatible if their shapes match — names don't matter
- **Erased**: types vanish at runtime. `typeof` at runtime gives JavaScript types, not TypeScript types.

```typescript
interface Point {
  x: number;
  y: number;
}

const p = { x: 10, y: 20, z: 30 };
const q: Point = p;  // works — p has at least x and y (structural)
```

This is fundamentally different from Go's structural interfaces. Go checks methods, TypeScript checks property shapes.

### Primitive Types

| Type | Runtime (typeof) | Notes |
|---|---|---|
| `number` | "number" | IEEE 754 double (64-bit). No int type. |
| `string` | "string" | UTF-16 encoded. Immutable. |
| `boolean` | "boolean" | |
| `undefined` | "undefined" | Variable declared but not assigned |
| `null` | "object" | Yes, `typeof null === "object"` is a JS bug from 1995 |
| `bigint` | "bigint" | Arbitrary precision integers |
| `symbol` | "symbol" | Unique identifiers |

### The `number` Problem

TypeScript has one number type: `number`. It's a 64-bit IEEE 754 double.

```typescript
const a = 0.1 + 0.2;    // 0.30000000000000004 — not 0.3
const b = 2 ** 53 + 1;  // 9007199254740992 — same as 2**53 (precision lost)
```

- Safe integer range: `Number.MIN_SAFE_INTEGER` to `Number.MAX_SAFE_INTEGER` (-(2^53-1) to 2^53-1)
- For larger integers: use `bigint`
- For currency: use integers (cents) or a decimal library. Never float.

Compare to Go where `int`, `float64`, `int32` etc. are separate types with explicit conversion.

### Your notes
<!-- -->


---

## Undefined, Null, and Void

This is TypeScript's biggest source of bugs. There are THREE "nothing" values:

| Value | Meaning | typeof |
|---|---|---|
| `undefined` | Declared but not assigned / missing property | "undefined" |
| `null` | Explicitly set to "no value" | "object" |
| `void` | Function returns nothing (type-level only) | N/A |

```typescript
let x: string;          // x is undefined at runtime
let y: string | null = null;  // explicitly no value

// strictNullChecks makes the compiler track these:
function len(s: string | null): number {
  // return s.length;       // error: s might be null
  return s?.length ?? 0;    // safe: optional chaining + nullish coalescing
}
```

With `strictNullChecks` on (and it should always be on), `null` and `undefined` are not assignable to other types. This is how TypeScript compensates for JavaScript's loose nullability.

### Your notes
<!-- -->


---

## Literal Types and Const

TypeScript infers **literal types** for `const`:

```typescript
let x = "hello";        // type: string (widened)
const y = "hello";      // type: "hello" (literal type — narrower than string)

const port = 8080;      // type: 8080 (not number)

let status = "active";  // type: string
// To force a literal type on let:
let mode = "debug" as const;  // type: "debug"
```

This matters for discriminated unions and function signatures:

```typescript
type Status = "active" | "inactive" | "pending";
const s: Status = "active";  // only these three strings are valid
```

### Your notes
<!-- -->


---

## Type Assertions and Type Guards

Since types are erased, sometimes you need to tell TypeScript what you know — or check at runtime:

```typescript
// Assertion — "trust me, I know the type"
const input: unknown = getConfig();
const port = (input as { port: number }).port;  // no runtime check

// Type guard — actual runtime check
function isString(x: unknown): x is string {
  return typeof x === "string";
}
```

Assertions are lies you tell the compiler. Type guards are truth you verify at runtime. Prefer guards.

### Your notes
<!-- -->


---

## Composite Types (Preview)

| Type | Syntax | Notes |
|---|---|---|
| Array | `number[]` or `Array<number>` | Resizable, single type |
| Tuple | `[string, number]` | Fixed length, mixed types |
| Object | `{ key: type }` | Structural shape |
| Record | `Record<string, number>` | Dictionary/map |
| Enum | `enum Direction { Up, Down }` | Compiled to object. Often avoided in favor of unions |

### Your notes
<!-- -->
