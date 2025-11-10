// scala/src/test/scala/leetcode/ArraysAndHashing/PalindromeNumberTest.scala

package leetcode.ArraysAndHashing

import org.scalatest.funsuite.AnyFunSuite

class PalindromeNumberTest extends AnyFunSuite {

  test("single digit numbers are palindromes") {
    assert(PalindromeNumber.isPalindrome(0))
    assert(PalindromeNumber.isPalindrome(7))
    assert(PalindromeNumber.isPalindrome(9))
  }

  test("negative numbers are not palindromes") {
    assert(!PalindromeNumber.isPalindrome(-121))
    assert(!PalindromeNumber.isPalindrome(-1))
    assert(!PalindromeNumber.isPalindrome(-101))
  }

  test("numbers ending in zero (except 0) are not palindromes") {
    assert(!PalindromeNumber.isPalindrome(10))
    assert(!PalindromeNumber.isPalindrome(100))
    assert(!PalindromeNumber.isPalindrome(1000))
  }

  test("even-length palindromes") {
    assert(PalindromeNumber.isPalindrome(11))
    assert(PalindromeNumber.isPalindrome(1221))
    assert(PalindromeNumber.isPalindrome(123321))
  }

  test("odd-length palindromes") {
    assert(PalindromeNumber.isPalindrome(121))
    assert(PalindromeNumber.isPalindrome(12321))
    assert(PalindromeNumber.isPalindrome(1234321))
  }

  test("non-palindrome numbers") {
    assert(!PalindromeNumber.isPalindrome(12))
    assert(!PalindromeNumber.isPalindrome(123))
    assert(!PalindromeNumber.isPalindrome(1234))
  }

  test("larger numbers") {
    assert(PalindromeNumber.isPalindrome(9999999))
    assert(PalindromeNumber.isPalindrome(12344321))
    assert(!PalindromeNumber.isPalindrome(1234567))
    assert(!PalindromeNumber.isPalindrome(9876543))
  }

  test("performance: 1K iterations") {
    val startTime = System.nanoTime()
    
    for (i <- 0 until 1000) {
      PalindromeNumber.isPalindrome(i)
    }
    
    val endTime = System.nanoTime()
    val durationMs = (endTime - startTime) / 1_000_000.0
    
    println(f"[1K iterations] Execution time: $durationMs%.3fms")
    assert(durationMs < 100)
  }

  test("performance: 10K iterations") {
    val startTime = System.nanoTime()
    
    for (i <- 0 until 10000) {
      PalindromeNumber.isPalindrome(i % 1000000)
    }
    
    val endTime = System.nanoTime()
    val durationMs = (endTime - startTime) / 1_000_000.0
    
    println(f"[10K iterations] Execution time: $durationMs%.3fms")
    assert(durationMs < 200)
  }

  test("performance: 100K iterations") {
    val startTime = System.nanoTime()
    
    for (i <- 0 until 100000) {
      PalindromeNumber.isPalindrome(i % 10000000)
    }
    
    val endTime = System.nanoTime()
    val durationMs = (endTime - startTime) / 1_000_000.0
    
    println(f"[100K iterations] Execution time: $durationMs%.3fms")
    assert(durationMs < 1000)
  }
}