public class TwoSum
{
    public int[] Solution(int[] nums, int target)
    {
        var numMap = new Dictionary<int, int>();

        for (int i = 0; i < nums.Length; i++)
        {
            int complement = target - nums[i];

            if (numMap.ContainsKey(complement))
            {
                return new int[] {numMap[complement], i};
            }
            numMap[nums[i]] = i;
        }
        return new int[] {};
    }
}