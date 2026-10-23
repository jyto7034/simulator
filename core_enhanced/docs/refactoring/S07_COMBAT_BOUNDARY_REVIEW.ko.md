# S07 전투 진입·결과 정산 상세 조사

작성일: 2026-10-16 · 상태: 대표 경계 조사와 선택 개선 B1·R1 완료

**현재 위치:** [B1 교정·검증·완료 검토](S07_B1_IMPLEMENTATION.ko.md)와 맨 아래 R1 실행·완료 검토를 마쳤다. 후속 [S01 대표 경계 조사·D1 완료 검토](S01_APPLICATION_BOUNDARY_REVIEW.ko.md)도 마쳤으며 최신 재개 위치는 [README](README.md)를 따른다.
원인·재현·미실행·계획은 제품 교정 전 조사 시점의 기록으로 보존한다. 현재 실행 결과와 재개 위치는
[출발점](README.md), B1 완료 기록과 이 문서의 R1 실행 절을 따른다.

## 범위와 조사 기준선

- 요청: [출발점](README.md)의 현재 상태·다음 행동을 확인하여 이어서 수행하고 결과·재개 위치 갱신.
- 선택 범위: 전투 노드 진입 → live 종료 → 정산 receipt → 결과 확인 → 보상/노드 진행,
  관련 거절·준비 취소·후퇴·Continue 및 호스트 저장 승인 경계. 제품 구현은 이번 단계에 포함하지 않는다.
- [기준선](evidence/s07_001/baseline.json): 지도 단계와 동일한 Core 입력 457개. 추가·삭제·내용 변경 없음.
  [기존 작업 트리](evidence/s07_001/git-status-before.txt)를 보존한다.
- 실제 계약은 `core_runtime_contract.md`의 Active Battle Result, Durable active-run, Application Authority 절과
  `game_rulebook.md`, `game_rules/results-and-save.md`의 관련 절로 대조한다.
- `docs/goals/**`의 과거 문서·완료 선언은 조사 근거에서 제외한다. 시스템 전체·전투 커널 전체 감사는 아니다.

## 확인한 사실과 판단

1. `world/application.rs::prepare_application_scoped`가 complete draft를 만들고 handler·후처리·Continue 갱신·검증을 수행한다.
   `commit_application`만 live root를 교체한다. domain-local `commit_*`가 fallible이어도 곧바로 live 부분 반영을 뜻하지 않는다.
2. 전투 종료 시 정산하고 `ConfirmedCombatResult`를 보관한다. `CompleteCombatResult`는 receipt를 소비한다.
   ChooseOne의 후보·grant context는 별도 보상 세션으로 유지하며 claim/exit와 원정 terminal 정리가 연결된다.
3. 수동 후퇴의 BattleEntrySnapshot은 최초 입장 비용 처리 **후** 상태다. 미완료 노드 Continue는 비용 처리 **전** 확정 상태다.
   두 복원을 통합하면 현재 규칙을 바꾸므로 분리 유지가 필요하다. 스트레스 후퇴는 노드를 소비하는 예외다.
4. GameCore draft 실패와 CoreApplication의 치명적 battle fault latch는 구분한다.
   ordinary refusal은 원본을 유지하지만 이미 공개된 전투의 치명적 fault는 첫 원인을 저장하고 이후 mutation을 차단한다.
5. R1 후보: 패배는 `stage_committed_combat_resolution`을 사용하지만 승리 planner는 같은 capture→post-resolution→consumable decrement를 다시 조립한다.
   현재 동작 차이는 확인하지 않았다. 승패 공통 후처리 순서의 변경 지점을 하나로 모으는 작은 동작 보존 후보다.
6. **B1 오류 확인:** snapshot은 `battle.rewards`로 결과 후보를 표시한 뒤 victory planner가 deterrence quantity 배율을 적용한다.
   Continue validator는 그 배율을 적용한 지급 후보를 원본 콘텐츠와 다르다고 거절한다.
   공개 API probe에서 같은 콘텐츠 세대로 방금 export한 변경 없는 저장 bytes의 Continue 실패를 재현했다.

## 좁은 실행 확인의 범위

- `/tmp/s07-boundary-probe`의 독립 실행 도구로 현재 Core library를 호출한다. 제품 파일은 변경하지 않는다.
- 통제 입력은 동일 seed·선택형 수량 보상에 초기 억제력 100/1을 대비한다. fixture builder 검증 후 공개 명령과 Continue API를 사용한다.
- 현재 소스에서 library와 도구를 빌드해 실행했다. 과거 실행 산출물을 현재 동작 증거로 재사용하지 않는다.
- 재현 입력 준비 중 시간 승리의 required-wave 조건, reference-only encounter 보상 금지, reward database 재구성 시 기존 pool 보존 검증에서 각각 거절됐다. 일반 encounter만 변경하고 기존 pool을 보존하도록 시험 입력을 교정했다. 이 거절은 제품 오류로 집계하지 않는다.

