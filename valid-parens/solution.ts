const s = "()";
const t = "(]";
const y = "()[]()";

// valid parens, must have open and closing.
// open and closing must be specific

// first approach
/**
 * 1. create a look up table or a key value pair store, where we associate () {} []
 * 2.
 */

function isValid(s: string): boolean {
  const stack: string[] = [];
  const parenMap: Map<string, string> = new Map([
    ["(", ")"],
    ["{", "}"],
    ["[", "]"],
  ]);

  for (const char of s) {
    if (parenMap.has(char)) {
      stack.push(char);
      console.log(stack);
    }
  }
  return false;
}

console.log(isValid(s));
