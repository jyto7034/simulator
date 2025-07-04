// src/auth/model.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

// =================================================================
// ENUM 타입 정의 (Type-safe Enums)
// =================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "player_status", rename_all = "snake_case")]
pub enum PlayerStatus {
    Active,
    Suspended,
    Banned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "card_rarity", rename_all = "snake_case")]
pub enum CardRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

// =================================================================
// 테이블 매핑 구조체 (Table Mapping Structs)
// =================================================================

// 1. 플레이어 계정 (Steam Player Account)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Player {
    pub id: i64, // [수정됨] SteamID64 (BIGINT)
    pub last_known_username: Option<String>,
    pub status: PlayerStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

// 2. 플레이어 프로필 (Game-specific Profile)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub player_id: i64, // [수정됨] SteamID64 (BIGINT), 외래 키
    pub mmr: f64,
    pub rd: f64,
    pub volatility: f64,
    pub last_rating_update_at: DateTime<Utc>,
    pub tier_id: i32,
    pub rank_points: i32,
    pub experience_points: i64,
    pub level: i32,
    pub updated_at: DateTime<Utc>,
}

// 3. 티어 정보 (Master Data - 변경 없음)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Tier {
    pub id: i32,
    pub name: String,
    pub order_key: i32,
    pub icon_url: Option<String>,
}

// 4. 게임 모드 (Master Data - 변경 없음)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct GameMode {
    pub id: i32,
    pub internal_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_ranked: bool,
    pub player_count_per_team: i32,
    pub team_count: i32,
    pub is_active: bool,
}

// 5. 게임 기록 (Match History)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct MatchHistory {
    pub id: Uuid,
    pub game_mode_id: i32,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: Option<i32>, // DB에서는 GENERATED 이지만, Rust에서는 읽기 전용이므로 Option<i32>
    pub winning_team_id: Option<i32>,
    pub additional_data: Option<serde_json::Value>,
}

// 6. 매치 참여자 (Match Participants)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct MatchParticipant {
    pub match_id: Uuid,
    pub player_id: i64, // [수정됨] SteamID64 (BIGINT), 외래 키
    pub team_id: i32,
    pub is_winner: bool,
    pub initial_mmr: f64,
    pub final_mmr: f64,
    pub mmr_change: f64, // DB에서는 GENERATED 이지만, Rust에서는 읽기 전용
    pub score: Option<i32>,
    pub stats: Option<serde_json::Value>,
    pub disconnected: bool,
}

// 7. 친구 관계 (Friendships) - [제거됨]

// 8. 카드 마스터 (Card Master Data - 변경 없음)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Card {
    pub id: i32,
    pub internal_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub rarity: Option<CardRarity>,
    pub mana_cost: i32,
    pub attack: Option<i32>,
    pub health: Option<i32>,
    pub card_type: Option<String>,
    pub image_url: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub is_collectible: bool,
}

// 9. 플레이어 소유 카드 (Player Card Collection)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PlayerCardCollection {
    pub player_id: i64, // [수정됨] SteamID64 (BIGINT), 외래 키
    pub card_id: i32,
    pub quantity: i32,
    pub is_new: bool,
}

// 10. 플레이어 덱 (Player Decks)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PlayerDeck {
    pub id: Uuid,
    pub player_id: i64, // [수정됨] SteamID64 (BIGINT), 외래 키
    pub deck_name: String,
    pub cover_card_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 11. 덱 구성 카드 (Deck Card Entries - 변경 없음)
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DeckCard {
    pub deck_id: Uuid,
    pub card_id: i32,
    pub quantity: i32,
}
