using Xunit;

namespace LeetCode.ArraysAndHashing.Tests;

public class TwoSumTest
{
    private readonly TwoSum _solution = new TwoSum();
    
    [Fact]
    public void TestBasicCase()
    {
        int[] nums = { 2, 7, 11, 15 };
        int target = 9;
        int[] result = _solution.Solution(nums, target);
        Assert.Equal(new int[] { 0, 1 }, result);
    }
    
    [Fact]
    public void TestDifferentPositions()
    {
        int[] nums = { 3, 2, 4 };
        int target = 6;
        int[] result = _solution.Solution(nums, target);
        Assert.Equal(new int[] { 1, 2 }, result);
    }
    
    [Fact]
    public void TestSameValueDifferentIndices()
    {
        int[] nums = { 3, 3 };
        int target = 6;
        int[] result = _solution.Solution(nums, target);
        Assert.Equal(new int[] { 0, 1 }, result);
    }
    
    [Fact]
    public void TestNegativeNumbers()
    {
        int[] nums = { -1, -2, -3, -4, -5 };
        int target = -8;
        int[] result = _solution.Solution(nums, target);
        Assert.Equal(new int[] { 2, 4 }, result);
    }
    
    [Fact]
    public void TestPerformance1K()
    {
        int[] nums = new int[1000];
        for (int i = 0; i < nums.Length; i++)
        {
            nums[i] = i;
        }
        int target = 1997;
        
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        int[] result = _solution.Solution(nums, target);
        stopwatch.Stop();
        
        System.Console.WriteLine($"[1K elements] Execution time: {stopwatch.Elapsed.TotalMilliseconds:F3}ms");
        
        Assert.Equal(new int[] { 998, 999 }, result);
    }
    
    [Fact]
    public void TestPerformance10K()
    {
        int[] nums = new int[10000];
        for (int i = 0; i < nums.Length; i++)
        {
            nums[i] = i;
        }
        int target = 19997;
        
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        int[] result = _solution.Solution(nums, target);
        stopwatch.Stop();
        
        System.Console.WriteLine($"[10K elements] Execution time: {stopwatch.Elapsed.TotalMilliseconds:F3}ms");
        
        Assert.Equal(new int[] { 9998, 9999 }, result);
    }
    
    [Fact]
    public void TestPerformance100K()
    {
        int[] nums = new int[100000];
        for (int i = 0; i < nums.Length; i++)
        {
            nums[i] = i;
        }
        int target = 199997;
        
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        int[] result = _solution.Solution(nums, target);
        stopwatch.Stop();
        
        System.Console.WriteLine($"[100K elements] Execution time: {stopwatch.Elapsed.TotalMilliseconds:F3}ms");
        
        Assert.Equal(new int[] { 99998, 99999 }, result);
    }
}
