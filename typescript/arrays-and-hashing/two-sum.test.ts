import { twoSum } from './two-sum';

describe('twoSum', () => {
    test('should return indices [0, 1] for nums=[2,7,11,15] and target=9', () => {
        const nums = [2, 7, 11, 15];
        const target = 9;
        const result = twoSum(nums, target);
        expect(result).toEqual([0, 1]);
    });

    test('should return indices [1, 2] for nums=[3,2,4] and target=6', () => {
        const nums = [3, 2, 4];
        const target = 6;
        const result = twoSum(nums, target);
        expect(result).toEqual([1, 2]);
    });

    test('should return indices [0, 1] for nums=[3,3] and target=6', () => {
        const nums = [3, 3];
        const target = 6;
        const result = twoSum(nums, target);
        expect(result).toEqual([0, 1]);
    });

    test('should handle negative numbers', () => {
        const nums = [-1, -2, -3, -4, -5];
        const target = -8;
        const result = twoSum(nums, target);
        expect(result).toEqual([2, 4]);
    });
});

describe('twoSum - Performance Benchmarks', () => {
    test('benchmark with 1,000 elements', () => {
        const nums = Array.from({ length: 1000 }, (_, i) => i);
        const target = 1997;
        
        const start = performance.now();
        const result = twoSum(nums, target);
        const end = performance.now();
        
        console.log(`[1K elements] Execution time: ${(end - start).toFixed(3)}ms`);
        expect(result).toEqual([998, 999]);
    });

    test('benchmark with 10,000 elements', () => {
        const nums = Array.from({ length: 10000 }, (_, i) => i);
        const target = 19997;
        
        const start = performance.now();
        const result = twoSum(nums, target);
        const end = performance.now();
        
        console.log(`[10K elements] Execution time: ${(end - start).toFixed(3)}ms`);
        expect(result).toEqual([9998, 9999]);
    });

    test('benchmark with 100,000 elements', () => {
        const nums = Array.from({ length: 100000 }, (_, i) => i);
        const target = 199997;
        
        const start = performance.now();
        const result = twoSum(nums, target);
        const end = performance.now();
        
        console.log(`[100K elements] Execution time: ${(end - start).toFixed(3)}ms`);
        expect(result).toEqual([99998, 99999]);
    });

    test('benchmark worst case - target at end', () => {
        const nums = Array.from({ length: 10000 }, (_, i) => i);
        const target = 19997; // Last two elements sum
        
        const start = performance.now();
        const result = twoSum(nums, target);
        const end = performance.now();
        
        console.log(`[Worst case - 10K] Execution time: ${(end - start).toFixed(3)}ms`);
        expect(result).toEqual([9998, 9999]);
    });
});