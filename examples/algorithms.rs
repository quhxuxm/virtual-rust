// Data structures and algorithms demo
struct Point {
    x: f64,
    y: f64,
}

fn distance(p1_x: f64, p1_y: f64, p2_x: f64, p2_y: f64) -> f64 {
    let dx = p2_x - p1_x;
    let dy = p2_y - p1_y;
    (dx * dx + dy * dy).sqrt()
}

fn is_prime(n: i64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

fn factorial(n: i64) -> i64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn main() {
    println!("=== Math & Algorithms Demo ===\n");

    // Distance calculation
    let d = distance(0.0, 0.0, 3.0, 4.0);
    println!("Distance from (0,0) to (3,4) = {}", d);

    // Prime numbers
    println!("\nPrime numbers up to 50:");
    let mut primes = vec![];
    for n in 2..50 {
        if is_prime(n) {
            primes = primes.push(n);
        }
    }
    println!("  {:?}", primes);

    // Factorials
    println!("\nFactorials:");
    for i in 0..11 {
        println!("  {}! = {}", i, factorial(i));
    }

    // GCD
    println!("\nGCD examples:");
    println!("  gcd(12, 8) = {}", gcd(12, 8));
    println!("  gcd(54, 24) = {}", gcd(54, 24));
    println!("  gcd(100, 75) = {}", gcd(100, 75));

    // FizzBuzz
    println!("\nFizzBuzz (1-30):");
    for i in 1..31 {
        if i % 15 == 0 {
            println!("  {} -> FizzBuzz", i);
        } else if i % 3 == 0 {
            println!("  {} -> Fizz", i);
        } else if i % 5 == 0 {
            println!("  {} -> Buzz", i);
        }
    }

    // Array operations with closures
    println!("\nArray operations:");
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("  Numbers: {:?}", numbers);

    let sum = numbers.iter().sum();
    println!("  Sum: {}", sum);

    let evens = numbers.iter().filter(|x| x % 2 == 0).collect();
    println!("  Evens: {:?}", evens);

    let squares = numbers.iter().map(|x| x * x).collect();
    println!("  Squares: {:?}", squares);

    // String operations
    println!("\nString operations:");
    let greeting = "Hello, Virtual Rust!";
    println!("  Original: {}", greeting);
    println!("  Uppercase: {}", greeting.to_uppercase());
    println!("  Length: {}", greeting.len());
    println!("  Contains 'Rust': {}", greeting.contains("Rust"));

    let words = greeting.split(" ");
    println!("  Words: {:?}", words);
}
