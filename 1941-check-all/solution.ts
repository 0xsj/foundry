function areOccurrencesEqual(s: string): boolean {
  const map = new Map<string, number>();

  for (const char of s) {
    if (map.has(char)) {
      map.set(char, map.get(char)! + 1);
    } else {
      map.set(char, 1);
    }
  }

  const occurence = Array.from(map.values());
  return occurence.every((count) => count === occurence[0]);
}

// try2
// maybe we can sort the array first. first examine how many times the first char occures
// lets say that the first char's instance is 3
// the 3 then becomes our "window". examine the array as such.

console.log(areOccurrencesEqual("abcabc"));
