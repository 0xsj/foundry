# TypeScript Reference — Variables and Types

> Extracted from the [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/)
> and [Variable Declarations](https://www.typescriptlang.org/docs/handbook/variable-declarations.html)
> for the `variables-and-types` module. Covers: var/let/const, primitive types, object
> types, union types, type aliases, interfaces, literal types, null/undefined, and
> destructuring.

---

## Variable Declarations

Source: [Variable Declarations](https://www.typescriptlang.org/docs/handbook/variable-declarations.html)

### `var` Declarations

`var` uses **function scoping** — accessible anywhere within the containing function,
regardless of block nesting.

```ts
function f(shouldInitialize: boolean) {
  if (shouldInitialize) {
    var x = 10;
  }
  return x; // accessible even outside the if block
}
f(true);  // 10
f(false); // undefined
```

Allows re-declaration in the same scope:

```ts
function f(x) {
  var x;
  var x;    // all refer to same variable
  if (true) {
    var x;  // still same variable
  }
}
```

Classic loop capture problem:

```ts
for (var i = 0; i < 10; i++) {
  setTimeout(function () {
    console.log(i);
  }, 100 * i);
}
// prints 10 ten times (not 0-9)
```

### `let` Declarations

`let` uses **block scoping** (lexical scoping) — constrained to the nearest
containing block.

```ts
function f(input: boolean) {
  let a = 100;
  if (input) {
    let b = a + 1;
    return b;
  }
  return b; // Error: 'b' doesn't exist here
}
```

**Temporal Dead Zone**: Cannot access before declaration:

```ts
a++;    // illegal to use 'a' before it's declared
let a;
```

**No re-declaration** in the same scope:

```ts
let x = 10;
let x = 20; // error: can't re-declare 'x' in the same scope
```

**Shadowing in nested blocks is allowed:**

```ts
function f(condition, x) {
  if (condition) {
    let x = 100;  // shadows outer x
    return x;
  }
  return x;
}
f(false, 0); // 0
f(true, 0);  // 100
```

**Loop scoping** — each iteration creates a new scope:

```ts
for (let i = 0; i < 10; i++) {
  setTimeout(function () {
    console.log(i);
  }, 100 * i);
}
// prints 0-9 (correct)
```

### `const` Declarations

Same scoping rules as `let`, but **cannot be re-assigned** after initialization.

```ts
const numLivesForCat = 9;
```

**Does not mean immutable** — object contents can still be mutated:

```ts
const kitty = {
  name: "Aurora",
  numLives: numLivesForCat,
};

kitty = { ... };      // Error: cannot reassign
kitty.name = "Rory";  // OK: mutation is fine
kitty.numLives--;     // OK
```

### Best Practice

Apply the **principle of least privilege**: use `const` by default, `let` only when
you need to reassign.

---

## Primitive Types

Source: [Everyday Types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html)

### `string`

Text values. Always use lowercase `string`, not `String`.

```ts
const greeting: string = "Hello, world";
const template: string = `Hello, ${name}`;
```

### `number`

All numeric values — no separate integer/float distinction. JavaScript uses IEEE 754
double-precision (64-bit) for all numbers.

```ts
const integer: number = 42;
const float: number = 3.14;
const hex: number = 0xff;
const binary: number = 0b1010;
const octal: number = 0o744;
```

### `bigint`

Arbitrary-precision integers. Cannot be mixed with `number` without explicit conversion.

```ts
const big: bigint = 100n;
```

### `boolean`

Two values: `true` and `false`.

```ts
const isDone: boolean = false;
```

### `symbol`

Globally unique identifiers created via `Symbol()`.

```ts
const sym: symbol = Symbol("description");
```

### `null` and `undefined`

Two separate types representing absence of value. Behavior depends on
`strictNullChecks`:

- **Off**: `null` and `undefined` can be assigned to any type (bug-prone).
- **On**: must explicitly include in union types and check before use.

```ts
function doSomething(x: string | null) {
  if (x === null) {
    // handle null
  } else {
    console.log(x.toUpperCase());
  }
}
```

**Non-null assertion operator** (`!`) — removes `null`/`undefined` without checking
(type-level only, no runtime effect):

```ts
function liveDangerously(x?: number | null) {
  console.log(x!.toFixed());  // asserts x is not null/undefined
}
```

---

## The `any` Type

Bypasses all type checking. Use sparingly.

```ts
let obj: any = { x: 0 };
obj.foo();           // OK — no checking
obj = "hello";       // OK
const n: number = obj; // OK
```

The `noImplicitAny` compiler flag errors on implicit `any` inference.

---

## Type Annotations on Variables

Add type information after variable names with `:`:

```ts
let myName: string = "Alice";
```

TypeScript often infers types automatically, so explicit annotations are frequently
unnecessary:

```ts
let myName = "Alice"; // inferred as string
```

---

## Arrays

Two equivalent syntaxes:

```ts
let numbers: number[] = [1, 2, 3];
let strings: Array<string> = ["a", "b"];
```

Note: `[number]` is a **tuple**, not an array.

---

## Object Types

Define objects by listing properties and their types:

```ts
function printCoord(pt: { x: number; y: number }) {
  console.log("x value is " + pt.x);
  console.log("y value is " + pt.y);
}
```

### Optional Properties

Mark with `?` — the property may be `undefined`:

```ts
function printName(obj: { first: string; last?: string }) {
  if (obj.last !== undefined) {
    console.log(obj.last.toUpperCase());
  }
}
```

---

## Union Types

Combine types with `|`:

```ts
function printId(id: number | string) {
  console.log("Your ID is: " + id);
}

printId(101);     // OK
printId("202");   // OK
printId([1, 2]);  // Error
```

### Narrowing

TypeScript only allows operations valid for **all** union members. Use type guards
to narrow:

```ts
function printId(id: number | string) {
  if (typeof id === "string") {
    console.log(id.toUpperCase());  // id is string here
  } else {
    console.log(id);                // id is number here
  }
}
```

Shared methods work without narrowing:

```ts
function getFirstThree(x: number[] | string) {
  return x.slice(0, 3);  // both types have slice()
}
```

---

## Type Aliases

Create reusable type names with `type`:

```ts
type Point = {
  x: number;
  y: number;
};

type ID = number | string;
```

**Key characteristic**: Aliases are purely structural — different aliases for the same
shape are interchangeable:

```ts
type Age = number;
type Weight = number;

const myAge: Age = 73;
const myWeight: Weight = myAge;  // OK — both are number
```

---

## Interfaces

Alternative to type aliases for naming object types:

```ts
interface Point {
  x: number;
  y: number;
}
```

TypeScript uses **structural typing** — only the structure matters, not the name.

### Type Aliases vs Interfaces

| Feature | `type` | `interface` |
|---|---|---|
| Extension | Intersection (`&`) | `extends` keyword |
| Declaration merging | No | Yes |
| Scope | Any type | Object types only |
| Error messages | May show expanded form | Shows name as-is |

---

## Literal Types

Specific string or number values as types:

```ts
let x: "hello" = "hello";
x = "howdy"; // Error
```

Useful in unions:

```ts
function printText(s: string, alignment: "left" | "right" | "center") {
  // ...
}
printText("Hello", "left");    // OK
printText("Hello", "centre");  // Error
```

Numeric and boolean literals:

```ts
function compare(a: string, b: string): -1 | 0 | 1 {
  return a === b ? 0 : a > b ? 1 : -1;
}
```

### Literal Inference

Object properties are inferred as general types, not literals:

```ts
const req = { url: "https://example.com", method: "GET" };
// method is inferred as string, not "GET"
```

Fix with `as const` or type assertions:

```ts
const req = { url: "https://example.com", method: "GET" as "GET" };
// or
const req = { url: "https://example.com", method: "GET" } as const;
```

---

## Type Assertions

Tell TypeScript about a more specific type:

```ts
const myCanvas = document.getElementById("main_canvas") as HTMLCanvasElement;

// Alternative angle-bracket syntax (not in .tsx files):
const myCanvas = <HTMLCanvasElement>document.getElementById("main_canvas");
```

TypeScript only allows assertions to more/less specific versions. For impossible
assertions, go through `any`:

```ts
const x = "hello" as number;           // Error
const x = "hello" as any as number;    // Allowed (but dangerous)
```

Assertions are **compile-time only** — no runtime checking.

---

## typeof Type Operator

Source: [typeof](https://www.typescriptlang.org/docs/handbook/2/typeof-types.html)

TypeScript's `typeof` works in **type contexts** to reference a variable's type:

```ts
function f() {
  return { x: 10, y: 3 };
}
type P = ReturnType<typeof f>;
// P = { x: number; y: number }
```

Restricted to identifiers and their properties only.

---

## Destructuring

Source: [Variable Declarations](https://www.typescriptlang.org/docs/handbook/variable-declarations.html)

### Array Destructuring

```ts
let [first, second] = [1, 2];
[first, second] = [second, first];  // swap

let [head, ...rest] = [1, 2, 3, 4];
// head = 1, rest = [2, 3, 4]

let [, second, , fourth] = [1, 2, 3, 4];
// second = 2, fourth = 4
```

### Tuple Destructuring

```ts
let tuple: [number, string, boolean] = [7, "hello", true];
let [a, b, c] = tuple;  // a: number, b: string, c: boolean

let [a, ...bc] = tuple;  // bc: [string, boolean]
```

### Object Destructuring

```ts
let { a, b } = { a: "foo", b: 12, c: "bar" };

// Property renaming:
let { a: newName1, b: newName2 } = o;

// With type annotation:
let { a, b }: { a: string; b: number } = o;

// Default values:
let { a, b = 1001 } = someObject;

// Rest:
let { a, ...passthrough } = o;
```

### Spread Operator

```ts
// Arrays:
let bothPlus = [0, ...first, ...second, 5];

// Objects (order matters — later overwrites earlier):
let search = { ...defaults, food: "rich" };
```

Object spread only includes own, enumerable properties — methods are lost:

```ts
class C {
  p = 12;
  m() {}
}
let clone = { ...new C() };
clone.p;   // OK
clone.m(); // Error — method not copied
```

---

## `using` Declarations (TC39 Stage 3)

Source: [Variable Declarations](https://www.typescriptlang.org/docs/handbook/variable-declarations.html)

Binds a variable's lifetime to its scope. On exit, `[Symbol.dispose]()` is called
automatically (RAII pattern):

```ts
function f() {
  using x = new C();
  doSomethingWith(x);
} // x[Symbol.dispose]() called automatically
```

Async variant:

```ts
async function f() {
  await using x = new C();
} // await x[Symbol.asyncDispose]() called
```

Implement `Disposable` or `AsyncDisposable` interface to define cleanup behavior.
