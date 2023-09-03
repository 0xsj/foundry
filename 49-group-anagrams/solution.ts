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

const groupAnagrams_hashmap = (strs: string[]): string[][] => {
  let anagramGroup = new Map<string, string[]>();

  for (let s of strs) {
    let frequency = Array.from({ length: 26 }, () => 0);

    for (let i = 0; i < s.length; i++) {
      frequency[s.charCodeAt(i) - 97]++;
    }

    let key = frequency.toString();

    if (!anagramGroup.has(key)) {
      anagramGroup.set(key, []);
    }

    anagramGroup.get(key)?.push(s);
  }

  return Array.from(anagramGroup.values());
};

console.log(groupAnagrams(strs));
