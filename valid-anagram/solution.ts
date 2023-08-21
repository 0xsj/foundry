const word1 = "anagram";
const word2 = "nagaram";
const word3 = "cat";
const word4 = "tab";

const pair = [word1, word2];
const pair2 = [word3, word4];

/**
 *
 */
function bruceForce(s: string, t: string, args?: string[]): boolean {
  return s.split("").sort().join("") === t.split("").sort().join("");
}

console.log("Brutce Force: " + bruceForce(word1, word2));
