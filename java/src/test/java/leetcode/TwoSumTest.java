package leetcode;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import static org.junit.jupiter.api.Assertions.*;

public class TwoSumTest {
    
    private final TwoSum solution = new TwoSum();
    
    @Test
    @DisplayName("Basic case: [2,7,11,15], target=9")
    public void testBasicCase() {
        int[] nums = {2, 7, 11, 15};
        int target = 9;
        int[] result = solution.twoSum(nums, target);
        assertArrayEquals(new int[]{0, 1}, result);
    }
    
    @Test
    @DisplayName("Different positions: [3,2,4], target=6")
    public void testDifferentPositions() {
        int[] nums = {3, 2, 4};
        int target = 6;
        int[] result = solution.twoSum(nums, target);
        assertArrayEquals(new int[]{1, 2}, result);
    }
    
    @Test
    @DisplayName("Same value different indices: [3,3], target=6")
    public void testSameValueDifferentIndices() {
        int[] nums = {3, 3};
        int target = 6;
        int[] result = solution.twoSum(nums, target);
        assertArrayEquals(new int[]{0, 1}, result);
    }
    
    @Test
    @DisplayName("Negative numbers: [-1,-2,-3,-4,-5], target=-8")
    public void testNegativeNumbers() {
        int[] nums = {-1, -2, -3, -4, -5};
        int target = -8;
        int[] result = solution.twoSum(nums, target);
        assertArrayEquals(new int[]{2, 4}, result);
    }
    
    @Test
    @DisplayName("Performance: 1K elements")
    public void testPerformance1K() {
        int[] nums = new int[1000];
        for (int i = 0; i < nums.length; i++) {
            nums[i] = i;
        }
        int target = 1997;
        
        long start = System.nanoTime();
        int[] result = solution.twoSum(nums, target);
        long end = System.nanoTime();
        
        double timeMs = (end - start) / 1_000_000.0;
        System.out.printf("[1K elements] Execution time: %.3fms%n", timeMs);
        
        assertArrayEquals(new int[]{998, 999}, result);
    }
    
    @Test
    @DisplayName("Performance: 10K elements")
    public void testPerformance10K() {
        int[] nums = new int[10000];
        for (int i = 0; i < nums.length; i++) {
            nums[i] = i;
        }
        int target = 19997;
        
        long start = System.nanoTime();
        int[] result = solution.twoSum(nums, target);
        long end = System.nanoTime();
        
        double timeMs = (end - start) / 1_000_000.0;
        System.out.printf("[10K elements] Execution time: %.3fms%n", timeMs);
        
        assertArrayEquals(new int[]{9998, 9999}, result);
    }
    
    @Test
    @DisplayName("Performance: 100K elements")
    public void testPerformance100K() {
        int[] nums = new int[100000];
        for (int i = 0; i < nums.length; i++) {
            nums[i] = i;
        }
        int target = 199997;
        
        long start = System.nanoTime();
        int[] result = solution.twoSum(nums, target);
        long end = System.nanoTime();
        
        double timeMs = (end - start) / 1_000_000.0;
        System.out.printf("[100K elements] Execution time: %.3fms%n", timeMs);
        
        assertArrayEquals(new int[]{99998, 99999}, result);
    }
}
