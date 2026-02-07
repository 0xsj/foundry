// ============================================================================
// Memory in TypeScript/JavaScript
// ============================================================================
// You don't have direct access to memory in JS/TS like you do in Go or Rust.
// But you CAN observe reference behavior and understand V8's model.
// Run with: npx ts-node memory.ts
// ============================================================================

// ============================================================================
// 1. PRIMITIVES vs OBJECTS: VALUE vs REFERENCE
// ============================================================================

// Primitives are passed/assigned by VALUE (copy)
let a = 42;
let b = a;
b = 100;
console.log("=== Primitive Copy ===");
console.log(`a: ${a}, b: ${b}`);
// Q: a is still 42. b got a copy. This is like Go's value semantics for ints.
//    Why can V8 do this efficiently? (hint: Smis)
//    (answer here)

// Objects are passed/assigned by REFERENCE (shared)
const x = { port: 8080 };
const y = x;
y.port = 9090;
console.log("\n=== Object Reference ===");
console.log(`x.port: ${x.port}, y.port: ${y.port}`);
// Q: x.port changed because x and y reference the SAME object.
//    How is this similar to Python's name/reference model?
//    (answer here)

// ============================================================================
// 2. TYPEOF — RUNTIME TYPE CHECKING
// ============================================================================

console.log("\n=== typeof ===");
console.log(`typeof 42:        ${typeof 42}`);
console.log(`typeof 3.14:      ${typeof 3.14}`);
console.log(`typeof "hello":   ${typeof "hello"}`);
console.log(`typeof true:      ${typeof true}`);
console.log(`typeof undefined: ${typeof undefined}`);
console.log(`typeof null:      ${typeof null}`);
console.log(`typeof {}:        ${typeof {}}`);
console.log(`typeof []:        ${typeof []}`);
// Q: typeof null is "object" — this is a 30-year-old bug in JavaScript.
//    typeof [] is also "object". How do you actually check for arrays?
//    (answer here)

// ============================================================================
// 3. EQUALITY TRAPS
// ============================================================================

console.log("\n=== Equality ===");
console.log(`42 == "42":    ${42 == "42"}`);     // true — type coercion
console.log(`42 === "42":   ${42 === "42"}`);    // false — strict equality
console.log(`null == undefined:  ${null == undefined}`);   // true
console.log(`null === undefined: ${null === undefined}`);  // false

// Q: Why should you ALWAYS use === in TypeScript?
//    How does TypeScript's compiler help prevent == bugs?
//    (answer here)

// ============================================================================
// 4. CONST DOESN'T MEAN IMMUTABLE
// ============================================================================

const config = { host: "localhost", port: 8080 };
config.port = 9090;  // works — const prevents reassignment, not mutation
// config = {};      // error — can't reassign the reference

console.log("\n=== const vs immutable ===");
console.log(`config: ${JSON.stringify(config)}`);

// To make truly immutable:
const frozen = Object.freeze({ host: "localhost", port: 8080 });
// frozen.port = 9090;  // runtime error in strict mode, silently fails otherwise

// Q: How does this compare to Java's `final` and Rust's `let`?
//    (answer here)

// ============================================================================
// 5. TYPE ERASURE IN ACTION
// ============================================================================

interface User {
  name: string;
  age: number;
}

const user: User = { name: "alice", age: 30 };

console.log("\n=== Type Erasure ===");
console.log(`typeof user: ${typeof user}`);
// Q: typeof says "object" — no trace of "User" at runtime.
//    How would you check at runtime whether an object matches the User shape?
//    (answer here)

// ============================================================================
// 6. NUMBER PRECISION
// ============================================================================

console.log("\n=== Number Precision ===");
console.log(`0.1 + 0.2 === 0.3? ${0.1 + 0.2 === 0.3}`);
console.log(`0.1 + 0.2 = ${0.1 + 0.2}`);
console.log(`Number.MAX_SAFE_INTEGER: ${Number.MAX_SAFE_INTEGER}`);
console.log(`2**53 + 1 === 2**53: ${2 ** 53 + 1 === 2 ** 53}`);
// Q: How would you safely handle money in TypeScript?
//    (answer here)

// ============================================================================
// 7. STRUCTUREDCLONE vs SPREAD (Shallow vs Deep Copy)
// ============================================================================

const original = { a: 1, nested: { b: 2 } };
const shallow = { ...original };
const deep = structuredClone(original);

shallow.nested.b = 999;

console.log("\n=== Copy Depth ===");
console.log(`original.nested.b: ${original.nested.b}`);
console.log(`deep.nested.b: ${deep.nested.b}`);
// Q: The spread operator (...) only copies one level deep.
//    shallow.nested and original.nested are the SAME object.
//    When would you need structuredClone vs spread?
//    (answer here)
