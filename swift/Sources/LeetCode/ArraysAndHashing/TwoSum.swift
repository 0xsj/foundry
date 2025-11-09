public class TwoSum {
    public init() {}

    public func twoSum(_ nums: [Int], _ target: Int) -> [Int] {
        var numMap: [Int: Int] = [:]

        for i in 0..<nums.count {
            let complement = target - nums[i]

            if let index = numMap[complement] {
                return [index, i]
            }

            numMap[nums[i]] = i
        }

        return []
    }
}