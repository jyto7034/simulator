# Docs Index

이 문서는 `core_enhanced/docs`의 입구다. 새 작업자는 먼저 이 파일을 읽고, 어떤 문서가 source of truth인지와 각 문서가 어떤 역할을 갖는지 확인한다.

이 저장소에는 루트 `README.md`가 없고, 문서 지도는 이 파일이 담당한다.

## Source Of Truth 순서

문서보다 실제 동작이 우선이다. 작업자는 아래 순서로 사실을 확인한다.

1. 실제 runtime code
2. live RON/data
3. Unity-facing snapshot/command/WebSocket 계약
4. 최신 정책 문서

문서와 코드가 충돌하면 먼저 코드를 읽고, 게임 정책 판단이 필요한 부분은 사용자와 의논한다.

## 최상위 문서

아래 문서들이 `core_enhanced` 쪽 최상위 문서다.

| 문서 | 역할 | 우선 읽는 경우 |
| --- | --- | --- |
| `docs/README.md` | 문서 지도. 각 문서의 역할과 source-of-truth 계층을 설명한다. | 문서 탐색을 시작할 때 |
| `docs/game_rulebook.md` | 현재 게임 규칙의 최상위 룰북. 런 흐름, 시설형 Node Map 탐사 규칙, 노드, 전투, 보상, 직원/장비/스킬 파편, 끝없는 탐사 연구/반복 조우/bonus objective, 보스 전조, 실패/후퇴 정책을 게임 루프 순서로 설명하고, 세부 구현 계약은 전문 문서로 연결한다. | gameplay rule, 노드 흐름, Node Map 진행/visibility/selectability, 보상/소비/성장, 전투 모드/Endless 연구 정책을 바꿀 때 |
| `docs/skill_target_contract.md` | 스킬/평타 타겟팅과 DefenseRoute 범위 계약의 도메인 source of truth. 내부 authoring source인 `defense_tile_range`, Unity-facing 최종 `range_previews` cell DTO, `StepTargetingMode`, 자동 적대 타겟 유용성, `TileArea` 의미를 설명한다. | 스킬 타겟, 범위 표시, 자동 시전, 면역/무효 대상 필터를 다룰 때 |
| `docs/refactor_preparation_plan.md` | 리팩토링 판단 기준. source of truth 축소, 레거시 제거, 과도한 추상화 방지, debug 산출물 분류를 설명한다. | 구조 정리, 파일 이동, 레거시 제거, 큰 goal을 시작할 때 |
| `docs/data_loading_contract.md` | embedded RON/live data loader ownership 계약. `GameDataBase::load_live_embedded()`가 소유하는 live bundle, map/combat-preview/run-policy domain builtin, test-only direct load의 경계를 설명한다. | RON loader, `include_str!`, live data ownership, server/test data loading 경계를 바꿀 때 |
| `docs/code_documentation_sync_guidelines.md` | 코드 변경 시 문서/테스트를 함께 갱신하기 위한 상위 작업 지침. source-of-truth 순서, 변경 유형별 갱신 문서, 완료 전 체크리스트를 포함한다. | 코드 변경이 문서/Unity 계약/테스트에 영향을 줄 때 |
| `docs/codex_goal_command.md` | 새 goal을 Codex에게 맡길 때 붙여 넣는 표준 명령어와 goal skeleton. | 장기 작업 goal 문서를 만들거나 Codex 작업 규칙을 통일할 때 |

## 설계 리뷰/감사 문서

아래 문서는 현재 정책의 source of truth가 아니라, 리팩토링 후보를 고르기 위한 참고 자료다. 코드 라인 근거는 시간이 지나면 낡으므로 goal 착수 시 반드시 runtime code와 live RON/data를 다시 확인한다.

| 문서 | 역할 | 우선 읽는 경우 |
| --- | --- | --- |
| `docs/component_design_review.md` | 2026-06-12 기준 컴포넌트 설계 검토와 refactor 후보 목록. | 리팩토링 후보를 고르거나 과거 설계 판단 맥락을 볼 때 |
| `docs/goal_completion_review_guide.md` | 완료 처리된 master/subgoal 항목이 실제 runtime/data/DTO/test까지 구현됐는지, 그리고 장기 방향에 맞는지 재검토하는 감사 절차. | 완료된 goal 체크박스를 검증하거나 후속 correction/refactor goal을 만들 때 |
| `docs/boss_omen_chain_policy_draft.md` | Phase 5 Boss Omen/Event 정책 초안의 과거 기록. 안정화된 정책은 `game_rulebook.md`와 `skills/skill_fragment_system.md`에 흡수됐으므로 source of truth가 아니다. | Phase 5 정책 결정 맥락이나 과거 질문을 추적할 때 |

