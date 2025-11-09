package leetcode.ArraysAndHashing

import org.scalatest.funsuite.AnyFunSuite

class TwoSumTest extends AnyFunSuite {
  
  test("basic case: [2,7,11,15], target=9") {
    val nums = Array(2, 7, 11, 15)
    val target = 9
    val result = TwoSum.twoSum(nums, target)
    assert(result.sameElements(Array(0, 1)))
  }
  
  test("different positions: [3,2,4], target=6") {
    val nums = Array(3, 2, 4)
    val target = 6
    val result = TwoSum.twoSum(nums, target)
    assert(result.sameElements(Array(1, 2)))
  }
  
  test("same value different indices: [3,3], target=6") {
    val nums = Array(3, 3)
    val target = 6
    val result = TwoSum.twoSum(nums, target)
    assert(result.sameElements(Array(0, 1)))
  }
  
  test("negative numbers: [-1,-2,-3,-4,-5], target=-8") {
    val nums = Array(-1, -2, -3, -4, -5)
    val target = -8
    val result = TwoSum.twoSum(nums, target)
    assert(result.sameElements(Array(2, 4)))
  }
  
  test("performance: 1K elements") {
    val nums = (0 until 1000).toArray
    val target = 1997
    
    val start = System.nanoTime()
    val result = TwoSum.twoSum(nums, target)
    val end = System.nanoTime()
    
    val timeMs = (end - start) / 1_000_000.0
    println(f"[1K elements] Execution time: $timeMs%.3fms")
    
    assert(result.sameElements(Array(998, 999)))
  }
  
  test("performance: 10K elements") {
    val nums = (0 until 10000).toArray
    val target = 19997
    
    val start = System.nanoTime()
    val result = TwoSum.twoSum(nums, target)
    val end = System.nanoTime()
    
    val timeMs = (end - start) / 1_000_000.0
    println(f"[10K elements] Execution time: $timeMs%.3fms")
    
    assert(result.sameElements(Array(9998, 9999)))
  }
  
  test("performance: 100K elements") {
    val nums = (0 until 100000).toArray
    val target = 199997
    
    val start = System.nanoTime()
    val result = TwoSum.twoSum(nums, target)
    val end = System.nanoTime()
    
    val timeMs = (end - start) / 1_000_000.0
    println(f"[100K elements] Execution time: $timeMs%.3fms")
    
    assert(result.sameElements(Array(99998, 99999)))
  }
}