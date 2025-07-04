// src/auth/db_operation.rs

use crate::auth_server::model::*;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

// =================================================================
// 1. 플레이어 계정 (Players) - 스팀 연동 버전
// =================================================================

/// 신규 플레이어를 생성하거나, 기존 플레이어의 로그인 정보를 업데이트합니다.
/// 스팀 인증 성공 후 호출되며, 플레이어 프로필이 없으면 함께 생성합니다.
pub async fn upsert_player_on_login(
    pool: &PgPool,
    steam_id: i64,
    username: &str,
) -> Result<Player> {
    // 트랜잭션 시작
    let mut tx = pool.begin().await?;

    // 플레이어 정보 INSERT 또는 UPDATE
    let player = sqlx::query_as!(
        Player,
        r#"
        INSERT INTO players (id, last_known_username, last_login_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (id) DO UPDATE
        SET last_known_username = EXCLUDED.last_known_username,
            last_login_at = NOW(),
            updated_at = NOW()
        RETURNING id, last_known_username, status AS "status: _", created_at, updated_at, last_login_at
        "#,
        steam_id,
        username,
    )
    .fetch_one(&mut *tx) // 트랜잭션 내에서 실행
    .await?;

    // 플레이어 프로필이 없는 경우에만 생성 (기본 티어 ID: 1)
    sqlx::query!(
        "INSERT INTO player_profiles (player_id, tier_id) VALUES ($1, 1) ON CONFLICT (player_id) DO NOTHING",
        steam_id
    )
    .execute(&mut *tx)
    .await?;

    // 트랜잭션 커밋
    tx.commit().await?;

    Ok(player)
}

pub async fn get_player_by_id(pool: &PgPool, player_id: i64) -> Result<Option<Player>> {
    sqlx::query_as!(
        Player,
        r#"SELECT id, last_known_username, status AS "status: _", created_at, updated_at, last_login_at FROM players WHERE id = $1"#,
        player_id
    )
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

pub async fn update_player_status(
    pool: &PgPool,
    player_id: i64,
    new_status: PlayerStatus,
) -> Result<Player> {
    sqlx::query_as!(
        Player,
        r#"
        UPDATE players SET status = $1 WHERE id = $2
        RETURNING id, last_known_username, status AS "status: _", created_at, updated_at, last_login_at
        "#,
        new_status as _,
        player_id
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

/// 테스트용: 특정 플레이어 계정과 관련 데이터를 모두 삭제합니다.
pub async fn delete_player_by_id(pool: &PgPool, player_id: i64) -> Result<()> {
    let mut tx = pool.begin().await?;

    // 1. match_participants 테이블에서 해당 플레이어의 기록 삭제
    sqlx::query!(
        "DELETE FROM match_participants WHERE player_id = $1",
        player_id
    )
    .execute(&mut *tx)
    .await?;

    // 2. players 테이블에서 플레이어 삭제 (ON DELETE CASCADE에 의해 다른 데이터도 삭제됨)
    let result = sqlx::query!("DELETE FROM players WHERE id = $1", player_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    if result.rows_affected() == 0 {
        return Err(anyhow::anyhow!("Player with ID {} not found", player_id));
    }

    Ok(())
}

// =================================================================
// 2. 플레이어 프로필 (Player Profiles)
// =================================================================

pub async fn get_player_profile(pool: &PgPool, player_id: i64) -> Result<Option<PlayerProfile>> {
    sqlx::query_as!(
        PlayerProfile,
        "SELECT * FROM player_profiles WHERE player_id = $1",
        player_id
    )
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

pub async fn update_player_mmr(
    pool: &PgPool,
    player_id: i64,
    new_mmr: f64,
    new_rd: f64,
    new_volatility: f64,
) -> Result<PlayerProfile> {
    sqlx::query_as!(
        PlayerProfile,
        r#"
        UPDATE player_profiles
        SET mmr = $1, rd = $2, volatility = $3, last_rating_update_at = NOW()
        WHERE player_id = $4
        RETURNING *
        "#,
        new_mmr,
        new_rd,
        new_volatility,
        player_id
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

// =================================================================
// 3. 티어 정보 (Tiers) - 변경 없음
// =================================================================
pub async fn get_all_tiers(pool: &PgPool) -> Result<Vec<Tier>> {
    sqlx::query_as!(Tier, "SELECT * FROM tiers ORDER BY order_key")
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

// =================================================================
// 5 & 6. 게임 기록 (Match History & Participants)
// =================================================================
pub struct MatchResult<'a> {
    pub player_id: i64, // Uuid -> i64
    pub team_id: i32,
    pub is_winner: bool,
    pub initial_mmr: f64,
    pub final_mmr: f64,
    pub score: Option<i32>,
    pub stats: Option<&'a serde_json::Value>,
}

pub async fn record_match_result(
    pool: &PgPool,
    game_mode_id: i32,
    winning_team_id: Option<i32>,
    participants: &[MatchResult<'_>],
) -> Result<MatchHistory> {
    let mut tx = pool.begin().await?;

    let now = chrono::Utc::now();
    let match_history = sqlx::query_as!(
        MatchHistory,
        r#"
        INSERT INTO match_history (game_mode_id, started_at, ended_at, winning_team_id)
        VALUES ($1, $2, $3, $4) RETURNING *
        "#,
        game_mode_id,
        now,
        now,
        winning_team_id
    )
    .fetch_one(&mut *tx)
    .await?;

    for p in participants {
        sqlx::query!(
            r#"
            INSERT INTO match_participants (match_id, player_id, team_id, is_winner, initial_mmr, final_mmr, score, stats)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            match_history.id,
            p.player_id,
            p.team_id,
            p.is_winner,
            p.initial_mmr,
            p.final_mmr,
            p.score,
            p.stats
        )
        .execute(&mut *tx)
        .await?;

        // MMR 업데이트
        sqlx::query!(
            "UPDATE player_profiles SET mmr = $1 WHERE player_id = $2",
            p.final_mmr,
            p.player_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(match_history)
}

pub async fn get_player_match_history(
    pool: &PgPool,
    player_id: i64, // Uuid -> i64
    limit: i64,
) -> Result<Vec<MatchHistory>> {
    sqlx::query_as!(
        MatchHistory,
        r#"
        SELECT mh.*
        FROM match_history mh
        JOIN match_participants mp ON mh.id = mp.match_id
        WHERE mp.player_id = $1
        ORDER BY mh.started_at DESC
        LIMIT $2
        "#,
        player_id,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

// =================================================================
// 9. 플레이어 카드 컬렉션 (Player Card Collection)
// =================================================================

pub async fn add_card_to_collection(
    pool: &PgPool,
    player_id: i64, // Uuid -> i64
    card_id: i32,
    quantity: i32,
) -> Result<PlayerCardCollection> {
    sqlx::query_as!(
        PlayerCardCollection,
        r#"
        INSERT INTO player_card_collection (player_id, card_id, quantity)
        VALUES ($1, $2, $3)
        ON CONFLICT (player_id, card_id)
        DO UPDATE SET quantity = player_card_collection.quantity + EXCLUDED.quantity
        RETURNING *
        "#,
        player_id,
        card_id,
        quantity
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn get_player_card_collection(
    pool: &PgPool,
    player_id: i64,
) -> Result<Vec<PlayerCardCollection>> {
    sqlx::query_as!(
        PlayerCardCollection,
        "SELECT * FROM player_card_collection WHERE player_id = $1",
        player_id
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

// =================================================================
// 10 & 11. 덱 및 덱 카드 (Player Decks & Deck Cards)
// =================================================================

pub async fn create_deck(pool: &PgPool, player_id: i64, deck_name: &str) -> Result<PlayerDeck> {
    sqlx::query_as!(
        PlayerDeck,
        "INSERT INTO player_decks (player_id, deck_name) VALUES ($1, $2) RETURNING *",
        player_id,
        deck_name
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn add_card_to_deck(
    pool: &PgPool,
    deck_id: Uuid,
    card_id: i32,
    quantity: i32,
) -> Result<DeckCard> {
    sqlx::query_as!(
        DeckCard,
        r#"
        INSERT INTO deck_cards (deck_id, card_id, quantity)
        VALUES ($1, $2, $3)
        ON CONFLICT (deck_id, card_id) DO UPDATE SET quantity = EXCLUDED.quantity
        RETURNING *
        "#,
        deck_id,
        card_id,
        quantity
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

// 덱 정보와 카드 목록을 함께 가져오는 함수
#[derive(Debug)]
pub struct DeckWithCards {
    pub deck_info: PlayerDeck,
    pub cards: Vec<DeckCard>,
}

pub async fn get_deck_with_cards(pool: &PgPool, deck_id: Uuid) -> Result<Option<DeckWithCards>> {
    let deck_info = match sqlx::query_as!(
        PlayerDeck,
        "SELECT * FROM player_decks WHERE id = $1",
        deck_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(deck) => deck,
        None => return Ok(None),
    };

    let cards = sqlx::query_as!(
        DeckCard,
        "SELECT * FROM deck_cards WHERE deck_id = $1",
        deck_id
    )
    .fetch_all(pool)
    .await?;

    Ok(Some(DeckWithCards { deck_info, cards }))
}

pub async fn get_player_decks(pool: &PgPool, player_id: i64) -> Result<Vec<PlayerDeck>> {
    sqlx::query_as!(
        PlayerDeck,
        "SELECT * FROM player_decks WHERE player_id = $1",
        player_id
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
