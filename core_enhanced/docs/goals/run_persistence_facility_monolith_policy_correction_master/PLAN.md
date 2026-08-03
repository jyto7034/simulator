# Run Persistence, Facility, And Monolith Policy Correction Master

Status: complete

Implementation complete: true

Goal name: `run_persistence_facility_monolith_policy_correction_master`

Created: 2026-07-28

Audit finding IDs: `F-01`, `F-02`, `D-01`, `P-01`, `P-02`, `P-03`

Execution authority: 2026-07-28 사용자 요청 및 정책 승인

## 1. 승인된 Goal 설명

`run_persistence_facility_monolith_policy_master`의 완료 주장을 현재 runtime, live data, server,
Unity와 재대조해 발견한 결함과 정책 충돌을 의존성 순서대로 수정한다.

이 Master는 서로 독립적인 네 source-of-truth 경계를 하나의 giant implementation patch로 합치지
않는다. Master는 phase registry, dependency, 공유 seam, phase 진입/완료 gate와 마지막 독립 재감사를
소유한다. 실제 production 조사·구현·검증은 활성 phase의 child Goal에서만 수행한다.

감사 근거와 사용자 승인 기록:

- [2026-07-28 재감사 임시 메모](../../../tmp/run_persistence_facility_monolith_reaudit_2026-07-28.md)
- 2026-07-28 사용자 승인: monolith stage 합성, combat-eligible certified power,
  checkpoint-owned Floor anchor
- 2026-07-28 사용자 승인: 개발 단계의 구 save/schema/compatibility는 보존하지 않고 제거

## 2. 해결할 사용자-visible 문제

- 실패한 일반 command나 직후 reconnect 때문에 이미 발견·격파한 도감 항목이 빈 상태로 보이지 않게
  한다.
- Critical/Breakdown처럼 현재 출전할 수 없는 직원이 상대 위협 계산을 부당하게 낮추지 않게 한다.
- 과거 checkpoint를 불러와 같은 Gate를 다시 진행할 때 Floor 난이도가 load 횟수에 따라 누적 상승하지
  않게 한다.
- Monolith stage 3이 위협 표시만 높은 전투가 아니라 stage 2보다 실제 적 HP·공격·방어가 높은 전투가
  되게 한다.
- Monolith reward가 중간 이중 반올림으로 명목 배율보다 튀지 않고, Unity가 내부 exact multiplier를
  노출하지 않게 한다.
- 기존 master/P07의 `Complete / High` 주장과 P00 기록을 실제 수정·검증 결과에 맞게 다시 판정한다.

## 3. Master 문서의 책임과 비책임

### 소유하는 것

- P00~P04 phase 순서와 hard/serial dependency
- child Goal 이름, 활성화 조건과 상태 registry
- compendium rollback, certified power, checkpoint anchor와 monolith tuning 사이의 공유 seam
- 각 phase의 사용자-visible exit gate
- 새 save compatibility를 만들지 않고 대체된 legacy를 제거하는 campaign-level 원칙
- 마지막 cross-component 독립 재감사와 master 종료 조건

### 소유하지 않는 것

- gameplay/data/save/wire의 durable canonical 자체
- child 내부 구현 순서와 구체 helper/type 배치
- 승인된 수치 밖의 신규 밸런스 조정
- hidden boss P06 콘텐츠, 계정/다중 profile persistence와 신규 UX
- 현재 finding과 독립적으로 구현·검증할 수 있는 문제

안정된 계약은 각 phase에서 가장 가까운 canonical에 반영한다. 이 Master와 child Goal은 실행 기록이며
durable gameplay source of truth가 아니다.

## 4. 필수 시작 및 재개 절차

Master를 시작하거나 재개할 때 production 조사·수정·테스트 전에 다음을 처음부터 끝까지 읽는다.

