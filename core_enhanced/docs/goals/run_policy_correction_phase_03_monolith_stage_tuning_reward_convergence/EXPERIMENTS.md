# Monolith Stage Tuning And Reward Convergence Experiments

Status: complete

Implementation complete: true

Created: 2026-07-28

## 2026-07-28 — Child Goal 초기화

- Master P00-P02 correction completion reviews: 모두 `Complete / High`.
- P03 child 경로는 생성 전에 존재하지 않았다.
- Master P-03이 exact stat/rounding/qualitative tier를 승인해 policy question은 없다.
- Core/RON/canonical과 Unity reader/presentation/test 경로는 기존 campaign 변경과 겹친다.
- Production code/schema/Unity 변경과 P03 test 실행은 아직 없다.
- 다음 작업: child 문서 완독, current end-to-end authority/rounding/consumer와 scoped diff 조사.

## 2026-07-28 — 변경 전 authority/rounding 기준선

- `cargo test -p game_core monolith --lib -- --nocapture`: 9 passed, 0 failed.
- 현재 difficulty에는 typed monolith stage가 없고 ledger만 `stage: u8`를 저장한다.
- conversion은 stage를 target power에만 반영하고 preview/setup stat은 공통 Monolith role tuning을 사용한다.
  그 결과 stage 2/3 모두 HP 165%, attack/defense 145%, wave budget 200%이다.
- reward는 battle start에서 role 250%를 반올림하고 victory completion에서 stage 100/125%를 다시
  반올림한다. 원수량 1의 stage 3 결과는 현재 4이며 승인된 one-pass 313% 결과 3과 다르다.
- Rust DTO와 Unity reader/presenter가 exact `reward_multiplier_percent`를 노출한다.
- Unity MCP active instance `ark@b49d32f6f2f70bbe`에서 관련 EditMode 2건을 실행했고
  2 passed, 0 failed를 확인했다.
- 다음 작업: typed stage와 role+stage 단일 resolver를 추가하고 target/stat/reward의 모든 consumer를
  해당 resolver로 수렴시킨다.

## 2026-07-30 — 구현 및 focused 회귀

- `MapNodeCombatDifficulty.monolith_stage`를 required `Option<Stage2|Stage3>` typed authority로 고정하고,
  Monolith role/stage/ledger 조합이 정확히 일치하지 않으면 live preview/combat와 checkpoint load/continue가
  fail closed하도록 확인했다.
- `RunPolicyData::resolved_tuning_for_manifested_node`가 role과 stage를 먼저 합성한다. Stage 2는
  target 220%, HP 165%, attack/defense 145%, reward 250%; stage 3은 target 275%, HP 207%,
  attack/defense 182%, reward 313%다. 두 stage 모두 generated-wave budget은 200%로 동일하다.
- reward는 원수량에 composite percent를 한 번만 nearest/minimum-one 적용한다. 원수량 1, stage 3은 3이다.
- Core wire와 Unity UI는 `enhanced`/`final_enhanced` qualitative tier만 노출한다. Unity reader는 새 tier와
  legacy `reward_multiplier_percent`가 함께 존재하는 mixed payload도 거부한다.
- checkpoint export JSON, 정상 continue, checkpoint load와 active-run transaction rollback이 Stage2를
  보존하고, stage 누락 및 ledger Stage2/difficulty Stage3 mismatch continue가 거부되는 새 회귀를 추가했다.
- event-started combat retry의 cached authoritative preview가 Event category gate 뒤에서 무시되던 broad
  regression을 발견했다. 캐시를 먼저 재사용하도록 좁게 보정했고 기존 패배 재진입 2건이 통과했다.
- `cargo test -p game_core monolith --lib -- --nocapture`: 17 passed, 0 failed.
- `cargo test -p game_core --test ron_loading -- --nocapture`: 23 passed, 0 failed.
- Unity MCP focused EditMode: 3 passed, 0 failed.

## 2026-07-30 — broad 검증 및 완료 판정

- `cargo fmt --all -- --check`: passed.
- `cargo check -p game_core -p game_server`: passed.
- `cargo test --workspace --all-targets -q`: passed. Core lib 798/798와 나머지 all-target suites
  (3, 4, 23, 13, 4, 54 tests) 모두 0 failed.
- Unity MCP EditMode: 260 passed, 0 failed, 0 skipped.
- Unity MCP PlayMode: 62 passed, 0 failed, 4 explicit live-server skips.
- Unity compile/console: changed scripts diagnostics 0 errors; Editor console error 0.
- Exact reward percent search는 internal policy/tuning 및 negative rejection tests만 남고 DTO/presenter의
  exact projection은 없다.
- `COMPLETION_REVIEW.md`: `Complete / High`.
