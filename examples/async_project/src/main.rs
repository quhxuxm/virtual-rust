mod api;
mod task;

#[tokio::main]
async fn main() {
    println!("=== Async Cargo Project — Powered by Virtual Rust ===\n");

    // 1. Run concurrent tasks with tokio
    println!("--- Concurrent Tasks (tokio::join!) ---");
    let (a, b, c) = tokio::join!(
        task::do_work("Download", 150),
        task::do_work("Parse", 100),
        task::do_work("Index", 200),
    );
    println!("  {a}");
    println!("  {b}");
    println!("  {c}");

    // 2. Spawn background tasks
    println!("\n--- Spawned Tasks (tokio::spawn) ---");
    let handles: Vec<_> = (1..=5)
        .map(|i| {
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                i * i
            })
        })
        .collect();

    let mut results = Vec::new();
    for h in handles {
        results.push(h.await.unwrap());
    }
    println!("  Squares: {:?}", results);

    // 3. Async channels
    println!("\n--- Async Channels (mpsc) ---");
    task::channel_demo().await;

    // 4. HTTP request
    println!("\n--- HTTP Request (reqwest) ---");
    api::fetch_ip().await;

    // 5. JSON serialization
    println!("\n--- JSON with serde ---");
    api::json_demo();

    println!("\n=== All async operations completed! ===");
}