- 이 디렉터리의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
- [문서 인덱스](../../README.md)
- [Engineering Workflow](../../engineering_workflow.md)
- [Goal 완료 검토 가이드](../../goal_completion_review_guide.md)
- [기존 구현 Master](../run_persistence_facility_monolith_policy_master/PLAN.md)와 P00~P05/P07 기록
- [2026-07-28 재감사 임시 메모](../../../tmp/run_persistence_facility_monolith_reaudit_2026-07-28.md)
- 활성 phase의 child Goal 문서 세 개

Unity-facing phase는 외부 프로젝트의 다음 문서와 실제 consumer를 함께 확인한다.

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

Child 문서는 해당 phase를 실제 활성화할 때만 생성한다. Master 초기화 때 빈 child Goal들을 미리
만들지 않는다.

## 5. 현재 runtime 및 live data 근거

다음은 2026-07-28 pre-Goal 재감사에서 확인한 입력이다. 새 Goal baseline을 대신하지 않으며 각 child가
현재 파일/diff와 focused baseline을 다시 확인해야 한다.

### F-01 compendium rollback

- `GameCoreState::clone_non_battle_transaction_state()`는 rollback 사본의 compendium을 default로 둔다.
- active-run host rollback은 live profile을 별도 보존·merge하지만 공통 `with_state_transaction()`은
  실패 시 default 사본을 그대로 복원한다.
- server는 player behavior 시작 전에는 profile을 sync하지만 `AttachSession` snapshot 전에는 sync하지
  않아 실패 직후 reconnect에서 빈 projection이 노출될 수 있다.
- live RON 변경은 이 경계와 무관하다.

### P-01 certified power

- 현재 runtime은 `is_alive()`인 모든 직원을 합산해 Critical/Breakdown도 certified power에 포함한다.
- 최초 approved decision은 실제 combat-eligible roster만 계산하고 current HP/Trauma는 readiness로
  분리한다고 규정한다.
- 현재 relative threat는 같은 certified total을 denominator로 사용한다.

### P-02 checkpoint anchor

- `run_peak_certified_power`와 `current_floor_power_anchor`가 모두 checkpoint 밖 monotonic metadata다.
- checkpoint payload는 current Floor anchor를 명시적으로 소유하지 않는다.
- load 뒤 map은 과거 Floor로 돌아가지만 anchor는 높은 세대에 남아 Gate 재진행 때 minimum rise가 다시
  누적될 수 있다.
- active-run envelope current/backup merge도 anchor 최대값을 보존한다.

### F-02/P-03 monolith stage와 reward

- live policy는 stage 2 stat/reward 100%, stage 3 stat/reward 125%를 author한다.
- conversion은 stage 배율을 `target_power`에만 반영하고 actual combat preview는 동일한
  `MonolithRemanifestation` role stat/wave tuning을 사용한다.
- stage 2와 3의 실제 enemy stat/wave가 동일하며 target power는 relative-threat projection에만 쓰인다.
- reward는 role multiplier 적용 후 stage multiplier를 다시 적용해 quantity를 두 번 반올림한다.
- Unity projection은 내부 `reward_multiplier_percent`를 exact 숫자로 표시해 P04 비노출 원칙과 충돌한다.

### D-01 기록 불일치

- P00 `PLAN.md`와 completion review는 complete지만 같은 child의 `EXPERIMENTS.md`와
  `EXPERIMENT_NOTES.md` header는 active/false다.

## 6. Source Of Truth와 소유 계층

| 책임 | Authority | 전달/소비 계층 |
| --- | --- | --- |
| transaction rollback과 gameplay state | Core `GameCoreState` / `GameCore` | game_server actor, snapshot |
| local compendium set union | Core compendium domain | host persistence, Unity projection |
| certified formula와 run peak | Core combat-power domain + versioned live RunPolicy | Floor manifestation, threat projection |
| checkpoint-owned state | Core checkpoint payload | active-run host envelope, continue/load |
| current/backup/lock/publish | game_server persistence | Core import/export, Unity status UX |
| node difficulty와 resolved combat tuning | Core map/difficulty resolver | combat preview, battle setup, reward |
| stage/balance authoring | live `game_resources/data/run/policy.ron` | strict Core loader/validation |
| wire/DTO mapping과 ordering | Core DTO + game_server | Unity strict reader/store |
| 표시 문구와 local presentation | Unity | 플레이어 |
| durable gameplay 의미 | `game_rulebook.md` | 모든 계층 |
| save/runtime 불변식 | `core_runtime_contract.md` | Core/server/test |
| live RON schema/loading | `data_loading_contract.md` | authoring/loader/test |
| Unity exact 계약 | 외부 `unity_core_contract.md` | Core/server/Unity |

