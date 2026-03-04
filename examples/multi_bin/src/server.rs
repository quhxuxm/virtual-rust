use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 { &args[1] } else { "8080" };

    println!("=== Multi-Binary Project: Server ===");
    println!("Starting server on port {port}...");
    println!("Listening for connections...");
    println!("Server ready! (simulated)");
}
