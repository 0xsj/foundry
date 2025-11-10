public struct PalindromeNumber {
    public static func isPalindrome(_ x: Int) -> Bool {
        if x < 0 {
            return false
        }

        if x % 10 == 0 && x != 0 {
            return false
        }

        let numStr = String(x)

        var left = 0
        var right = numStr.count - 1

        while left < right {
            let leftChar = numStr[numStr.index(numStr.startIndex, offsetBy: left)]
            let rightChar = numStr[numStr.index(numStr.startIndex, offsetBy: right)]

            if leftChar != rightChar {
                return false
            }

            left += 1
            right -= 1
        }

        return true
    }
}