## 7. 유지할 불변식

- Local compendium은 active-run/checkpoint와 별도 monotonic union이며 run rollback으로 감소하지 않는다.
- 실패한 transaction은 run-local mutation 일부만 commit하지 않지만 이미 commit된 profile knowledge를
  삭제하지 않는다.
- `run_peak_certified_power`와 successful `loads_used`는 checkpoint load/backup recovery로 감소하지 않는다.
- current Floor anchor와 생성된 node difficulty는 같은 checkpoint manifestation 세대에 속한다.
- 같은 checkpoint branch의 같은 Gate 재진행은 같은 run peak와 live policy에서 같은 다음 anchor를 만든다.
- Certified power는 current combat-eligible 전체 roster의 persistent loadout 입력만 사용하며 실제 배치된
  subset을 사후 재계산하지 않는다.
- 같은 node의 target projection, combat preview와 battle setup은 하나의 resolved difficulty authority를
  사용한다.
- Monolith stage 3은 stage 2보다 실제 enemy HP·attack·defense가 높고 wave budget은 stage role 값을
  유지한다.
- Quantity reward는 role+stage 배율을 원본 quantity에 한 번 적용·반올림한다.
- Exact certified/anchor/target/stat/reward multiplier는 Unity에 노출하지 않는다.
- 새 schema는 strict하고 대체된 구 save/schema/reader/writer/fixture를 호환층으로 남기지 않는다.
- 기존 사용자 변경과 현재 Goal 밖의 production 동작을 보존한다.

## 8. 승인된 정책과 근거

### Compendium rollback

- 기존 P02 decision에 따라 rollback 전후 profile을 set union으로 보존한다.
- server AttachSession snapshot 전에도 durable profile을 Core에 merge한다.

### Certified power

- `alive && physical_damage <= 4 && trauma < 100`인 combat-eligible 전체 roster만 합산한다.
- current HP/Trauma의 정확한 값과 Minor/Major damage는 coefficient가 아니라 readiness 의미다.
- Critical/Breakdown/Dead는 current certified total에서 제외하지만 warning은 유지한다.
- run peak는 과거 combat entry 최댓값으로 계속 비되감기다.

### Checkpoint anchor

- current Floor anchor와 power model version은 checkpoint-owned다.
- `loads_used`와 run peak만 checkpoint 밖 monotonic metadata다.
- backup은 이전 checkpoint의 anchor를 복원하고 loads/run peak만 current 값과 최대 merge한다.

### Monolith stage와 reward

- Stage stat은 target power와 enemy max-health/attack/defense에만 합성하고 wave에는 적용하지 않는다.
- Current live 기준 stage 2는 health 165%, attack/defense 145%; stage 3은 health 207%,
  attack/defense 182%다.
- Role+stage percentage는 한 번 합성하고 base stat에 한 번 적용한다.
- Reward도 원본 quantity에 role+stage를 한 번 합성해 nearest rounding/minimum-one을 한 번만 적용한다.
- Unity는 exact percentage 대신 `강화 보상`/`최종 강화 보상` 계열 qualitative tier를 소비한다.
- Map difficulty는 typed monolith stage authority를 보존하고 monolith ledger와 mismatch면 fail closed한다.

### Legacy/save compatibility

- 현재는 개발 단계이며 과거 save 구조를 유지하지 않는다.
- Schema/version 변경 시 구 save는 explicit unsupported/reset 대상이다.
- Migration, dual-read/write, alias, missing-field default fallback을 추가하지 않는다.
- 새 정책으로 대체된 legacy type, reader, writer, field, fixture, mock, test와 canonical 표현을 제거한다.

## 9. 미결정 정책

