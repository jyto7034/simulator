mod compiler;
mod content;

#[cfg(feature = "desktop")]
#[tauri::command]
async fn compile_graph(input: serde_json::Value) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || compiler::compile(input))
        .await
        .map_err(|error| error.to_string())?
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn content_files(
    app: tauri::AppHandle,
    input: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use tauri::Manager;
    let directory = if cfg!(debug_assertions) {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("프로젝트 경로를 찾지 못했습니다.")?
            .join("content")
    } else {
        app.path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("content")
    };
    content::operation(&directory, &input)
}

#[cfg(feature = "desktop")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![content_files, compile_graph])
        .run(tauri::generate_context!())
        .expect("failed to run content assembler");
}
