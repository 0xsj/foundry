package leetcode.ArraysAndHashing

object PalindromeNumber {
    def isPalindrome(x: Int): Boolean = {
        if (x < 0) {
            false
        } else if (x % 10 == 0 && x != 0) {
            false
        } else {
            val numStr = x.toString

            def checkPalindrome(left: Int, right: Int): Boolean = {
                if (left >= right) {
                    true
                } else if (numStr(left) != numStr(right)) {
                    false
                } else {
                    checkPalindrome(left + 1, right - 1)
                }
            }
            checkPalindrome(0, numStr.length - 1)
        }
    }
}