## 외부 Unity Canonical 문서

Unity-facing 계약/구현 문서는 이 저장소의 `docs/`가 아니라 외부 Unity 프로젝트가 canonical이다.

```text
F:\unity projects\ark\docs
/mnt/f/unity projects/ark/docs
```

특히 아래 문서는 외부 위치를 기준으로 확인하고 갱신한다.

| 외부 문서 | 역할 |
| --- | --- |
| `/mnt/f/unity projects/ark/docs/unity_core_contract.md` | Unity 클라이언트와 `/game` WebSocket의 통합 계약. snapshot, command, server message, enum casing, Node Map DTO, live battle transport를 설명한다. |
| `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md` | Unity 클라이언트 구현 goal 지침. UI/scene 구현 순서, WebSocket 수신 파이프라인, 검증 기준을 설명한다. |
| `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md` | core <-> Unity 전투 통신의 canonical 통합 계약. `battle_setup_snapshot`, `battle_update.events_delta`, `battle_update.checkpoint`, `battle_resync`, 전투 종료 snapshot 흐름을 함께 설명한다. |
| `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md` | 과거 setup snapshot 분리 계약. 현재는 transport 통합 문서가 supersede하며, migration context 확인용으로만 본다. |
| `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md` | 과거 battle update 분리 계약. 현재는 transport 통합 문서가 supersede하며, migration context 확인용으로만 본다. |

이 저장소 안에는 `docs/unity_core_contract.md`, `docs/unity_client_implementation_goal.md`를 보관하지 않는다. 같은 이름의 문서가 다시 생기면 stale copy로 간주하고 외부 canonical을 확인한다.

## 콘텐츠/스킬 문서

`docs/skills/` 아래 문서는 환상체, E.G.O 장비, 스킬 파편 콘텐츠 설계를 다룬다.

| 문서 | 역할 |
| --- | --- |
| `docs/skills/lobotomy_content_catalog.md` | 환상체 콘텐츠 카탈로그. 최종 보스급, 일반 보스급, 엘리트, 도구 로스터와 파편/장비 설계 원칙을 정리하는 콘텐츠 source of truth다. |
| `docs/skills/abnormality_skill_design_notes.ko.md` | 확정 로스터 환상체의 원작 정체성, 기믹, E.G.O/스킬 모티프를 우리 게임의 조우/파편/장비 설계 언어로 번역한 한국어 설계 노트다. 밸런스표나 구현 완료 보고서가 아니다. |
| `docs/skills/skill_fragment_system.md` | 스킬 파편의 상위 게임 정책. 보스 격리 보상, 위험 자원으로서의 파편, WhiteNight 같은 특수 파편 체인을 설명한다. |
| `docs/skills/skill_fragment_wiki.md` | 현재 live placeholder/구현 스킬 파편 목록. fragment id, rarity, 현재 효과 유형, source data를 확인하는 색인이다. |
| `docs/skills/ego_equipment_wiki.md` | 현재 live placeholder E.G.O 장비 목록. 장비 definition id, rarity, source data를 확인하는 색인이다. |
| `docs/skills/tool_abnormality_wiki.md` | 확정 도구형 환상체 목록. 이벤트, 정비, 지원 노드, 리스크 선택지, 장비/강화 이벤트로 사용할 도구의 역할과 실패 비용을 정리한다. |

콘텐츠 문서는 정책과 카탈로그를 제공하지만, 실제 사용 가능 여부와 필드 shape는 live RON/data와 runtime validation을 먼저 확인한다.

## Goal 문서

`docs/goals/<goal_name>/`은 장기 작업의 작업 기억장치다. 정식 정책 문서가 아니라, 진행 중 또는 최근 완료된 goal의 계획/실험/판단 기록이다.

goal 디렉터리는 보통 아래 세 파일을 가진다.

| 파일 | 역할 |
| --- | --- |
| `PLAN.md` | 목표, 범위, 완료 조건, 중단 조건, 검증 명령을 기록한다. |
| `EXPERIMENTS.md` | 시도/실패/성공 결과, 테스트 실패 원인, 수정 내용, 재검증 결과를 기록한다. |
| `EXPERIMENT_NOTES.md` | 작업 중 판단, source-of-truth 충돌, 정책 질문, 후속 후보를 기록한다. |

