# Safezone Inventory Interaction Contract — Experiments

이 문서에는 실제로 수행한 검증만 시간순으로 기록한다. 각 기록은 가설, 변경 내용, 정확한 명령
또는 runner job, 실제 test 수와 pass/fail/skip, 실제 결과, 실패 원인, 되돌림과 판단을 포함한다.

## 2026-07-17 — Production 변경 전 Core equipment/consumable baseline

- 가설: 현재 dirty worktree의 장비, 소비 아이템, 파편 및 Maintenance focused 동작에 기존 실패가 없다.
- 수행: `cargo test --lib game::world::tests::equipment -- --nocapture`
- 변경: 없음.
- 실제 test 수: 23.
- 결과: 23 pass / 0 fail / 0 ignored. 자동 조합 2개 test와 modifier overwrite test도 현재 동작으로 통과했다.
- 실패 원인/되돌림: 없음.
- 판단: 변경 후 제거/교체되는 자동 조합 기대와 새 atomic replacement 기대를 분리해 비교할 수 있다.

## 2026-07-17 — Production 변경 전 Core capability baseline

- 가설: 현재 snapshot capability test는 Safezone inventory action 누락을 기존 동작으로 고정한다.
- 수행:
  `cargo test --lib game::world::tests::snapshots_and_start::game_core_projects_context_capabilities_and_revalidates_on_state_transition -- --exact`
- 변경: 없음.
- 실제 test 수: 1.
- 결과: 1 pass / 0 fail / 0 ignored. test는 ViewingMap의 `EquipItem` capability가 없음을 assert한다.
- 실패 원인/되돌림: 없음.
- 판단: 이번 Goal이 의도적으로 교체할 baseline 계약이다.

## 2026-07-17 — Production 변경 전 live RON baseline

- 가설: 현재 live bundle은 equipment recipe를 포함한 상태로 로드되며 다른 RON baseline 실패가 없다.
- 수행: `cargo test --test ron_loading -- --nocapture`
- 변경: 없음.
- 실제 test 수: 20.
- 결과: 20 pass / 0 fail / 0 ignored.
- 실패 원인/되돌림: 없음.
- 판단: recipe schema/data 제거 후 같은 suite로 live loading 회귀를 비교한다.

## 2026-07-17 — Production 변경 전 Unity item/loadout focused baseline

- 가설: 현재 Unity는 소비 아이템을 즉시 보내고 장비 교체를 두 command로 보내는 기존 계약을 test로 고정한다.
- 수행: Unity EditMode runner job `3d96b1e4fe0443d8a3255f28f715fdb3`에서 다음 3개 test 실행.
  - `ItemUseActionHandler_DropUsesPayloadWithoutPersistingTargetSelection`
  - `ItemUseActionHandler_RejectsForeignDropPayload`
  - `LoadoutDropHandler_LiveReplacementUnequipsExistingBeforeEquip`
- 변경: 없음.
- 실제 test 수: 3.
- 결과: 3 pass / 0 fail / 0 skip.
- 실패 원인/되돌림: 없음.
- 판단: atomic single-command replacement와 overwrite confirmation으로 의도적으로 바꿀 baseline이다.

## 2026-07-17 — Core schema/runtime/capability 구현과 중간 실패

- 가설: recipe/schema/`Combined`를 제거하고 `EquipItem`을 staged replacement로 바꾸면 기존 장비와
  consumable 계약을 보존하면서 새 원자성 test가 통과한다.
- 변경:
  - equipment combination metadata/field/index/validation/live recipe 2개 제거
  - `EquipItemOutcomeDto::Combined`와 자동 grant mutation 제거
  - cloned roster/inventory 기반 점유 슬롯 replacement 및 직원 간 transfer 구현
  - Safezone inventory action capability와 동일 context handler validation 추가
  - 자동 조합 test를 replacement/transfer/atomic failure/serialized-shape test로 교체
- 실행:
  - `cargo check --lib`
  - `cargo test --lib game::world::tests::equipment -- --nocapture`
  - `cargo test --lib game::world::tests::snapshots_and_start::game_core_projects_context_capabilities_and_revalidates_on_state_transition -- --exact --nocapture`
  - `cargo test --test ron_loading -- --nocapture`
- 중간 실패:
  - 병렬 focused 첫 실행은 test helper가 중복 정의되어 compile 실패했다. 중복 helper를 제거했다.
  - capability test는 기존 `EquipItem == None` 기대 때문에 0 pass / 1 fail이었다. 승인된 새 계약에
    맞춰 5개 Safezone inventory capability의 존재/enablement 기대를 수정했다.
- 최종 결과:
  - `cargo check --lib`: 성공, test runner 수치 없음.
  - equipment focused: 24 pass / 0 fail / 0 ignored.
  - capability focused: 1 pass / 0 fail / 0 ignored.
  - live RON: 20 pass / 0 fail / 0 ignored.
- 되돌림: 자동 조합 구현/test는 승인된 제거 대상이라 복원하지 않았다.
- 판단: Core runtime, DTO, schema, live data, capability가 승인 정책으로 정렬됐다.

## 2026-07-17 — Removed recipe strict rejection 검증

- 가설: 구 equipment RON의 `recipes` field는 alias나 silent ignore 없이 startup parse에서 거부된다.
- 변경: `removed_combination_recipes_field_is_rejected` test 추가.
- 첫 실행:
  `cargo test --lib game::data::equipment_data::tests::removed_combination_recipes_field_is_rejected -- --exact`
