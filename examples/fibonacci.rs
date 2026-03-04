// Fibonacci sequence - demonstrates functions, loops, and recursion
fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    let mut a = 0;
    let mut b = 1;
    let mut i = 2;
    while i <= n {
        let temp = a + b;
        a = b;
        b = temp;
        i += 1;
    }
    b
}

fn fibonacci_recursive(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2)
    }
}

fn main() {
    println!("=== Fibonacci Sequence ===\n");

    // Iterative version
    println!("Iterative Fibonacci:");
    for i in 0..15 {
        let result = fibonacci(i);
        println!("  fib({}) = {}", i, result);
    }

    println!("\nRecursive Fibonacci:");
    for i in 0..15 {
        let result = fibonacci_recursive(i);
        println!("  fib({}) = {}", i, result);
    }

    // Verify both give same results
    println!("\nVerification:");
    let mut all_match = true;
    for i in 0..20 {
        let iter_result = fibonacci(i);
        let rec_result = fibonacci_recursive(i);
        if iter_result != rec_result {
            println!("  MISMATCH at n={}", i);
            all_match = false;
        }
    }
    if all_match {
        println!("  All results match! ✓");
    }
}
