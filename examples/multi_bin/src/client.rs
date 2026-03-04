use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    let host = if args.len() > 1 { &args[1] } else { "localhost" };
    let port = if args.len() > 2 { &args[2] } else { "8080" };
    println!("=== Multi-Binary Project: Client ===");
    println!("Connecting to {host}:{port}...");
    println!("Sending request...");
    println!("Response received: 200 OK (simulated)");
}