## 실제 흐름과 상태 전달

경로는 `core_enhanced/src/` 기준이다. 다음은 함수 본문·호출자·상태 쓰기·관련 테스트 기대값을 연결한 정적 조사다.

| 단계 | 실제 입력·주 소유자 | 처리와 결과 | 실패·취소·재개 경계 |
| --- | --- | --- | --- |
| 명령 자격과 draft | `application.rs::prepare_command_with_generation` → `world.rs::prepare_with_transport_context` → `world/application.rs::prepare_application_scoped` | current context 검증 → complete clone → authority/cursor/action gate → handler → reconcile·Continue 갱신·최종 context 검증 → typed prepared token | 일반 Err는 draft 폐기. 명령 시도 revision과 fatal fault capsule은 CoreApplication 밖의 별도 책임 |
| 노드 진입 | `world/node_flow.rs::handle_confirm_enter_node_resuming` | Continue용 비용 전 상태 확보 → 입장 비용·유예·필수 선택 처리 → BattleEntrySnapshot capture → 노드 enter → 전투 handler | 수동 후퇴 snapshot은 비용 후 상태. 소진 선택이 필요하면 전투 시작 전 일단 반환 |
| 전투 구성 | `world/combat.rs::plan_explicit_combat_node_with_state`, `commit_planned_live_battle_start` | roster·inventory·fragment·검증 데이터 → preview/policy·BattleCore·attempt roster → run resource mount → 진입 power·attempt·UUID → InBattle → time-zero step | planner/구성/초기 projection 실패도 바깥 draft 폐기. 현재 공식 실행은 live DefenseRoute 경계이며 비지원 non-live mission은 거절 |
| Event 전투 | `world/event_node.rs::handle_resolve_event_choice_check` 내부 StartCombat 분기 | 검사 결과·event session을 draft에 기록 → 같은 live start 경계 | event용 reentry snapshot은 NodeConfirm·session을 다시 구성. durable Continue는 event 선택 전 노드 진입 기준 |
| live 진행 | `advance_active_battle_staged` | cursor/revision 자격 → kernel candidate step → attempt condition reconcile → terminal result 산출 → VN trigger·최종 battle projection → 종료 전환 | projection이 active root를 필요로 하므로 active 제거 전에 준비. time-zero terminal에도 같은 순서 유지 |
| 종료 확정 | `transition_finished_live_battle_to_result` | authored 후보 확정 → authoring terminal → 비권위 trace → CombatBattle content·summary → 정산 → receipt·CombatResult | trace 파일 실패는 warning. 다른 fallible 처리는 바깥 draft를 통해 commit 전 실패 처리 |
| 승리/패배 정산 | `settle_combat_result`와 victory plan/commit | typed run resource 결과, attempt 직원 상태·사망 자산·생존 XP·소비품 기간, 노드/encounter 진행, 보상·도감·회복 경로 | 패배는 성공 보상 없이 노드 소비. 실패 선택/최종 실패 분기는 `fail_run_with_reasons`가 담당 |
| 결과 확인 | `handle_complete_combat_result` | receipt take → pending reward가 있으면 InReward, 없으면 continuation phase → 저장 acknowledgement 반환 | capability는 CombatResult + receipt를 요구. 성공 후 재호출은 원래 결과를 다시 지급하는 API가 아님 |
| 선택 보상 | `world/reward.rs` claim/exit, `action_capability.rs::evaluate_reward_action` | 선택 후보와 원 grant context로 지급 → InRewardClaimed → Exit에서 return phase로 이동 | can_skip=false면 미수령 Exit 거절. InRewardClaimed에서는 Exit만 허용. terminal pending choice는 claim 후에도 Exit 전까지 자산·Continue 유지 |
| 외부 저장 | `PreparedApplication` → 서버 `publish_prepared_application_bound` → `ApprovedPersistence::publish` | profile→active lock, revision 비교, 필요 시 accepted intent → `prepared.commit()` | 승인 전 Err는 준비 객체 폐기. 승인 후 파일 게시 복구가 남으면 commit + recovery_pending. 평범한 tick은 checkpoint/profile 불변이면 저장 생략 |

### 세 가지 복원과 전투 fault

