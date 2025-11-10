// typescript/arrays-and-hashing/palindrome-number.test.ts
import {isPalindrome} from './palindrome-number'

describe('isPalindrome', () => {
  // Import the function (we'll create this next)
  const isPalindrome = require('./palindrome-number').isPalindrome;

  describe('Correctness Tests', () => {
    test('single digit numbers are palindromes', () => {
      expect(isPalindrome(0)).toBe(true);
      expect(isPalindrome(7)).toBe(true);
      expect(isPalindrome(9)).toBe(true);
    });

    test('negative numbers are not palindromes', () => {
      expect(isPalindrome(-121)).toBe(false);
      expect(isPalindrome(-1)).toBe(false);
      expect(isPalindrome(-101)).toBe(false);
    });

    test('numbers ending in zero (except 0) are not palindromes', () => {
      expect(isPalindrome(10)).toBe(false);
      expect(isPalindrome(100)).toBe(false);
      expect(isPalindrome(1000)).toBe(false);
    });

    test('even-length palindromes', () => {
      expect(isPalindrome(11)).toBe(true);
      expect(isPalindrome(1221)).toBe(true);
      expect(isPalindrome(123321)).toBe(true);
    });

    test('odd-length palindromes', () => {
      expect(isPalindrome(121)).toBe(true);
      expect(isPalindrome(12321)).toBe(true);
      expect(isPalindrome(1234321)).toBe(true);
    });

    test('non-palindrome numbers', () => {
      expect(isPalindrome(12)).toBe(false);
      expect(isPalindrome(123)).toBe(false);
      expect(isPalindrome(1234)).toBe(false);
    });

    test('larger palindromes', () => {
      expect(isPalindrome(9999999)).toBe(true);
      expect(isPalindrome(12344321)).toBe(true);
    });

    test('larger non-palindromes', () => {
      expect(isPalindrome(1234567)).toBe(false);
      expect(isPalindrome(9876543)).toBe(false);
    });
  });

  describe('Performance Benchmarks', () => {
    test('performance with small numbers (1-1000)', () => {
      const start = performance.now();
      
      for (let i = 0; i < 1000; i++) {
        isPalindrome(i);
      }
      
      const end = performance.now();
      const duration = end - start;
      
      console.log(`Small numbers (1K iterations): ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(100); // Should be very fast
    });

    test('performance with medium numbers (10K iterations)', () => {
      const start = performance.now();
      
      for (let i = 0; i < 10000; i++) {
        isPalindrome(i % 1000000); // Numbers up to 1 million
      }
      
      const end = performance.now();
      const duration = end - start;
      
      console.log(`Medium numbers (10K iterations): ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(200);
    });

    test('performance with large numbers (100K iterations)', () => {
      const start = performance.now();
      
      for (let i = 0; i < 100000; i++) {
        isPalindrome(i % 10000000); // Numbers up to 10 million
      }
      
      const end = performance.now();
      const duration = end - start;
      
      console.log(`Large numbers (100K iterations): ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(1000);
    });

    test('worst case: large palindromes', () => {
      const start = performance.now();
      
      for (let i = 0; i < 10000; i++) {
        isPalindrome(123454321); // 9-digit palindrome
        isPalindrome(987656789); // 9-digit palindrome
      }
      
      const end = performance.now();
      const duration = end - start;
      
      console.log(`Large palindromes (10K iterations): ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(200);
    });
  });
});