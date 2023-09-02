/** */
const mergeAlternatively_join = (word1: string, word2: string) => {
  let result = "";

  for (let i = 0; i < Math.max(word1.length, word2.length); i++) {
    if (i < word1.length) {
      result += word1[i];
    }
    if (i < word2.length) {
      result += word2[i];
    }
  }
  return result;
};
