package leetcode.ArraysAndHashing

object TwoSum {
  def twoSum(nums: Array[Int], target: Int): Array[Int] = {
    nums.indices.foldLeft((Map[Int, Int](), Array[Int]())) { 
      case ((map, result), i) =>
        if (result.nonEmpty) {
          (map, result)
        } else {
          val complement = target - nums(i)
          if (map.contains(complement)) {
            (map, Array(map(complement), i))
          } else {
            (map + (nums(i) -> i), result)
          }
        }
    }._2
  }
}