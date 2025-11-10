// c/arrays-and-hashing/palindrome_number_test.c

#include <stdio.h>
#include <stdbool.h>
#include <time.h>
#include "../test_framework.h"

// Forward declaration - we'll implement this
bool isPalindrome(int x);

void test_single_digit_numbers() {
    ASSERT_EQ(isPalindrome(0), true, "0 should be palindrome");
    ASSERT_EQ(isPalindrome(7), true, "7 should be palindrome");
    ASSERT_EQ(isPalindrome(9), true, "9 should be palindrome");
}

void test_negative_numbers() {
    ASSERT_EQ(isPalindrome(-121), false, "-121 should not be palindrome");
    ASSERT_EQ(isPalindrome(-1), false, "-1 should not be palindrome");
    ASSERT_EQ(isPalindrome(-101), false, "-101 should not be palindrome");
}

void test_numbers_ending_in_zero() {
    ASSERT_EQ(isPalindrome(10), false, "10 should not be palindrome");
    ASSERT_EQ(isPalindrome(100), false, "100 should not be palindrome");
    ASSERT_EQ(isPalindrome(1000), false, "1000 should not be palindrome");
}

void test_even_length_palindromes() {
    ASSERT_EQ(isPalindrome(11), true, "11 should be palindrome");
    ASSERT_EQ(isPalindrome(1221), true, "1221 should be palindrome");
    ASSERT_EQ(isPalindrome(123321), true, "123321 should be palindrome");
}

void test_odd_length_palindromes() {
    ASSERT_EQ(isPalindrome(121), true, "121 should be palindrome");
    ASSERT_EQ(isPalindrome(12321), true, "12321 should be palindrome");
    ASSERT_EQ(isPalindrome(1234321), true, "1234321 should be palindrome");
}

void test_non_palindromes() {
    ASSERT_EQ(isPalindrome(12), false, "12 should not be palindrome");
    ASSERT_EQ(isPalindrome(123), false, "123 should not be palindrome");
    ASSERT_EQ(isPalindrome(1234), false, "1234 should not be palindrome");
}

void test_larger_numbers() {
    ASSERT_EQ(isPalindrome(9999999), true, "9999999 should be palindrome");
    ASSERT_EQ(isPalindrome(12344321), true, "12344321 should be palindrome");
    ASSERT_EQ(isPalindrome(1234567), false, "1234567 should not be palindrome");
    ASSERT_EQ(isPalindrome(9876543), false, "9876543 should not be palindrome");
}

void test_performance_1k() {
    clock_t start = clock();
    for (int i = 0; i < 1000; i++) {
        isPalindrome(i);
    }
    clock_t end = clock();
    double timeMs = ((double)(end - start) / CLOCKS_PER_SEC) * 1000.0;
    printf("[1K iterations] Execution time: %.3fms\n", timeMs);
}

void test_performance_10k() {
    clock_t start = clock();
    for (int i = 0; i < 10000; i++) {
        isPalindrome(i % 1000000);
    }
    clock_t end = clock();
    double timeMs = ((double)(end - start) / CLOCKS_PER_SEC) * 1000.0;
    printf("[10K iterations] Execution time: %.3fms\n", timeMs);
}

void test_performance_100k() {
    clock_t start = clock();
    for (int i = 0; i < 100000; i++) {
        isPalindrome(i % 10000000);
    }
    clock_t end = clock();
    double timeMs = ((double)(end - start) / CLOCKS_PER_SEC) * 1000.0;
    printf("[100K iterations] Execution time: %.3fms\n", timeMs);
}

int main() {
    RUN_TEST(test_single_digit_numbers);
    RUN_TEST(test_negative_numbers);
    RUN_TEST(test_numbers_ending_in_zero);
    RUN_TEST(test_even_length_palindromes);
    RUN_TEST(test_odd_length_palindromes);
    RUN_TEST(test_non_palindromes);
    RUN_TEST(test_larger_numbers);
    RUN_TEST(test_performance_1k);
    RUN_TEST(test_performance_10k);
    RUN_TEST(test_performance_100k);
    
    printf("\nAll tests passed!\n");
    return 0;
}