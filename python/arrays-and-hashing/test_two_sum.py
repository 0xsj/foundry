import time

import pytest
from two_sum import two_sum


class TestTwoSum:
    def test_basic_case(self):
        nums = [2, 7, 11, 15]
        target = 9
        result = two_sum(nums, target)
        assert result == [0, 1]
    
    def test_different_positions(self):
        nums = [3, 2, 4]
        target = 6
        result = two_sum(nums, target)
        assert result == [1, 2]
    
    def test_same_value_different_indices(self):
        nums = [3, 3]
        target = 6
        result = two_sum(nums, target)
        assert result == [0, 1]
    
    def test_negative_numbers(self):
        nums = [-1, -2, -3, -4, -5]
        target = -8
        result = two_sum(nums, target)
        assert result == [2, 4]


class TestTwoSumPerformance:
    def test_benchmark_1k_elements(self):
        nums = list(range(1000))
        target = 1997
        
        start = time.perf_counter()
        result = two_sum(nums, target)
        end = time.perf_counter()
        
        print(f"\n[1K elements] Execution time: {(end - start) * 1000:.3f}ms")
        assert result == [998, 999]
    
    def test_benchmark_10k_elements(self):
        nums = list(range(10000))
        target = 19997
        
        start = time.perf_counter()
        result = two_sum(nums, target)
        end = time.perf_counter()
        
        print(f"\n[10K elements] Execution time: {(end - start) * 1000:.3f}ms")
        assert result == [9998, 9999]
    
    def test_benchmark_100k_elements(self):
        nums = list(range(100000))
        target = 199997
        
        start = time.perf_counter()
        result = two_sum(nums, target)
        end = time.perf_counter()
        
        print(f"\n[100K elements] Execution time: {(end - start) * 1000:.3f}ms")
        assert result == [99998, 99999]
    
    def test_benchmark_worst_case(self):
        nums = list(range(10000))
        target = 19997
        
        start = time.perf_counter()
        result = two_sum(nums, target)
        end = time.perf_counter()
        
        print(f"\n[Worst case - 10K] Execution time: {(end - start) * 1000:.3f}ms")
        assert result == [9998, 9999]
