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

  const isClosing = (char: string) => {
    return parenMap.has(char);
  };

  for (const char of s) {
    if (isClosing(char)) {
      stack.push(char);
    } else {
      const topOfStack = stack.pop();

      if (!topOfStack || parenMap.get(topOfStack) !== char) {
        return false;
      }
    }
  }
  return stack.length === 0;
}

console.log(isValid(s));
