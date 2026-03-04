mod math;
mod greeting;

fn main() {
    // Use the greeting module
    let msg = greeting::hello("Virtual Rust");
    println!("{msg}");

    // Use the math module
    let nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Numbers: {:?}", nums);
    println!("Sum:     {}", math::sum(&nums));
    println!("Mean:    {:.1}", math::mean(&nums));
    println!("Max:     {}", math::max(&nums).unwrap());
    println!("Min:     {}", math::min(&nums).unwrap());

    // Fibonacci from math module
    println!("\nFibonacci sequence:");
    for i in 0..10 {
        print!("{}", math::fibonacci(i));
        if i < 9 { print!(", "); }
    }
    println!();

    // Farewell
    println!("\n{}", greeting::farewell("World"));
}
