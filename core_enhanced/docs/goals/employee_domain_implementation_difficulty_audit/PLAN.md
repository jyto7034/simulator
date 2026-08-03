# Employee Domain Implementation Difficulty Audit

## Status

`complete`

## Goal

실제 Core runtime, live RON, checkpoint/save, DTO, server와 접근 가능한 Unity 소비자를 조사해 승인된 직원 도메인 정책별 구현 난이도와 결합 관계를 근거 기반으로 평가한다. 평가 결과를 직원 도메인 정책 논의 문서에 기록하고, 완결된 source-of-truth 경계에 맞는 후속 Goal 묶음과 수행 순서를 제안한다.

## User-visible Problem

현재 직원 도메인 구현 로드맵은 정책 문서와 대화 수준의 추정에 머물러 있다. 실제 생산자·상태 mutation·저장·wire·소비자 범위를 읽지 않고 Goal을 나누면 중간 dual schema, 누락된 소비자 또는 과소평가된 migration이 생길 수 있다.

## Audit Finding

없음. 사용자 요청에서 시작된 구현 난이도 조사다.

## Target Repositories and Paths

- Core 및 현재 Goal 문서: `/mnt/f/work/simulator/core_enhanced`
- live data: `/mnt/f/work/simulator/game_resources`
- server: `/mnt/f/work/simulator/game_server`
- 접근 가능한 Unity 소비자와 canonical 문서: `/mnt/f/unity projects/ark`
- 최종 갱신 대상: `docs/decisions/employee_domain_policy_2026-07-17.md`

## Current Runtime and Live-data Evidence

- `src/game/employee.rs`의 `Employee`가 run UUID, name, level/XP, life/availability, trauma, injuries, Run HP, loadout, skill fragments, combat profile과 trust를 한 aggregate로 소유한다. authored definition ID는 보존하지 않는다.
- `src/game/data/employee_data.rs`와 live data는 starter/recruitment 후보를 별도 DB/RON으로 로드한다.
- `src/game/world/map_content.rs`는 HQ 후보를 node seed로 전체 recruitment DB에서 매번 shuffle/truncate하고 `src/game/world/headquarters.rs`는 roster에 새 UUID employee를 추가한다. finite queue와 definition dedupe는 없다.
- `src/game/battle/types.rs`의 weapon profile 적용은 attack interval을 직접 overwrite하며 `src/game/battle/stat_pipeline.rs`에 직원 고유 percent 단계가 없다.
- `src/game/employee_trust.rs`는 UUID modulo 단일 trait, append-only memory와 dormant trust feature surface를 가진다.
- `src/game/world/state.rs`와 `src/game/world/combat.rs`는 attempt-local roster, instance processed ledger, staged result commit와 entry snapshot retreat를 소유한다.
- `src/game/world/helpers.rs`, `event_node.rs`, `state.rs`가 목적마다 다른 availability/life predicate를 사용한다.
- `src/game/world/node_flow.rs`의 Run checkpoint는 in-memory clone payload이며 roster를 포함하지만 queue는 없다.
- game_server는 shared Core behavior/DTO를 transport하고 별도 직원 의미를 계산하지 않는다.
- Unity `RosterModel`, starter/selected-event reader와 여러 composer/test가 현재 employee candidate/health/life/availability JSON을 직접 소비한다.

## Source of Truth and Ownership

- 현재 authored candidate source: 두 employee RON과 두 candidate DB.
- 현재 persistent run employee source: Core `EmployeeRoster`/`Employee`.
- 현재 active battle employee condition source: `BattleEmployeeAttemptState.roster`; 결과 확인 시 persistent roster로 commit한다.
- 현재 retreat source: `BattleEntrySnapshot`의 world clone.
- 현재 Run checkpoint source: Core `RunCheckpointPayload` in-memory clone.
- 현재 public employee projection source: Core `EmployeeSnapshotDto`; game_server는 의미 없는 serializer/transport다.
- 현재 Unity presentation consumer: `RosterModel`, `StarterSelectionModel`, `SelectedEventModel`과 screen composers.
- 승인된 목표 source: 하나의 authored EmployeeDefinition, run-owned Queue/Session/Roster, EmployeeCondition central evaluator와 run-local Psychology다. 실제 runtime은 아직 목표 정책으로 migration되지 않았다.

## Approved Policies

