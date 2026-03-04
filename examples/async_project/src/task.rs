use std::time::{Duration, Instant};

/// Simulates an async task that takes `ms` milliseconds.
pub async fn do_work(name: &str, ms: u64) -> String {
    let start = Instant::now();
    tokio::time::sleep(Duration::from_millis(ms)).await;
    format!("{name}: done in {:.0?}", start.elapsed())
}

/// Demonstrates async mpsc channels.
pub async fn channel_demo() {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(10);

    let producer = tokio::spawn(async move {
        for word in ["async", "channels", "are", "awesome"] {
            tx.send(word.to_string()).await.unwrap();
        }
    });

    let consumer = tokio::spawn(async move {
        let mut messages = Vec::new();
        while let Some(msg) = rx.recv().await {
            messages.push(msg);
        }
        messages
    });

    producer.await.unwrap();
    let messages = consumer.await.unwrap();
    println!("  Received: {:?}", messages);
}
