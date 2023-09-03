let strs = ["eat", "tea", "tan", "ate", "nat", "bat"];
//[["bat"],["nat","tan"],["ate","eat","tea"]]

/** */
function groupAnagrams(strs: string[]): string[][] {
  const wordCount = new Map<string, string[]>();
  for (const str of strs) {
    for (const char of str) {
      const count = Array(26).fill(0);

      const index = char.charCodeAt(0) - "a".charCodeAt(0);
      count[index]++;

      const key = count.toString();
      if (!wordCount.has(key)) {
        wordCount.set(key, []);
      }
      wordCount.get(key)?.push(str);
    }
  }
  return Array.from(wordCount.values());
}

groupAnagrams(strs);
