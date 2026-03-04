/// Returns a greeting message for the given name.
pub fn hello(name: &str) -> String {
    format!("Hello, {name}! Welcome to the loose modules example.")
}

/// Returns a farewell message for the given name.
pub fn farewell(name: &str) -> String {
    format!("Goodbye, {name}! Thanks for trying loose modules.")
}
