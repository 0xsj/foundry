function findClosestSubsetIndices(case1: string[], keywords: Set<string>): number[] | null {
  const keywordArray = Array.from(keywords);
  const keywordCount = keywordArray.length;

  let minDistance = Infinity;
  let minIndices: number[] | null = null;

  const keywordLastIndices: Map<string, number> = new Map();

  for (let i = 0; i < case1.length; i++) {
    const currentWord = case1[i];

    if (keywords.has(currentWord)) {
      keywordLastIndices.set(currentWord, i);

      if (keywordLastIndices.size === keywordCount) {
        const minIndex = Math.min(...Array.from(keywordLastIndices.values()));

        const currentDistance = i - minIndex;

        if (currentDistance < minDistance) {
          minDistance = currentDistance;
          minIndices = Array.from({ length: currentDistance + 1 }, (_, index) => minIndex + index);
        }
      }
    }
  }

  return minIndices;
}

// Example usage:
const case1 = ["hello", "world", "world", "at", "this", "is", "of", "not", "think", "okay", "got"];
const keywords = new Set(["world", "this", "not"]);

const result = findClosestSubsetIndices(case1, keywords);
console.log(result); // Output: [2, 3, 4, 5, 6, 7]
