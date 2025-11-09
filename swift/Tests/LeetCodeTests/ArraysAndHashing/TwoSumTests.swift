import XCTest
@testable import LeetCode

final class TwoSumTests: XCTestCase {
    let solution = TwoSum()
    
    func testBasicCase() {
        let nums = [2, 7, 11, 15]
        let target = 9
        let result = solution.twoSum(nums, target)
        XCTAssertEqual(result, [0, 1])
    }
    
    func testDifferentPositions() {
        let nums = [3, 2, 4]
        let target = 6
        let result = solution.twoSum(nums, target)
        XCTAssertEqual(result, [1, 2])
    }
    
    func testSameValueDifferentIndices() {
        let nums = [3, 3]
        let target = 6
        let result = solution.twoSum(nums, target)
        XCTAssertEqual(result, [0, 1])
    }
    
    func testNegativeNumbers() {
        let nums = [-1, -2, -3, -4, -5]
        let target = -8
        let result = solution.twoSum(nums, target)
        XCTAssertEqual(result, [2, 4])
    }
    
    func testPerformance1K() {
        let nums = Array(0..<1000)
        let target = 1997
        
        let start = CFAbsoluteTimeGetCurrent()
        let result = solution.twoSum(nums, target)
        let end = CFAbsoluteTimeGetCurrent()
        
        let timeMs = (end - start) * 1000
        print(String(format: "[1K elements] Execution time: %.3fms", timeMs))
        
        XCTAssertEqual(result, [998, 999])
    }
    
    func testPerformance10K() {
        let nums = Array(0..<10000)
        let target = 19997
        
        let start = CFAbsoluteTimeGetCurrent()
        let result = solution.twoSum(nums, target)
        let end = CFAbsoluteTimeGetCurrent()
        
        let timeMs = (end - start) * 1000
        print(String(format: "[10K elements] Execution time: %.3fms", timeMs))
        
        XCTAssertEqual(result, [9998, 9999])
    }
    
    func testPerformance100K() {
        let nums = Array(0..<100000)
        let target = 199997
        
        let start = CFAbsoluteTimeGetCurrent()
        let result = solution.twoSum(nums, target)
        let end = CFAbsoluteTimeGetCurrent()
        
        let timeMs = (end - start) * 1000
        print(String(format: "[100K elements] Execution time: %.3fms", timeMs))
        
        XCTAssertEqual(result, [99998, 99999])
    }
}