//! A library consumer deliberately does not install a process global allocator.
#[cfg(feature = "runtime-measurement")]
fn main() {
    println!(
        "{}",
        serde_json::to_string(&game_server::runtime_measurement::process_heap())
            .expect("finite measurement")
    );
}
#[cfg(not(feature = "runtime-measurement"))]
fn main() {
    println!("runtime-measurement is disabled");
}
