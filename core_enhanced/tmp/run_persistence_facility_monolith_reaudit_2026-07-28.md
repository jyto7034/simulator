# Run Persistence / Facility / Monolith 재감사 임시 메모

Status: audit complete; policy approved; correction pending

Date: 2026-07-28

Policy approval: 2026-07-28 사용자 승인 — R-02, R-03, R-04 권고안을 수정 구현의 확정 정책으로 채택

Legacy save approval: 2026-07-28 사용자 승인 — 개발 단계이므로 과거 save schema 호환·migration은
제공하지 않으며 새 정책 수립 시 구 구조와 호환 경로를 제거

이 파일은 `run_persistence_facility_monolith_policy_master`와 P00~P05/P07의 완료 주장을 현재
runtime code, live RON/data, game_server, 외부 Unity consumer 및 실행 검증과 다시 대조한 임시 감사
문서다. Gameplay/runtime/data/wire 정책의 canonical이 아니며, 보정 작업이 끝난 뒤 폐기하거나 정식
review 문서로 옮긴다.

## 감사 원칙

- 증거 우선순위는 runtime code, live data, 실제 DTO/consumer, durable canonical, goal 기록 순이다.
- 완료 문서와 기존 test 결과는 증거 포인터이지 현재 구현의 증명이 아니다.
- phase별 producer, validation, serialization, consumer와 rollback/duplicate/legacy 경계를 확인한다.
- 이상하거나 통념적이지 않은 정책은 구현 불일치와 분리해 `정책 검토 메모`로 기록한다.
- 기존 dirty worktree 변경은 사용자 소유로 간주하고 수정하거나 되돌리지 않는다.

## 시작 상태

- Master `PLAN.md`: `Status: complete`, `Implementation complete: true`.
- Registry: P00~P05 및 P07은 `Complete / High`; hidden-boss P06은 현재 master 밖 후반 콘텐츠다.
- 현재 scoped worktree는 Core, live data, game_server, canonical 전반에 대규모 tracked/untracked 변경이
  있다. 이번 감사는 이 상태를 현재 기준선으로 읽는다.
- 임시 메모 외 production code/data/document 수정은 감사 범위가 아니다.

## Phase 판정표

| Phase | 주장 | 현재 판정 | 핵심 증거/불일치 |
| --- | --- | --- | --- |
| P00 | SavePoint/Gate Trauma 및 Support/Rest 제거 | 구현 일치 / 기록 보정 필요 | tagged `TraumaRecovery`, live 15%/10%, alive-only recovery, 원자 완료와 strict Unity reader가 현재 코드에 있다. 단, P00 `EXPERIMENTS.md`와 `EXPERIMENT_NOTES.md` 머리말은 여전히 active/false다. |
| P01 | 실패/checkpoint/durable active-run | 구현 일치 | recovery restriction, failure 사전 선택, 최대 3회 load, 마지막 SavePoint payload, version/checksum/current+backup/single-writer/invalidation과 서버 rollback이 실제 구현·시험에 있다. |
| P02 | local-install 영구 도감 | **부분 불일치 / 수정 필요** | 결과 소비 시점의 monotonic discovery와 별도 파일 저장은 구현됐다. 그러나 일반 non-battle transaction 실패가 Core의 in-memory 도감을 기본값으로 되돌릴 수 있다(F-01). |
| P03 | authored facility/assignment/manifestation | 구현 일치 | live 7개 template, Standard/Endless assignment schedule, 방향 edge, concealed manifestation, exact Core/Unity template contract가 일치한다. |
| P04 | certified power/Floor difficulty | 구현 일치하나 정책 충돌·재검토 필요 | difficulty/role stat·wave·reward 소비와 비되감기 peak/anchor는 구현됐다. 다만 최초 decision의 “실제 출전 가능 직원”과 후속 canonical/runtime의 “살아 있는 전원”이 충돌한다(P-01). |
| P05 | Elite monolith/remanifestation/exchange | **부분 불일치 / 수정 필요** | 자연 Elite ledger, Shop, conversion, attempt, unique fragment, residual/exchange/dormant key는 구현됐다. `stage_3_stat_multiplier_percent`는 실제 enemy stat/wave에 적용되지 않는다(F-02). |
| P07 | cross-component 최종 감사 | 기존 Complete/High 판정 무효화 | P02와 P05 material gap 및 P00 기록 불일치 때문에 현재 상태를 `Complete / High`로 유지할 수 없다. |

