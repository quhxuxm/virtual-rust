use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IpResponse {
    pub origin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub roles: Vec<String>,
}

/// Fetches the caller's IP address from httpbin.org.
pub async fn fetch_ip() {
    match reqwest::get("https://httpbin.org/ip").await {
        Ok(resp) => {
            let status = resp.status();
            print!("  GET https://httpbin.org/ip -> {status}");
            match resp.json::<IpResponse>().await {
                Ok(ip) => println!(" (origin: {})", ip.origin),
                Err(_) => println!(),
            }
        }
        Err(e) => {
            println!("  Request failed (no network?): {e}");
        }
    }
}

/// Demonstrates serde JSON serialization/deserialization.
pub fn json_demo() {
    let user = User {
        name: "Rustacean".to_string(),
        age: 28,
        roles: vec!["developer".to_string(), "contributor".to_string()],
    };

    let json = serde_json::to_string_pretty(&user).unwrap();
    println!("  Serialized:\n  {}", json.replace('\n', "\n  "));

    let back: User = serde_json::from_str(&json).unwrap();
    println!(
        "  Deserialized: {} (age {}), roles: {:?}",
        back.name, back.age, back.roles
    );
}
