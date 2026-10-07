//! Local finite measurement playback; no listening socket or save overwrite.
#[cfg(feature = "runtime-measurement")]
#[global_allocator]
static ALLOCATOR: game_server::runtime_measurement::CountingAllocator<'static, std::alloc::System> =
    game_server::runtime_measurement::process_allocator();

#[cfg(feature = "runtime-measurement")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use game_server::runtime_measurement::playback::{self, MANIFEST_BYTE_CAP};
    let arguments: Vec<_> = std::env::args_os().collect();
    if arguments.len() != 3 {
        return Err("usage: measure_runtime MANIFEST_JSON NEW_OUTPUT_JSONL".into());
    }
    let bytes = playback::read_bounded(std::path::Path::new(&arguments[1]), MANIFEST_BYTE_CAP)?;
    let summary = playback::execute(&bytes, std::path::Path::new(&arguments[2]))?;
    println!("{summary}");
    Ok(())
}
#[cfg(not(feature = "runtime-measurement"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("measure_runtime requires the runtime-measurement feature".into())
}