| 상황 | 복원·보존하는 사실 | 근거 |
| --- | --- | --- |
| 일반 수동 후퇴 | 입장 비용·부화 후 entry snapshot을 복원하고 성공 후퇴 횟수만 증가. RNG/UUID·사본·직원·SavePoint·VN 복원. 영구 profile/peak는 snapshot 필드에 포함하지 않음 | `BattleEntrySnapshot::capture/restore_shared/restore`, `handle_retreat_battle` |
| 스트레스 전투 중 후퇴 | entry 자산은 복원하되 노드를 완료하고 미수령 보상을 잃음. 일반 후퇴의 재도전 ledger 경로를 사용하지 않음 | `handle_retreat_battle`의 stress 분기, `u164_manual_retreat_restores_battle_assets_consumes_node_and_preserves_entry_cost` |
| 미완료 전투 Continue | immutable pre-entry continuation을 복원. 실제 live battle을 저장하지 않음. 성공 후퇴 ledger만 그 원본에 덧씌움 | `prepare_continue_before_operation`, `refresh_continue_after_operation`, `restore_continue_document` |
| 확정 결과/필수 보상 Continue | compact result·reward·context·phase 복원. 결과 확인·claim을 다시 정산하지 않음. pending choice 없는 최종 terminal은 Continue 없음 | continuation capture/validate/restore, `durable_run_is_terminal` |
| 명시적 SavePoint load | 별도 checkpoint 시점으로 되감고 load 횟수 증가. 위 두 복원과 다른 계약 | runtime 계약의 SavePoint·Continue 구분. codec 전체 상세 조사는 S04 후속 |
| 일반 입력 거절 | 원본 gameplay authority 유지, 잘못된 authority·cursor·target·자원 부족 등을 실패로 반환 | world gate·application draft, fault 분류의 비치명적 제외 |
| 공개된 전투의 치명적 runtime 실패 | 마지막 commit된 GameCore 보존 + CoreApplication의 첫 fault latch 보존, 후속 mutation/export 차단 | `application/fault.rs::runtime_error/ensure_battle_not_faulted`, `fault_tests.rs` |
| 준비된 명령 폐기·authoring 취소 | commit 전 complete draft와 준비 결과를 폐기. 도메인 중간 commit은 live root를 바꾸지 않음 | `PreparedApplication::commit`, executor의 `control::poll`·typed commit |

`GameCore` 내부 재시도 테스트를 근거로 치명적 fault 이후의 공개 CoreApplication 재시도까지 허용된다고 해석하지 않는다.
또한 `ConfirmedCombatResult.runtime_facts`는 `serde(skip)`인 제한된 live 관측이며 durable history가 아니다.

## 기존 검증의 실제 내용과 공백

아래는 테스트 **본문을 읽어 확인한 보호 의도**다. 이번 조사에서 기존 전체 테스트를 실행했다는 의미는 아니다.
`world/tests/combat.rs`의 `settle_test_combat_result`는 application draft를 거치지만 terminal payload를 직접 주입한다.
따라서 이 fixture의 정산 검증과 실제 kernel 종료→공개 API 검증을 구분한다.

| 위치·테스트 | 확인한 기대값 | 범위 한계 |
| --- | --- | --- |
| `world/tests/combat.rs::combat_terminal_settlement_failure_does_not_partially_apply_rewards_on_retry` | node completion 조건을 깨뜨린 반복 정산 시 자원·직원·진행·도감 불변 | 내부 terminal payload 주입. 일반 정상 플레이 재현 아님 |
| 같은 파일 `combat_result_commits_attempt_conditions_and_survival_xp_once_per_employee` | 재배치한 같은 직원의 condition·생존 XP 1회, ACK에서 inventory/fragment/UUID/재화 불변 | 승리 분기 중심 |
| 같은 파일 `combat_defeat_commits_attempt_incapacitation_without_reapplying_it` | 패배에서도 attempt 부상·trauma를 다시 계산하지 않음 | 소비품 기간까지 함께 확인하지 않음 |
| 같은 파일 `consumable_modifier_survives_abandonment_and_expires_on_committed_result` | 두 번 후퇴는 기간 보존, 패배 정산은 소진 | 승리와 수량 배율의 조합은 이 테스트 대상 아님 |
| 같은 파일 `finishing_advance_rejects_future_cursor_without_consuming_terminal_state` | 실제 elapsed 진입에서 잘못된 cursor가 terminal 상태를 소비하지 않음 | host 저장 실패는 별도 |
| `world/tests/u04_continuation.inc.rs::u04_live_terminal_commits_before_ack_and_continue_cannot_repeat_grants` | 실제 timed live 종료 → 노드 완료 → Continue → ACK → 중복 ACK 거절 → 재복원 | 기본 배율의 보상 |
| 같은 파일 `u04_choose_one_two_candidates_and_participant_context_survive_select_claim_restarts` | 선택·claim 전후 재시작, 미선택 claim/exit 거절, 원 참가자 XP, 중복 claim 거절 | 무억제 배율 2와 결합하지 않음 |
| 같은 파일 `u04_terminal_choice_survives_ack_and_claim_restart_then_exit_invalidates_continue` | terminal에서도 ACK·claim까지 자산 보유, Exit에서 정리, 반복 명령 무효 | Tutorial terminal fixture |
| 같은 파일 `u04_continue_unfinished_battle_preserves_two_retreats_and_third_entry_without_quit_charge` | 두 후퇴·세 번째 진입·반복 Continue에서 후퇴 예산 불변 | live 세부 효과 검증 아님 |
| `application/fault_tests.rs::u04_prepared_command_and_host_tick_drop_preserve_roots_and_commit_matches_immediate_ports` | entry·pause·tick 준비 폐기 시 root/checkpoint 유지, commit과 직접 실행의 결과 일치 | 모든 결과·보상 조합을 포함하지 않음 |
| 같은 파일 `u04_prepared_host_tick_fault_retains_first_origin_and_original_authority` | 공개 tick work limit 실패에서 root와 첫 fault 보존 | 일반 입력 거절과 별도 정책 |

