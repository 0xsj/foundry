# python/arrays-and-hashing/test_palindrome_number.py

import pytest
import time
from palindrome_number import is_palindrome


class TestPalindromeNumber:
    """Test cases for palindrome number problem"""

    def test_single_digit_numbers(self):
        """Single digit numbers are palindromes"""
        assert is_palindrome(0) == True
        assert is_palindrome(7) == True
        assert is_palindrome(9) == True

    def test_negative_numbers(self):
        """Negative numbers are not palindromes"""
        assert is_palindrome(-121) == False
        assert is_palindrome(-1) == False
        assert is_palindrome(-101) == False

    def test_numbers_ending_in_zero(self):
        """Numbers ending in zero (except 0) are not palindromes"""
        assert is_palindrome(10) == False
        assert is_palindrome(100) == False
        assert is_palindrome(1000) == False

    def test_even_length_palindromes(self):
        """Even-length palindromes"""
        assert is_palindrome(11) == True
        assert is_palindrome(1221) == True
        assert is_palindrome(123321) == True

    def test_odd_length_palindromes(self):
        """Odd-length palindromes"""
        assert is_palindrome(121) == True
        assert is_palindrome(12321) == True
        assert is_palindrome(1234321) == True

    def test_non_palindromes(self):
        """Non-palindrome numbers"""
        assert is_palindrome(12) == False
        assert is_palindrome(123) == False
        assert is_palindrome(1234) == False

    def test_larger_numbers(self):
        """Larger numbers"""
        assert is_palindrome(9999999) == True
        assert is_palindrome(12344321) == True
        assert is_palindrome(1234567) == False
        assert is_palindrome(9876543) == False


class TestPalindromeNumberPerformance:
    """Performance benchmarks for palindrome number"""

    def test_performance_1k(self):
        """Performance test with 1K iterations"""
        start_time = time.perf_counter()
        
        for i in range(1000):
            is_palindrome(i)
        
        end_time = time.perf_counter()
        duration_ms = (end_time - start_time) * 1000
        
        print(f"\n[1K iterations] Execution time: {duration_ms:.3f}ms")
        assert duration_ms < 100

    def test_performance_10k(self):
        """Performance test with 10K iterations"""
        start_time = time.perf_counter()
        
        for i in range(10000):
            is_palindrome(i % 1000000)
        
        end_time = time.perf_counter()
        duration_ms = (end_time - start_time) * 1000
        
        print(f"\n[10K iterations] Execution time: {duration_ms:.3f}ms")
        assert duration_ms < 200

    def test_performance_100k(self):
        """Performance test with 100K iterations"""
        start_time = time.perf_counter()
        
        for i in range(100000):
            is_palindrome(i % 10000000)
        
        end_time = time.perf_counter()
        duration_ms = (end_time - start_time) * 1000
        
        print(f"\n[100K iterations] Execution time: {duration_ms:.3f}ms")
        assert duration_ms < 1000