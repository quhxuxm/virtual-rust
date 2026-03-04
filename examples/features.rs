// Comprehensive feature demo for Virtual Rust

fn main() {
    // === Variables and Types ===
    println!("=== Variables and Types ===");
    let x: i32 = 42;
    let y: f64 = 3.14;
    let name = "Virtual Rust";
    let active = true;
    let ch = 'R';
    println!(
        "  int: {}, float: {}, string: {}, bool: {}, char: {}",
        x, y, name, active, ch
    );

    // Mutable variables
    let mut counter = 0;
    counter += 10;
    counter *= 2;
    println!("  counter = {}", counter);

    // === Type Casting ===
    println!("\n=== Type Casting ===");
    let a = 65;
    let c = a as char;
    println!("  {} as char = '{}'", a, c);
    let f = 3.99;
    let i = f as i32;
    println!("  {} as i32 = {}", f, i);

    // === Tuples ===
    println!("\n=== Tuples ===");
    let point = (10, 20);
    println!("  Point: {:?}", point);
    let px = 10;
    let py = 20;
    println!("  x={}, y={}", px, py);

    // === Arrays ===
    println!("\n=== Arrays ===");
    let arr = [1, 2, 3, 4, 5];
    println!("  Array: {:?}", arr);
    println!("  Length: {}", arr.len());
    println!("  First: {}", arr[0]);
    println!("  Last: {}", arr[4]);

    // Array repeat
    let zeros = [0; 5];
    println!("  Zeros: {:?}", zeros);

    // === Vec ===
    println!("\n=== Vec ===");
    let v = vec![10, 20, 30, 40, 50];
    println!("  Vec: {:?}", v);
    println!("  Length: {}", v.len());

    // === Control Flow ===
    println!("\n=== Control Flow ===");

    // If/else as expression
    let status = if x > 40 { "big" } else { "small" };
    println!("  {} is {}", x, status);

    // While loop
    let mut n = 1;
    let mut sum = 0;
    while n <= 100 {
        sum += n;
        n += 1;
    }
    println!("  Sum 1..100 = {}", sum);

    // For loop with range
    let mut product = 1;
    for i in 1..=5 {
        product *= i;
    }
    println!("  5! = {}", product);

    // Loop with break
    let mut count = 0;
    let result = loop {
        count += 1;
        if count == 10 {
            break count * count;
        }
    };
    println!("  Loop result: {}", result);

    // === Match ===
    println!("\n=== Match ===");
    let number = 13;
    let description = match number {
        1 => "one",
        2 => "two",
        13 => "thirteen (lucky!)",
        _ => "something else",
    };
    println!("  {} is {}", number, description);

    // Match with ranges
    let score = 85;
    let grade = match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    };
    println!("  Score {} = Grade {}", score, grade);

    // === Functions ===
    println!("\n=== Functions ===");
    println!("  add(3, 4) = {}", add(3, 4));
    println!("  multiply(6, 7) = {}", multiply(6, 7));
    println!("  power(2, 10) = {}", power(2, 10));

    // === Closures ===
    println!("\n=== Closures ===");
    let double = |x: i64| x * 2;
    let add_n = |a: i64, b: i64| a + b;
    println!("  double(21) = {}", double(21));
    println!("  add_n(30, 12) = {}", add_n(30, 12));

    // Higher-order functions
    let nums = vec![1, 2, 3, 4, 5];
    let doubled = nums.iter().map(|x| x * 2).collect();
    println!("  {:?} doubled = {:?}", nums, doubled);

    let big = nums.iter().filter(|x| x > 3).collect();
    println!("  {:?} filter(>3) = {:?}", nums, big);

    let total = nums.iter().fold(0, |acc, x| acc + x);
    println!("  {:?} fold(+) = {}", nums, total);

    // === String Operations ===
    println!("\n=== String Methods ===");
    let s = "Hello, World!";
    println!("  \"{}\"", s);
    println!("  len() = {}", s.len());
    println!("  to_uppercase() = {}", s.to_uppercase());
    println!("  to_lowercase() = {}", s.to_lowercase());
    println!("  contains(\"World\") = {}", s.contains("World"));
    println!(
        "  replace(\"World\", \"Rust\") = {}",
        s.replace("World", "Rust")
    );
    println!("  trim() = \"{}\"", "  spaced  ".trim());

    // === Assertions ===
    println!("\n=== Assertions ===");
    assert_eq!(2 + 2, 4);
    assert_eq!(fibonacci(10), 55);
    assert!(true);
    println!("  All assertions passed! ✓");

    println!("\n=== Demo Complete ===");
}

fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn multiply(a: i64, b: i64) -> i64 {
    a * b
}

fn power(base: i64, exp: i64) -> i64 {
    if exp == 0 {
        return 1;
    }
    let mut result = 1;
    let mut i = 0;
    while i < exp {
        result *= base;
        i += 1;
    }
    result
}

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
