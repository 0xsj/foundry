function longestPalindrome(s: string): string {
  if (!s) {
    return "";
  }

  const n = s.length;
  const dp: boolean[][] = new Array(n).fill(0).map(() => new Array(n).fill(false));
  let longest = s[0];

  for (let i = 0; i < n; i++) {
    dp[i][i] = true;
  }

  let maxLen = 1;

  for (let start = n - 1; start >= 0; start--) {
    for (let end = start + 1; end < n; end++) {
      if (s[start] === s[end]) {
        if (end - start === 1 || dp[start + 1][end - 1]) {
          dp[start][end] = true;
          if (end - start + 1 > maxLen) {
            maxLen = end - start + 1;
            longest = s.substring(start, end + 1);
          }
        }
      }
    }
  }

  return longest;
}
