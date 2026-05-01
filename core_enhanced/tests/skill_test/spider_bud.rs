use game_core::{
    ecs::resources::Position,
    game::{
        battle::{buffs::BuffId, timeline::TimelineEvent},
        enums::Side,
    },
};

use super::{
    buff_ids, buffs_applied_by, passive_dummy_patch, run_abnormality_scenario, scenario_from_board,
    skill_dummy_board_legend, step_ids, target_unit_ids, BoardEntry, PlacedUnitKind,
};

#[test]
// 목적:
// Spider Bud의 Poison Stack이 근처 적 여러 명 중 하나만 골라
// poison 버프를 단일 대상에게만 적용하는지 검증한다.
fn spider_bud_applies_poison_only_to_the_nearest_enemy() {
    let mut legend = skill_dummy_board_legend();
    legend
        .units
        .insert('E', BoardEntry::Opponent(PlacedUnitKind::SkillDummy));
    legend.patches.insert('!', passive_dummy_patch(2_000, None));

    let board = r#"
        . . . . . . .
        . C . E! . . .
        . D! . . . . .
    "#;

    let result = run_abnormality_scenario("o-01-45", scenario_from_board(board, &legend));
    let near_enemy = result
        .unit_instance_at(Position::new(1, 2), Some(Side::Opponent))
        .unwrap();

    let cast = result.first_cast_of("spider_bud_poison_stack");
    assert!(matches!(
        &cast.event,
        TimelineEvent::AbilityCast {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == near_enemy
    ));

    let steps = result.first_cast_steps("spider_bud_poison_stack");
    assert_eq!(step_ids(&steps), vec!["poison_apply"]);

    let buffs = buffs_applied_by(result.timeline(), steps[0].seq);
    assert_eq!(buffs.len(), 1);
    assert_eq!(target_unit_ids(&buffs), vec![near_enemy]);
    assert_eq!(buff_ids(&buffs), vec![BuffId::from_name("poison")]);
}

/*

• 좋습니다. 방향이 훨씬 선명해졌습니다. 제 판단은 이렇습니다.

  1. 상대 stage 표현
  TFT 증강창처럼 전체를 덮는 방식은 이번 게임엔 메인 패턴으로 쓰지 않는 게 맞습니다. 말씀대로 게임 공간
  과 선택 공간이 분리돼 보여서 Bazaar 쪽 몰입감이 약해집니다.

  제가 추천하는 건 상대 필드 위에 카드가 뜨되, 필드를 완전히 지우지 않는 방식입니다.

  구조는 이렇게 가면 됩니다.

  ┌──────────────────────── opponent stage ────────────────────────┐
  │ enemy board frame / encounter backdrop                         │
  │                                                                │
  │    [ choice card ] [ choice card ] [ choice card ]             │
  │                                                                │
  │ faint enemy field / stage ornaments remain visible             │
  └────────────────────────────────────────────────────────────────┘

  핵심 규칙:

  - 상대 필드 배경은 남긴다
  - 카드가 그 위에 “올라온다”
  - 배경은 20~35% 정도만 어둡게 한다
  - 전체 화면 암전은 하지 않는다
  - 플레이어 필드는 그대로 보이게 둔다

  즉 modal이 아니라 stage overlay입니다.

  더 구체적으로는:

  - 이벤트/보상은 상대 필드 중앙 60~70% 폭 안에 카드 3장을 배치
  - 카드 뒤에는 상대 필드 타일/장식/적 실루엣을 희미하게 유지
  - 선택된 카드는 살짝 확대
  - 비선택 카드는 살짝 눌린 느낌
  - 상세 설명은 카드 안에 최대한 넣고, 보조 설명만 하단 HUD에 둠

  상점은 더 Bazaar처럼 가야 하므로:

  - 상대 stage 상단 또는 중앙에 판매 품목 카드열
  - 필드는 뒤에 남아 있음
  - “상대 진영이 이번에 제공하는 상품”처럼 보여야 함

  즉:

  - 이벤트/보상 = 상대 stage 중앙 카드 오버레이
  - 상점 = 상대 stage에 붙은 상품 진열대 느낌
  - 전투/준비 = 실제 적 필드

  2. 7x8 필드
  좋습니다. 그러면 논리적으로는:

  - 전체 보드: 7 x 8
  - 상대 진영: 7 x 4
  - 아군 진영: 7 x 4

  이건 UI상 매우 좋습니다.
  왜냐하면 상대/아군을 완전히 같은 규칙으로 보여줄 수 있기 때문입니다.

  표현은 이런 식이 좋습니다.

    enemy side
    7 x 4

    [ ][ ][ ][ ][ ][ ][ ]
    [ ][ ][ ][ ][ ][ ][ ]
    [ ][ ][ ][ ][ ][ ][ ]
    [ ][ ][ ][ ][ ][ ][ ]

    player side
    7 x 4

    [ ][ ][ ][ ][ ][ ][ ]
    [ ][ ][ ][ ][ ][ ][ ]
    [ ][ ][ ][ ][ ][ ][ ]
    [ ][ ][ ][ ][ ][ ][ ]

  실제 목업에서는 모든 칸을 강하게 그리지 말고:

  - 타일선은 얇게
  - 선택/점유/하이라이트 때만 강하게
  - 평소엔 보드 판 자체가 먼저 보이게

  3. 하단 HUD 줄 수
  지금 요구사항이면 2줄 구조가 가장 안전합니다.

  추천 구조:

  하단 HUD 1줄:
  [ 미니 전설이 ] [ 골드 / 자원 ] [ 현재 선택 정보 ] [ 주요 액션 버튼 ]

  하단 HUD 2줄:
  [ 유닛 인벤토리 / 아이템 인벤토리 / 아티팩트 슬롯 ]

  또는 좌우 분할로 보면:

  ┌───────────────────────────────────────────────────────────────┐
  │ mini legend | gold | enkephalin | selected target | actions │
  ├───────────────────────────────────────────────────────────────┤
  │ units inventory | item inventory | artifact slots           │
  └───────────────────────────────────────────────────────────────┘

  제 권장은:

  - 1줄은 너무 빡빡함
  - 3줄은 필드를 너무 먹음
  - 그래서 2줄이 가장 적절

  비율은 대략:

  - 상단 정보줄: 35~40%
  - 하단 소유물줄: 60~65%

  4. 벤치
  벤치는 TFT 용어로, 필드에 배치되지 않은 유닛이 대기하는 줄입니다.

  즉:

  - 보드 위 = 현재 전투/배치 중인 유닛
  - 벤치 = 아직 보드에 안 올린 보유 유닛

  당신 게임에서는 굳이 벤치라는 말을 안 써도 됩니다.
  더 맞는 표현은 아마:

  - 대기 유닛
  - 보유 유닛
  - 편성 대기열

  일 겁니다.

  지금 문맥에선 “하단 HUD에 놓인, 아직 필드에 안 올라간 유닛 슬롯들”을 의미하면 됩니다.

  5. 정보를 더 크게
  그게 맞습니다. 이 게임은 미니멀 HUD보다 읽는 재미가 더 중요해 보입니다.
  그래서 정보는 작게 숨기지 말고, 오히려 크게 잡는 게 맞습니다.

  제 추천 우선순위:

  - 크게: 카드 일러스트, 카드 이름, 비용, 주요 효과, 자원량
  - 중간: 아이템/아티팩트 아이콘, 선택 대상 정보
  - 작게: 부가 설명, 세부 수치, 장식성 텍스트

  즉 화면 철학은:
  보드 중심 + 카드와 자원은 큼 + 보조 정보는 눌러 담음

  정리
  제가 지금 추천하는 확정안은 이겁니다.

  - 상대 stage는 전체 모달이 아니라 필드 위 stage overlay
  - 필드는 7x8, 상대/아군 7x4로 정확히 절반 분할
  - 하단 HUD는 2줄
  - 벤치는 하단 대기 유닛 구역
  - 정보는 작고 세련되게 숨기기보다, 크고 읽기 쉽게

  원하면 다음엔 이걸 바탕으로
  정확한 하단 HUD ASCII
  와
  이벤트 선택 / 상점 / suppression 준비
  3개 상태의 정밀 레이아웃을 다시 그리겠습니다.



 */
