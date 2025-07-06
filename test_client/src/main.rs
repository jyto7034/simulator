use futures_util::{StreamExt, SinkExt};
use serde_json::json;
use tokio_tungstenite::connect_async;
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player_id = Uuid::new_v4();
    println!("[Client {}] Starting...", player_id);

    // 1. Connect to the matchmaking server
    let url = Url::parse("ws://127.0.0.1:8080/ws/")?;
    let (mut ws_stream, _) = connect_async(url.as_str()).await.expect("Failed to connect");
    println!("[Client {}] Connected to matchmaking server.", player_id);

    // 2. Send an enqueue message
    let enqueue_msg = json!({
        "type": "enqueue",
        "player_id": player_id,
        "game_mode": "1v1_ranked"
    });
    ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(enqueue_msg.to_string())).await?;
    println!("[Client {}] Sent enqueue request.", player_id);

    // 3. Wait for messages from the server
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
            println!("[Client {}] Received: {}", player_id, text);
            
            // Exit after receiving the match found message
            if text.contains("match_found") {
                break;
            }
        }
    }

    println!("[Client {}] Test finished.", player_id);
    Ok(())
}