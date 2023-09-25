/**
 * Try 1
 */
function closeStrings(word1: string, word2: string): boolean {
  const frequencyMap1 = new Map();
  const frequencyMap2 = new Map();

  for (const char of word1) {
    frequencyMap1.set(char, (frequencyMap1.get(char) || 0) + 1);
  }
  for (const char of word2) {
    frequencyMap2.set(char, (frequencyMap2.get(char) || 0) + 1);
  }

  const set1 = new Set(word1);
  const set2 = new Set(word2);

  if (set1.size !== set2.size) {
    return false;
  }

  for (const char of set2) {
    if (!set1.has(char)) {
      return false;
    }
  }

  const count1 = Array.from(frequencyMap1.values()).sort((a, b) => a - b);
  const count2 = Array.from(frequencyMap2.values()).sort((a, b) => a - b);

  if (count1.length !== count2.length) {
    return false;
  }

  for (let i = 0; i < count1.length; i++) {
    if (count1[i] !== count2[i]) {
      return false;
    }
  }
  return true;
}

console.log(closeStrings("abc", "bca"));
console.log(closeStrings("a", "aa"));
