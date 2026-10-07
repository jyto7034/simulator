# R06 잔여 전체 구현 지시서 — C07 이후부터 최종 감사 인계까지

2026-09-29 · v3 · R06→R07 연속 요청의 R06 상세 지침. 지시서 작성으로 제품 구현을 시작하지 않음.

**최신 요청:** [R06→R07 연속 구현 지시서](R06_R07_CONTINUOUS_IMPLEMENTATION_REQUEST_20260929.ko.md)가 전체 범위·전환·종료점을 소유한다. 아래의 R06 종료 후 반환/R07 착수 금지는 R06 단독 요청의 경계였으며 이번 연속 실행에는 적용하지 않는다. R06 경계 target을 보존한 뒤 사용자 답변·감사 대기 없이 R07의 계획 준비와 구현으로 이어간다. R06의 나머지 상세 설계·수용·검증 조건은 유지한다.

## 1. 요청과 종료점

**C07 완료 소스를 이어받아 R06의 남은 구현·교정·자체 검증을 끝까지 수행한다.** C08 하나를 닫고 다음 요청을 기다리는 방식은 이번 실행에 적용하지 않는다. 내부적으로 작업을 나누고 focused 시험을 수행하되, 남은 R06을 진행하여 최종 통합 target을 동결한 뒤 멈춘다.

사용자의 이번 전체 잔여 구현 요청에 맞춘 실행 지시다. 시작 요청을 받으면 [GEX](../../../../../goal_execution_pipeline(1).md)의 `GEX-00` 인수 후 `GEX-10`을 수행한다. 최종 종료 문구는 **구현·자체 검증 종료 / 최종 target 동결 / 감사 요청 대기**다. `GEX-20` 독립 완료 감사는 별도 요청과 fresh reviewer의 업무다. 구현자가 `Complete / High`, 독립 PASS 또는 R06 최종 완료를 선언하지 않는다. 현재 구조는 Single Goal이므로 새 Phase나 GEX-30을 만들지 않는다.

이번 실행에 한해 기존 PLAN, 부분 구현 지시서 및 [단계검토 템플릿](../../../../../templates/IMPLEMENTATION_CHECKPOINT_REQUEST.template.ko.md)의 다음 제한을 대체한다.

- 첫 의미 있는 부분에서 동결하고 검토 요청 대기로 중단하는 제한.
- 다음 C/WP에 착수하려면 별도 재개 요청을 받아야 한다는 제한.
- D07-B 교정만 허용하고 WP03/04에 진입하지 말라는 부분 범위 제한.

정책·품질·원자료 보존·실제 코드 검토·세 기록 역할·최종 독립 감사 경계는 유지한다. 원본 지침·템플릿·동결 기록은 수정하지 않는다.

### 사용자 부재 중 정책 결정 위임

사용자는 작업 시작 후 답변할 수 없으므로 구현 중 생기는 미정 정책도 구현자가 재량으로 결정하고 계속 수행하도록 명시적으로 위임했다. 이번 R06 실행에서는 GEX 및 선행 문서의 미정 정책 질문/owner 반환 조건에 이 사용자 위임을 적용한다. 질문 답변 대기만을 이유로 중단하지 않는다.

1. 이미 명시적으로 합의한 gameplay·범위·저장 호환·비용 정책을 우선한다. 미정 부분은 기존 동작 보존, 데이터 보존, 결정적 실행, 작고 되돌릴 수 있는 변경을 우선하여 선택한다. 구현 편의를 위한 수용 기준 삭제·지원 범위 축소·실패 은폐는 재량 결정이 아니다.
2. 실제 코드와 기존 계약에서 가능한 선택지를 비교하고 채택 정책·근거·영향·대안·되돌리는 방법을 `EXPERIMENT_NOTES.md`에 기록한다. `EXPERIMENTS.md`에 결정 시점과 검증을, PLAN에 현재 적용 결정을 연결한다. 과거 사용자 확정 정책이나 독립 검토 PASS로 표시하지 않는다.
3. 선택한 정책을 필요한 제품 문서·구현·시험에 반영하고 다음 작업으로 진행한다. 나중에 개선 가능한 선택은 별도 의존으로 분리하되, 필수 완료 조건을 후속 작업으로 밀어 완료를 주장하지 않는다. 최종 인계에 구현자가 결정한 정책 목록을 모아 사용자가 돌아온 뒤 확인할 수 있게 한다.
4. 기존 사용자 데이터 삭제나 손실을 수반하는 전환 대신 보존 가능한 경로를 택한다. 범위 밖 기능·R07 착수·독립 감사 자동 실행으로 확대하지 않는다. 20분 시험 제외와 concurrent peak 단독 초과 경고 정책도 유지한다.
5. 필수 자료 접근 불가·환경 권한 제한·충돌하는 동시 수정처럼 판단만으로 해결할 수 없는 장애는 안전한 우회와 독립 작업을 먼저 수행한다. 끝내 진행할 수 없다면 실제 blocker와 재개 방법을 남기며 성공으로 가장하지 않는다. 이 위임은 환경의 필수 승인 절차를 우회하는 권한이 아니다.