## 구현 불일치 및 결함

### F-01 — High — 실패한 일반 transaction이 local compendium을 비울 수 있음

문서 계약:

- local compendium은 active-run/checkpoint rollback 밖의 local-install monotonic profile이다.
- 실패한 command나 run rollback이 discovery/defeat를 되감거나 지우면 안 된다.

현재 코드:

- `src/game/world/state.rs:131-157`의 `clone_non_battle_transaction_state()`는 rollback 사본의
  `encounter_compendium`을 의도적으로 `default()`로 만든다.
- active-run host rollback인 `src/game/world/active_run.rs:40-54`는 복구 직전 live compendium을 따로
  보관하고 복구 뒤 merge하므로 이 설계와 맞는다.
- 반면 공통 `src/game/world/node_flow.rs:793-808`의 `with_state_transaction()`은 실패 시 그 default
  사본을 `self.state`에 그대로 대입하며 profile 보존/merge를 하지 않는다.
- 이 helper는 node confirm/completion, checkpoint load, monolith conversion, combat-result completion 등
  여러 command가 사용한다. 따라서 도감이 채워진 상태에서 해당 operation이 `Err`를 반환하면 그
  command가 도감을 건드리지 않았더라도 Core 메모리 값이 빈 profile로 바뀔 수 있다.

영향 경계:

- 서버는 다음 정상 player behavior 시작 전에 persistence profile을 merge하므로 보통 다음 command에서
  복구된다. 저장소 자체도 full-union monotonic merge여서 디스크 진행은 지워지지 않는다.
- 하지만 실패 직후 같은 actor에 reconnect하면 `AttachSession`은 먼저 sync하지 않고 snapshot을 만든다
  (`game_server/src/game/player_game_actor/handlers.rs:26-50`). 이때 빈 도감 snapshot이 노출될 수 있다.
- Core 직접 consumer에서는 다음 server sync라는 완화가 없으므로 contract 위반이 그대로 보인다.
- 기존 P02/서버 test는 active-run persistence 실패 전용 rollback merge는 검증하지만 이 공통 helper의
  오류 rollback은 직접 고정하지 않는다.

권장 보정:

- `with_state_transaction()`도 restore 직전 live compendium을 보존해 rollback 뒤 merge하거나,
  rollback snapshot이 profile을 복제하도록 하되 active-run payload 직렬화 경계와 혼동하지 않게 한다.
- 최소 회귀 test는 non-empty profile에서 의도적으로 실패하는 일반 transaction, 실패 직후 snapshot,
  같은 actor reconnect의 세 경계를 포함한다.

### F-02 — High — Monolith stage 3 stat 125%가 실제 전투 난이도에 적용되지 않음

문서/데이터 계약:

- P05 `EXPERIMENTS.md:128-129`와 `COMPLETION_REVIEW.md:43-47`은 stage stat/reward scaling이 P04
  `MonolithRemanifestation` role과 합성되고 stage 3이 stage 2보다 엄격히 강하다고 주장한다.
- live `game_resources/data/run/policy.ron:41-44`는 stage 2 stat/reward 100%, stage 3 125%를 author한다.

현재 코드:

- 변환 시 `src/game/world/node_flow.rs:553-577`은 difficulty class를 동일한
  `Role(MonolithRemanifestation)`으로 만들고 `target_power`에만 stage stat multiplier를 곱한다.
- 실제 preview/battle enemy stat 및 generated wave는
  `src/game/combat_preview/mod.rs:407-432`에서 `tuning_for_manifested_node()` 결과를 사용한다.
- `src/game/data/run_policy_data.rs:772-792`의 lookup은 difficulty class만 보고 role tuning을 돌려주며
  `target_power`나 monolith stage를 읽지 않는다. 따라서 stage 2와 3 모두 실제 health 165%, attack/
  defense 145%, wave 200%로 동일하다.
