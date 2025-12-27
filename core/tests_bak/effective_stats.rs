use std::collections::HashMap;
use std::sync::Arc;

use game_core::game::ability::AbilityId;
use game_core::game::battle::{GrowthId, GrowthStack, OwnedUnit};
use game_core::game::data::GameDataBase;
use game_core::game::enums::Tier;

mod common;

use common::load_game_data_from_ron;

/// Scorched Girl 를 기반으로 성장, 장비, 아티팩트를 모두 얹었을 때
/// UnitStats 가 기대한 값으로 계산되는지 검증한다.
///
/// 계산식:
/// - Base (Scorched Girl): HP=150, ATK=40, DEF=5, ATK_INT=1500
/// - Growth (KillStack=10): ATK += 10 → 50
/// - Equipment
///   - justitia: ATK += 40, ATK_INT += (-200) → ATK=90, ATK_INT=1300
///   - standard_suit: HP += 80, DEF += 5 → HP=230, DEF=10
/// - Artifacts
///   - one_sin: DEF += 10 → DEF=20
///   - beauty_and_beast:
///       ATK += 20% of 90  → +18 → 108
///       HP  += 15% of 230 → +34 → 264
#[test]
fn effective_stats_applies_growth_equipment_and_artifacts() {
    let game_data: Arc<GameDataBase> = load_game_data_from_ron();

    // Given: Scorched Girl 메타데이터
    let abnormality = game_data
        .abnormality_data
        .get_by_id("f-01-02")
        .expect("Scorched Girl abnormality must exist");

    // Given: 장비는 justitia + standard_suit
    let justitia = game_data
        .equipment_data
        .get_by_id("justitia")
        .expect("justitia equipment must exist");
    let standard_suit = game_data
        .equipment_data
        .get_by_id("standard_suit")
        .expect("standard_suit equipment must exist");

    // Given: 아티팩트는 one_sin + beauty_and_beast
    let one_sin = game_data
        .artifact_data
        .get_by_id("one_sin")
        .expect("one_sin artifact must exist");
    let beauty_and_beast = game_data
        .artifact_data
        .get_by_id("beauty_and_beast")
        .expect("beauty_and_beast artifact must exist");

    // Given: Growth는 KillStack 10
    let mut growth_map = HashMap::new();
    growth_map.insert(GrowthId::KillStack, 10);

    let unit = OwnedUnit {
        base_uuid: abnormality.uuid,
        level: Tier::I,
        growth_stacks: GrowthStack { stacks: growth_map },
        equipped_items: vec![justitia.uuid, standard_suit.uuid],
    };

    let artifact_uuids = vec![one_sin.uuid, beauty_and_beast.uuid];

    // When: 스탯 계산 수행
    let stats = unit
        .effective_stats(&game_data, &artifact_uuids)
        .expect("effective_stats should succeed");

    // Then: 최종 스탯이 기대값과 일치해야 함
    assert_eq!(
        stats.max_health, 264,
        "max_health should include flat + percent bonuses"
    );
    assert_eq!(
        stats.attack, 108,
        "attack should include growth, equipment and percent artifact"
    );
    assert_eq!(stats.defense, 20, "defense should include suit + artifact");
    assert_eq!(
        stats.attack_interval_ms, 1300,
        "attack_interval_ms should include weapon and artifact modifiers"
    );
    // Then: 영구 성장/효과가 적용된 최종 max_health 기준으로 전투 시작은 풀피여야 함
    assert_eq!(
        stats.current_health, stats.max_health,
        "battle should start at full HP after permanent growth/effects"
    );
}

/// AbnormalityMetadata 가 RON 의 abilities 필드를 통해
/// AbilityId 목록을 정상적으로 역직렬화하는지 검증한다.
#[test]
fn abnormality_metadata_deserializes_abilities_from_ron() {
    let game_data: Arc<GameDataBase> = load_game_data_from_ron();

    // Given: Scorched Girl
    let scorched = game_data
        .abnormality_data
        .get_by_id("f-01-02")
        .expect("Scorched Girl abnormality must exist");
    assert_eq!(
        scorched.abilities,
        vec![AbilityId::ScorchedExplosion],
        "Scorched Girl should have ScorchedExplosion ability"
    );

    // Given: Plague Doctor
    let plague = game_data
        .abnormality_data
        .get_by_id("o-02-56")
        .expect("Plague Doctor abnormality must exist");
    assert_eq!(
        plague.abilities,
        vec![AbilityId::PlagueMassHeal],
        "Plague Doctor should have PlagueMassHeal ability"
    );

    // Given: 랜덤 이벤트 전용 Unknown Distortion
    let random_abno = game_data
        .abnormality_data
        .get_by_id("random_event_abnormality_1")
        .expect("random_event_abnormality_1 must exist");
    assert_eq!(
        random_abno.abilities,
        vec![AbilityId::UnknownDistortionStrike],
        "Random event abnormality should have UnknownDistortionStrike ability"
    );
}