현재 파일이 있는 goal 디렉터리:

| Goal | 의미 |
| --- | --- |
| `docs/goals/abnormality_skill_design_notes/` | 환상체별 원작 기믹을 우리 게임용 스킬/장비/조우 설계 노트로 정리한 콘텐츠 조사 goal. |
| `docs/goals/allowed_actions_policy/` | `allowed_actions` 결정 로직을 `GameState`와 active context 기준의 단일 source of truth로 모으는 goal. |
| `docs/goals/automatic_hostile_target_usefulness/` | 피해 또는 적대적 대상 효과가 없는 적을 자동 평타/스킬 후보에서 제외하는 정책 구현 goal. |
| `docs/goals/basic_attack_lifecycle_refactor/` | 기본 공격 스케줄링, 타겟 선택, windup/impact, projectile lifecycle을 응집시키는 refactor goal. |
| `docs/goals/basic_attack_projectile_target_only_collision/` | 기본 공격 투사체를 고정 시간 자동 적중이 아니라 원래 locked target body와의 target-only continuous sweep 적중으로 정리한 goal. |
| `docs/goals/battle_attack_movement_time_consistency/` | 공격 판정 시점과 이동 runtime position이 같은 core battle time 기준을 쓰도록 고정하는 goal. |
| `docs/goals/combat_setup_usage_audit/` | `src/game/combat_setup/` 파일별 실사용 여부를 감사하고 스텁/죽은 파일을 정리하는 goal. |
| `docs/goals/core_file_hierarchy_refactor/` | `src/game` 파일 계층을 world/admin, world/tests, combat_preview 등 명확한 하위 모듈로 정리하는 대형 refactor goal. |
| `docs/goals/core_policy_implementation_master_completion_review/` | `core_policy_implementation_master`의 완료 처리된 subgoal들이 실제 runtime/data/DTO/test와 장기 방향까지 맞게 구현됐는지 감사하는 goal. |
| `docs/goals/core_test_contract_audit/` | core/server 테스트가 gameplay, Unity-facing DTO, live RON/data, timing/order 계약을 실질적으로 고정하는지 검토하는 감사 goal. |
| `docs/goals/core_unity_battle_setup_snapshot_contract/` | Unity 전투 scene construction source인 `battle_setup_snapshot` 계약을 구현한 goal. |
| `docs/goals/core_unity_battle_transport_recovery_repair/` | 전투 command 후 전체 `state_snapshot` 억제, invalid resync cursor 검증, setup-loss/reconnect recovery를 정리한 transport repair goal. |
| `docs/goals/core_unity_battle_update_contract/` | live battle transport를 `battle_update.events_delta`와 `battle_update.checkpoint` 중심으로 구현한 goal. |
| `docs/goals/data_validation_refactor/` | `data/mod.rs` cross-reference validation을 dedicated validation module로 분리하고 빈 replay 디렉터리를 정리하는 goal. |
| `docs/goals/documentation_runtime_contract_audit/` | 주요 문서가 실제 runtime code, live RON/data, 외부 Unity-facing 계약과 같은 내용을 말하는지 감사하는 goal. |
| `docs/goals/defense_route_path_continuity_repair/` | generated DefenseRoute가 endpoint 직선 이동으로 void에 막히지 않도록 valid adjacent route path와 live movement progress를 고정하는 repair goal. |
| `docs/goals/defense_route_single_runtime_model/` | DefenseRoute를 단일 live runtime 모델로 통합하고 SplitRoom/Encirclement 별도 모드 정책을 제거하는 goal. |
| `docs/goals/defense_route_sticky_block_engagement/` | 명일방주식 지속 저지 engagement를 구현해 지상 적이 blocker를 관통하지 않도록 고정한 goal. |
| `docs/goals/item_inventory_cleanup/` | artifact 제거 정책과 abnormality-as-item 레거시를 정리하는 inventory goal. |
| `docs/goals/live_movement_event_immutability/` | live `MovementSegmentStarted` event를 seq 부여 후 수정하지 않고 실제 이동 tick별 presentation source로 고정한 goal. |
| `docs/goals/movement_tick_backend_refactor/` | Direct/Rapier movement backend의 중복 tick control flow를 줄이는 refactor goal. |
| `docs/goals/pending_deployment_range_preview_contract/` | 배치 확정 전 위치와 방향을 기준으로 Unity가 core에 최종 range preview cell을 요청하는 계약을 구현한 goal. |
| `docs/goals/pve_enemy_wave_model_refactor/` | PVE wave enemy identity를 `Abnormality`/`CorrodedEmployee` variant로 분리하고, legacy `enemies` field와 preview-local generated wave resolver를 제거하는 goal. |
| `docs/goals/skill_cast_interrupt_epoch/` | `InterruptCast` 효과, cast-start seq 기반 pending cast token, `SkillCastInterrupted` event를 구현하는 goal. |
| `docs/goals/stat_pipeline_inventory_contract_repair/` | battle-entry stat pipeline 단계와 artifact removal 표현을 보완하는 repair goal. |
| `docs/goals/stats_pipeline_unification/` | 장비, artifact, skill fragment, growth stack, enhancement, consumable modifier의 전투 진입 스탯 합산 entry point와 적용 순서를 통합하는 goal. |
| `docs/goals/step_targeting_contract/` | `StepTargetingMode::{ReuseCastTarget, RetargetOnStep}` 계약을 문서화한 goal. |
| `docs/goals/tile_based_attack_delivery_contract/` | 공식 기본 공격/스킬 판정을 타일 기반으로 정리하고, hit-scan/projectile/directional collision delivery 경계를 고정하는 goal. |
| `docs/goals/unity_range_preview_final_tiles_contract/` | Unity가 `defense_tile_range`, `effective_weapon_profile`, `effective_basic_attack`을 조합하지 않고 core가 계산한 최종 range cell DTO만 쓰도록 정리한 goal. |

