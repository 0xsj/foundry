/**
 * one liner
 * 1. the first and most immediate goal of this is to get rid of trailing and leading white spaces
 */
function reverseWords_try1(s: string): string {
  return s.replace(/\s+/g, " ").trim().split(" ").reverse().join(" ");
}

/**
 * 1. similarly, we are cleaning the string
 * 2. we start at the last index (trimmed.length -1), and for each word that we encounter, we are pushing it into the results array.
 * 3. its just a long way of reversing
 */
function reverseWords_try2(s: string): string {
  const trimmed = s.replace(/\s+/g, " ").trim().split(" ");
  const results: string[] = [];
  const left = trimmed.length - 1;

  for (let i = left; i >= 0; i--) {
    results.push(trimmed[i]);
  }

  return results.join(" ");
}
