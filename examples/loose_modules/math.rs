/// Computes the sum of a slice of integers.
pub fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

/// Computes the arithmetic mean of a slice of integers.
pub fn mean(numbers: &[i32]) -> f64 {
    if numbers.is_empty() {
        return 0.0;
    }
    sum(numbers) as f64 / numbers.len() as f64
}

/// Returns the maximum value in a slice, or None if empty.
pub fn max(numbers: &[i32]) -> Option<i32> {
    numbers.iter().copied().max()
}

/// Returns the minimum value in a slice, or None if empty.
pub fn min(numbers: &[i32]) -> Option<i32> {
    numbers.iter().copied().min()
}

/// Computes the n-th Fibonacci number.
pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a: u64 = 0;
            let mut b: u64 = 1;
            for _ in 2..=n {
                let tmp = a + b;
                a = b;
                b = tmp;
            }
            b
        }
    }
}
