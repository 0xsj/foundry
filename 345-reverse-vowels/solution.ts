function reverseVowels(s: string): string {
  const vowels = "aeiouAEIOU";
  const vowelArray = s.split("");
  let start = 0;
  let end = s.length - 1;

  while (start < end) {
    while (start < end && vowels.indexOf(vowelArray[start]) === -1) {
      start++;
    }

    while (start < end && vowels.indexOf(vowelArray[end]) === -1) {
      end--;
    }

    if (start < end) {
      const temp = vowelArray[start];
      vowelArray[start] = vowelArray[end];
      vowelArray[end] = temp;
      start++;
      end--;
    }
  }

  return vowelArray.join("");
}

console.log(reverseVowels("leetcode"));
