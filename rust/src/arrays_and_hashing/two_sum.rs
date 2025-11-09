use std::collections::HashMap;

pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    let mut num_map: HashMap<i32, usize> = HashMap::new();
    
    for (i, &num) in nums.iter().enumerate() {
        let complement = target - num;
        
        if let Some(&index) = num_map.get(&complement) {
            return vec![index as i32, i as i32];
        }
        
        num_map.insert(num, i);
    }
    
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        let nums = vec![2, 7, 11, 15];
        let target = 9;
        let result = two_sum(nums, target);
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_different_positions() {
        let nums = vec![3, 2, 4];
        let target = 6;
        let result = two_sum(nums, target);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_same_value_different_indices() {
        let nums = vec![3, 3];
        let target = 6;
        let result = two_sum(nums, target);
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_negative_numbers() {
        let nums = vec![-1, -2, -3, -4, -5];
        let target = -8;
        let result = two_sum(nums, target);
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_performance_1k() {
        let nums: Vec<i32> = (0..1000).collect();
        let target = 1997;
        
        let start = std::time::Instant::now();
        let result = two_sum(nums, target);
        let duration = start.elapsed();
        
        println!("[1K elements] Execution time: {:.3}ms", duration.as_secs_f64() * 1000.0);
        
        assert_eq!(result, vec![998, 999]);
    }

    #[test]
    fn test_performance_10k() {
        let nums: Vec<i32> = (0..10000).collect();
        let target = 19997;
        
        let start = std::time::Instant::now();
        let result = two_sum(nums, target);
        let duration = start.elapsed();
        
        println!("[10K elements] Execution time: {:.3}ms", duration.as_secs_f64() * 1000.0);
        
        assert_eq!(result, vec![9998, 9999]);
    }

    #[test]
    fn test_performance_100k() {
        let nums: Vec<i32> = (0..100000).collect();
        let target = 199997;
        
        let start = std::time::Instant::now();
        let result = two_sum(nums, target);
        let duration = start.elapsed();
        
        println!("[100K elements] Execution time: {:.3}ms", duration.as_secs_f64() * 1000.0);
        
        assert_eq!(result, vec![99998, 99999]);
    }
}