- 무한 랜덤 직원 생성 대신 authored 고정 직원 풀을 사용한다.
- 모든 직원을 하나의 `EmployeeDefinition` 카탈로그로 통합하고 시작 자격을 명시한다.
- 미합류 definition은 run-seeded 순환 `RecruitmentQueue`, 활성 채용 세션 또는 roster 중 정확히 한 곳에만 존재한다.
- `employee_definition_id`는 authored identity, `employee_uuid`는 합류 후 run-local identity다.
- 각 definition은 두 축 personality를 정확히 하나씩 필수로 가진다.
- UUID 기반 trait 자동 배정과 trust 내부 trait 소유를 제거한다.
- 전문성은 문서상 확장 후보일 뿐 production schema/runtime/DTO/save에 추가하지 않는다.
- `CombatAptitude`는 초기 `Balanced`, `Defensive`, `Offensive`를 사용해 역할 가시성을 제공하되 장비 제한이나 불일치 페널티를 만들지 않는다.
- 직원 고유 공격 속도는 양의 정수 `basic_attack_speed_percent`로 작성하고 무기 기본 공격 주기에 `ceil(base_interval * 100 / percent)`로 적용한다.
- `Run HP`, 범용 injury와 범용 availability를 제거하고 신체 손상도와 트라우마를 독립 condition으로 사용한다.
- 신체 손상 파생 구간은 정상 0, 경상 1~3, 중상 4, 치명상 5, 사망 6이다.
- 손상 0~4는 전투/Event/위험 행동 참여 가능, 손상 5는 해당 활동 불가지만 치료와 장비 관리는 가능하다.
- 손상 상태명은 수치에서 파생하며 중복 mutable state로 저장하지 않는다.
- 트라우마는 0~100이고 100은 사망이 아닌 Breakdown이다. Breakdown은 전투/Event/위험 행동을 막지만 치료와 장비 관리는 허용한다.
- 전투불능 RuntimeUnit instance마다 손상도 +1과 data-driven 트라우마 증가를 정확히 한 번 적용한다. 같은 전투의 재배치에 즉시 반영하고 결과 commit 시 roster에 반영하며 retreat 시 복원한다.
- 사망 방지 효과는 손상 6 도달을 5로 막고 트라우마를 별도로 회복하지 않는다.
- 시작 Battle HP는 `floor(final_battle_max_hp * (100 - trauma) / 100)`이며 trauma 1~99는 최소 1, trauma 100은 draft를 만들지 않는다.
- 목적별 eligibility는 Core 중앙 evaluator가 typed disabled reason과 함께 계산하고 snapshot과 command validation이 공유한다.
- 신체 손상은 정수 단계 `ReducePhysicalDamage` 같은 명시 Treatment effect만 회복하며 Rest/SavePoint/Gate는 암묵적으로 치료하지 않는다.
- 일반 사망은 장비와 공유 스킬 파편을 원자적으로 회수하고, 회수 불가 원인은 typed death policy로만 명시한다.
- 사망한 EmployeeRunState는 run roster 기록에 남고 같은 definition은 다시 채용하지 않는다.
- 신뢰도는 -100..100, 초기 0의 run-local 상태이며 band는 현재 다섯 구간에서 파생한다.
- 기억은 의미 단위로 병합하며 typed subject, occurrence count, intensity, 최초/최근 발생, 선택적 expiry와 authoritative source dedupe를 가진다.
- 고위험 거부, trust 전투/Event modifier와 실제 dialogue/reaction content는 콘텐츠가 생길 때까지 보류한다.
- compatibility, dual-read, dual-write, fallback 또는 schema alias를 만들지 않는다. 구형 계약이 새 필수 계약을 만족하지 않으면 명시적으로 거부한다.

## Invariants

- Core가 직원 identity, condition, eligibility, psychology mutation의 authority다.
- Unity는 직원 내부 필드를 조합해 gameplay 가능 여부나 modifier를 재계산하지 않는다.
- retreat와 checkpoint load는 승인된 run-local 직원 상태를 정확히 복원한다.
- 같은 employee definition은 한 run에서 중복 소유되거나 중복 채용되지 않는다.
- 현재 Goal은 구현하지 않고 실제 코드 근거에 기반한 난이도와 Goal 경계만 기록한다.
- 기존 사용자 변경을 수정하거나 정리하지 않는다.

## Scope

- 승인 정책 각각의 현재 runtime/data/DTO/save/server/Unity 영향 경로 조사
- 구현 난이도 기준 정의와 정책별 평가
- 정책 사이의 shared mutation/source-of-truth 결합 관계 분석
- 후속 Goal 묶음, 의존성과 권장 수행 순서 제안
- 직원 도메인 정책 논의 문서에 근거와 평가 반영

## Explicit Non-scope

- production Rust, live RON, server 또는 Unity 코드 구현
- DTO, save 또는 wire schema 변경
- canonical gameplay 문서를 현행 runtime과 다르게 변경
- 직원별 최종 스탯, personality 배정, 치료량과 트라우마 증가량 밸런스
- trust high-risk/refusal, modifier 또는 dialogue 콘텐츠 구현
- 후속 Goal 또는 Master Goal 자동 생성
- 현재 Goal과 무관한 dirty 파일 수정 또는 정리

## Undecided Policies

없음. 실제 reaction cue/high-risk 콘텐츠, 직원별 최종 stats/personality와 Treatment 수치는 명시적 후속 authoring/비범위이며 이번 난이도 산정에 필요하지 않다.

## Implementation Scope

문서 조사와 난이도 평가만 수행한다. 생산 코드 구현은 없다.