무억제 수량 배율과 ChooseOne·Continue의 교차 검증은 위 읽은 테스트에서 확인하지 못했다.
이는 저장소 전체에 그런 테스트가 없다는 선언은 아니다. 해당 입력을 재현 도구에서 직접 확인한다.

## 구조 평가 12개 항목의 적용

| 항목 | 이 흐름의 판단 |
| --- | --- |
| 책임·도메인 경계 | S07은 S03/S05/S06/S08을 연결하는 조정 책임. 단순 파일 분할만으로 개선된다고 판단하지 않음 |
| 상태 소유권 | live session·attempt roster·정산 receipt·reward session의 서로 다른 수명을 확인. 두 보상 표현의 일치 문제는 B1 조사 |
| 상태 전이·수명 | 후퇴/Continue/SavePoint와 terminal pending choice를 분리 유지 |
| 실패·commit | complete application draft와 호스트 accepted 경계 유지. fatal fault latch를 rollback 대상에 넣지 않음 |
| 규칙 중복 | 승패 공통 후처리 조립은 R1 후보. 보상 배율 결정·표시·복원 검사는 B1 조사 |
| 의존 방향·공개 범위 | 서버는 공개 PreparedApplication을 소비. S07용 새 public API나 새 commit port는 필요 없음 |
| 변경 용이성 | 직원 후처리 단계를 추가할 때 현재 승패 양쪽 순서를 수동 동기화해야 함. 기존 helper 재사용으로 좁힐 수 있음 |
| 데이터·외부 계약 | reward option·result summary·durable entitlement의 값 일치 필요. DTO/codec 변경은 단순 리팩토링으로 취급하지 않음 |
| 결정론·재개 | snapshot UUID와 immutable continuation 원본 유지. 난수 소비 순서를 바꾸는 최적화는 제안하지 않음 |
| 오류·관측 | trace 실패는 비권위. authoring 관측·projection 실패는 commit 전 처리. fatal error는 별도 capsule |
| 검증 가능성 | 내부 payload 정산 테스트와 실제 live/public API 테스트를 함께 사용해야 함. 교차 입력을 좁은 probe로 보완 |
| 성능·자원 | clone/admission·Arc snapshot·Continue cache를 확인했지만 성능 측정은 없음. clone 제거나 예산 변경 후보를 확정하지 않음 |

## 작은 동작 보존 후보 R1

- **문제·근거:** `combat.rs:734`의 `stage_committed_combat_resolution`은 capture → 직원 후처리 → 소비품 기간 감소를 묶는다.
  패배 경로(`settle_combat_result`)는 이를 쓰지만 승리 planner(`plan_player_victory_combat_result_completion`)는 같은 세 단계를 다시 조립한다.
- **영향:** 공통 후처리 순서나 단계를 변경할 때 두 조립 위치를 수동으로 맞춰야 한다. 승패 공통 계약과 기존 helper의 경계가 어긋난다.
  현재 승패 동작이 이미 다르다는 주장은 아니다.
- **개선:** 승리 planner의 해당 세 줄을 기존 helper 호출로 대체한다. node completion·도감 검증보다 먼저 호출하도록 옮기지 않는다.
  추가 type/trait, 별도 transaction, crate/파일 분리 없이 같은 함수 안에서 처리한다.
