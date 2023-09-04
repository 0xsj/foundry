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

/** */

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

/**
 * 1. we are creating a new map called anagram groups, where the key is the anagram string, and string[] is the anagrams
 * 2. go through the entire strs array
 * 3. sortedStr - key. we sort everything alphabetically, use this as the key
 * 4. if the key doesnt exist in the map, create a new entry with an empty array as the value.
 * 5. push original str into the array associated with the key
 */

function groupAnagrams_keySort(strs: string[]): string[][] {
  const anagramGroups = new Map<string, string[]>();

  for (const str of strs) {
    const sortedStr = str.split("").sort().join("");

    if (!anagramGroups.has(sortedStr)) {
      anagramGroups.set(sortedStr, []);
    }

    anagramGroups.get(sortedStr)?.push(str);
  }

  return Array.from(anagramGroups.values());
}