## User-visible Completion Conditions

- 승인된 모든 직원 정책에 난이도 등급과 실제 코드 근거가 연결된다.
- 각 정책의 Core, live data, checkpoint/save, DTO, server, Unity 영향 여부가 구분된다.
- 하나의 Goal로 묶어야 하는 정책과 분리해야 하는 계약 경계가 이유와 함께 기록된다.
- 후속 Goal의 권장 수행 순서와 선행 의존성이 기록된다.
- 확인하지 못한 외부 소비자와 남은 위험을 명시한다.
- 직원 도메인 정책 논의 문서가 평가 결과를 포함한다.

## DTO Migration

현재 Goal에서는 없음. 후속 구현 난이도 평가 대상으로 조사한다.

## Data or Live RON Migration

현재 Goal에서는 없음. 후속 구현 난이도 평가 대상으로 조사한다.

## Save Migration

현재 Goal에서는 없음. 후속 구현 난이도 평가 대상으로 조사한다.

## Planned Investigation

1. 완료 — 직원 definition, aggregate, profile/trust와 recruitment runtime 경로 조사.
2. 완료 — live employee data와 loader/validation 조사.
3. 완료 — combat incapacitation, redeploy, retreat, result commit, treatment/recovery와 death mutation 조사.
4. 완료 — checkpoint/save clone/restore와 snapshot/behavior DTO serialization 조사.
5. 완료 — server mapping과 Unity roster/starter/recruitment/condition 소비자 조사.
6. 완료 — 관련 tests와 canonical 문서 검색·비교.
7. 완료 — 정책별 난이도와 A~D Goal 경계를 대상 문서에 기록.

## Focused Verification

- 조사에 사용한 정확한 주요 `rg`, `sed` 명령과 범위를 `EXPERIMENTS.md`에 기록한다.
- 최종 문서의 대표 근거 경로가 존재하는지 확인한다.
- 문서 section/table/Goal 순서가 존재하는지 `rg`로 확인한다.
- scoped status와 문서 내용을 검토해 이번 Goal 외 파일을 수정하지 않았는지 확인한다.

## Broad Verification

Production 코드와 live data를 변경하지 않으므로 Rust/Unity test는 실행하지 않는다. Core, game_server와 Unity 전체의 repository-wide employee symbol/consumer search를 broad verification으로 수행했으며 runner test 수는 해당 없음으로 기록한다.

## Skipped Verification and Rationale

- Rust/game_server/Unity tests: 읽기 전용 난이도 감사이며 runtime behavior를 변경하지 않는다.
- formatter: Markdown 문서만 수동 갱신하고 관련 없는 dirty 파일을 건드리지 않는다.

## Difficulty Audit Result

- Goal A `employee_definition_profile_contract`: 높음.
- Goal B `employee_recruitment_queue_ownership`: 높음.
- Goal C `employee_condition_lifecycle_migration`: 매우 높음.
- Goal D `employee_psychology_state_contract`: 높음.
- 권장 순서: A → B → C → D.

## Completion Evidence

- 정책별 난이도: `docs/decisions/employee_domain_policy_2026-07-17.md`의 13.3 표가 승인 정책 전체를 해당 없음/낮음/중간/높음/매우 높음으로 분류한다.
- 계층 영향: 같은 문서 13.2와 각 표의 근거 열이 Core, live data, checkpoint, DTO/server와 Unity 소비자를 구분한다.
- Goal 경계: 같은 문서 14절이 A~D 각 Goal의 묶음, 분리 이유와 사용자-visible 완료 조건 후보를 기록한다.
- 수행 순서: 같은 문서 15절이 A → B → C → D 의존성과 Dice/Event 진입 조건을 기록한다.
- 미검증/위험: 같은 문서 16절과 `EXPERIMENT_NOTES.md`가 dirty Unity, 미확인 durable save consumer와 초안/canonical 차이를 기록한다.
- 문서 무결성: 대표 근거 경로, 필수 Goal 문서 3개, 난이도 정책/Goal/order keyword와 trailing whitespace 검증이 exit code 0으로 완료됐다.
- 변경 범위: production code/data/server/Unity/canonical 문서는 수정하지 않았고 새 Goal 문서와 기존 untracked 직원 도메인 임시 문서만 갱신했다.

## Stop Conditions

- 승인된 정책 자체가 모호하거나 서로 충돌해 난이도를 하나로 산정할 수 없음
- 필수 외부 consumer 또는 저장 경로에 접근할 수 없어 완료 조건을 증명할 수 없음
- 대상 문서에 현재 Goal과 겹치는 사용자 변경이 있어 안전하게 병합할 수 없음
- 조사 결과 현재 Goal이 구현이나 서로 독립적인 변경을 요구함

## Compatibility

현재 Goal은 compatibility를 구현하지 않는다. 승인된 목표 정책은 compatibility를 허용하지 않으며, 난이도에는 complete cutover 비용을 포함한다.
