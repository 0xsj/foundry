const word1 = "anagram";
const word2 = "nagaram";
const word3 = "cat";
const word4 = "tab";

const pair = [word1, word2];
const pair2 = [word3, word4];

/**
 * 1. we just compare two strings that we run maniulates through.
 * 2. we split the string, sort it, which would give us a specific sequence of characters
 * 3. we then join the characters array that is sorted to complete / form a new string.
 * 4. we do the same for the second argument / string
 * 5. if the two strings match, it means that it is an anagram
 */
function bruceForce(s: string, t: string, args?: string[]): boolean {
  return s.split("").sort().join("") === t.split("").sort().join("");
}

console.log("Brutce Force: " + bruceForce(word1, word2));
