use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventStreamMessage {
    pub event_type: String,
    pub player_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

use std::fmt;

pub struct ExpectedEvent {
    pub event_type: String,
    pub player_id: Option<Uuid>,
    pub data_matcher: Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>,
    pub timeout: Duration,
}

impl fmt::Debug for ExpectedEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExpectedEvent")
            .field("event_type", &self.event_type)
            .field("player_id", &self.player_id)
            .field("data_matcher", &"<function>")
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Clone for ExpectedEvent {
    fn clone(&self) -> Self {
        Self {
            event_type: self.event_type.clone(),
            player_id: self.player_id,
            data_matcher: Box::new(|_| true), // Default matcher for cloning
            timeout: self.timeout,
        }
    }
}

impl ExpectedEvent {
    pub fn new(
        event_type: String,
        player_id: Option<Uuid>,
        matcher: Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>,
        timeout: Duration,
    ) -> Self {
        Self {
            event_type,
            player_id,
            data_matcher: matcher,
            timeout,
        }
    }

    pub fn simple(event_type: String, player_id: Option<Uuid>) -> Self {
        Self::new(
            event_type,
            player_id,
            Box::new(|_| true),
            Duration::from_secs(10),
        )
    }

    pub fn matches(&self, event: &EventStreamMessage) -> bool {
        // Check event type
        if self.event_type != event.event_type {
            return false;
        }

        // Check player ID
        if let Some(expected_player_id) = self.player_id {
            if event.player_id != Some(expected_player_id) {
                return false;
            }
        }

        // Check data matcher
        (self.data_matcher)(&event.data)
    }
}

#[derive(Debug)]
pub struct EventObserver {
    pub match_server_url: String,
    pub expected_sequence: Vec<ExpectedEvent>,
    pub received_events: Vec<EventStreamMessage>,
    pub current_step: usize,
    pub test_name: String,
}

impl Clone for EventObserver {
    fn clone(&self) -> Self {
        Self {
            match_server_url: self.match_server_url.clone(),
            expected_sequence: self.expected_sequence.clone(),
            received_events: self.received_events.clone(),
            current_step: self.current_step,
            test_name: self.test_name.clone(),
        }
    }
}

impl EventObserver {
    pub fn new(match_server_url: String, test_name: String) -> Self {
        Self {
            match_server_url,
            expected_sequence: Vec::new(),
            received_events: Vec::new(),
            current_step: 0,
            test_name,
        }
    }

    pub fn expect_event(&mut self, event: ExpectedEvent) {
        self.expected_sequence.push(event);
    }

    pub fn expect_queued(&mut self, player_id: Uuid) {
        self.expect_event(ExpectedEvent::new(
            "server_message".to_string(),
            Some(player_id),
            Box::new(|data| data.get("Queued").is_some()),
            Duration::from_secs(5),
        ));
    }

    pub fn expect_start_loading(&mut self, player_id: Uuid) {
        self.expect_event(ExpectedEvent::new(
            "server_message".to_string(),
            Some(player_id),
            Box::new(|data| data.get("StartLoading").is_some()),
            Duration::from_secs(10),
        ));
    }

    pub fn expect_match_found(&mut self, player_id: Uuid) {
        self.expect_event(ExpectedEvent::new(
            "server_message".to_string(),
            Some(player_id),
            Box::new(|data| data.get("MatchFound").is_some()),
            Duration::from_secs(10),
        ));
    }

    pub async fn start_observation(
        &mut self,
        player_id: Option<Uuid>,
    ) -> Result<ObservationResult> {
        info!("Starting event observation for test: {}", self.test_name);

        // Build WebSocket URL with query parameters
        // match_server_url이 이미 "ws://127.0.0.1:8080" 형태
        let final_url = if let Some(pid) = player_id {
            format!("{}/events/stream?player_id={}", self.match_server_url, pid)
        } else {
            format!("{}/events/stream", self.match_server_url)
        };

        info!("Connecting to WebSocket: {}", final_url);

        // Connect to event stream
        let (ws_stream, _) = connect_async(&final_url).await?;
        let (mut _write, mut read) = ws_stream.split();

        let start_time = Instant::now();
        let mut _last_event_time = start_time;

        while self.current_step < self.expected_sequence.len() {
            let current_expected = &self.expected_sequence[self.current_step];
            let step_start = Instant::now();

            // Wait for the expected event with timeout
            loop {
                if step_start.elapsed() > current_expected.timeout {
                    return Ok(ObservationResult::timeout(
                        self.current_step,
                        format!(
                            "Timeout waiting for event: {:?}",
                            current_expected.event_type
                        ),
                        self.received_events.clone(),
                    ));
                }

                // Try to receive a message
                match tokio::time::timeout(Duration::from_millis(100), read.next()).await {
                    Ok(Some(Ok(msg))) => {
                        if let Message::Text(text) = msg {
                            match serde_json::from_str::<EventStreamMessage>(&text) {
                                Ok(event) => {
                                    info!("Received event: {:?}", event);
                                    self.received_events.push(event.clone());
                                    _last_event_time = Instant::now();

                                    if current_expected.matches(&event) {
                                        info!(
                                            "✓ Step {} matched: {}",
                                            self.current_step, event.event_type
                                        );
                                        self.current_step += 1;
                                        break;
                                    } else {
                                        warn!("Event doesn't match expected: {:?}", event);
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to parse event: {}", e);
                                }
                            }
                        }
                    }
                    Ok(Some(Err(e))) => {
                        return Ok(ObservationResult::error(
                            self.current_step,
                            format!("WebSocket error: {}", e),
                            self.received_events.clone(),
                        ));
                    }
                    Ok(None) => {
                        return Ok(ObservationResult::error(
                            self.current_step,
                            "WebSocket connection closed".to_string(),
                            self.received_events.clone(),
                        ));
                    }
                    Err(_) => {
                        // Timeout, continue the loop
                        continue;
                    }
                }
            }
        }

        Ok(ObservationResult::success(
            self.received_events.clone(),
            start_time.elapsed(),
        ))
    }
}

#[derive(Debug)]
pub enum ObservationResult {
    Success {
        events: Vec<EventStreamMessage>,
        duration: Duration,
    },
    Timeout {
        failed_step: usize,
        reason: String,
        events: Vec<EventStreamMessage>,
    },
    Error {
        failed_step: usize,
        reason: String,
        events: Vec<EventStreamMessage>,
    },
}

impl ObservationResult {
    pub fn success(events: Vec<EventStreamMessage>, duration: Duration) -> Self {
        Self::Success { events, duration }
    }

    pub fn timeout(failed_step: usize, reason: String, events: Vec<EventStreamMessage>) -> Self {
        Self::Timeout {
            failed_step,
            reason,
            events,
        }
    }

    pub fn error(failed_step: usize, reason: String, events: Vec<EventStreamMessage>) -> Self {
        Self::Error {
            failed_step,
            reason,
            events,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn get_summary(&self) -> String {
        match self {
            Self::Success { events, duration } => {
                format!("✓ Test passed - {} events in {:?}", events.len(), duration)
            }
            Self::Timeout {
                failed_step,
                reason,
                events,
            } => {
                format!(
                    "✗ Test failed at step {}: {} ({} events received)",
                    failed_step,
                    reason,
                    events.len()
                )
            }
            Self::Error {
                failed_step,
                reason,
                events,
            } => {
                format!(
                    "✗ Test error at step {}: {} ({} events received)",
                    failed_step,
                    reason,
                    events.len()
                )
            }
        }
    }
}
