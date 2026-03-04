//! [dependencies]
//! tokio = { version = "1", features = ["full"] }
//! reqwest = { version = "0.12", features = ["json"] }

use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("=== Virtual Rust with Async Runtime (Tokio) ===\n");

    // Spawn concurrent tasks
    let start = Instant::now();

    let (r1, r2, r3) = tokio::join!(
        async_greet("Alice", 100),
        async_greet("Bob", 200),
        async_greet("Charlie", 150),
    );

    println!("{r1}");
    println!("{r2}");
    println!("{r3}");
    println!(
        "\nAll greetings completed in {:.0?} (ran concurrently!)",
        start.elapsed()
    );

    // Demonstrate tokio::spawn
    println!("\n--- Spawned Tasks ---");
    let mut handles = vec![];
    for i in 1..=5 {
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            i * i
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    println!("Squares (via tokio::spawn): {:?}", results);

    // Demonstrate async HTTP request with reqwest
    println!("\n--- HTTP Request (reqwest) ---");
    match reqwest::get("https://httpbin.org/ip").await {
        Ok(resp) => {
            let status = resp.status();
            println!("GET https://httpbin.org/ip -> {status}");
            if let Ok(body) = resp.text().await {
                println!("Response: {}", body.trim());
            }
        }
        Err(e) => {
            println!("Request failed (possibly no network): {e}");
        }
    }

    // Demonstrate channels
    println!("\n--- Async Channels ---");
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(10);

    let producer = tokio::spawn(async move {
        for msg in ["hello", "from", "tokio", "channel"] {
            tx.send(msg.to_string()).await.unwrap();
        }
    });

    let consumer = tokio::spawn(async move {
        let mut messages = vec![];
        while let Some(msg) = rx.recv().await {
            messages.push(msg);
        }
        messages
    });

    producer.await.unwrap();
    let messages = consumer.await.unwrap();
    println!("Received: {:?}", messages);

    println!("\n=== Done! ===");
}

async fn async_greet(name: &str, delay_ms: u64) -> String {
    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    format!("Hello, {name}! (after {delay_ms}ms)")
}
