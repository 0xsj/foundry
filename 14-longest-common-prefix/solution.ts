let strs = ["flower", "flow", "flight"];

function longestCommonPrefix(strs: string[]): string {
  // we just care about the first x amount of letters.

  // only lower case

  // what if we create a set?

  const results = new Set(strs);

  results.forEach((item) => {
    console.log(item);
  });

  return "";
}

console.log(longestCommonPrefix(strs));
