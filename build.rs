use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Get the current time as seconds since the Unix epoch
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Pass the current time as an environment variable to the Rust code
    println!("cargo:rustc-env=BUILD_TIME={}", current_time);
}