- `target_power`의 runtime consumer는 `src/game/world/node_flow.rs:202-258`의 상대 위협 band projection뿐이다.
  기존 `stage_three_target_power_is_strictly_above_stage_two` test도 이 표시용 수치만 비교한다.
- reward는 role 250%가 battle reward에 먼저 적용되고, 승리 처리에서 stage 100%/125%가 다시 적용되므로
  reward 쪽 합성은 구현되어 있다.

영향:

- stage 3은 UI 상대 위협과 보상은 올라가지만 실제 적 stat/wave는 stage 2와 같다.
- 문서의 핵심 progression claim과 다르고 현재 test suite가 이 gap을 검출하지 못한다.

권장 보정:

- manifested difficulty가 stage-specific combat tuning authority를 보존하도록 typed field/class를 두거나,
  preview 생성 시 validated remanifestation stage modifier를 stat/wave tuning에 합성한다.
- stage 2/3의 생성된 enemy health/attack/defense와 wave budget을 직접 비교하는 회귀 test가 필요하다.

### D-01 — Low — P00 goal 기록 머리말이 완료 상태와 불일치

- `PLAN.md`는 complete/true이고 completion review는 Complete/High다.
- 같은 폴더의 `EXPERIMENTS.md:3-8`은 `Status: active`, `Implementation complete: false`, “아직 production
  구현이나 completion review를 수행하지 않았다”고 남아 있다.
- `EXPERIMENT_NOTES.md:3-5`도 active notes/false다.
- runtime 결함은 아니지만 phase registry와 감사 자동화/사람의 상태 판독을 서로 다르게 만든다.

## 정책 검토 메모

구현이 현재 canonical과 일치하더라도 플레이어 경험, 저장 안정성, 일반적인 시스템 설계 관점에서
재검토할 만한 항목이다.

### P-01 — Certified power의 “실제 출전 가능” 의미가 source 간 충돌함

- 최초 승인 decision `combat_power_floor_difficulty_policy_draft_2026-07-26.md:31-40`은 전투 진입 때
  **실제 출전 가능한 직원**만 계산하고 Breakdown/출전 가능 인원은 readiness로 분리한다고 썼다.
- 후속 P04 notes와 현재 rulebook은 살아 있지만 Critical 또는 Breakdown인 직원도 raw certified power에
  남기는 것으로 바뀌었다. runtime도 `src/game/combat_power.rs:35-40`에서 `is_alive()`만 필터링한다.
- 결과적으로 실제 전투에 한 명도 배치할 수 없어 `NoDeployableEmployees`인 roster도 상당한 certified
  power를 가질 수 있고, node 상대 위협이 실제 체감보다 낮게 표시될 수 있다.
- “현재 HP/trauma를 raw 장기 전력에서 빼고 readiness로 보낸다”는 분리는 타당하지만, 출전 불가능한
  인원의 장비/stat까지 현재 전투력 합계에 더하는 것은 통상적인 battle-ready power와 다르다.
- 어느 의미가 최종 승인인지 명시적으로 결정하고 decision/canonical/runtime를 한 방향으로 맞춰야 한다.

### P-02 — 비되감기 current Floor anchor는 과거 checkpoint 재진행 때 난이도를 누적 상승시킴

- `RunPowerAuthorityState::manifest_floor()`는 현재 anchor에 minimum rise를 더하고, checkpoint restore는
  anchor를 복원하지 않는다. rulebook도 peak와 current Floor anchor가 load로 감소하지 않는다고 명시한다.
- 예: Floor 4 SavePoint 이후 Floor 5로 넘어가 anchor가 오른 뒤 Floor 4 checkpoint를 load하면 map은
  Floor 4로 돌아가지만 anchor는 Floor 5 값이다. Gate를 다시 확정하면 그 높은 값에 최소 상승분을 한 번
  더한다. 최대 3회 load 제한만큼 같은 구간 재진행으로 future anchor가 추가 상승할 수 있다.
- 약화 load exploit을 막는 anti-rewind 의도는 분명하지만, “현재 Floor anchor”라는 이름과 달리 map Floor와
  서로 다른 세대의 값이 공존하며 재진행 자체가 난이도 penalty가 되는 비통상 정책이다.