- 첫 결과: compile fail. 현재 ron crate의 함수가 `ron::from_str`이 아니라 `ron::de::from_str`이었다.
- 수정: 정확한 import로 교체.
- 재실행: 같은 명령.
- 최종 실제 test 수: 1.
- 결과: 1 pass / 0 fail / 0 ignored.
- 판단: 승인되지 않은 recipe compatibility가 남지 않는다.

## 2026-07-17 — Unity atomic equip와 consumable overwrite confirmation 구현

- 가설: Unity가 replacement/transfer를 `EquipItem` 하나로 보내고 active modifier가 있을 때만 modal을
  열면 partial mutation과 무경고 손실을 함께 막을 수 있다.
- 변경:
  - `LoadoutDropHandler`의 선행 unequip command 제거
  - active modifier typed reader와 pending confirmation selection state 추가
  - ItemUse composer/modal action, Shell dispatch, confirm/cancel handler 연결
  - modifier 없음 즉시 전송, modifier 있음 confirm/cancel, replacement/transfer single-command tests 추가
- compile: `mcp__unity__refresh_unity(mode=if_dirty, scope=scripts, compile=request, wait_for_ready=true)`.
  Console을 비운 뒤 최종 compile 확인에서 error/warning 0건.
- focused 최종 runner job: `4cd6562ef5ee46ea849b70ccb58ea386`.
- 실제 test 수: 6.
- 결과: 6 pass / 0 fail / 0 skip.
- 중간 runner 참고: job `58b2b7e01342470fa13001d69d59114b`는 status succeeded와 progress 6/6,
  failure 0을 반환했지만 summary result가 null이라 수치를 완료 증거로 사용하지 않고 최종 job을 재실행했다.
- 판단: Unity client-local confirmation과 Core-authoritative mutation 경계가 분리됐다.

## 2026-07-17 — Broad Core와 game_server 검증

- 가설: schema/DTO/runtime migration이 다른 gameplay와 server bridge에 새 회귀를 만들지 않는다.
- 실행:
  - Core: `cargo test --all-targets --quiet`
  - game_server: `cargo test --all-targets --quiet`
- 실제 결과:
  - Core non-zero suites 합계 755 pass / 0 fail / 0 ignored. lib 711, integration suites
    3 + 4 + 20 + 13 + 4.
  - game_server 30 pass / 0 fail / 0 ignored. 추가 target은 0 tests였다.
- 실패 원인/되돌림: 없음.
- 판단: 접근 가능한 Core producer와 server consumer가 같은 새 계약을 사용한다.

## 2026-07-17 — Broad Unity 검증

- 가설: 새 constructor/modal/action enum과 atomic drop 변경이 전체 client 계약 및 render/input 경계를
  깨뜨리지 않는다.
- 실행:
  - EditMode job `4f2e5818a6d0418391992261c05849fe`
  - PlayMode job `788df16550a34e18ab2f3f3b9c660978`
- 실제 결과:
  - EditMode 231 pass / 0 fail / 0 skip.
  - PlayMode 55 pass / 0 fail / 4 explicit-live skip, 총 59.
- skip: live Event admin, live Maintenance admin, live combat, live endless briefing 네 test로 이번
  Safezone inventory 완료 조건과 직접 관련 없다.
- 실패 원인/되돌림: 없음.
- 판단: 일반 Unity 전체 suite에 새 실패가 없다.

## 2026-07-17 — 실제 game_server WebSocket Safezone inventory probe

- 가설: 실제 producer snapshot이 Safezone capability를 내보내고 wire command가 replacement/transfer와
  consumable overwrite를 새 계약대로 수행한다.
- server 기동:
  `ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run --quiet`
  (`config/development.toml`의 `ws://127.0.0.1:8081/game` 사용).
- probe: 기존 WebSocket framing/helper를 import한 inline Python 진단으로 auth → StartNewGame → HQ VN
  advance → starter selection → admin equipment/consumable grant → command를 실행했다.
- 실제 test 수: test runner가 아닌 단일 live scenario이므로 runner 수치를 보고하지 않음.
- 결과: exit 0.
  - `ViewingMap`에서 `EquipItem`, `UnEquipItem`, `EquipSkillFragment`, `UnequipSkillFragment`,
    `UseConsumableItem` 5개 capability 존재
  - 같은 type 장비 replacement result `EquipItem`, `Combined`/ingredient field 없음
  - 기존 장비는 owned 상태로 가방 복귀, 새 장비는 target에 장착
  - 같은 장비를 두 번째 직원으로 한 command에 이동
  - 두 번째 consumable 사용 result `ConsumableItemUsed`, `replaced_modifier` 존재
- 종료: Ctrl+C graceful shutdown 후 8081 listener가 남지 않음을 확인했다.
- 판단: mock끼리의 일치가 아니라 실제 Core→server wire 경계를 증명했다.

## 2026-07-17 — Formatting, scoped diff와 legacy hygiene

- 실행:
  - `cargo fmt --all -- --check`
  - Core/game_resources/Unity 이번 대상 파일에 한정한 `git diff --check -- <scoped files>`
  - production/canonical 전체에서 combination symbol과 old Unity two-command message 검색
- 결과:
  - format check 성공.
  - scoped diff check exit 0.
  - active production/canonical에서 recipe type/lookup/`Combined`/ingredient result symbol 0건.
  - 전체 Unity `git diff --check`는 이번 범위 밖 기존 CRLF `.gitignore`/IDE 파일 whitespace를
    보고했으므로 완료 증거로 사용하지 않았다. 해당 사용자 파일은 수정하지 않았다.
- 판단: 이번 변경 범위에는 whitespace 오류와 제거 대상 legacy가 남지 않았다.
