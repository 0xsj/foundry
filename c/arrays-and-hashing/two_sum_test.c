#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "../test_framework.h"

// Include the implementation
int* twoSum(int* nums, int numsSize, int target, int* returnSize);

void test_basic_case() {
    int nums[] = {2, 7, 11, 15};
    int target = 9;
    int returnSize;
    int* result = twoSum(nums, 4, target, &returnSize);
    
    ASSERT_EQ(returnSize, 2, "Return size should be 2");
    ASSERT_EQ(result[0], 0, "First index should be 0");
    ASSERT_EQ(result[1], 1, "Second index should be 1");
    
    free(result);
}

void test_different_positions() {
    int nums[] = {3, 2, 4};
    int target = 6;
    int returnSize;
    int* result = twoSum(nums, 3, target, &returnSize);
    
    ASSERT_EQ(returnSize, 2, "Return size should be 2");
    ASSERT_EQ(result[0], 1, "First index should be 1");
    ASSERT_EQ(result[1], 2, "Second index should be 2");
    
    free(result);
}

void test_same_value_different_indices() {
    int nums[] = {3, 3};
    int target = 6;
    int returnSize;
    int* result = twoSum(nums, 2, target, &returnSize);
    
    ASSERT_EQ(returnSize, 2, "Return size should be 2");
    ASSERT_EQ(result[0], 0, "First index should be 0");
    ASSERT_EQ(result[1], 1, "Second index should be 1");
    
    free(result);
}

void test_negative_numbers() {
    int nums[] = {-1, -2, -3, -4, -5};
    int target = -8;
    int returnSize;
    int* result = twoSum(nums, 5, target, &returnSize);
    
    ASSERT_EQ(returnSize, 2, "Return size should be 2");
    ASSERT_EQ(result[0], 2, "First index should be 2");
    ASSERT_EQ(result[1], 4, "Second index should be 4");
    
    free(result);
}

void test_performance_1k() {
    int* nums = (int*)malloc(1000 * sizeof(int));
    for (int i = 0; i < 1000; i++) {
        nums[i] = i;
    }
    int target = 1997;
    int returnSize;
    
    clock_t start = clock();
    int* result = twoSum(nums, 1000, target, &returnSize);
    clock_t end = clock();
    
    double timeMs = ((double)(end - start) / CLOCKS_PER_SEC) * 1000.0;
    printf("[1K elements] Execution time: %.3fms\n", timeMs);
    
    ASSERT_EQ(result[0], 998, "First index should be 998");
    ASSERT_EQ(result[1], 999, "Second index should be 999");
    
    free(nums);
    free(result);
}

void test_performance_10k() {
    int* nums = (int*)malloc(10000 * sizeof(int));
    for (int i = 0; i < 10000; i++) {
        nums[i] = i;
    }
    int target = 19997;
    int returnSize;
    
    clock_t start = clock();
    int* result = twoSum(nums, 10000, target, &returnSize);
    clock_t end = clock();
    
    double timeMs = ((double)(end - start) / CLOCKS_PER_SEC) * 1000.0;
    printf("[10K elements] Execution time: %.3fms\n", timeMs);
    
    ASSERT_EQ(result[0], 9998, "First index should be 9998");
    ASSERT_EQ(result[1], 9999, "Second index should be 9999");
    
    free(nums);
    free(result);
}

int main() {
    RUN_TEST(test_basic_case);
    RUN_TEST(test_different_positions);
    RUN_TEST(test_same_value_different_indices);
    RUN_TEST(test_negative_numbers);
    RUN_TEST(test_performance_1k);
    RUN_TEST(test_performance_10k);
    
    printf("\nAll tests passed!\n");
    return 0;
}