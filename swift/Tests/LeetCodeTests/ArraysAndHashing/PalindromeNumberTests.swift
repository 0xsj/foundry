// swift/Tests/LeetCodeTests/ArraysAndHashing/PalindromeNumberTests.swift

import XCTest
@testable import LeetCode

final class PalindromeNumberTests: XCTestCase {
    
    func testSingleDigitNumbers() {
        XCTAssertTrue(PalindromeNumber.isPalindrome(0))
        XCTAssertTrue(PalindromeNumber.isPalindrome(7))
        XCTAssertTrue(PalindromeNumber.isPalindrome(9))
    }
    
    func testNegativeNumbers() {
        XCTAssertFalse(PalindromeNumber.isPalindrome(-121))
        XCTAssertFalse(PalindromeNumber.isPalindrome(-1))
        XCTAssertFalse(PalindromeNumber.isPalindrome(-101))
    }
    
    func testNumbersEndingInZero() {
        XCTAssertFalse(PalindromeNumber.isPalindrome(10))
        XCTAssertFalse(PalindromeNumber.isPalindrome(100))
        XCTAssertFalse(PalindromeNumber.isPalindrome(1000))
    }
    
    func testEvenLengthPalindromes() {
        XCTAssertTrue(PalindromeNumber.isPalindrome(11))
        XCTAssertTrue(PalindromeNumber.isPalindrome(1221))
        XCTAssertTrue(PalindromeNumber.isPalindrome(123321))
    }
    
    func testOddLengthPalindromes() {
        XCTAssertTrue(PalindromeNumber.isPalindrome(121))
        XCTAssertTrue(PalindromeNumber.isPalindrome(12321))
        XCTAssertTrue(PalindromeNumber.isPalindrome(1234321))
    }
    
    func testNonPalindromes() {
        XCTAssertFalse(PalindromeNumber.isPalindrome(12))
        XCTAssertFalse(PalindromeNumber.isPalindrome(123))
        XCTAssertFalse(PalindromeNumber.isPalindrome(1234))
    }
    
    func testLargerNumbers() {
        XCTAssertTrue(PalindromeNumber.isPalindrome(9999999))
        XCTAssertTrue(PalindromeNumber.isPalindrome(12344321))
        XCTAssertFalse(PalindromeNumber.isPalindrome(1234567))
        XCTAssertFalse(PalindromeNumber.isPalindrome(9876543))
    }
    
    func testPerformance1K() {
        let startTime = CFAbsoluteTimeGetCurrent()
        
        for i in 0..<1000 {
            _ = PalindromeNumber.isPalindrome(i)
        }
        
        let timeElapsed = (CFAbsoluteTimeGetCurrent() - startTime) * 1000
        print("[1K iterations] Execution time: \(String(format: "%.3f", timeElapsed))ms")
        
        XCTAssertLessThan(timeElapsed, 100)
    }
    
    func testPerformance10K() {
        let startTime = CFAbsoluteTimeGetCurrent()
        
        for i in 0..<10000 {
            _ = PalindromeNumber.isPalindrome(i % 1000000)
        }
        
        let timeElapsed = (CFAbsoluteTimeGetCurrent() - startTime) * 1000
        print("[10K iterations] Execution time: \(String(format: "%.3f", timeElapsed))ms")
        
        XCTAssertLessThan(timeElapsed, 200)
    }
    
    func testPerformance100K() {
        let startTime = CFAbsoluteTimeGetCurrent()
        
        for i in 0..<100000 {
            _ = PalindromeNumber.isPalindrome(i % 10000000)
        }
        
        let timeElapsed = (CFAbsoluteTimeGetCurrent() - startTime) * 1000
        print("[100K iterations] Execution time: \(String(format: "%.3f", timeElapsed))ms")
        
        XCTAssertLessThan(timeElapsed, 1000)
    }
}