- 의도라면 rulebook/UX에 penalty를 명시하고 cross-floor rewind/re-advance test를 추가하는 편이 안전하다.
  의도가 아니라면 run peak만 비되감기로 두고 checkpoint-owned floor anchor를 별도로 복원해야 한다.

### P-03 — Monolith reward preview가 stage 배율만 보여 실제 합성 배율을 오해하기 쉬움

- 실제 quantity reward는 P04 role 250%를 적용한 뒤 P05 stage 100%/125%를 다시 적용한다.
- Unity-facing remanifestation DTO와 화면의 `Reward multiplier`는 stage 값 100%/125%만 표시한다.
- exact internal power를 숨기는 P04 원칙과는 별개로, “125%”가 총배율인지 추가배율인지 표기가 없다.
  `Stage bonus 125%`처럼 의미를 명시하거나 최종 reward tier/band를 투영하는 편이 통념적이다.

### P-04 — local-install singleton은 계정/사용자 경계가 아님

- active run과 compendium은 transport player UUID가 아니라 설치 단위 singleton이다. 서버 test도 서로 다른
  transport player가 같은 도감을 공유함을 고정한다.
- 오프라인 단일 사용자 제품이면 일관된 선택이지만, 로그인/다중 프로필을 기대하는 배포에서는 계정 간
  진행 공유 및 active-run 교체가 놀라운 동작이다. 제품 경계를 README/타이틀 UX에 명시할 필요가 있다.

### P-05 — corrupted monolith는 의도적으로 consumer 없는 보상 key임

- residual 네 개 교환은 `corrupted_monolith_acquired`를 만들지만 P06 hidden boss가 master 범위 밖이라
  현재 consumer가 없다. rulebook은 이를 dormant key라고 명시하므로 구현 누락으로 판정하지 않았다.
- 플레이어에게 완결 보상처럼 제시된다면 당장은 dead-end다. 후속 콘텐츠 전에는 “향후 사용” 상태를
  명확히 투영하거나 노출 정책을 재검토할 가치가 있다.

## Legacy / fallback 검색

- production Core/server/live data에 P00의 `Support/Rest` node/command producer는 없다. 남은 `support`는
  HeadquartersContact의 별도 도메인 용어이고, `choose_support Rest`는 제거 계약의 negative deserialize
  test다.
- P03 구 procedural facility generation policy/old depth-lane authority는 official facility runtime에서
  발견되지 않았다. `battlefield_generation_policy`는 전투장 생성의 별도 시스템이다.
- P04 `floor_combat_scaling`/`FloorCombatScaling` production consumer는 없다. 관련 이름은 compatibility
  rejection 또는 과거 기록에만 남아 있다.
- P06 hidden-boss/capstone consumer는 없고 corrupted monolith key만 dormant 상태다. 현재 master의 명시적
  제외 범위와 일치한다.
- Unity map/checkpoint/compendium reader는 exact-field validation을 사용하며 legacy alias나 임의 좌표
  fallback을 확인하지 못했다.

## 실행 검증

- `cargo test --workspace --all-targets`: exit 0.
  - `game_core` library 787 passed.
  - 현재 integration suites(포함: RON loading 23, skill runtime contract 13, skill suite 4) 모두 통과.
  - `game_server` library 52 passed.
- Unity MCP instance: `ark@b49d32f6f2f70bbe`, Unity `6000.3.4f1`, idle/ready 상태에서 실행.
  - broad EditMode job `251d8ae8a412449b88938067f575a7f4`: 259 passed, 0 failed, 0 skipped.
  - broad PlayMode job `35485845406d4f9fa916b3a4a813526c`: 62 passed, 0 failed, 4 explicit skipped.
  - skip 4개는 실행 중인 `game_server`/admin-enabled server가 필요한 live integration test다.
- 위 broad suite 통과는 F-01/F-02를 반증하지 않는다. 두 결함을 직접 고정하는 test assertion이 없고,
  static call/data flow상 현재 구현이 문서 계약과 어긋난다.

## 최종 결론

