def longest_palindromic_substring(s):
    if not s:
        return ""

    n = len(s)
    dp = [[False] * n for _ in range(n)]
    longest = ""

    for i in range(n):
        dp[i][i] = True
        longest = s[i]

    max_length = 1

    for start in range(n - 1, -1, -1):
        for end in range(start + 1, n):
            if s[start] == s[end]:
                if end - start == 1 or dp[start + 1][end - 1]:
                    dp[start][end] = True
                    if end - start + 1 > max_length:
                        max_length = end - start + 1
                        longest = s[start:end + 1]

    return longest

# Example usage:
s = "babad"
result = longest_palindromic_substring(s)
print(result)  # Output: "bab" or "aba"
