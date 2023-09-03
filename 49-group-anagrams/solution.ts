let strs = ["eat", "tea", "tan", "ate", "nat", "bat"];
//[["bat"],["nat","tan"],["ate","eat","tea"]]

/** */
function groupAnagrams(strs: string[]): string[][] {
  const anagramGroup = new Map<string, string[]>();

  for (const str of strs) {
    const charFrequency = Array(26).fill(0);

    for (const char of str) {
      const index = char.charCodeAt(0) - "a".charCodeAt(0);
      charFrequency[index]++;
    }

    const key = charFrequency.join(",");

    if (!anagramGroup.has(key)) {
      anagramGroup.set(key, []);
    }

    anagramGroup.get(key)?.push(str);
  }

  return Array.from(anagramGroup.values());
}

console.log(groupAnagrams(strs));
