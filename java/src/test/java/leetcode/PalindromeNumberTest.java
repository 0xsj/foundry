// java/src/test/java/leetcode/PalindromeNumberTest.java

package leetcode;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class PalindromeNumberTest {

    @Test
    public void testSingleDigitNumbers() {
        assertTrue(PalindromeNumber.isPalindrome(0));
        assertTrue(PalindromeNumber.isPalindrome(7));
        assertTrue(PalindromeNumber.isPalindrome(9));
    }

    @Test
    public void testNegativeNumbers() {
        assertFalse(PalindromeNumber.isPalindrome(-121));
        assertFalse(PalindromeNumber.isPalindrome(-1));
        assertFalse(PalindromeNumber.isPalindrome(-101));
    }

    @Test
    public void testNumbersEndingInZero() {
        assertFalse(PalindromeNumber.isPalindrome(10));
        assertFalse(PalindromeNumber.isPalindrome(100));
        assertFalse(PalindromeNumber.isPalindrome(1000));
    }

    @Test
    public void testEvenLengthPalindromes() {
        assertTrue(PalindromeNumber.isPalindrome(11));
        assertTrue(PalindromeNumber.isPalindrome(1221));
        assertTrue(PalindromeNumber.isPalindrome(123321));
    }

    @Test
    public void testOddLengthPalindromes() {
        assertTrue(PalindromeNumber.isPalindrome(121));
        assertTrue(PalindromeNumber.isPalindrome(12321));
        assertTrue(PalindromeNumber.isPalindrome(1234321));
    }

    @Test
    public void testNonPalindromes() {
        assertFalse(PalindromeNumber.isPalindrome(12));
        assertFalse(PalindromeNumber.isPalindrome(123));
        assertFalse(PalindromeNumber.isPalindrome(1234));
    }

    @Test
    public void testLargerNumbers() {
        assertTrue(PalindromeNumber.isPalindrome(9999999));
        assertTrue(PalindromeNumber.isPalindrome(12344321));
        assertFalse(PalindromeNumber.isPalindrome(1234567));
        assertFalse(PalindromeNumber.isPalindrome(9876543));
    }

    @Test
    public void testPerformance1K() {
        long startTime = System.nanoTime();
        
        for (int i = 0; i < 1000; i++) {
            PalindromeNumber.isPalindrome(i);
        }
        
        long endTime = System.nanoTime();
        double durationMs = (endTime - startTime) / 1_000_000.0;
        
        System.out.printf("[1K iterations] Execution time: %.3fms%n", durationMs);
        assertTrue(durationMs < 100);
    }

    @Test
    public void testPerformance10K() {
        long startTime = System.nanoTime();
        
        for (int i = 0; i < 10000; i++) {
            PalindromeNumber.isPalindrome(i % 1000000);
        }
        
        long endTime = System.nanoTime();
        double durationMs = (endTime - startTime) / 1_000_000.0;
        
        System.out.printf("[10K iterations] Execution time: %.3fms%n", durationMs);
        assertTrue(durationMs < 200);
    }

    @Test
    public void testPerformance100K() {
        long startTime = System.nanoTime();
        
        for (int i = 0; i < 100000; i++) {
            PalindromeNumber.isPalindrome(i % 10000000);
        }
        
        long endTime = System.nanoTime();
        double durationMs = (endTime - startTime) / 1_000_000.0;
        
        System.out.printf("[100K iterations] Execution time: %.3fms%n", durationMs);
        assertTrue(durationMs < 1000);
    }
}