현재 구현은 P00, P01, P03의 핵심 계약과 P04의 후속 canonical 대부분을 충족한다. P02와 P05에는 각각
profile rollback 격리와 stage-specific 실제 전투 scaling이라는 material gap이 있어 master/P07의
`Complete / High` 최종 판정은 현재 코드 기준으로 유지할 수 없다.

수정 우선순위:

1. F-01 공통 transaction rollback에서 compendium 보존 및 reconnect regression test.
2. F-02 stage 3 stat/wave 실제 합성 및 생성된 전투 수치 비교 test.
3. P04 certified-power source-of-truth 결정과 P00 stale goal header 정리.
4. anchor rewind penalty, reward 배율 표기, local-install singleton/dormant reward의 제품 정책 확인.

이번 감사에서는 이 임시 메모만 작성했고 production code/data/canonical은 변경하지 않았다.

## 수정 전 정책 결정 검토

### 정책 결정 상태 요약

| 항목 | 사용자 정책 결정 | 검토 결론 |
| --- | --- | --- |
| F-01 compendium rollback | 불필요 | 기존 P02 정책이 이미 “run transaction과 분리된 monotonic union”으로 확정한다. live profile 보존이 유일하게 일관된 수정이다. |
| F-02 monolith stage stat | **승인 완료** | R-02: target/HP/attack/defense에만 stage 배율을 한 번 합성하고 wave에는 적용하지 않는다. |
| P-01 certified power 대상 | **승인 완료** | R-03: 현재 combat-eligible 전체 roster만 합산하고 run peak는 계속 비되감기로 유지한다. |
| P-02 Floor anchor rewind | **승인 완료** | R-04: current Floor anchor는 checkpoint-owned, loads와 run peak만 monotonic으로 유지한다. |
| P-03 reward preview | **승인 완료** | R-02에 포함: reward는 한 번 합성·반올림하고 exact percentage 대신 qualitative tier를 투영한다. |
| P-04 local singleton | 현재는 불필요 | 승인 decision이 설치 단위 하나로 명시한다. 계정/다중 profile 제품 요구가 생길 때 별도 migration goal로 다룬다. |
| P-05 dormant key | 현재는 불필요 | P06 전까지 consumer 없음이 명시적 범위다. 오해 방지 표기만 UX 보정 후보로 남긴다. |
| D-01 P00 stale header | 불필요 | 사실관계에 맞춘 기록 보정이다. |

### R-01 — F-01 rollback 정책: 기존 monotonic profile 계약을 그대로 적용

확정 정책:

- 실패한 non-battle/run transaction은 active-run state만 이전 상태로 복원한다.
- transaction 실행 중 이미 Core가 commit한 local compendium discovery/defeat union은 보존한다.
- rollback 사본의 profile과 live profile 중 하나를 선택하지 않고 set union으로 merge한다.
- command 시작 전 profile도 사라지지 않고, 결과 소비 뒤의 discovery도 active-run persistence 실패 때문에
  사라지지 않는다.
- server는 AttachSession snapshot 전에도 persistence profile을 Core에 merge해 메모리와 파일 authority가
  항상 수렴하도록 한다.

이 결정은 새로운 gameplay 정책이 아니라 P02 `decision` 2~3절의 누락 구현을 복구하는 것이다.

### R-02 — F-02 Monolith stage 합성 정책 — 승인 완료

검토한 선택지:

1. `target_power`만 높인다.
   - 현재 구현이다. 실제 전투가 강해지지 않으므로 기각한다.
2. stage multiplier를 stat과 generated-wave budget 모두에 적용한다.
   - stage 3에서 적 개체 수/구성과 개별 stat이 동시에 뛰어 이중 난이도 상승이 된다.
   - field 이름과 decision은 `stat multiplier`이고 wave multiplier는 P04 role이 별도로 소유하므로 기각한다.
3. stage multiplier를 target power와 enemy health/attack/defense에만 적용한다.
   - 문서의 “더 높은 stat profile”과 P04/P05 ownership을 모두 만족한다.
   - **승인된 채택안이다.**

확정 세부 계약:

- Stage 2: P04 `MonolithRemanifestation` role의 target/stat/wave/reward에 stage stat 100%, reward 100%를
  합성한다.
