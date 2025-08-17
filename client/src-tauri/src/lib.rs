use steamworks::{Client, TicketForWebApiResponse};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_steam_ticket() -> Result<(u64, String), String> {
    // This is a blocking operation, so we run it in a separate thread
    tokio::task::spawn_blocking(|| {
        let client = Client::init().map_err(|e| e.to_string())?;
        let steam_id = client.user().steam_id().raw();

        let (tx, rx) = std::sync::mpsc::channel();

        let _cb = client.register_callback(move |resp: TicketForWebApiResponse| {
            if resp.result.is_ok() {
                let ticket_hex = hex::encode(resp.ticket);
                tx.send(Some(ticket_hex)).unwrap();
            } else {
                tx.send(None).unwrap();
            }
        });

        client
            .user()
            .authentication_session_ticket_for_webapi("test_identity");

        // Wait for the ticket callback
        for _ in 0..100 {
            client.run_callbacks();
            if let Ok(Some(ticket)) = rx.try_recv() {
                return Ok((steam_id, ticket));
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        Err("Failed to get web api ticket".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create steam_appid.txt for Steamworks initialization
    std::fs::write("steam_appid.txt", "480").expect("Failed to write steam_appid.txt");

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_steam_ticket])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