- **보존:** capture 횟수, admission·clone 예산, 오류 순서, 직원 condition·사망 자산·생존 XP·consumable 기간,
  승리 전용 보상/진행과 패배 전용 실패 처리, ACK와 Continue 의미.
- **검증:** 위 승리 condition/XP, 패배 condition, 소비품 후퇴/정산, 정산 실패·재시도 테스트를 변경 전후 실행한다.
  새 helper 호출 여부만 검사하는 테스트는 만들지 않는다. 필요하면 기존 소비품 테스트를 승리/패배 양쪽으로 확장한다.
- **구조 수용 조건:** 공통 후처리 순서의 조립 지점이 하나이고 두 결과 분기가 이를 사용한다. 보상 정책 분기까지 억지로 합치지 않는다.
- **선행·종료:** B1이 실제 오류로 확인되면 먼저 처리하거나 독립적으로 분리해 우선순위를 명시한다.
  R1만 끝나도 유효한 상태여야 하며, 변경 후 별도 시점의 완료 검토에서 diff·동작·검증을 다시 확인한다.

## 인접 영역 후속과 제한

- S04 저장 계약 문서에 journal V3가 적혀 있지만 현재 `player_profile_store/application_intent.rs`의 `JOURNAL_VERSION`은 4다.
  불일치는 확인했으나 전체 호환성·정책 변경 경위를 조사하지 않았다. 문서 버전만 임의 교정하거나 저장 migration으로 확대하지 않는다.
- 서버 accepted publication 호출부는 확인했지만 실제 filesystem crash/recovery와 Unity 표시 소비자는 이번 조사 대상 밖이다.
- 전투 내부의 공격·이동·취소 모든 branch, 모든 encounter와 모든 데이터 세대 조합은 조사하지 않았다.
- [경계 입력 해시](evidence/s07_001/boundary-inputs.json)는 읽은 계약·서버 경계 파일을 식별하며 그 파일 전체 감사의 증거는 아니다.

## B1 — 보상 수량 해석이 결과·지급 후보·Continue에서 어긋남

**분류:** 실행 재현한 오류 교정 + 보상 해석 책임 정리. 주 담당 S07, 관련 S04/S05/S06.
**우선순위:** 현재 지원된 데이터 형태의 정상 저장을 읽지 못하므로 작은 구조 정리 R1보다 우선한다.

### 원인과 도달 경로

1. `combat_preview/types.rs::BattleDeterrenceSnapshot::for_entry`는 무억제 승리의 quantity multiplier를 2로 정한다.
2. `combat.rs::combat_battle_state_from_live_result`는 outcome policy의 보상과 deterrence를 따로 전달한다.
3. `transition_finished_live_battle_to_result`가 `snapshot.rs`를 호출하면 `battle.rewards`의 아직 배율 적용 전 effect가 summary에 들어간다.
4. `plan_player_victory_combat_result_completion`은 그 뒤 rewards 사본에 `apply_deterrence_quantity_multiplier`를 적용한다.
   ChooseOne이면 이 다른 값이 지급 후보로 보관된다. ClaimAll도 결과 표시와 실제 적용량이 갈릴 수 있는 같은 생산 경계를 사용한다.
5. `active_run/continuation.rs::validate_continue_payload`의 일반 reward 검사는 원본 또는 난이도 percent 적용본만 비교한다.
   합법적인 무억제 quantity 적용본을 생성하지 않아 `pending reward differs from retained content`로 실패한다.
6. 이어지는 result/entitlement 대조도 `display.effects == option.effects`를 요구한다.
   따라서 첫 validator만 느슨하게 하거나 화면 숫자만 바꾸는 수정으로는 전체 경계가 해결되지 않는다.

### 재현 입력과 의미

- 공식 embedded 데이터를 기반으로 `GameDataBuilder`의 명시적 fixture capability를 사용했다.
  일반 encounter의 승리 조건을 10ms 시간 승리로 바꾸고 required wave 플래그를 그 계약에 맞췄다.
  ChooseOne 후보는 `GrantEnkephalin(17)` + 기존 장비 한 사본이며 기존 reward pool은 보존했다.
- 같은 seed 7001과 같은 노드 진행에 초기 억제력만 100/1로 대비했다. 최초 활동과 전투 진입은 공개 명령으로 수행한다.
  무억제 전환의 Continue 선택도 공개 명령을 사용한다. runtime state·checkpoint bytes를 직접 고치지 않는다.