- Stage 3: role target과 enemy max health/attack/defense에 stat 125%를 합성한다. Generated-wave budget은
  role의 200%를 그대로 유지한다.
- 현재 live 값의 최종 stat scale은 stage 2가 health 165%, attack/defense 145%, stage 3이 health 207%,
  attack/defense 182%다. 결합 percentage는 `ceil(role_percent * stage_percent / 100)`으로 한 번 계산하고
  base stat에 한 번 적용해 중간 이중 반올림을 피한다.
- `MapNodeCombatDifficulty`에는 typed source modifier
  `MonolithRemanifestationStage { stage }`에 해당하는 authority를 보존한다. Resolver 하나가 role과 stage를
  합성하고 target projection, combat preview와 battle setup이 같은 resolved tuning을 소비한다.
- monolith ledger mapping과 difficulty modifier의 node/stage가 다르면 compatibility fallback 없이
  checkpoint/live-state validation error로 처리한다.
- live stage/role 수치가 바뀌면 `power_model_version`도 반드시 올린다. 구 진행 run을 새 계수로 조용히
  재스케일하지 않는다.

Reward 합성/표시 확정 계약:

- 내부 수량형 reward는 role 250%와 stage 100%/125%를 하나의 유리수 배율로 합성해 원래 authored
  quantity에 **한 번** 반올림한다. 현재처럼 role 반올림 후 stage를 다시 반올림하면 소량 보상이
  명목 312.5%보다 크게 튈 수 있다.
- 기존 reward 정책의 nearest rounding과 최소 1 규칙은 최종 한 번에만 적용한다.
- P04는 exact multiplier를 Unity에 공개하지 않는다고 규정한다. 따라서 현재 `Reward multiplier 125%`는
  제거하고 `강화 보상`/`최종 강화 보상` 같은 qualitative tier를 Core가 투영한다.
- stage identity와 stage 3이 더 보상적이라는 상대 의미는 표시하되 250%/312.5% 내부 수치는 숨긴다.

### R-03 — P-01 Certified power 대상 — 승인 완료

검토한 선택지:

1. 살아 있는 전원을 합산하고 Critical/Breakdown을 readiness warning으로만 표시한다.
   - 현재 구현이다. 장기 잠재력은 안정적이지만 전원 출전 불가여도 높은 current certified power와 낮은
     relative threat를 만들 수 있다.
2. 모든 생존 전력과 현재 출전 가능 전력을 두 개의 독립 power authority로 유지한다.
   - 표시 문제는 해결하지만 산식/version/persistence 의미가 중복되고 P04의 단일 authority 원칙을
     불필요하게 복잡하게 만든다.
3. combat-eligible roster만 certified power에 포함하고, 개별 eligible 직원의 현재 HP/trauma 수치는
   산식에서 제외한다.
   - 최초 승인 decision과 “전투 진입에 실제 사용할 수 있는 persistent loadout” 의미에 맞는다.
   - **승인된 채택안이다.**

확정 세부 계약:

- `physical_damage <= 4 && trauma < 100 && alive`인 현재 combat-eligible 직원만 합산한다.
- Minor/Major damage, HP와 trauma의 정확한 수치는 power coefficient로 쓰지 않고 readiness에 둔다.
- Critical/Breakdown/Dead는 current certified 합계에서 제외한다. 별도 warning은 계속 투영한다.
- `run_peak_certified_power`는 지금까지 combat entry에서 기록한 최댓값이므로 직원 부상/Breakdown/사망이나
  checkpoint load로 내려가지 않는다. 따라서 손실로 다음 Floor가 자동 약화되는 exploit은 생기지 않는다.
- live deployment 방식상 전투 시작 전에 고정 squad를 고르지 않으므로 “실제 출전 직원”은 battle에
  deploy 가능한 전체 roster를 뜻한다. 실제로 전장에 배치된 subset만 사후 계산하지 않는다.
- current relative threat도 같은 eligible certified total을 denominator로 사용한다. 0이면 `VeryHigh`와
  `NoDeployableEmployees`를 함께 투영한다.

### R-04 — P-02 Checkpoint와 Floor anchor 소유권 — 승인 완료

