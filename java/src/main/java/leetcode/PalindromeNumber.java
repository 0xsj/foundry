package leetcode;

public class PalindromeNumber {
    public static boolean isPalindrome(int x) {
        if (x < 0) {
            return false;
        }

        if (x % 10 == 0 && x != 0) {
            return false;
        }

        String numStr = String.valueOf(x);

        int left = 0;
        int right = numStr.length() - 1;

        while (left < right) {
            if (numStr.charAt(left) != numStr.charAt(right)) {
                return false;
            }

            left++;
            right--;
        }

        return true;
    }
}