- 데이터 builder의 validation을 통과하고 실제 `CoreApplication` 명령·advance·export·profile install·Continue로 도달했다.
  이 실행은 시험 데이터에 대한 library 재현이다. 수정하지 않은 공식 배포 데이터 전체의 장애나 production pack 수용을 검증한 것은 아니다.
- 배율 1에서는 summary와 저장 후보가 모두 17이고 Continue가 성공했다.
  배율 2에서는 summary 17, 저장 후보 34이며 동일 세대의 새 Core에서 원본 export를 그대로 읽었을 때 거절됐다.

최종 실행은 [probe 원본](evidence/s07_001/probe/main.rs), [manifest](evidence/s07_001/probe/Cargo.toml),
[출력](evidence/s07_001/probe-run006.log), [6개 대조 결과와 저장 해시](evidence/s07_001/probe-results.json)에 보존했다.
실패한 초기 입력 준비는 [별도 메모](evidence/s07_001/probe-setup-failures.txt)에 남겼다.
[공식 데이터·설정 식별](evidence/s07_001/probe-support-inputs.json)은 실행 후 캡처이며 전후 불변 증거가 아니다.
실제 실행의 콘텐츠 정체성은 각 원본 checkpoint에도 보존되어 있다.

| 배율 | 결과 화면 | ACK 후 | claim 후 | 수령 전→후 재화 |
| --- | --- | --- | --- | --- |
| 1 (초기 억제력 100) | Continue 성공, 표시/후보 17 | Continue 성공 | Continue 성공 | 500→517 |
| 2 (초기 억제력 1) | Continue 거절, 표시 17/후보 34 | Continue 거절 | Continue 거절 | 500→534 |

세 거절 모두 `invalid_static_data`: `active-run continuation pending reward differs from retained content`다.
ACK 뒤 summary가 없어져도 거절되므로 결과 화면만의 문제가 아니다. 원 세션의 claim은 34를 실제 지급한다.
이 6건은 오류 재현 실험이며 “회귀 테스트 6개 통과”로 집계하지 않는다. 빌드 시 기존 Core warning 36개가 출력됐다.
제품의 `src/**`, `tests/**`, manifest·lock·build script는 수정하지 않았다.

재현 예시(저장소 루트, Cargo 및 기존 offline dependency cache 사용):

```bash
mkdir -p /tmp/s07-boundary-probe
env PATH=/home/blast/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin CARGO_HOME=/tmp/wp02-cargo-home CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target CARGO_PROFILE_DEV_DEBUG=0 cargo run --offline --locked --manifest-path core_enhanced/docs/refactoring/evidence/s07_001/probe/Cargo.toml
```

도구는 결과 JSON을 stdout으로 내보내며 원본 export를 `/tmp/s07-boundary-probe/checkpoint-*.json`에 쓴다.
오류를 출력으로 관측하기 때문에 process exit 0만으로 문제가 해결됐다고 판단하면 안 된다.
교정 후에는 여섯 출력의 `continue_ok`가 모두 true이고 실제 수량·ACK/claim 의미도 맞는지 확인한다.

### 하나의 작업으로 구현할 교정 범위

**작업 B1:** 전투 보상 수량을 확정하는 경계를 통일하고, 정상 writer 산출물을 reader가 수용하도록 함께 교정한다.

- 포함: `combat.rs`의 terminal reward 준비·victory planner, `combat_setup/rewards.rs`의 공통 수량 변환,
  `snapshot.rs` 소비 방식, `active_run/continuation.rs`의 정상 생성 보상 비교, 관련 회귀 검증.
- 제외: 보상 밸런스·새 지급 제한·새 gameplay 정책, 전투 내부 효과 재설계, 별도 commit port, 서버 저장 알고리즘 변경,
  임의 외부 변조에 대한 방어 확대, 전체 codec 재작성.
- 목표: revival 등 authored 후보를 확정한 뒤 **유효한 최종 보상 옵션을 한 번만 계산**하고 결과 표시·자동 지급·pending choice가 같은 값을 소비한다.
  기존 난이도 percent와 무억제 quantity의 적용 대상·순서, singular 사본 grant 불변, checked overflow 처리를 보존한다.
  bonus/encounter reward와 생존 XP는 서로 다른 지급 경로이므로 기존 배율을 누락하거나 두 번 적용하지 않는다.
- reader는 원본 reward 정의와 지원되는 수량 변환 규칙을 같은 계산 경계로 검증한다.
  현행의 원본/난이도 보상 허용 범위를 유지하면서 실제 producer가 만드는 무억제 결과를 비교 대상으로 포함한다.
  arbitrary amount를 허용하거나 summary/entitlement의 동일성 검사·후보 identity 검사를 삭제하는 방식은 채택하지 않는다.