없음.

승인된 범위를 구현하는 중 새로운 사용자-visible 의미, 밸런스 값, external DTO 또는 배포 compatibility가
필요하면 해당 child를 `policy_decision_required`로 전환한다. Master가 대신 결정하지 않는다.

## 10. Dependency Graph와 실행 순서

```text
P00 Compendium transaction rollback integrity
  -> P01 Combat-eligible certified power
      -> P02 Checkpoint-owned Floor anchor / save schema
          -> P03 Monolith stage tuning / reward projection
              -> P04 Cross-component final completion review
```

Dependency 근거:

- P00은 공통 `with_state_transaction()`을 바로잡는다. P03 monolith conversion도 이 helper를 사용하므로
  stage schema/mutation 검증 전에 rollback invariant를 먼저 고정해야 한다.
- P01은 P02 anchor와 run peak가 소비하는 certified input 의미와 `power_model_version`을 먼저 고정한다.
  P02를 먼저 구현하면 anchor 기대값과 save version을 다시 바꿔야 한다.
- P02는 checkpoint payload/envelope와 node manifestation 세대를 먼저 정리한다. P03이 추가하는 typed
  stage difficulty가 저장·load·backup에서 보존될 안정된 seam을 제공한다.
- P03은 P00 transaction, P01 power model과 P02 checkpoint/save seam을 모두 소비한다.
- P04는 child 완료 주장을 증거로 사용하지 않고 current runtime/data/server/Unity/canonical을 다시
  독립 감사한다.

P00과 P01은 논리적으로 일부 병렬 조사할 수 있지만, dirty baseline과 회귀 원인 분리를 위해 승인된
기본 실행은 직렬이다. 활성 phase가 `Complete / High` completion review를 얻기 전에는 다음 production
phase를 시작하지 않는다.

## 11. Phase Registry

| Phase | Child Goal | Current status | Hard prerequisite | 핵심 결과 |
| --- | --- | --- | --- | --- |
| P00 | `run_policy_correction_phase_00_compendium_transaction_rollback_integrity` | complete; Complete/High 2026-07-28 | 없음 | failed transaction/reconnect에서 profile union 보존 |
| P01 | `run_policy_correction_phase_01_combat_eligible_certified_power` | complete; Complete/High 2026-07-28 | P00 Complete/High | combat-eligible current power와 비되감기 run peak |
| P02 | `run_policy_correction_phase_02_checkpoint_owned_floor_anchor` | complete; Complete/High 2026-07-28 | P01 Complete/High | checkpoint-owned anchor, 새 strict save schema와 legacy 제거 |
| P03 | `run_policy_correction_phase_03_monolith_stage_tuning_reward_convergence` | complete; Complete/High 2026-07-30 | P00-P02 Complete/High | 실제 stage stat, one-pass reward, qualitative Unity projection |
| P04 | `run_policy_correction_master_final_completion_review` | complete; Complete/High 2026-07-30 | P00-P03 Complete/High | 독립 재감사, D-01 기록 보정과 Master 종료 판정 |

이 표가 correction campaign의 phase 상태와 활성 순서에 대한 master-level source다. Child 문서는 해당
phase를 활성화할 때 만들며 initial baseline과 현재 dirty state를 그때 다시 기록한다.

## 12. Phase 00 — Compendium Transaction Rollback Integrity

### 범위

- Core 공통 non-battle transaction rollback에서 live compendium을 보존·union merge한다.
- active-run host rollback과 공통 rollback이 같은 profile lifetime 의미를 사용하게 한다.
- server AttachSession이 snapshot을 만들기 전에 durable profile을 Core에 merge한다.
- failed command, persistence failure와 same-actor reconnect의 profile projection을 검증한다.

### 명시적 비범위

- compendium 발견/격파 gameplay producer 재설계
- profile file schema/current+backup 정책 변경
- 계정/transport-player별 profile namespace
- certified power, anchor와 monolith tuning 수정

### 사용자-visible exit gate