## 2. 기준 문서와 현재 시작점

일상 진입점은 계속 [ROADMAP_STATUS](../../../../../roadmaps/content_authoring/ROADMAP_STATUS.md) 하나다. 다음 문서는 연결된 상세 근거다.

| 역할 | 기준 |
| --- | --- |
| 최신 Goal | [successor PLAN_HANDOFF](../../../successors/r06_scope_20260927_001/PLAN_HANDOFF.ko.md), [Goal Set](../../../successors/r06_scope_20260927_001/P2G_50_GOAL_SET.ko.md) |
| 정확한 identity | `GS-CAVER-SCOPE50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, `HA-CAVER-SCOPE60-001` |
| 정책·실행 노드 | `FPF-CAVER-SCOPE30-001`, `PP-CAVER-SCOPE40-001`, `PVR-CAVER-SCOPE40-002`; `AG-CAVER-01`, `WP-CAVER-04` |
| 상세 설계 | [CODE_REVIEW_003 D01–D10/V01–V21](../../../CODE_REVIEW_003.ko.md), 최신 successor의 집계·비용 변경이 우선 |
| 최신 소스 | [C07 인계](../corrections/wp02_d07_b_f03_order_001/REVIEW_HANDOFF.ko.md), [manifest](../corrections/wp02_d07_b_f03_order_001/target/d07_b_f03_order_001/MANIFEST.json) |
| 미완료 관측 조건 | [D07-B 교정 지시서 C08–C16](WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md), 원 B01–B20도 유지 |
| 비용 변경 | [R06 비용 정책](R06_COST_POLICY_UPDATE_20260927.ko.md) |
| 이번 조사 | [코드 근거](R06_COMPLETION_CODE_EVIDENCE_20260929.json), [문서 검증](R06_COMPLETION_DOCUMENT_VALIDATION_20260929.json) |

C07 manifest SHA-256: `41b98656a421e489a04d528d2dbc9ac196ff247ab8f56ec69d30cfbea4d7a882`.
347개 source digest: `ff05f531a378146effa2c3e3321b250db7763b17e5a849fecb0b25c3006d510a`.
이 지시서 작성 시 currentness·source/test membership·345→347 및 역방향 격리 복원을 확인했다. 이는 **C07 독립 동작 검토 PASS가 아니다**. C07의 자체 검증 결과는 신규4개·전체1,560개이며 제품 실행 코드 변경 없이 시험만 보강한 결과다. 최종 감사는 아직 독립 검토되지 않은 C07도 포함한다.

F01, F02/C02, C03 runtime, C04/C05, C06의 기존 부분 PASS를 이력으로 이어받는다. 이전 시험 수를 최종 목표 숫자로 고정하지 않는다. 기존 구현은 현재 계약과 코드가 맞으면 재작성하지 않고 남은 연결·교정·증거를 채운다.

반드시 닫을 두 관찰 사항:

1. **C07-OBS01 → C09:** 적중한 관통 투사체가 마지막에 빈 상태로 만료하면 native carrier는 닫혀도 cast/pending delivery가 남는다. [재현·사용자 결정](../corrections/wp02_d07_b_f03_order_001/evidence/C07_OBS01.json)을 읽고 교정한다.
2. **C03-OBS01 → C15:** RON의 `i32::MIN` 직접 역직렬화 실패. typed runtime 통과와 공개 RON 성공을 구별하고 이번 통합에서 교정한다. 해결 대신 허용 범위를 줄이는 정책 변경은 임의로 하지 않는다.

## 3. 인수와 세 기록 역할

Canonical execution-document directory는 기존 `execution/single_goal/`다. 최초 동결 root PLAN을 고치거나 별도 Goal 실행 디렉터리를 만들지 않는다. 그 안의 연결된 이력 분할인 `continuations/r06_completion_001/`을 이번 current locator로 사용한다. 시작 시 존재 여부부터 확인하고, 이미 있다면 재개한다.

| 역할 | canonical directory 기준 실제 locator |
| --- | --- |
| PLAN | `continuations/r06_completion_001/PLAN.md` |
| EXPERIMENTS | `continuations/r06_completion_001/EXPERIMENTS.md` |
| EXPERIMENT_NOTES | `continuations/r06_completion_001/EXPERIMENT_NOTES.md` |
| 최종 인계 | `continuations/r06_completion_001/REVIEW_HANDOFF.ko.md` |
| 최종 감사 | 별도 감사 요청 시 같은 canonical directory 아래 `reviews/`에 배정; 구현 중 감사 PASS 파일을 만들지 않음 |

이 매핑은 current PLAN에 한 번 기록하고 ROADMAP_STATUS에서 연결한다. 이전 segment/correction PLAN은 당시 snapshot이며 독립 current 원본이 아니다. EXPERIMENTS는 명령·시도·실패·수정·결과·원자료, NOTES는 현재 판단·채택/폐기 이유·미검증 가정·재발 방지를 소유한다. 시작/의미 있는 실패/설계 변경/검증/인계 시 갱신하며 종료 직전에 기억으로 일괄 작성하지 않는다.

구현 전 다음을 수행한다.

1. C07의 PLAN/EXPERIMENTS/NOTES 및 직접 관련 검토·실패 이력을 인수한다. 현재 HEAD/status/dirty/untracked와 347개 manifest를 대조한다. 차이가 있으면 실제 변경 주체·내용을 분리해 before를 보존한다. 사용자 작업을 reset/stash/clean하지 않는다.
2. 기존 `freeze.py verify`를 읽고 읽기 전용 검증을 수행한다. 이전 `freeze`를 다시 실행하거나 frozen 파일을 덮어쓰지 않는다. 새 파일 membership까지 확인한다.
3. 현재 source/data/tests/config/lock/toolchain/실행 환경과 before bytes를 고정한다. R01/R04/R05 선행 근거의 관련 입력도 연결한다. C07의 전체 기본 시험은 동일 입력임을 입증하면 initial baseline으로 재사용할 수 있다. 없는 feature/shared-contract 근거는 한 번 확보한다. 단순 날짜 변경 때문에 동일 시험을 반복하지 않는다.
4. [이전 build V7 fixture](../evidence/preimplementation/active_run_v7.pre_r06.json)와 [생성 근거](../evidence/preimplementation/PREIMPLEMENTATION_FIXTURES.json)를 확인한다. 이 원본을 새 코드로 재생성하지 않는다. literal fixture와 old content canonical identity도 보존한다.
5. 아래 실제 symbol을 읽고 producer→consumer, rollback, 저장·계량 경계를 PLAN에 적은 후 수정한다. source 차이가 생겼으면 이 문서의 줄 번호보다 현재 symbol/계약을 우선한다.

## 4. 내부 구현 순서

순서는 의존성을 위한 것이며 중간 검토 중단 지점이 아니다.

| 순서 | 완료할 책임 | 다음 책임으로 가기 전 자체 확인 |
| --- | --- | --- |
| A | C08–C14 관측 identity·수명·실패·한도, C07-OBS01 교정 | 실제 실행과 조회/rollback/cleanup의 focused 시험 |
| B | WP03 stat/무기/사본 강화/cadence, CloneShape·계량 | 단일 합성·copy isolation·정상/오류 경로 |
| C | D07-C 저장·fingerprint·effective input 연결 | raw old V7와 nonzero 새 입력·내용 identity |
| D | C15 RON MIN, 최종 production capability 공개, 외부 RON/CLI | 지원 식 실제 소비 및 미지원 typed 거부 |
| E | WP04 전체 21 IVA/31 acceptance/10 dimensions, 문서·전체 회귀·비용 | 최종 source 검증 후 단 한 번의 최종 감사 target 고정 |

B/C의 schema·stat·저장 계약은 구현 전 함께 설계한다. A 일부 시험이 최종 공개 경로를 요구하면 D에서 마저 닫고 추적표에 의존을 적는다. 부분 단계의 `authored expression 차단 유지`는 준비 중에 유지하되, **R06 최종 완료에는 지원 범위의 공개 실행 성공이 필요**하다. 마지막까지 blanket 차단한 상태로 E를 완료 처리하지 않는다.

## 5. A — D07-B 잔여 교정·검증

주요 코드: `src/game/battle/core/value_expr_runtime.rs`, `skill_runtime/{cast,projectile}.rs`, `src/game/world/authoring_observation.rs`, `src/authoring/{amount_query,control,clone_shape}.rs`, TrialSession 및 기존 `tests/amount_*` 모듈. 아래 `src` 경로는 모두 `core_enhanced/` 기준이다.

| 조건 | 구현·시험 지침과 필수 oracle |
| --- | --- |
| C08 | 실제 selector로 wrong battle/ordinal를 거부한다. generation/site/fingerprint/source-copy 손상은 private 시험에서만 만든다. 정상 고정 record는 source unit/item/action/delivery 제거 뒤에도 조회된다. 조회 때문에 살아 있는 action 권한을 재발급하거나 현재 unit을 다시 찾아 과거 record를 거부하지 않는다. |
| C09 | active→terminal→새 active→새 terminal에서 active와 마지막 terminal만 보관한다. 이전 terminal Exact는 not-retained. 실제 controlled clone은 immutable Arc 공유이며 반복 평가에 비례해 기록이 늘지 않는다. 아래 C07-OBS01을 함께 교정한다. |
| C10 | 성공 record A 뒤 target/effect/timestamp/receipt 실패·취소를 주입한다. accepted HP/resource/RNG/UUID/queue/log/time/frontier와 A/ordinal 모두 보존, 실패 B 노출 없음. 정상 재시도에서 이중 적용 없음. 내부 결과값만 비교하지 말고 transaction 경계를 검증한다. |
| C11 | allocation 전 admission, 실제 소비 성공 후 commit을 유지한다. old+new record 교체 peak, frame/views/trace/scratch, active와 terminal 소유 payload, clone 비용을 분리한다. exact limit 성공·부족 typed 거부·checked size overflow를 실제 할당 경계에서 검증한다. Arc 핸들만 세고 payload를 누락하지 않는다. |
| C12 | 실제 numeric query의 work/metadata/result 경계 및 trace 중간 취소. query 실패/성공 모두 accepted state·RNG·queue·resources·identity·work/index/receipt 무변경, 동일 성공의 문서/usage 결정적. 취소는 결정적 제어 지점으로 만들고 sleep 경합에 의존하지 않는다. |
| C13 | optional trace off/on/포화, 일반 event log off/eviction에서 필수 amount/node/member trace 동일. 짧은 시나리오와 작은 한도로 포화를 만든다. |
| C14 | 실제 평가 중간값 `2^100+1`을 만들고 마지막 값은 정상 narrowing 범위로 줄인다. 실제 record→typed query encoder→원문에서 정확한 십진수·순서를 검증한다. `serde_json::Value`/f64를 정확성 oracle로 쓰지 않는다. D에서 공개 동적 CLI까지 이어 검증한다. |

`encode_record_for_test`는 수치 인코딩 시험에 재사용 가능하지만 selector·전투 교체 증거가 아니다. C07의 `public_amount_trial_fixture`와 공개 TrialSession 시험을 참고한다. 시험만 추가해서 통과하면 충분한 항목은 제품 코드를 불필요하게 고치지 않는다.

### C07-OBS01 교정 방향

`advance_skill_projectile_runtime`은 과거 hit가 있는 projectile이 새 hit 없이 만료할 때 `expired` 분기로 그냥 반환한다. terminal impact가 없으므로 `resolve_skill_step_delivery`의 pending 감소/후속 정리에 도달하지 않는 경로가 있다. 실제 event/native carrier 종료 순서까지 읽고 수정한다.

- **effect 실행과 delivery 종료 통지를 분리**한다. 과거 hit 뒤 빈 만료를 가짜 impact로 만들어 Damage/Heal/Resources를 다시 실행하는 방식은 금지한다.
- 해당 cast/step의 delivery를 정확히 한 번 terminal 처리하고 기존 `release_deferred_skill_steps`/`cleanup_finished_skill_cast`로 연결한다. 기존 result prefix를 보존하고 새 amount/ordinal/피해 fact를 만들지 않는다. 이미 terminal hit를 처리한 경로에서 중복 감소하지 않는다.
- 기존 `previous_step_damage_gate`는 실제 누적 dealt_damage를 판단한다. zero 피해의 종료를 gate 성공으로 만들지 않는다. terminal unsatisfied와 pending을 구별해 후속 step의 기존 skip/정리 규칙을 따른다. 기존 `native_action_piercing_empty_final_expiry_closes_carrier_without_releasing_gate`의 gate oracle을 약화하지 않는다.
- 양수/0 피해의 두 적중 후 빈 만료, 처음부터 무대상 만료, max_pierces terminal hit, 후속 damage-gated step 유무, 실패·취소·재시도를 검사한다. native action settlement와 cast/pending cleanup이 함께 닫혀야 한다. C07의 latest record/ordinal, D04-F01 무대상 후속 resource 완료를 유지한다.

`capture_authoring_terminal`은 terminal capsule로 observer/trace를 이동한다. 기존 admission 계약에 빠진 실제 payload가 있으면 닫힌 owner의 cached extent와 이전/새 payload 공존 preflight를 보강한다. 새 전투/세션 합산 상한 정책을 임의로 추가하지 않는다.

## 6. B — WP03 effective stat·실제 사본·cadence

### 6.1 단일 합성

현재 `stat_pipeline::persistent_stats_for_draft_profile`은 base→GrowthStack→binding Permanent/enhancement를 처리한다. weapon numeric contribution은 아직 없고 enhancement는 `base_uuid`로 여러 row의 level을 합산한다. 이를 아래 순서로 교정한다.

1. `UnitStats.spell_power:u32=0`, `StatId::SpellPower`, weapon numeric contribution의 attack/spell_power 기본0을 추가한다. `WeaponCombatProfile` 또는 그 안의 작은 typed 구조를 사용하되 0 직렬화 생략 계약을 지킨다.
2. 기존 employee→fragment rank/growth profile→consumable battle profile→behavioral weapon override 순서를 보존한다. `apply_weapon_profile`은 행동/cadence만 맡고 numeric contribution을 더하지 않는다.
3. `persistent_stats_for_draft_profile` 한 곳에서 **실제 선택된 equipped weapon 수치 한 번 → GrowthStack → equipped binding 순서 Permanent+그 사본 enhancement**를 합성한다. 그 뒤 battle 경로만 기존 scenario scale/starting HP를 적용한다. 중복 weapon 거부를 유지한다.
4. 새 weapon base의 u32 합산 overflow는 draft construction의 typed error다. SpellPower Flat/Percent는 기존 Attack의 saturating·정수 절삭을 따른다. expression의 checked arithmetic와 혼동하지 않는다. 기존 Percent 연산을 합치거나 재정렬하지 않는다.
5. Certified는 같은 persistent 합성 함수를 사용하고 기존 consumable/readiness/current HP/scenario scale 제외를 유지한다. 유효 roster 자격과 그 안의 일시적 readiness 수치를 구별한다.

필수 기대값: employee40+weapon40+KillStack10+Permanent5+enhancement5=100; weapon만0이면60, growth만0이면90. 동일 profile의 재구성이 값을 누적하지 않는다. SpellPower20→35는 SpellPower 식만15 증가하고 Attack/Physical·Magic type은 불변이다. Magic type이나 기존 Attack 배율·scenario override에서 SpellPower를 추정하지 않는다.

### 6.2 사본 강화 결속

`BattleEquipmentEnhancement`에는 현재 `base_uuid`/level만 있다. `BattleEquipmentBinding.persistent_copy_uuid`와 정확히 결속할 사본 ID를 보강한다. `combat_setup/player_spawns.rs`의 일반 draft와 `certified_combat_profile_for_employee` 양 producer를 변경하고 inventory의 실제 owned copy/base를 검증한다.

- 같은 definition인 A level1/B level2, level당+5 → 총+15이며 기존 잘못된+30을 허용하지 않는다.
- 같은 persistent copy 중복 row, copy/base 불일치, 없는 owned copy, binding과 enhancement 결속 불일치는 typed 거부한다. definition UUID 합산/첫 사본 선택으로 fallback하지 않는다.
- 시나리오의 persistent copy 없는 장비는 강화0의 기존 의미를 유지한다. 출처 없는 nonzero 강화를 definition으로 붙이지 않는다. 실제 생성 경로 전체에서 typed validation이 도달하는지 확인한다.
- D04의 runtime item ID 직접 조회·owner/base/copy/internal instance 검사는 보존한다. 강화용 persistent identity를 runtime ID로 바꾸지 않는다.

### 6.3 AttackSpeedPoints와 runtime 입력

`UnitCombatProfile.basic_attack_speed_percent`는 유지하고 typed selector `AttackSpeedPoints`의 원본으로 연결한다(100=base). `scale_attack_interval_ms`와 `for_employee`/`apply_weapon_profile`에는 assert/expect 경계가 있으므로 모든 외부 입력 경로에서 base>0, points>0, `ceil(base*100/points) <= u64::MAX`를 panic 전에 검사한다. 필요하면 checked Result helper를 도입하고 실제 caller를 옮긴다.

- 1500@200→750ms, 1001@150→668ms, 1@200→1ms; 0/overflow는 typed 거부.
- basic attack interval만 변한다. projectile flight/area tick/focus/bombardment timer 불변. 기존 fragment interval modifier는 기존 적용 순서를 보존한다.
- `RuntimeUnit`은 현재 points를 보관하지 않는다. `core/build.rs`의 양 spawn 경로에서 검증된 원본 points를 전달할 저장 위치를 정하고 Snapshot/Live frame reader에 연결한다. 반올림된 interval에서 points를 역산하지 않는다.
- `value_expr_runtime`의 `SpellPower | AttackSpeedPoints => MissingContext` 임시 분기를 실제 stat/points 읽기로 교체한다. Snapshot은 release 값, Live는 실제 effect 시작 값이다. member selector의 공개 지원 범위를 이유 없이 확장하지 않는다.
- 모든 struct literal, serde, canonical hash, `CloneShape`, retained/clone/preflight 계량과 최종 explain 입력 설명을 함께 갱신한다.

## 7. C — D07-C 저장·Certified input fingerprint

`combat_power.rs`는 현재 model version/total/member power를 반환하고 input fingerprint는 없다. 기존 total 계산/weight/model version을 변경하지 않고 별도 `CertifiedInputFingerprint`를 추가한다.

- persistent employee identity/level, effective Attack/SpellPower/cadence, selected weapon numeric/profile, 실제 copy/level, growth/modifier, content semantic digest를 canonical sort/hash한다. 같아야 하는 입력 순서는 정규화하되 의미 있는 modifier 적용 순서는 지운 채 hash하지 않는다.
- 실제 공개/내부 소비 경로에서 fingerprint를 확인할 수 있게 연결한다. 입력 의미가 바뀌면 fingerprint 변화, 동일 의미 ordering은 동일. 새 SpellPower input을 기존 Certified total이 평가한다고 표시하지 않고 `evaluation_model_pending` 등 별도 capability로 진실하게 설명한다. 새 weight·가짜 cache는 만들지 않는다.
- `world/combat.rs` 등 소비자를 조사한다. 외부 서버/Unity 수정이 필수인 비호환 변경 대신 core의 additive 관측을 우선한다. 외부 변경 필요가 드러나면 경계를 사용자에게 알린다.

`world/active_run.rs`의 V7 import는 typed deserialize→재직렬화 checkpoint SHA 비교다. 단순 serde default만 추가하면 old checksum이 깨질 수 있다.

1. 새 persistent/definition 0필드는 `default + skip_serializing_if` 또는 동일한 zero-struct omission을 적용한다. old field 누락 bytes의 canonical serialization과 기존 공식 content digest를 보존한다.
2. 이전 raw fixture SHA `47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9`를 새 코드에 그대로 넣어 정상 import/재개한다. 새 코드가 만든 round-trip만으로 old 호환을 주장하지 않는다.
3. nonzero SpellPower/growth/실제 copy enhancement를 새 process/application에서 round-trip하고 effective stat·checksum을 대조한다. nonzero 데이터는 실제 저장/hash에 포함된다. 변조 checksum/content identity는 거부하고 기존 store를 보존한다.
4. transient action/result/resource lease를 persistent authority로 복원하지 않는다. 전역 버전 bump로 old save를 전부 거부하거나 checksum/identity를 우회하지 않는다. 사용자 save 자동 변환·삭제도 하지 않는다.

## 8. D — 공개 입력·실행과 C15

### RON MIN

현재 `ability.rs::SkillEffectDef::{Damage,Heal}.amount`는 derive Deserialize의 i32다. RON 0.5.1은 음수의 절댓값을 i32로 파싱하는 경로 때문에 MIN이 실패한다. 우선 해당 signed amount 필드에 더 넓은 정수로 읽고 checked i32 변환하는 좁은 serde adapter를 검토한다. wire 숫자 형태·Serialize·deny_unknown_fields·범위 진단을 유지한다. 전체 parser/의존성 교체는 필요성과 영향 증거 없이 선택하지 않는다.

외부 raw RON으로 MIN/MIN+1/-1/0/MAX 및 범위 밖 양끝을 검증한다. Heal의 기존 signed runtime 의미는 유지하고, **공개 음수 Damage 거부는 그대로**다. 정상 parse와 semantic 거부를 구별한다. signed 필드의 공유 adapter 적용 범위를 명시하며 expression의 bounded parse/i128/깊이·노드 한도를 약화하지 않는다.

### 최종 capability 전환

`ValueExprCatalog::ensure_runtime_publishable`은 현재 모든 authored expression을 `content_capability_unsupported`로 거부한다. A–C의 실제 consumer 연결 후 **검증된 R06 지원 집합의 공개를 허용**하도록 전환한다. 무조건 Ok로 바꾸는 방식은 금지한다.

- compiler의 unit/context/delivery/result availability와 bound 검증을 유지한다. unsupported relation·unknown opcode·필수 context 없는 Snapshot·future/stale result 등은 typed 거부한다.
- public RON→winning compiled catalog→실제 runtime effect→record→TrialSession preview/explain→release CLI까지 연결한다. 테스트 내부 catalog 주입이나 `test-fixtures`만 통과한 것을 공개 지원으로 보고하지 않는다.
- legacy 및 authored expression 각각 immediate/projectile/area의 지원 조합, source/target/context와 owner grouping을 시험한다. 앞 effect의 실제 피해4를 뒤 heal4가 읽는 prefix와 action/effect/delivery identity를 확인한다.
- 실제 큰 i128 trace 출력, CLI trace on/off amount 문서 byte 동등, selector와 최대4문서 shape를 검증한다. F01 오류 입력의 exit2/panic 없음도 유지한다.
- static explain은 `performed=false`, 실제 문맥 없는 미래 amount는 명시적 unavailable/needs-runtime-context다. DamageExpr/mixed usefulness는 계산 예측 없이 `Indeterminate`와 기존 target eligibility를 유지한다. amount는 final damage와 구별한다.
- 현재 publication 차단을 기대하는 시험은 지원 사례 성공/실제 미지원 사례 거부로 갱신한다. 거부 시험을 삭제해 통과시키지 않는다.

효과·상태의 출처 장비별 집계(R11/R17), empty tile 후속 기반, 일반 함수/VM, R07/R19 정책, 편집기, 실험실 스트레스/모드 cache, 안전 설정 RON화, metrics, 자동 스킬 취소는 이번 범위가 아니다. 20분 시험은 모든 내부 단계에서 제외한다.

## 9. E — 전체 수용표·검증·성능

최신 Goal Set의 **31 acceptance와 IVA-CAVER-01–21 전체, 10 Validation Dimensions**를 최종 coverage 표에 연결한다. D07의 C01–C16과 Goal의 사용자 완료 C01–C08은 서로 다른 번호다. 한 시험이 여러 조건을 증명해도 되지만 각 row에 실제 selector/입력/기대·실제 결과/소스 identity/원자료/재사용 또는 신규 여부를 기록한다. 미구현/미검증을 N/A로 바꾸지 않는다.

| IVA | 최종 확인할 핵심 |
| --- | --- |
| 01–03 | 단위, four rounding, checked overflow/0 division/clamp/final narrowing와 정확한 오류 site |
| 04 | 기존 literal의 signed/zero/unsigned_abs 의미; 공개 거부와 내부 compatibility 구별 |
| 05–06 | 실제 release Snapshot/소비 Live, 시간·HP, target availability·source 제거·기존 cancellation |
| 07–08 | finite empty/Max/Sum/Count·permutation, WholeSet/tile/Side/ActualEmployee/EquippedSlot·명시적 필터 |
| 09–11 | 단일 stat 합성·실제 copy 강화·SpellPower·checked cadence |
| 12–14 | resource owner/lease 격리·무변경 조회, settled result/prefix, native/controlled 실패 원자성·재시도 |
| 15–16 | raw old V7/새 입력 fresh-process round-trip·identity 거부, override/zero omission/fingerprint |
| 17–18 | 실제 동일 문맥 amount/trace 및 공개 RON/CLI 지원·미지원·action/effect/delivery 결속 |
| 19–20 | parse/compile/capture/impact/query 경계, limit−1/limit/+1, 취소·finite workload·retained/clone/cleanup·비용 |
| 21 | 기존 공식 literal pack, R01/R04/R05 영향 회귀, 전체 feature/build와 후속 독립 감사 연결 |

10 dimensions는 success, invalid_input, failure_rollback, retry_idempotency, ordering_concurrency, boundary_empty_max, disconnect_restart_persistence, live_data, producer_consumer, legacy_fallback_absence다. 동시성 행은 deterministic ordering/논리적 single writer 계약이며 병렬 mutation 허가가 아니다.

### 시험 실행

cwd는 repository root, Cargo는 `/home/blast/.cargo/bin/cargo`, 확인된 cache는 `CARGO_HOME=/tmp/wp02-cargo-home`이다. PATH/CARGO_TARGET_DIR/toolchain/lock identity를 기록한다. 제품 전체 시험은 기본 병렬이며 `RUST_TEST_THREADS`의 외부 강제 설정도 확인한다. 아래는 최종 source의 필수 명령이다.

```bash
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --lib authoring --offline --locked
cargo test -p game_core --lib value_expr --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo test -p game_core --all-targets --features test-fixtures --offline --locked
cargo check -p game_core --all-targets --features event_log_checks --offline --locked
cargo build -p game_core --release --bins --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
cargo fmt --check
```

추가로 실제 작성한 durable tests/외부 fixtures의 selector를 `--list`로 확인하고 0건 성공을 배제한다. `value_expr_contract`가 아직 없으면 WP04의 공개 integration seam으로 만들거나 동등한 실제 test owner와 mapping을 명시한다. release `author_content`/`package_official_content`/`consume_content_pack`의 현재 usage를 읽고 실제 fixture 경로로 호출한다. placeholder 명령을 실행했다고 기록하지 않는다.

Clippy의 두 allow는 기존 baseline과 동일한 알려진 lint 처리이며 새로운 경고를 숨기는 포괄 allow를 추가하지 않는다. repo fmt의 기존 무관한 차이는 초기 결과와 구별하고 변경 파일은 형식을 맞춘다. 전체 fmt 실패를 전체 PASS로 기록하거나 unrelated mass-format을 섞지 않는다. 최종 계약의 baseline 예외 판단이 필요한 경우 원자료와 변경 범위를 인계한다.

로그는 실행 시작부터 파일에 기록하고 진행을 표시한다. 래퍼가 완료까지 출력을 숨기지 않게 한다. 실행 중단·재시도·환경 실패도 보존하고 실패 한 번 뒤 성공한 값을 유일한 실행처럼 쓰지 않는다. 각 C마다 전체 회귀를 반복할 필요는 없다. focused 검증 후 공유 schema/runtime/save 변경의 통합 target에서 full baseline을 수행하고, 이후 변경이 무효화한 검증만 재실행한다.

### PC36·R05·신규 workload

이번 실행은 runtime/stat/소유권/계량을 바꾸므로 최종 PC36/R05는 **재측정**한다. F02의 old 근거는 비교 기준이며 새 결과를 대신하지 않는다. 정확한 cwd/env/명령의 선행 영수증은 `corrections/wp02_d07_b_f02_001/evidence/{pc_final_001,r05_final_001}.json`이다. 출력 경로만 새 실행 evidence로 바꾸고 old 파일을 덮어쓰지 않는다.

- PC36: release lib의 `pc_load_baseline --ignored --nocapture --test-threads=1`, `R06_BASELINE_SAMPLES=11`, `R06_BASELINE_OUTPUT`을 새 절대 경로로 지정. 이 측정의 단일 스레드는 제품 전체 시험의 단일 스레드와 다르다. 기존 성공31/typed 거부5와 gameplay/state/error 원자료를 대조한다.
- R05: cwd `core_enhanced/`, `cargo run --manifest-path tools/runtime_architecture_final_validation/Cargo.toml --bin final_performance --release --features allocation-observer --offline --locked -- evidence/P01_PERFORMANCE_BASELINE.json` 뒤에 새 raw JSON 절대 경로를 전달한다. build/run 영수증과 원자료를 보존한다.
- concurrent peak cap `9,030,188 bytes` **단독 초과만 경고**다. 기존 `9,049,037`/초과18,849는 F02 측정값이며 이번 결과가 아니다. raw exit101을0으로 바꾸지 않고 정책 판정을 별도 기록한다. 다른 operation/retained/parity 등 hard failure는 교정한다. cap을 올리거나 좋은 sample만 선택하지 않는다.
- 별도 finite many-cast/repeated-area/copy-isolated expression workload로 관측 on/off gameplay, 입력 규모별 work/retained/clone, 교체·terminal cleanup을 검증한다. 측정치와 safety limit를 공식 밸런스·전투당 실제 RSS로 표현하지 않는다. 20분 soak는 넣지 않는다.

## 10. 최종 문서와 동결

canonical 계약 문서(`core_runtime_contract`, `data_loading_contract`, `skill_target_contract`, `game_rulebook`, 기존 authoring architecture owner)를 찾아 구현된 의미만 반영한다. 예시를 실제 지원 문법으로 검증하고 old literal/미지원 경계를 설명한다. R 순서·의존성, successor 정책/P2G, GEX, validator, AGENTS와 전역 설정은 변경하지 않는다.

최종 인계에는 다음을 포함한다.

1. F03/D07-B/D07-C/WP03/WP04별 실제 완료와 31 acceptance·21 IVA·10 dimensions 결과, C07 최종 검토 포함 범위, 두 OBS의 재현→교정→회귀 증거. Goal 완료 C08의 독립 감사는 대기로 표시한다.
2. 초기 C07 대비 이번 delta와 최초 R06 baseline 대비 누적 변경을 연결한다. 이번 실행이 작성하지 않은 변경을 구별한다. `tests/battle_showcase.rs` 등 기존/외부 추가 파일도 membership에서 누락하지 않는다.
3. 필요한 source/data/test/config/lock/fixture/toolchain identity, before/after archive, 신규 파일 부재↔존재, 정·역 patch, 원자료·실패 이력·명령 영수증·재사용 판정과 필수 지시서/검토를 고정한다. 하드코딩된 347개/1,560개 성공을 복사하지 않는다.
4. 새 manifest SHA-256와 source digest, currentness 및 격리 디렉터리에서 양방향 byte 복원 결과를 제공한다. 검증 도구는 대상 목록의 실제 추가/삭제를 검사해야 한다. hash만 남기고 원본 byte를 버리지 않는다.
5. ROADMAP_STATUS는 `R06 구현·자체 검증 완료 / 최종 감사 대기`로 갱신하고 최종 target 하나를 가리킨다. 미해결 hard failure가 있으면 이 문구를 쓰지 않고 정확한 blocker와 재개 위치를 남긴다.

최종 target 고정 뒤 제품/target 기록 mutation을 멈춘다. 사용자에게 인계서와 manifest를 전달하고 감사 요청 대기에서 종료한다. 구현자가 독립 감사 결과를 대신 쓰거나 다음 R을 시작하지 않는다.
