---
title: "TypeScript: Implicit any"
languages: [typescript]
category: fundamentals
severity: high
related: [[variables-and-types]], [[type-safety]]
---

# TypeScript: Implicit any

## The Mistake

```typescript
function process(data) {  // implicitly typed as 'any'
    return data.foo();    // no type checking
}

process({ bar: 42 });     // compiles fine, runtime error
```

**What goes wrong:** TypeScript treats the parameter as `any`, disabling all type checking. The code compiles but fails at runtime.

---

## Why It Happens

**Mental Model Gap:** Coming from JavaScript, you might not realize TypeScript needs explicit types (or enough context to infer them).

When TypeScript can't infer a type, it defaults to `any` — which is essentially "opt out of type checking for this value."

**Common cases:**
- Function parameters without types
- Empty arrays: `const items = []` → `any[]`
- Destructuring without types: `const { foo } = obj`
- JSON parsing: `JSON.parse(str)` → `any`

```typescript
// All of these are implicitly 'any'
function process(data) { ... }          // parameter

const items = [];                       // array
items.push("hello");                    // still any[]
items.push(42);                         // still any[]

const result = JSON.parse(response);    // any
```

---

## The Fix

### Option 1: Enable `noImplicitAny` (Recommended)

In `tsconfig.json`:

```json
{
  "compilerOptions": {
    "strict": true,           // includes noImplicitAny
    // or individually:
    "noImplicitAny": true
  }
}
```

Now TypeScript will error on implicit `any`:

```typescript
function process(data) {  // ERROR: Parameter 'data' implicitly has an 'any' type
    return data.foo();
}
```

### Option 2: Add Explicit Types

```typescript
interface Data {
    foo: () => void;
}

function process(data: Data) {
    return data.foo();  // type-checked
}
```

### Option 3: Use Type Inference

Let TypeScript infer when possible:

```typescript
const items: string[] = [];      // explicitly typed array
items.push("hello");             // OK
items.push(42);                  // ERROR

// Or infer from initialization
const items = ["hello"];         // inferred as string[]
```

---

## How to Avoid

1. **Enable `strict` mode in `tsconfig.json`** — this includes `noImplicitAny` and many other safety checks
2. **Type all function parameters** — even if you think they're obvious
3. **Type empty arrays explicitly**: `const items: string[] = []`
4. **Type JSON parsing results**: Define an interface and assert
   ```typescript
   interface User {
       name: string;
       age: number;
   }
   const user = JSON.parse(response) as User;
   ```
5. **Use ESLint rules**: `@typescript-eslint/no-explicit-any`

---

## Comparison to Other Languages

| Language | Behavior |
|---|---|
| TypeScript (no `noImplicitAny`) | Silently defaults to `any` |
| TypeScript (`noImplicitAny` on) | Compile error |
| Go | Compile error (no type inference for function parameters) |
| Rust | Compile error (must annotate or infer from usage) |
| Python (with type hints) | `mypy` catches missing annotations |

**TypeScript is unique in allowing the "escape hatch" by default.** This was a design decision to make TypeScript easy to adopt incrementally from JavaScript, but it's a footgun if you're not aware.

---

## Related Concepts

- [[variables-and-types#type-annotations]]
- [[type-safety-config]]
- `any` vs `unknown` (prefer `unknown` for "I don't know the type yet")

---

## Frequency

⭐⭐⭐⭐ (Common in TypeScript projects without strict mode)

---

## Real-World Impact

**Production incidents:** This bug class has caused runtime errors in production when:
- API response shapes changed but types weren't updated
- Null/undefined values weren't handled (because `any` bypasses checks)
- Refactors broke calling code without compile errors

**Example:**
```typescript
function sendEmail(config) {  // implicitly any
    smtp.send({
        to: config.to,        // no autocomplete
        from: config.sender,  // typo: should be 'from'
    });
}

// Compiles fine, fails at runtime
sendEmail({ to: 'user@example.com', from: 'me@example.com' });
```

**Lesson:** Always enable `strict` mode in production TypeScript projects. The short-term convenience of implicit `any` is not worth the long-term maintenance cost.
