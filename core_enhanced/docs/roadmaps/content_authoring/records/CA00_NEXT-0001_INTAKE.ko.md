# CA00 — NEXT-0001 인수·주요 의존 경계 1차 조사

작성일: 2026-09-09 · 작성자: Codex / 이번 사용자 요청 세션 · 상태: **NEXT-0001 조사·기록 완료 / CA00 전체는 일부 진행**

## 1. 요청과 작업 경계

- 요청 원문: 이번 대화의 “첫 진입점: core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md … 요청 시작 시 기록된 ‘다음 작업’ 하나를 수행하세요.”
- 요청 시작 카드: [ROADMAP_STATUS 기록 0002](../ROADMAP_STATUS.md)의 `NEXT-0001 — 기존 진행 인수·주요 의존 경계 1차 확인·다음 범위 확정`.
- worktree: `/mnt/f/work/simulator`; branch `cleanup/simulator-dirty-worktree-20260803`; HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`; 기존 미커밋 변경이 있는 상태를 인수한다.
- 허용: 원본·코드·기존 실행 증거 조회, 필요한 CA00 조사 기록과 현황 갱신, 안전한 비파괴 로컬 진단.
- 종료점: 기존 진행·주요 공유 경계·다음 범위 하나와 직접 전제·미검토 부분을 이 기록에 남긴 뒤 현황을 갱신하고 종료한다. CA00 전체 요구 추적이나 다음 작업 실행까지 확대하지 않는다.
- P2G/GEX 적용: `inspect`와 조사 문서 작성이다. 정식 P2G-20/40, Freeze/Goal 작성, GEX 구현·검증·독립 감사는 이번 작업이 아니다.
- 코드·RON·저장 데이터·검증기·AGENTS·P2G/GEX 규칙·기존 Goal/감사 원본은 수정하지 않는다. 이 파일은 정식 Topic/Goal을 대신하지 않는 CA00 조사 원본이다.

## 2. 작업공간과 지침 인수

상세 해시·조회 결과·읽은 코드 구간은 [이번 조사 증거](CA00_NEXT-0001_EVIDENCE.json)에 둔다. 이 JSON은 조사 부속 자료이며 정식 P2G/GEX Receipt나 전역 의존성 장부가 아니다.

- `git status --porcelain=v1 -uall -z`: 기존 16,483행 = 수정 167, 삭제 24, 미추적 16,292. 이번 조사 파일은 제외했다. 축약 `git status --short`의 디렉터리 수와 비교하지 않는다.
- 코드·RON·관련 문서의 Git 목록에서 선택한 텍스트 경로 1,680개의 해시/부재를 비교 기준으로 저장했다. 전체 자산 해시 시도는 중단하고 이 범위로 좁혔다. 무시된 파일·실제 개인 저장 파일까지 동일성을 보증하는 기록은 아니다.
- `ability.rs`, `data/{live_ron,mod,skill_data}.rs`, `skill_fragment.rs`, `battle/core/sim.rs`, `world/{state,active_run}.rs`, Cargo 입력은 이미 수정 상태였다. `src/application.rs`, Boss runtime·기존 P2G/GEX 실행 폴더와 로드맵은 미추적 경로도 포함한다. 현재 사실은 clean HEAD가 아니라 **위 HEAD + 실제 dirty 파일 해시**에 귀속한다.
- 조상 경로 및 저장소 내 `AGENTS.md` 검색 결과 없음. `.agents`, `.codex`에서도 추가 파일을 찾지 못했다. 기존 기록과 이번 사용자의 범위를 따른다.
- 이번 thread의 `get_goal` 결과는 `goal: null`이다. 이를 저장소 Goal 부재로 해석하지 않는다. 기존 관련 Master 실행은 아래 원본에서 terminal임을 확인했다. 격리된 process 조회만으로 다른 세션의 부재를 확인할 수는 없으며, 같은 파일의 외부 동시 수정은 종료 시 비교로 확인한다.

| 지침 | 실제 위치·identity | 이번 적용 |
| --- | --- | --- |
| 운영 규칙 | [OPERATING_RULES](../OPERATING_RULES.md), v1.1 | NEXT-0001 한 단위와 원본/현황 분리 |
| 로드맵 | [ROADMAP](../roadmap/ROADMAP.ko.md), v2.1; [CA00](../roadmap/goals/CA00.md), [의존성](../roadmap/DEPENDENCIES.ko.md) | 후보·실제 코드·실행 증거를 구분 |
| 설계·K 계약 | [DESIGN_WORKFLOW](../roadmap/DESIGN_WORKFLOW.ko.md), [K owner](../roadmap/CONTRACTS.ko.md), [A01](../roadmap/architecture/01_SYSTEM.ko.md), [A02](../roadmap/architecture/02_AUTHORING_AND_COMPILER.ko.md) | 제안 타입을 현재 API로 간주하지 않음; 정식 D2/DG 수행 아님 |
| 현재 P2G 문서 | [저장소 원문](../../../policy_to_goal_pipeline_revised_v2.md), `2026-09-09-r2`, SHA-256 `e439da72…079b` | Reader, §8.5, §8.9 적용; `inspect`와 조사 기록 |
| P2G 배포 사본 | [guides 사본](../guides/policy_to_goal_pipeline_revised_v2.md) | 저장소 원문과 **바이트 동일**. 별도 authority로 편집하지 않음 |
| 과거 P2G | [archive 원문](../../../archive/policy_to_goal_pipeline.md), SHA-256 `87474280…4c1c` | CRAR `PP-CRAR-P2G40-002`에 묶인 지침 해시와 동일. 과거 artifact 해석에 사용하고 r2 진행 규칙과 혼합하지 않음 |
| GEX | [저장소 GEX](../../../goal_execution_pipeline.md), SHA-256 `5c19b62a…6c72` | Reader의 inspect/execute·감사 분리 확인. 과거 Master target의 GEX 해시와 동일 |

**문서 경로 문제:** 기존 `core_enhanced/docs/policy_to_goal_pipeline.md`는 현재 없으며 GEX·engineering_workflow 등에 그 옛 경로 링크가 남아 있다. archive의 내용 identity와 새 r2의 실제 경로는 위와 같이 찾았다. 지침 내용 조회에는 필요한 원문이 확보됐지만, runner/AGENTS의 r2 라우팅 적용까지 확인된 것은 아니다. 이번 권한으로 링크를 맞추려 P2G/GEX·AGENTS를 고치지 않는다. 후속 정식 실행은 실제 r2 경로를 명시하고 필수 runner가 옛 경로를 강제하는 경우 해당 지침 owner에 반환한다.

## 3. 기존 Topic·Goal·실행·감사 인수

검색 범위는 `core_enhanced/docs/goals`, `docs/decisions`, `docs/architecture`의 제작 기반/커뮤니티/CA00/CA01 표기와 Working Policy·Review·Phase·Handoff 파일명, 로드맵 패키지다. **이 범위에서 커뮤니티 제작 기반의 정식 Topic/Goal·개별 COV 실행 행은 발견하지 못했다.** `roadmap/goals/CAxx.md`는 정식 Goal이 아니다. 기존 기능과 완료 기록은 다음처럼 인수한다.

| 원래 owner | 인수한 identity·진행 | 재사용 범위와 한계 |
| --- | --- | --- |
| [프로젝트 지도 Goal](../../../goals/project_architecture_map/PLAN.md) / [문서 검토](../../../goals/project_architecture_map/COMPLETION_AUDIT.md) | 2026-09-06, `complete`, M01–M10 충족 | [project_map](../../../project_map.ko.md)의 입력→검증→실행→상태/저장 지도를 탐색 입구로 재사용. 신규 제작 기능·전 콘텐츠 표현력 감사나 테스트 결과가 아님 |
| [CRAR Working Policy](../../../goals/current_runtime_architecture_remediation/WORKING_POLICY.ko.md) | `WPD-CRAR-2026-08-22`, `WR-CRAR-P2G20-COMPLETE-002` | 런타임 결함 교정 Topic. 임시 유물/침식 구현으로 확대하지 않았던 범위를 보존 |
| [CRAR Code Review](../../../goals/current_runtime_architecture_remediation/P2G_20_CORE_ONLY_REVIEW_002.ko.md) / [Freeze](../../../goals/current_runtime_architecture_remediation/P2G_30_CORE_ONLY_FREEZE_002.ko.md) / [Phase Plan](../../../goals/current_runtime_architecture_remediation/P2G_40_CORE_ONLY_PHASE_PLAN_002.ko.md) | `RB-CRAR-P2G20-002`, `RSB-CRAR-P2G20-002`, `FPF-CRAR-P2G30-002`, `PP-CRAR-P2G40-002` | 기존 구조 교정의 순서·경계 근거다. 신규 CA01에 자동 적용되는 Phase Plan이 아님 |
| [CRAR Goal Set](../../../goals/current_runtime_architecture_remediation/P2G_50_CORE_ONLY_GOAL_SET_002.ko.md) / [Handoff](../../../goals/current_runtime_architecture_remediation/P2G_60_CORE_ONLY_HANDOFF_002.ko.md) | `GS-CRAR-P2G50-002`, `HA-CRAR-P2G60-002` | `001` chain과 중단된 `p01_preservation_baseline`은 이력. 새로 실행하지 않음 |
| [CRAR Master 실행](../../../goals/current_runtime_architecture_remediation/execution/master_goal_set/PLAN.md) / [Master 감사](../../../goals/current_runtime_architecture_remediation/execution/master_goal_set/COMPLETION_AUDIT.md) / [최종 receipt](../../../../evidence/P08_GOAL_SET_GEX30_RECEIPT_002.json) | `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-MASTER)`, GEX-30 Attempt 3 `Complete / High` | P2G 문서에 남은 `Authored / Not Activated`는 당시 authoring checkpoint다. **더 진행된 GEX 원본을 우선**하며 초기화하지 않음 |
| [CRAR P02 감사](../../../goals/current_runtime_architecture_remediation/execution/p02_validated_data/COMPLETION_AUDIT.md) / [currentness 007](../../../goals/current_runtime_architecture_remediation/execution/p02_validated_data/SUCCESSOR_CURRENTNESS_AUDIT_007.md) | Attempt 1 `Not Complete` → 교정 뒤 Attempt 2 `Complete / High`; `ATR-CRAR-C2-P02-GEX20-007` | 이미 닫힌 bare construction 우회 결함을 다시 미해결로 분류하지 않음. opaque publication을 재사용하되 외부 loader 완료로 확대하지 않음 |
| [Boss v1 Master 감사](../../../goals/boss_runtime_v1_implementation_master/COMPLETION_AUDIT.md) | 2026-08-09 `Complete / High` | 기존 Boss graph·tracking·Little Red 참고 구현의 이력. 신규 공통 ActionHandle/최종 보스 콘텐츠의 완료 근거가 아님 |
| [조건부 Wave·보스 소환 Working Policy](../../../goals/conditional_wave_and_boss_summon_policy/WORKING_POLICY.ko.md) / [Master 감사](../../../goals/conditional_wave_and_boss_summon_policy/execution/master/COMPLETION_AUDIT.md) | `HA-CWBS-P2G60-002`, `GS-CWBS-P2G50-002`, `GOAL-CWBS-MASTER`, `AUDIT-CWBS-MASTER-GEX30-001`: `Complete / High` | Wave·Spawn·소환·결과의 기존 구현 이력. 현재 전체 target 재감사는 안 했으므로 CA14/16/19 전체의 current 완료로 표시하지 않음 |

**이번 currentness 대조:** [Master target](../../../../evidence/P08_MASTER_AUDIT_TARGET_002.json)의 32개 member, [교정 submanifest](../../../../evidence/P08_AUDIT_TARGET_002.json)의 16개 member, [구현 submanifest](../../../../evidence/P08_AUDIT_TARGET.json)의 95개 member가 모두 현재 파일 해시와 일치했다. 부가 repository/submanifest 바인딩까지 포함한 비교는 각각 36/36, 17/17, 95/95다. 새 GEX 감사는 아니며 target에 없는 모든 코드·RON·환경의 currentness를 증명하지 않는다. 과거 Core 1,159개 PASS는 그 receipt의 실행 결과이며 이번에 재실행하지 않았다.

## 4. 주요 의존 경계 1차 확인

아래는 선택 범위의 코드 관찰과 후속 검토 제안이다. 로드맵 edge·정식 Phase Plan은 변경하지 않았다. source identity와 읽은 구간은 증거 JSON에 연결한다.

| 경계 | 현재 생산자 → 검증/실행/소비자 | 확인한 관계와 다음 owner |
| --- | --- | --- |
| B01 loader·K02/K03 | [live_ron.rs](../../../../src/game/data/live_ron.rs) `GameplayRonSourceSet::embedded` → `parse_gameplay_ron_sources`/`into_validated`; [data/mod.rs](../../../../src/game/data/mod.rs) `load_live_sources` → generated preview 검증 → `ValidatedGameData`; [application.rs](../../../../src/application.rs) `CoreApplication::new` | embedded manifest 1 + 24 source, 실패 시 publication token 미반환은 기존 경계. CA01 최소 schema는 이 검증 경계를 소비한다(**기능 필수**, 기존 일부 충족). 외부 다중 팩/세대 fingerprint는 현재 이 경로에 없으며 CA02/03 전체를 CA01의 선행으로 요구할 근거는 없다. 정식 P2G-20/40이 최소 최신 slice·publication의 동시 전환을 확정 |
| B02 작성·K01/K02/K07 | [skill_data.rs](../../../../src/game/data/skill_data.rs) `RawSkillDatabase`/`try_into_unvalidated_database`/range preset 해석 → `SkillDef`; [ability.rs](../../../../src/game/ability.rs) `SkillStepDef`, `SkillEffectDef` | step에는 ID·target·delivery·effects가 있지만 개별 effect에는 stable ID/tag/property descriptor가 없다. step의 공통 target/delivery를 effect별 변경으로 오인하면 다른 effect도 바뀐다. CA01의 identity/EffectSite·ActionSite 구분은 CA12 selector의 **기능 필수 계약**. 모든 유물 활성 알고리즘을 CA01에 선구현하는 뜻은 아님 |
| B03 행동·K04/K06/K10 | [sim.rs](../../../../src/game/battle/core/sim.rs) `build_skill_step_commands` → [commands.rs](../../../../src/game/battle/core/commands.rs) `ApplyDamage`/`ApplyHeal` → committed damage/HP 변경; `record_skill_step_live_signals`는 전술 자원 효과를 별도 전달 | 현재 고정량 Damage/Heal/Instant 경로를 재사용할 수 있다. `ModifyDamage`는 step의 Damage에 합산된다. 전술 자원 match가 비었다고 미지원으로 단정하지 않음. 최소 slice의 parser·validator·executor·지원 설명은 **동시 전환**, 기존 positive/negative 시험은 **검증 선행**. 모든 CA04–09의 미래 opcode를 먼저 만드는 관계는 확인되지 않음 |
| B04 상태·사본·K01/K05 | [skill_fragment.rs](../../../../src/game/skill_fragment.rs) `stacks`/`progress: BTreeMap<SkillFragmentId,…>`와 `SkillFragmentLoadout`; [reward.rs](../../../../src/game/reward.rs) `GrantSkillFragment`; [maintenance.rs](../../../../src/game/world/maintenance.rs) equip/awaken | 종류별 수량·공통 성장/개화가 current actual. 목표 U08 사본별 개화는 미구현 delta다. 사본 ID·지급·장착·성장·투영 및 B06 저장 전환은 **동시 전환 검토 대상**. CA08의 모든 지속 효과가 사본 보유에 필수라는 근거는 없고, 형태가 그 기능을 소비하는 부분만 연결할 것: CA10/04 및 P2G-20/40 owner |
| B05 전투 identity·K01/K05/K08 | [build.rs](../../../../src/game/battle/core/build.rs) `make_item_instance_id(equipment_uuid, side, unit_instance, index)`; [types.rs](../../../../src/game/battle/core/types.rs) `RuntimeItem`, `AbilityProcKey`; [sim.rs](../../../../src/game/battle/core/sim.rs) `should_fire_triggered_ability` | 전투 item instance와 proc key는 이미 있다. 영구 사본 ID와 재배치 epoch를 구분하지 않은 채 유물 전투당 횟수를 재사용할 수는 없다. 이 관찰만으로 재배치 전후 횟수의 실제 동작을 검증했다고 주장하지 않음. CA04/11/13에서 source binding과 수명을 확인 |
| B06 snapshot·저장·K03/K05 | [state.rs](../../../../src/game/world/state.rs) `RunCheckpointPayload::{capture,restore_into_state}`, `BattleEntrySnapshot::{capture,restore}`는 inventory/roster/skill_fragments를 함께 보유; [active_run.rs](../../../../src/game/world/active_run.rs) V5 writer, V4/V5 reader | 사본화·새 Run 상태를 도입하는 owner가 해당 capture/restore·검증을 **동시 전환**해야 한다. CA19까지 지연하면 오래된 ID로 복원된다. 현재 `checkpoint_sha256`은 payload 무결성이고 resolved content fingerprint가 아니다. CA02 세대 pin/저장 일치와 CA19 최종 통합을 분리 검토 |
| B07 보스·K04/K09 | [boss_runtime_data.rs](../../../../src/game/data/boss_runtime_data.rs) raw/compiled task 및 미지원 거부 → [boss_runtime.rs](../../../../src/game/battle/core/boss_runtime.rs) `CompiledBossTask` dispatcher | RouteCombat/AcquireTarget/Move/Rejoin/Wait/Summon/EmitSignal 경로가 있고 `RepathToProtectedObjective`는 명시 거부한다. raw/compiled task에 명시 스킬 invocation task는 없다. CA16의 ActionHandle·완료/취소 연결은 후속 delta다. CA09/14 전체를 보스 첫 스킬 호출의 필수 선행이라고 확정하지 않고 해당 기믹 소비 부분만 P2G-20/40에서 판단 |

**순서 판단의 범위:** CA03→CA04는 공통 CLI 전체보다 필요한 관찰 포트가 선행인지, CA08→CA10은 보유/개화와 지속 효과 적용을 나눌 수 있는지, CA09/14→CA16은 스킬 호출과 개별 기믹을 나눌 수 있는지 구체화했다. 제거가 확정된 edge는 없다. 각각의 fault/취소/복원·중간 상태와 최소 Atomic Group은 아직 정식 P2G-20/40 대상이다.

## 5. CA01 첫 후속 후보와 직접 전제

첫 **정식 검토 후보**는 CA01의 “효과별 identity·변경 가능한 property·최소 Damage/Heal/Instant 작성→검증→기존 kernel 실행 계약”이다. 이유는 B01–B03의 현재 경로를 활용하면서 CA02의 최신 schema와 CA12의 effect 선택이 소비할 계약을 좁게 검토할 수 있기 때문이다. 정식 Topic/Goal identity는 아직 배정하지 않는다.

| 직접 전제 | 실제 충족 수준 | 무시했을 때의 문제·후속 담당 |
| --- | --- | --- |
| 전체 승인 요구와 원문 상태 추적 | [COVERAGE](../roadmap/COVERAGE.ko.md)는 축/대표 예시/작성 열만 있는 계획. 전체 개별 COV 행은 미작성 | 최소 AST가 대표 예시만 표현하고 기본/개화·탱/딜·보스/보상 차이를 빠뜨릴 위험. **다음 작업 CA00 조사 owner**가 원문을 읽고 요구 baseline 작성 |
| 최소 실제 parser/검증/publication | B01–B03을 코드로 재확인. 기존 P02 감사도 인수 | CA01 P2G-20이 source map/unsupported 진단과 compiler→executor/descriptor의 실제 delta를 검토. 새로운 타입의 실행 증거는 아직 없음 |
| 현재 피해→자기 회복 소비 지점 | `SkillStepDef` target은 step 단위. 기존 `hit_gated_self_heal_and_buff_stack_repeat_attack_work_together`는 피해 후 1회 자기 회복을 assert | CA01 최소 fixture를 후반 공식 C02 이식과 혼동하지 않음. 기존 amount는 상수이며 Attack/HP 수식·accepted hit 의미는 CA05/06의 별도 delta/PG |
| 검증 관찰 포트 | [ron_loading](../../../../tests/ron_loading.rs)과 [skill_runtime_contract](../../../../tests/skill_runtime_contract.rs), `skill_data.rs`의 부정 fixture를 읽음 | 실제 실행은 0. `BattleCore`/fixture builder의 debug·test/test-fixtures 가시성은 default release 공용 포트와 다르다. CA01은 해당 시험을, CA03은 제작자용 공개 검증/실행 포트를 소유 |
| identity·상태 범위의 최소 계약 | `SkillId`, step ID, cast sequence, damage source, proc key는 현재 존재. effect/copy/action 새 계약은 제안 | type 이름만 도입한 성공을 금지. CA01이 필요한 port sketch를 정하고 실제 상태가 추가되는 CA owner가 저장·취소를 함께 닫음 |

**정책 basis:** [POLICY_GATES U01–U13](../roadmap/POLICY_GATES.ko.md), [유물 정책 기록](../../../tmp/run_relic_system_design_notes_2026-08-21.ko.md), [침식 정책 기록](../../../tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md), [세 콘텐츠 원문 입구](../roadmap/SOURCES.ko.md#s08--공식-콘텐츠와-15축)를 연결했다. 유물·침식 원문은 사용자 결정과 초안/보류를 분리하며 코드 검토·동결 전이라고 명시한다. 세 콘텐츠 원문 전체를 읽은 것은 아니며 이번에는 문서 상태·15축 설명과 풍운승려 피해/자기 회복 절 등을 표본 확인했다.

U04–U08의 일부는 패키지에서 이전 대화 결정으로만 연결돼 있다. 이번 조사에서 그 대화 원문을 새로 확보했다고 주장하지 않는다. 이미 기록된 결정을 임의로 되묻거나 뒤집지 않고, CA00 나머지 추적에서 provenance 수준을 분리한다. exact approval가 필수인 정식 P2G 단계에서 확보할 수 없으면 해당 요소와 필요한 확인만 질문한다.

## 6. 남은 문제와 다음 작업 하나

다음은 **NEXT-0002 — CA00 남은 원문별 요구·정책 추적 기준선 작성**이다. 담당은 다음 요청의 CA00 조사 작성자다. 초기 workspace/의존 경계 조사를 처음부터 반복하지 않고 이 기록과 변경분을 인수한다.

포함: 세 콘텐츠 원문 전체의 합의된 기계적 요구를 [COVERAGE의 필수 열](../roadmap/COVERAGE.ko.md#개별-요구-행의-필수-열)로 개별 기록한다. 기본/개화·탱/딜·보스/플레이어 보상, 단계·분기·병행·개별 효과를 분리하고 현재 사실/P/R/미정, 담당 CA·PG·최소 긍정/부정 시험을 연결한다. 보스 제작 능력도 원문에서 목록화한다. 기록된 결정의 provenance를 연결하고 CA01 최소 검토 범위의 unresolved assumptions를 정리한다. 조사 원본은 이 `records/` 아래에 작성하며 계획 원본 COVERAGE의 상태를 실행 완료로 덮어쓰지 않는다.

종료점: 전체 원문 행과 미정·비범위 이유가 확인 가능하고 CA01 정식 Topic 범위 제안과 필수 근거/질문이 구체적인 상태, 또는 확보 불가능한 필수 근거·새 정책 선택의 중단 조건. 정식 Freeze/Phase/Goal 작성·P2G 실행·게임 구현·전체 로드맵 후반 함수 조사는 포함하지 않는다.

남은 쟁점의 owner:

- **CA00:** 전체 COV/T00, U 결정의 원문 추적과 최신 문서 상태. 현재 CA00 전체 완료·DG 통과를 주장하지 않는다.
- **CA01 P2G-20/40:** typed slice·지원 descriptor·진단·기존 kernel 소비·최소 동시 전환. CA01 코드·RON·tests는 아직 작성하지 않았다.
- **CA02/상태 도입 owner:** generation pin/fingerprint와 save 일치, 사본화 reader/writer 전환. 현재 V4 importer를 제거하라는 신규 승인으로 NEXT-0001을 해석하지 않는다.
- **문서/정책 owner:** [data_loading_contract](../../../data_loading_contract.md)의 durable 문단은 Core payload V4만을 기술하지만 실제 Core writer는 V5다. 또한 기존 CRAR의 V4 제거 인증 조건과 새 U04 no-legacy 목표는 서로 다른 시점/범위의 계약이다. 다음 해당 Topic에서 전환 범위를 명시해야 하며 이번에 코드·canonical·과거 Freeze를 교정하지 않았다.
- **후반 CA owner:** 지속 효과/사본 상태의 상세 수명, Boss action milestone·취소·signal, 소환/방어의 기믹별 선행, 효과 선택/연쇄, 전투력·획득·침식·독립 재조합·성능은 미검토 또는 일부 입구 확인이다. 이들이 현재 조사 종료를 막는 전역 선행은 아니다.

## 7. 실행·검증과 종료 판단

| 활동 | 이번 실제 결과 |
| --- | --- |
| 문서 작성 | 이 CA00 조사 원본·부속 증거 작성 후 ROADMAP_STATUS 요약/다음 카드 갱신 |
| 코드 조사 | B01–B07 표본과 CA01 직접 전제. 전체 함수·모든 의존성 검토 아님 |
| 과거 근거 진단 | Master/교정/구현 member 32/32·16/16·95/95 해시 일치. P2G 두 r2 사본 동일·구판 archive identity 확인 |
| 문서·보존 검사 | JSON 파싱, 이번 작성 문서의 로컬 링크/절, 제한 범위 before/after 해시·Git 상태를 검사. 구체 수량과 결과는 부속 증거의 `final_checks`에 기록 |
| 게임 구현·실행 | production/RON/store/검증기/지침 변경 0. 빌드·Rust 테스트·probe·benchmark·P2G validator 실행 0 |
| 공식 검토·감사 | P2G-20/40, D2/DG, GEX-20/30 새 실행 없음. 독립 reviewer나 새 감사 verdict를 만들지 않음 |

NEXT-0001의 기존 진행 인수·주요 의존 경계 1차 확인·다음 범위 특정이라는 종료점에 도달했다. 현재 조사에 필요한 원문은 확보했고 사용자에게 넘겨야 할 내부 기술 선택은 없다. 새 NEXT-0002는 이번 요청으로 실행하지 않는다.