파일이 없는 빈 `docs/goals/*/` 디렉터리는 현재 source of truth가 아니다. 과거 계획 이름 또는 cleanup 잔재일 수 있으므로, 내용을 근거로 삼지 않는다.

goal 완료 후 유지해야 할 정책은 `game_rulebook.md`, `skill_target_contract.md`, `refactor_preparation_plan.md`, 외부 Unity canonical 문서 중 가장 가까운 source-of-truth 문서로 흡수한다. goal 문서는 장기 보관 문서가 아니다.

## 문서 선택 가이드

- 게임 규칙, 시설형 Node Map 진행/visibility/selectability, 노드 흐름, 보상, 직원 성장, 전투 성공/실패 판정: `docs/game_rulebook.md`
- Node Map JSON DTO, `map_template_id`, `slot_id`, `map_navigation.selectable_node_ids`, Unity 렌더링/선택 계약: 외부 `unity_core_contract.md`
- 스킬/평타 타겟팅, 범위, 자동 시전, 유효 적대 대상 정책: `docs/skill_target_contract.md`
- Unity WebSocket, DTO shape, command/result: 외부 `unity_core_contract.md`
- 전투 시작, live update, checkpoint, resync, 전투 종료 snapshot 흐름: 외부 `core_unity_battle_transport_contract.md`
- Unity 구현 순서와 클라이언트 작업 지침: 외부 `unity_client_implementation_goal.md`
- 리팩토링 방향, 레거시 제거, debug 산출물 분류: `docs/refactor_preparation_plan.md`
- embedded RON loader ownership, `GameDataBase::load_live_embedded()`, domain-owned builtin loader 경계: `docs/data_loading_contract.md`
- 코드 변경 시 문서/테스트 동기화 규칙: `docs/code_documentation_sync_guidelines.md`
- 새 Codex goal을 만들 때 붙여 넣을 명령: `docs/codex_goal_command.md`
- 환상체/파편/장비 콘텐츠 설계: `docs/skills/*`
- 과거 작업 판단, 실패 이유, 검증 명령: `docs/goals/<goal_name>/*`

## 정리 원칙

- docs에는 현재 의사결정과 client-facing 계약만 남긴다.
- 문서를 무조건 따르지 않는다. 실제 runtime code와 live RON/data를 먼저 확인한다.
- 삭제된 문서, 빈 goal 디렉터리, debug output, 오래된 구현 기록은 source of truth로 보지 않는다.
- 새 정책과 충돌하는 레거시는 compatibility layer나 dual schema로 보존하지 않는다.
- 정책이 모호하면 구현 전에 사용자와 의논한다.