- non-empty compendium 상태에서 실패한 일반 transaction 뒤에도 discovered/defeated union이 동일하다.
- 실패 직후 같은 actor reconnect snapshot도 durable union을 투영한다.
- active-run publish 실패 rollback은 run state만 되돌리고 profile 진행을 보존한다.
- 기존 profile file failure/dirty/reset UX가 바뀌지 않는다.
- focused Core/server와 영향 범위 broad 검증이 통과하고 completion review가 `Complete / High`다.

## 13. Phase 01 — Combat-Eligible Certified Power

### 범위

- certified roster filter를 current combat eligibility와 일치시킨다.
- current threat projection, Floor-1 initialization과 combat-entry run peak가 같은 formula/version을 쓴다.
- Critical/Breakdown/Dead 제외, warning 유지와 zero-power threat 의미를 고정한다.
- formula 의미 변경에 맞춰 `power_model_version`과 live validation/canonical을 갱신한다.

### 명시적 비범위

- power coefficient 재밸런싱
- 별도 potential/capacity power authority 추가
- Floor anchor checkpoint ownership 변경
- monolith stage stat 적용

### 사용자-visible exit gate

- Critical/Breakdown/Dead의 stat/loadout이 current certified total과 relative threat를 낮추지 않는다.
- combat-eligible 직원의 current HP/trauma 수치는 formula coefficient가 아니다.
- 직원 손실이나 checkpoint load가 historical run peak를 낮추지 않는다.
- power model version mismatch와 구 save는 fail closed하며 compatibility fallback이 없다.
- focused formula/threat/run-peak/live-RON 및 broad 검증이 통과하고 completion review가
  `Complete / High`다.

## 14. Phase 02 — Checkpoint-Owned Floor Anchor

### 범위

- checkpoint payload가 power model version과 current Floor anchor를 명시적으로 소유하게 한다.
- load/continue가 checkpoint map/node difficulty와 같은 세대의 anchor를 복원하게 한다.
- host monotonic metadata와 backup merge에서 current anchor를 제거하고 loads/run peak만 유지한다.
- active-run envelope/core payload version을 올리고 구 schema/field/fixture를 제거한다.
- cross-Floor load 후 같은 Gate 재진행의 deterministic anchor를 검증한다.

### 명시적 비범위

- checkpoint load 최대 횟수 변경
- run peak rewind 허용
- save migration/dual reader
- node difficulty formula나 monolith stage tuning 변경

### 사용자-visible exit gate

- checkpoint load가 당시 Floor anchor와 node difficulty를 함께 복원한다.
- 같은 checkpoint/Gate branch를 반복해도 load 자체로 next-Floor anchor가 추가 상승하지 않는다.
- run peak와 successful loads는 load/restart/backup recovery로 감소하지 않는다.
- current/backup은 각 checkpoint-owned anchor를 보존하며 구 envelope/schema는 명시적으로 거부한다.
- 실제 filesystem focused test와 Core/server/Unity 영향 검증이 통과하고 completion review가
  `Complete / High`다.

## 15. Phase 03 — Monolith Stage Tuning And Reward Convergence

### 범위

- map difficulty에 typed monolith stage modifier를 저장하고 ledger와 cross-validation한다.
- 하나의 resolver가 role+stage target/stat/reward를 합성한다.
- stage 3 enemy HP/attack/defense를 stage 2보다 높이고 generated-wave budget은 role 200%로 유지한다.
- quantity reward를 원본 authored amount에 한 번 합성·반올림한다.
- exact `reward_multiplier_percent` Unity DTO/reader/presentation을 qualitative reward tier로 교체한다.
- 새 internal/save/DTO 계약에 맞춰 legacy field, fixture, mock, test와 canonical을 제거·갱신한다.

### 명시적 비범위

- monolith 가격, 시도 횟수, use와 residual/exchange 정책 변경
- encounter wave composition 또는 reward pool identity 재밸런싱
- hidden boss/P06 콘텐츠
- P00~P02 정책 재설계

### 사용자-visible exit gate