검토한 선택지:

1. `current_floor_power_anchor`를 run-wide high-water로 유지한다.
   - 현재 구현이다. load 때마다 이전 Floor 재진행이 미래 난이도를 추가 상승시키는 명시적 penalty가 된다.
   - 이 penalty가 의도된 것이 아니라면 이름, UX와 checkpoint 기대에 어긋난다.
2. `run_peak_certified_power`만 비되감기로 유지하고 current Floor anchor는 checkpoint-owned manifestation
   state로 복원한다.
   - 같은 checkpoint branch를 다시 진행하면 같은 Floor 난이도를 얻고, 더 강한 장비를 실제 전투에서
     사용해 기록한 run peak만 anti-rewind high-water로 남는다.
   - **승인된 채택안이다.**
3. current anchor와 별도 `max_manifested_anchor`를 모두 보존한다.
   - 다시 high-water penalty를 도입하면서 authority만 늘리므로 기각한다.

확정 세부 계약:

- checkpoint payload가 `power_model_version`과 checkpoint Floor의 `current_floor_power_anchor`를 명시적으로
  소유한다. Map node의 각 difficulty에 있는 anchor만 역추론하지 않는다.
- load/continue는 checkpoint anchor와 map/node difficulty를 함께 복원한다.
- `loads_used`와 `run_peak_certified_power`만 checkpoint 밖 monotonic metadata로 유지한다.
- 다음 Floor anchor는 복원된 current anchor + authored rise, authored minimum, 비되감기 run peak의
  최댓값으로 다시 계산한다. 같은 branch 재진행은 원래와 같은 값을 만든다.
- active-run backup이 이전 checkpoint 세대로 복구되면 그 checkpoint의 anchor를 사용하고, run peak와
  loads만 current/backup 최대값으로 merge한다.
- active-run envelope/core payload schema version을 올리고 구 version은 explicit unsupported로 처리한다.
- 현재는 개발 단계이므로 구 save 보존, one-shot migration, dual-read/write, alias/default compatibility를
  구현하지 않는다. 현재 환경에 남은 개발용 구 save 파일은 reset/제거 대상이다.

### R-05 — 제품 경계 항목

- Local singleton: 현재 문서는 Standard/Endless, 모든 transport player가 설치 단위 하나의 active run과
  compendium을 공유한다고 명시한다. 현재 제품이 로컬 단일 사용자라면 유지한다. 계정 또는 게임 내부
  profile 선택이 제품 요구가 될 때만 namespace, 기존 파일 귀속, 충돌·migration을 별도 goal로 결정한다.
- Dormant corrupted monolith: P06 전까지 consumer가 없는 것이 승인 경계다. 지금 hidden boss를 붙이지
  않고 UI에 `현재 run에서는 추가 용도 없음` 또는 future-content 표기를 제공하는 수준만 권고한다.

### 사용자 승인 기록

2026-07-28 사용자 응답으로 다음 세 묶음을 수정 구현의 확정 정책으로 승인했다.

1. **Monolith:** stage stat은 target/HP/attack/defense에만 적용, wave에는 미적용; role+stage는 한 번 합성·
   반올림; exact reward percentage는 qualitative tier로 교체.
2. **Certified power:** 현재 combat-eligible 전체 roster만 합산하고 run peak는 계속 비되감기.
3. **Checkpoint:** current Floor anchor는 checkpoint-owned, `loads_used`와 run peak만 monotonic; save schema는
   version bump/구 version explicit rejection이며 개발용 구 save는 reset/제거.

승인 후 구현 원칙:

- 위 세 계약을 다시 선택지로 열지 않고 production code, live policy, persistence schema, Unity projection,
  canonical과 회귀 test를 같은 방향으로 수직 수렴시킨다.
- F-01 compendium rollback과 D-01 P00 기록 보정은 별도 사용자 결정 없이 기존 승인 정책대로 수정한다.
- 새 정책이 수립되면 대체된 schema/type/reader/writer/test fixture와 호환 분기를 제거한다. 과거 save를
  위한 migration, dual-read/write, field alias와 serde default fallback은 추가하지 않는다.
