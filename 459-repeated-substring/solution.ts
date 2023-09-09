let teststring = "abcabcabcabc";

let teststring2 = "aba";

function repeatedSubstringPattern(s: string): boolean {
  const n = s.length;

  for (let len = 1; len <= n / 2; len++) {
    if (n % len === 0) {
      const sub = s.substring(0, len);
      let repeated = true;

      for (let i = len; i < n; i += len) {
        if (s.substring(i, i + len) !== sub) {
          repeated = false;
          break;
        }
      }

      if (repeated) {
        return true;
      }
    }
  }

  return false;
}

console.log(repeatedSubstringPattern(teststring2));
