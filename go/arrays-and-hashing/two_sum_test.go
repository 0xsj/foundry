package arrays_and_hashing

import (
	"testing"
	"time"
)

func TestTwoSum(t *testing.T) {
	tests := []struct {
		name   string
		nums   []int
		target int
		want   []int
	}{
		{
			name:   "basic case",
			nums:   []int{2, 7, 11, 15},
			target: 9,
			want:   []int{0, 1},
		},
		{
			name:   "different positions",
			nums:   []int{3, 2, 4},
			target: 6,
			want:   []int{1, 2},
		},
		{
			name:   "same value different indices",
			nums:   []int{3, 3},
			target: 6,
			want:   []int{0, 1},
		},
		{
			name:   "negative numbers",
			nums:   []int{-1, -2, -3, -4, -5},
			target: -8,
			want:   []int{2, 4},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := TwoSum(tt.nums, tt.target)
			if len(got) != len(tt.want) {
				t.Errorf("TwoSum() = %v, want %v", got, tt.want)
				return
			}
			for i := range got {
				if got[i] != tt.want[i] {
					t.Errorf("TwoSum() = %v, want %v", got, tt.want)
					return
				}
			}
		})
	}
}

func BenchmarkTwoSum1K(b *testing.B) {
	nums := make([]int, 1000)
	for i := range nums {
		nums[i] = i
	}
	target := 1997

	start := time.Now()
	result := TwoSum(nums, target)
	elapsed := time.Since(start)

	b.Logf("[1K elements] Execution time: %.3fms", float64(elapsed.Microseconds())/1000)

	if result[0] != 998 || result[1] != 999 {
		b.Errorf("Expected [998, 999], got %v", result)
	}
}

func BenchmarkTwoSum10K(b *testing.B) {
	nums := make([]int, 10000)
	for i := range nums {
		nums[i] = i
	}
	target := 19997

	start := time.Now()
	result := TwoSum(nums, target)
	elapsed := time.Since(start)

	b.Logf("[10K elements] Execution time: %.3fms", float64(elapsed.Microseconds())/1000)

	if result[0] != 9998 || result[1] != 9999 {
		b.Errorf("Expected [9998, 9999], got %v", result)
	}
}

func BenchmarkTwoSum100K(b *testing.B) {
	nums := make([]int, 100000)
	for i := range nums {
		nums[i] = i
	}
	target := 199997

	start := time.Now()
	result := TwoSum(nums, target)
	elapsed := time.Since(start)

	b.Logf("[100K elements] Execution time: %.3fms", float64(elapsed.Microseconds())/1000)

	if result[0] != 99998 || result[1] != 99999 {
		b.Errorf("Expected [99998, 99999], got %v", result)
	}
}
