// rust/src/arrays_and_hashing/palindrome_number.rs

pub fn is_palindrome(x: i32) -> bool {
    // Implementation will go here
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_single_digit_numbers() {
        assert_eq!(is_palindrome(0), true);
        assert_eq!(is_palindrome(7), true);
        assert_eq!(is_palindrome(9), true);
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(is_palindrome(-121), false);
        assert_eq!(is_palindrome(-1), false);
        assert_eq!(is_palindrome(-101), false);
    }

    #[test]
    fn test_numbers_ending_in_zero() {
        assert_eq!(is_palindrome(10), false);
        assert_eq!(is_palindrome(100), false);
        assert_eq!(is_palindrome(1000), false);
    }

    #[test]
    fn test_even_length_palindromes() {
        assert_eq!(is_palindrome(11), true);
        assert_eq!(is_palindrome(1221), true);
        assert_eq!(is_palindrome(123321), true);
    }

    #[test]
    fn test_odd_length_palindromes() {
        assert_eq!(is_palindrome(121), true);
        assert_eq!(is_palindrome(12321), true);
        assert_eq!(is_palindrome(1234321), true);
    }

    #[test]
    fn test_non_palindromes() {
        assert_eq!(is_palindrome(12), false);
        assert_eq!(is_palindrome(123), false);
        assert_eq!(is_palindrome(1234), false);
    }

    #[test]
    fn test_larger_numbers() {
        assert_eq!(is_palindrome(9999999), true);
        assert_eq!(is_palindrome(12344321), true);
        assert_eq!(is_palindrome(1234567), false);
        assert_eq!(is_palindrome(9876543), false);
    }

    #[test]
    fn test_performance_1k() {
        let start = Instant::now();
        
        for i in 0..1000 {
            is_palindrome(i);
        }
        
        let duration = start.elapsed();
        println!("[1K iterations] Execution time: {:.3}ms", duration.as_secs_f64() * 1000.0);
        
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_performance_10k() {
        let start = Instant::now();
        
        for i in 0..10000 {
            is_palindrome(i % 1000000);
        }
        
        let duration = start.elapsed();
        println!("[10K iterations] Execution time: {:.3}ms", duration.as_secs_f64() * 1000.0);
        
        assert!(duration.as_millis() < 200);
    }

    #[test]
    fn test_performance_100k() {
        let start = Instant::now();
        
        for i in 0..100000 {
            is_palindrome(i % 10000000);
        }
        
        let duration = start.elapsed();
        println!("[100K iterations] Execution time: {:.3}ms", duration.as_secs_f64() * 1000.0);
        
        assert!(duration.as_millis() < 1000);
    }
}