- 현재 phase의 억제력만으로 진입 배율을 역추론하지 않는다. 봉인 승리 등으로 종료 때 억제력이 변할 수 있다.
  저장된 option과 retained definition의 합법적 변환 비교를 우선 사용하며, 현재 지원 배율의 원본을 중복 상수로 만들지 않는다.
- 원칙적으로 DTO 필드·저장 버전 추가 없이 현재 유효 producer 값의 일치를 고친다.
  구체화 과정에서 새 provenance 필드나 호환성 변경이 실제로 필요하면 구현 전 작업 경계와 계약 영향을 다시 기록한다.
  이번 계획은 기존 비일치 저장을 임의 수정해 복구하는 migration 승인이 아니다. 그 복구는 별도 범위로 남긴다.

**보존 조건:** 전투 종료 시 정산, ACK 시 재지급 금지, ChooseOne 후보/선택/context 고정, claim/exit의 1회 처리,
일반 실패의 draft 폐기, fatal fault latch, 수동 후퇴·Continue 비용 차이, 원정 terminal 정리 시점.

**구현 순서와 안정된 종료점:**

1. 보존된 probe로 현재 실패와 배율 1 대조 성공을 재확인하고 같은 입력의 회귀 검증을 배치한다.
2. 최종 옵션의 단일 계산·소비와 reader 비교를 한 작업에서 바꾼다. 한쪽만 수정한 상태를 완료로 인계하지 않는다.
3. 수량 배율 1/2 × ChooseOne/ClaimAll, 수량 grant/사본 grant, 결과 확인·선택·수령 전후 Continue를 확인한다.
   기존 난이도 percent와 결합한 입력, 후보 0개, terminal pending choice, 정산 오류 rollback도 필요한 기존 테스트로 연결한다.
4. 최신 구현 diff·실행 근거를 별도 시점에 다시 읽어 완료 검토하고 README·지도에 반영한다.
   이 단계에서 새 agent를 자동 생성하지 않는다.

**수용 조건:** 정상 생성 저장의 Continue 성공, summary·지급 후보의 수량 일치, 실제 지급량 정확,
ACK/재시작/중복 claim에서 중복 지급 없음, unrelated source/DTO/저장 계약 변화 없음.
개선 효과는 보상 수량 해석이 생산·표시·복원마다 따로 구현되지 않는지로 확인한다.

## 조사 종료 시점의 다음 실행과 인계

- 다음 순서: **B1 오류 교정·관련 회귀 검증 → 별도 완료 검토 → R1 공통 후처리 통합·검증**.
  둘은 별도 원인이다. R1만 구현한 뒤 B1까지 해결된 것으로 보고하지 않는다.
- 기존 테스트 선택은 위 표의 보호 조건을 기준으로 한다. 실행 예시는
  `cargo test -p game_core --lib u04_choose_one --offline --locked`와
  `cargo test -p game_core --lib combat_result_commits_attempt_conditions_and_survival_xp_once_per_employee --offline --locked`다.
  실제 feature·target·캐시·필터의 실행 건수는 인수 시 확인하고 0개 실행을 통과로 취급하지 않는다.
- unit test build에는 현재 goals 아래 Rust/fixture 참조가 연결되어 있다. 해당 현재 검증 입력과 과거 문서의 권위를 구분한다.
  이번 probe는 library dependency만 빌드했으며 Core unit-test target 전체를 빌드·실행하지 않았다.
- 다음 재개는 이 문서의 B1 원인과 보존된 재현 도구부터다. 이후 S01/S04의 전체 저장·실행 상세 조사로 연결한다.
  S07 전투 내부 전체 또는 Core 전체 리팩토링 완료를 의미하지 않는다.

## 조사 단계 완료 검증과 남은 작업

- 선택한 대표 흐름의 정적 조사, 12개 평가 항목, 유지/오류/리팩토링/후속 분류와 작업 순서를 기록했다.
- B1은 현재 library의 공개 API로 재현했고 R1은 호출·중복 순서 분석으로 확인했다. 둘 다 아직 제품에 수정하지 않았다.
- [최종 확인](evidence/s07_001/verification.json)에서 조사 기준선과 제품 입력의 동일성, 경계 입력·문서 링크·근거 목록을 확인한다.
- 실행 중인 명령은 없다. 전체 Core unit/integration test·release build·host crash recovery는 미실행이다.
  다음 구현 작업이 필요한 회귀 검증과 별도 완료 검토를 수행한다.

## R1 구현과 완료 검토