- 고정 fixture/seed에서 stage 3 실제 enemy HP/attack/defense가 stage 2보다 높다.
- stage 2와 3 generated-wave budget/구성은 stage 배율 때문에 달라지지 않는다.
- target projection, preview와 battle setup이 같은 stage authority를 사용한다.
- reward의 작은 quantity도 중간 이중 반올림 없이 승인된 one-pass 결과를 낸다.
- Unity는 stage와 qualitative reward tier를 표시하고 exact internal multiplier를 읽거나 표시하지 않는다.
- checkpoint/continue/rollback 뒤 stage modifier와 ledger가 일치하며 mismatch는 fail closed한다.
- focused Core/RON/save/server/Unity와 broad 검증이 통과하고 completion review가 `Complete / High`다.

## 16. Phase 04 — Final Completion Review And Record Correction

### 범위

- P00~P03 완료 주장을 현재 code/data/DTO/server/Unity/canonical과 독립 재대조한다.
- original master/P07의 잘못된 `Complete / High` 판정을 대체하는 현재 correction evidence를 기록한다.
- D-01 P00 `EXPERIMENTS.md`/`EXPERIMENT_NOTES.md` stale header를 사실관계에 맞게 보정하되 과거 실험
  결과와 history를 재작성하지 않는다.
- 제거 대상 legacy/compatibility의 runtime/data/schema/DTO/mock/fixture/test/canonical 잔존을 감사한다.

### 명시적 비범위

- P00~P03에서 발견한 독립 신규 문제 구현
- 과거 Goal 실험 결과의 미화 또는 기계적 재서술
- hidden boss, 계정 profile과 신규 콘텐츠

### 사용자-visible exit gate

- F-01/F-02와 승인된 P-01/P-02/P-03 계약에 각각 current evidence가 연결된다.
- producer와 접근 가능한 Unity consumer가 같은 strict 계약을 사용한다.
- 구 save/schema/field와 승인되지 않은 compatibility/fallback이 남지 않는다.
- P00 기록 header와 실제 완료 history가 모순되지 않는다.
- 필수 focused/broad 검증 결과와 test 수가 기록되고 final review가 `Complete / High`일 때만 Master를
  `complete`로 변경한다.

## 17. Migration 및 Compatibility 계획

| Phase | DTO migration | Live data/RON migration | Save migration |
| --- | --- | --- | --- |
| P00 | 없음 | 없음 | 없음 |
| P01 | shape 없음; 값 의미 변경 | `power_model_version` 상승, coefficient 변경 없음 | migration 없음; 구 model save reset/reject |
| P02 | 기존 Unity-visible checkpoint shape 유지가 목표 | 없음 | checkpoint payload/envelope breaking version; 구 schema 제거·reject |
| P03 | exact reward percent 제거, qualitative tier로 breaking 교체 | stage 값 유지, 필요 시 power model version 상승 | typed difficulty stage 추가; migration 없이 구 save 제거·reject |
| P04 | 없음 | 없음 | 없음 |

Compatibility 지원 대상과 기간은 없음이다. 새 compatibility 필요성이 발견되면 구현하지 않고 사용자
승인을 요청한다.

## 18. 필수 Focused 검증

각 child는 실제 조사 뒤 정확한 test 이름과 수를 보완한다. 최소 계약은 다음과 같다.

- P00: failed transaction profile preservation, active-run rollback union, AttachSession-before-snapshot sync,
  file dirty/fatal regression
- P01: eligibility matrix, Critical/Breakdown/Dead exclusion, zero denominator threat, run peak monotonic,
  live power-model version validation
- P02: checkpoint anchor round-trip, cross-Floor load/re-advance determinism, current/backup recovery,
  loads/peak-only max merge, unsupported old version, publish failure atomicity
- P03: stage 2/3 resolved target/stat exactness, unchanged wave budget, one-pass reward rounding edge cases,
  ledger/difficulty mismatch rejection, checkpoint/continue round-trip, exact legacy DTO rejection,
  Unity qualitative presentation
- P04: all finding regressions, legacy search, cross-layer serialized shape와 fixed-seed combined flow

Compile 성공이나 mock-only snapshot은 사용자-visible 완료 증거로 사용하지 않는다.

