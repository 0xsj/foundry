// go/arrays-and-hashing/palindrome_number_test.go

package arrays_and_hashing

import (
	"testing"
	"time"
)

func TestSingleDigitNumbers(t *testing.T) {
	tests := []struct {
		input    int
		expected bool
	}{
		{0, true},
		{7, true},
		{9, true},
	}

	for _, test := range tests {
		result := IsPalindrome(test.input)
		if result != test.expected {
			t.Errorf("IsPalindrome(%d) = %v; expected %v", test.input, result, test.expected)
		}
	}
}

func TestNegativeNumbers(t *testing.T) {
	tests := []struct {
		input    int
		expected bool
	}{
		{-121, false},
		{-1, false},
		{-101, false},
	}

	for _, test := range tests {
		result := IsPalindrome(test.input)
		if result != test.expected {
			t.Errorf("IsPalindrome(%d) = %v; expected %v", test.input, result, test.expected)
		}
	}
}

func TestNumbersEndingInZero(t *testing.T) {
	tests := []struct {
		input    int
		expected bool
	}{
		{10, false},
		{100, false},
		{1000, false},
	}

	for _, test := range tests {
		result := IsPalindrome(test.input)
		if result != test.expected {
			t.Errorf("IsPalindrome(%d) = %v; expected %v", test.input, result, test.expected)
		}
	}
}

func TestEvenLengthPalindromes(t *testing.T) {
	tests := []struct {
		input    int
		expected bool
	}{
		{11, true},
		{1221, true},
		{123321, true},
	}

	for _, test := range tests {
		result := IsPalindrome(test.input)
		if result != test.expected {
			t.Errorf("IsPalindrome(%d) = %v; expected %v", test.input, result, test.expected)
		}
	}
}

func TestOddLengthPalindromes(t *testing.T) {
	tests := []struct {
		input    int
		expected bool
	}{
		{121, true},
		{12321, true},
		{1234321, true},
	}

	for _, test := range tests {
		result := IsPalindrome(test.input)
		if result != test.expected {
			t.Errorf("IsPalindrome(%d) = %v; expected %v", test.input, result, test.expected)
		}
	}
}

func TestNonPalindromes(t *testing.T) {
	tests := []struct {
		input    int
		expected bool
	}{
		{12, false},
		{123, false},
		{1234, false},
	}

	for _, test := range tests {
		result := IsPalindrome(test.input)
		if result != test.expected {
			t.Errorf("IsPalindrome(%d) = %v; expected %v", test.input, result, test.expected)
		}
	}
}

func TestLargerNumbers(t *testing.T) {
	tests := []struct {
		input    int
		expected bool
	}{
		{9999999, true},
		{12344321, true},
		{1234567, false},
		{9876543, false},
	}

	for _, test := range tests {
		result := IsPalindrome(test.input)
		if result != test.expected {
			t.Errorf("IsPalindrome(%d) = %v; expected %v", test.input, result, test.expected)
		}
	}
}

func BenchmarkPalindromeSmall(b *testing.B) {
	start := time.Now()
	for i := 0; i < 1000; i++ {
		IsPalindrome(i)
	}
	duration := time.Since(start)
	b.Logf("[1K iterations] Execution time: %.3fms", float64(duration.Microseconds())/1000.0)
}

func BenchmarkPalindromeMedium(b *testing.B) {
	start := time.Now()
	for i := 0; i < 10000; i++ {
		IsPalindrome(i % 1000000)
	}
	duration := time.Since(start)
	b.Logf("[10K iterations] Execution time: %.3fms", float64(duration.Microseconds())/1000.0)
}

func BenchmarkPalindromeLarge(b *testing.B) {
	start := time.Now()
	for i := 0; i < 100000; i++ {
		IsPalindrome(i % 10000000)
	}
	duration := time.Since(start)
	b.Logf("[100K iterations] Execution time: %.3fms", float64(duration.Microseconds())/1000.0)
}