2026-10-16 사용자 재개 요청에 따라 R1 구현·영향 회귀·별도 완료 검토를 수행했다. 이 절은 위 조사 시점 기록의 후속이다.

- 변경 전 `world/combat.rs`는 B1 검토 대상과 해시가 같았다. [R1 기준선](evidence/s07_r1_001/baseline.json)과
  [편집 전 사본](evidence/s07_r1_001/combat.rs.before)을 보존하고, B1의 condition·XP·소비품·rollback 통과 결과를 변경 전 근거로 재사용했다.
- 실제 제품 변경은 `plan_player_victory_combat_result_completion`의 세 단계 조립을
  `let mut staged = self.stage_committed_combat_resolution(battle)?;`로 바꾼 한 곳이다.
  [이번 diff](evidence/s07_r1_001/source.diff)와 [검증·대상 식별](evidence/s07_r1_001/verification.json)을 연결한다.
- helper 본문은 기존과 같다. capture → 직원 후처리 → 소비품 기간 감소의 조립 지점이 하나이고 승리·패배가 함께 사용한다.
  승리의 node completion·도감 검증 이후라는 위치, 세 단계의 호출 횟수·순서·오류 전파, clone/admission과 각 결과의 후속 지급을 보존했다.
- [성장과 보상](../game_rules/growth-and-rewards.md#xp와-보상-확정)의 직원별 생존 XP 1회·패배 지급·후퇴 미지급과
  [런타임 계약](../core_runtime_contract.md)의 소비품 기간 차감·후퇴 복원을 대조했다. 새 정책·API·저장 형식·테스트 helper는 추가하지 않았다.

변경 후 [영향 검사](evidence/s07_r1_001/regression001.log)는 **15개 통과, 실패·ignore 0개, 종료 코드 0**이다.
빌드 38.21초, 검사 21.46초이며 기존 warning 33개가 남아 있다. 선택 범위는 다음과 같다.

- 승리 condition·생존 XP, 패배 condition, 후퇴 후 소비품 유지·정산 시 소진, 정산 실패·재시도의 원자성.
- live 종료·ACK, ChooseOne 선택·수령·Continue, 빈 후보와 terminal 정리, 준비된 명령 폐기/commit·fatal fault.
- 봉인 보스 승리·억제력 보충·보너스 1회, 스트레스 전투 전체의 누적 지급.
- B1 공개 API 테스트 2개가 포함한 8개 입력: 1배/2배 × 선택형/자동 지급 × 난이도 보너스 없음/125%.

기존 테스트를 그대로 사용했다. 소비품 테스트 자체는 후퇴·패배 정산을 직접 확인하며, 승리의 소비품 호출 보존은
변경 전 세 연산과 helper 본문의 동일성 및 동일 호출 위치를 함께 대조했다. 단순 위임 변경에 구현을 복제한 새 테스트를 만들지 않았다.
전체 Core suite·release build·호스트 crash recovery는 실행하지 않았으며, 이전 B1의 55개·probe 6건을 이번 재실행 수로 더하지 않는다.

재현 명령(저장소 루트):

```bash
env PATH=/home/blast/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin CARGO_HOME=/tmp/wp02-cargo-home CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p game_core --lib --offline --locked -- combat_terminal_settlement_failure combat_result_commits_attempt_conditions combat_defeat_commits consumable_modifier_survives s07_ u04_live_terminal u04_choose_one u04_terminal_choice u04_terminal_zero u04_prepared u14_seal_retreat u164_five_rounds
```

구현·검증 종료 후 대상 소스를 수정하지 않는 별도 검토 pass에서 diff·helper 본문·두 호출 위치·원시 테스트 결과를 다시 대조했다.
유일한 변경이 위임 호출임을 확인했고 보상 배율 확정·지급·reader 비교 등 B1 교정은 유지됐다. 같은 실행자의 시점을 분리한 검토이며
비참여 외부 감사는 아니다. **판정: R1 완료.** 현재 범위의 필수 미해결과 실행 중인 명령은 없다.

다음 재개는 **S01 애플리케이션 준비·commit·오류 경계 상세 조사와 개선 계획**이다.
`application.rs::prepare_command_with_generation`, `prepare_active_battle_for_host_tick`에서 시작해
`world/application.rs::prepare_application_scoped`, `commit_application`, 준비 객체 공개/폐기와 `application/fault.rs`를 연결한다.
S07에서 확인한 전투 흐름을 재사용하고 일반 명령·거절·취소·호스트 승인 연결의 미확인을 보완한다.
S04 journal 문서/구현 버전 차이는 저장·복원 상세 조사에 인계하며, R1 완료만으로 해당 계약을 교정하지 않는다.