## 19. 필수 Broad 검증

최종 정확한 명령은 child baseline에서 현재 workspace 구조에 맞춰 기록한다.

- Core: `cargo test -p game_core --lib`와 관련 integration suites
- Live data: `cargo test -p game_core --test ron_loading`
- Server/persistence: `cargo test -p game_server`
- Workspace final: `cargo test --workspace --all-targets`
- Unity: MCP broad EditMode와 PlayMode; server가 필요한 explicit live tests는 실행 환경을 마련할 수 있는지
  확인하고, 실행하지 못하면 정확한 skip 이유와 completion 영향 기록
- Legacy: scoped symbol/schema/JSON field search와 strict rejection tests
- Documentation: scoped `git diff --check`, Goal/canonical 상태·link 검사

각 runner가 보고한 실제 pass/fail/skip 수를 `EXPERIMENTS.md`에 기록한다.

## 20. 생략할 검증과 근거

초기 계획에서 승인된 생략은 없다.

조사 결과 특정 계층이 phase와 무관하면 child가 파일/호출 근거를 남기고 생략할 수 있다. 필수 외부
consumer나 실제 filesystem 검증에 접근할 수 없으면 해당 child를 `externally_blocked`로 두며 정적
검토만으로 complete 처리하지 않는다.

## 21. Dirty Worktree 및 변경 제약

- Master 초기화 직전 `/mnt/f/work/simulator`와 `/mnt/f/unity projects/ark`의 `git status --short`는
  모두 출력이 없었다.
- 이후 생기는 기존/신규 변경은 사용자 소유로 간주한다.
- 각 child production 수정 전 관련 저장소 status와 대상 file diff를 다시 확인한다.
- reset, restore, clean, stash, unrelated format과 snapshot 일괄 갱신을 하지 않는다.
- 기존 변경과 안전하게 병합할 수 없으면 해당 child를 완료하지 않고 충돌 근거를 보고한다.

## 22. 중단 조건

- 승인된 phase보다 독립적인 source-of-truth를 함께 수정해야만 진행 가능한 경우
- 새로운 사용자-visible 의미, balance value, identity/lifetime 또는 external DTO 선택이 필요한 경우
- 배포된 구 save/compatibility 보존 요구가 새로 생긴 경우
- 필수 Unity consumer, filesystem runner, live data 또는 dependency에 접근할 수 없는 경우
- dirty 사용자 변경과 안전하게 병합할 수 없는 경우
- baseline 없이 변경 전후 회귀를 구분할 수 없는 경우

범위 선택이 필요하면 `scope_decision_required`, 정책이면 `policy_decision_required`, 필수 외부 요소면
`externally_blocked`로 전환하고 선택에 영향받는 production 수정을 중단한다.

## 23. Master 완료 조건

- P00~P03 child가 각각 사용자-visible exit gate와 필수 focused/broad 검증을 충족한다.
- 각 child completion review가 `Complete / High`다.
- P04가 child 완료 주장을 독립적으로 재검증하고 `Complete / High`다.
- F-01/F-02/D-01과 승인된 P-01/P-02/P-03에 current evidence가 연결된다.
- Core/live data/save/server/Unity/canonical이 같은 승인 계약을 사용한다.
- 구 save/schema/DTO/reader/writer/mock/fixture/test와 승인되지 않은 compatibility가 남지 않는다.
- Goal 범위 밖 변경이 섞이지 않고 기존 사용자 변경 보존 결과가 기록된다.
- 실행한 모든 검증의 실제 test 수와 pass/fail/skip이 기록된다.
- 필수 미검증, 정책 질문과 blocker가 없다.

위 조건을 모두 충족하기 전에는 Master의 `Status`와 `Implementation complete`를 완료로 바꾸지 않는다.

## 24. Completion Record

- Completed: 2026-07-30
- P00~P04: 모두 `Complete / High`
- Final review: `COMPLETION_REVIEW.md`
- Explicit live Maintenance fixture 1건의 독립 실패는 P04 review에 범위 밖 remaining risk로 공개했다.
