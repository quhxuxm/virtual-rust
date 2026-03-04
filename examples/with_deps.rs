//! [dependencies]
//! rand = "0.8"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
    hobbies: Vec<String>,
}

fn main() {
    println!("=== Virtual Rust with Dependencies ===\n");

    // Using rand crate
    let mut rng = rand::thread_rng();
    let secret: u32 = rng.gen_range(1..=100);
    println!("Random number (1-100): {}", secret);

    let roll: f64 = rng.gen();
    println!("Random float (0-1):    {:.4}", roll);

    // Using serde + serde_json
    let person = Person {
        name: "Rustacean".to_string(),
        age: 28,
        hobbies: vec![
            "coding".to_string(),
            "open source".to_string(),
            "coffee".to_string(),
        ],
    };

    let json = serde_json::to_string_pretty(&person).unwrap();
    println!("\nSerialized to JSON:\n{}", json);

    let deserialized: Person = serde_json::from_str(&json).unwrap();
    println!("\nDeserialized back:");
    println!("  Name:    {}", deserialized.name);
    println!("  Age:     {}", deserialized.age);
    println!("  Hobbies: {:?}", deserialized.hobbies);

    // Generate random data
    println!("\n--- Random People ---");
    let names = ["Alice", "Bob", "Charlie", "Diana", "Eve"];
    for name in &names {
        let age: u32 = rng.gen_range(18..=65);
        let p = Person {
            name: name.to_string(),
            age,
            hobbies: vec!["coding".to_string()],
        };
        let j = serde_json::to_string(&p).unwrap();
        println!("  {}", j);
    }

    println!("\n=== Done! ===");
}
