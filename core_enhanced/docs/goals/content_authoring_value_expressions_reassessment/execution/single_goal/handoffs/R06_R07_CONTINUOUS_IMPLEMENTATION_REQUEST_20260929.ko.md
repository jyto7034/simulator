# R06 → R07 연속 구현 지시서

2026-09-29 · v1 · 사용자 범위 확대·정책 결정 위임 반영. 현재는 지시서 작성이며 제품 구현·계획 검증 완료를 뜻하지 않는다.

## 1. 이번 실행 범위와 종료점

**R06 잔여 전체를 구현·자체 검증한 뒤 R07의 계획 수립과 전체 구현·자체 검증까지 연속 수행한다.** 사용자는 자리를 비우므로 미정 정책은 구현자가 결정하고 근거·대안·영향·검증을 기록한다. 질문 답변이나 R06 감사 대기로 멈추지 않는다. R08 이상은 시작하지 않는다.

[GEX](../../../../../goal_execution_pipeline(1).md)를 지침으로 사용한다. R06과 R07은 서로 다른 구현 책임이며 기존 R06 Goal을 임의로 R07까지 확장하지 않는다. 이 요청은 **두 구현 작업을 순서대로 수행할 권한**이며 독립 감사 권한은 아니다.

1. R06의 GEX-00 인수 → GEX-10 잔여 구현·자체 검증 → R06 경계 target 보존.
2. R07 계획 인수/작성·검증 → GEX-00 → GEX-10 구현·자체 검증.
3. R07 변경이 반영된 최종 source에서 R06/R07 통합 회귀·비용 검증 → 최종 target 동결.
4. **R06·R07 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기**에서 종료.

R06 경계의 target 보존은 비교·복원용 내부 인계다. **이번 사용자 범위 확대에 따라 그 지점에서 최종 응답으로 종료하거나 R06 독립 감사 완료를 R07 착수 조건으로 요구하지 않는다.** 이는 R07의 기능 선행 R01/R05, 진행상 권장 R06과 맞는다. R06 source 변경이 생기면 보존한 target을 수정하지 않고 새 최종 source에서 영향 조건을 재검증한다.

GEX의 증거·기록·검증·독립성 요구는 유지한다. 나중에 별도 요청하는 감사는 두 범위와 경계 간 회귀를 포함하며, R06 미검토 C07도 빠뜨리지 않는다. 두 Single Goal을 순서대로 요청한 것이므로 여기서 가짜 Master Goal/GEX-30 완료를 만들지 않는다. 구현자가 독립 PASS나 최종 `Complete / High`를 선언하지 않는다.

## 2. 기준 문서와 우선순위

- R06 상세 구현·시험: [R06 구현 지시서](R06_COMPLETION_IMPLEMENTATION_REQUEST_20260929.ko.md). 소스 identity, C08–C15/OBS, WP03/D07-C/WP04 및 수용표를 그대로 따른다.
- R07 범위: [R07 카드](../../../../../roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R07.ko.md), [AT07·의존성](../../../../../roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/DEPENDENCIES.ko.md), [PG05/PG06](../../../../../roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/POLICY_GATES.ko.md).
- 최신 파편 목표: [장비·파편](../../../../../game_rules_next/equipment-and-fragments.md), [미정 사항](../../../../../game_rules_next/open-decisions.md), [현행 비교 기준](../../../../../game_rulebook.md). 차기 설계 전체가 구현되었다고 가정하지 않는다.
- 조사 근거: [R07 코드·문서 근거](R06_R07_CODE_EVIDENCE_20260929.json). 아래 코드 경로는 `core_enhanced/` 기준이다.

이번 문서가 기존 R06 지시서의 `R07 착수 금지`, `R06 완료 후 즉시 반환`, `정책 위임은 R06에만 적용` 문구보다 우선한다. 나머지 R06 수용 조건을 삭제하지 않는다. R06 source에서는 유지한 파편 rank 경로를 R07에서 승인된 일반 강화 제거로 교체할 수 있지만, 그 차이는 R07 변경으로 기록하고 R06 수치 합성 회귀는 새 파편 입력에 맞게 다시 증명한다.

오래된 카드의 intrinsic/passive 조합과 최신 목표의 **기본 active 파편 슬롯1개·파편 일반 강화 없음**을 구별한다. 이번 R07은 최신 목표에 따라 active 사본1개를 기본으로 한다. PG06의 직원 공용3슬롯을 파편 active3슬롯으로 해석하지 않는다(최신 문서의 직원3슬롯은 유물 정책). passive/channel/override 확장은 R11에 남긴다. 이 비교와 채택을 R07 정책 기록에 명시한다.

## 3. R07 정식 계획 준비와 실행 기록

현재 R07 카드는 `proposed_not_implemented`인 조사·권고 문서이며 정식 Goal/Handoff가 아니다. 존재하지 않는 P2G PASS나 Handoff ID를 인수했다고 기록하지 않는다.

이번 구현 시작 요청은 R07에 필요한 계획 준비와 이어지는 구현까지 포함한다. R06의 기록을 인수한 뒤 R07의 정책/코드 조사와 상세 계획을 준비할 수 있으며, R07 제품 mutation은 R06 경계 인계 뒤 수행한다. [P2G](../../../../../policy_to_goal_pipeline_revised_v2.md)에 따라 R07의 기존 산출물 존재 여부부터 확인하고, 없다면 필요한 정책·코드 대조·freeze·Phase Plan/검증·Goal Set·Handoff를 작성한다. 위임된 미정 정책은 작성자가 근거를 남겨 결정하고 계획 종료만을 이유로 사용자 재승인을 기다리지 않는다. 실제 검증 도구/결과와 미수행 항목을 구별한다. 상세 계획은 아래 설계를 현재 R06 완료 source와 재대조해야 한다.

새 계획 저장 위치는 `docs/goals/content_authoring_fragment_copies/`를 권장한다. 같은 R07을 가리키는 기존 정식 실행 기록이 있으면 그 위치를 재사용한다. R07은 가능한 한 하나의 Single Goal로 AT07을 묶고, exact Goal/Freeze/Plan/Handoff ID는 실제 산출물에서 확정한다. 이 권장 경로를 존재하는 실행 기록으로 표시하지 않는다.

| 기록 책임 | 위치/역할 |
| --- | --- |
| 전체 사용자 진입점 | 기존 `docs/roadmaps/content_authoring/ROADMAP_STATUS.md`만 사용 |
| R06 세 기록 | 기존 `execution/single_goal/continuations/r06_completion_001/`의 PLAN/EXPERIMENTS/EXPERIMENT_NOTES |
| R07 세 기록 | R07 Handoff가 선택한 하나의 execution directory. 신규라면 `docs/goals/content_authoring_fragment_copies/execution/single_goal/` |
| 최종 통합 인계 | R07 실행 디렉터리의 REVIEW_HANDOFF에서 R06 경계 target와 두 수용표·최종 통합 target을 연결 |

R06의 동결 P2G/감사/원자료를 바꾸지 않는다. R07 시작 시 실제 source baseline, R06 경계 target, 변경 예정 shared schema와 원본 저장 fixture를 고정한다. 이후 R06 회귀 교정도 해당 owner와 원인을 기록하며 같은 단일 writer가 처리한다.

## 4. R07 실제 코드에서 확인한 문제와 구현 방향

### A. 단일 사본 저장소

`src/game/skill_fragment.rs::SkillFragmentInventory`는 현재 definition별 `stacks`와 `progress`를 따로 보관한다. progress의 upgrade_level/awakening_progress/awakened가 같은 종류 모든 사본에 공유된다. `SkillFragmentLoadout.active_fragment_id`도 definition ID다.

- `OwnedFragment`에 독립 copy UUID, definition ID, unlocked form 집합, selected form, 장착 owner 결속을 둔다. inventory는 copy UUID로 keyed된 **하나의 authoritative aggregate**를 사용한다. definition별 count/목록은 파생 조회이며 별도 mutable truth로 유지하지 않는다.
- employee loadout은 실제 copy를 참조하고 inventory reverse binding과 함께 검증한다. copy→definition→selected form→compiled skill binding을 하나의 resolver로 연결한다. 장착에 definition만 넣어 첫 번째 사본을 고르는 fallback은 제거한다.
- 일반 파편 강화·공통 progress를 새 gameplay reader에서 제거한다. `upgrade_level`, `upgrade_skill_ids`, dust 기반 강화→개화 진척 경로의 schema/명령/검증/DTO/consumer를 함께 전환한다. old decode 전용 표현이 필요하면 읽기 전용 compatibility 모듈로 격리한다.

### B. 정상 획득·UUID·장착·이전·사망 회수

`src/game/reward.rs::GrantExecutor`의 GrantSkillFragment는 현재 count만 증가시키고 `SkillFragmentGrantDiffDto`도 count 전후만 전달한다. `src/game/world/grant_state.rs::EconomyGrantState`는 inventory/fragments/roster/UUID/enkephalin을 함께 clone/commit하므로 이 transaction을 재사용한다.

- `src/game/managers/uuid_manager.rs`에 파편 사본 전용 deterministic namespace를 연결한다. 기존 장비/직원/전투 UUID stream을 변경하지 않는다. 실패한 grant는 count뿐 아니라 UUID stream과 영수증도 보존한다. 반환 DTO에는 실제 지급 copy identity를 포함한다.
- 정상 보상/관리 grant/시작 입력 등 실제 producer 모두 사본 생성으로 모은다. 기존 grant/reward 재실행 제어를 유지하고 중복 결과 적용 시 사본을 추가 발급하지 않는다. 랜덤 UUID나 definition hash를 copy ID로 쓰지 않는다.
- `src/game/resources/equipment_loadout.rs::EquipmentLoadoutTransition`의 plan/projected validation/commit 패턴을 파편에 적용한다. `world/maintenance.rs`, `world/helpers.rs`, `employee.rs`의 definition·count 기반 장착 검사를 copy 독점과 양방향 결속 검사로 바꾼다. 직원 이전은 기존 owner 해제+새 owner 장착을 하나의 성공/실패 경계로 처리한다.
- `world/combat.rs::recover_assets_for_new_deaths`는 현재 employee active ID만 해제한다. 새 signature/candidate state에 fragment inventory도 연결해 실제 사본 reverse binding까지 함께 정리하고, 같은 사본의 form/unlock을 보존한다. 임의의 다른 사본을 회수하지 않는다.
- 관리 허용 상태, 직원 출전/장착 자격과 무기 profile compatibility를 Core가 재검증한다. Safezone 장착 허용과 Maintenance 개화/분쇄 허용을 구별한다.

### C. 개화 해금·형태 선택·recipe·분쇄

`SkillFragmentEffectDef::ActiveSkill`의 현재 imitation/awakened SkillId를 출발점으로 definition에 stable form ID→지원 skill/compatibility를 표현한다. base form은 항상 남고 해금과 선택을 서로 다른 저장 값으로 둔다. form별 compiled 경로를 실제 validation/catalog에 연결한다.

- 개화는 지정 copy만 바꾸며 A의 해금이 같은 definition의 B를 바꾸지 않는다. 이미 해금한 form의 재개화는 재료를 중복 차감하지 않는 명시적 결과로 처리한다. 잠긴 form 선택은 typed 거부한다.
- 대표 정책: 개화는 해금만 하고 선택은 유지, 형태 선택은 Maintenance에서 별도 수행, 같은 형태 재선택은 무변경. 구현자가 현재 계약상 다른 선택이 필요하면 위임 결정으로 이유·시험을 기록한다.
- 현재 dust 공식(강화4/개화threshold7 등)을 새 공식 recipe로 복사하지 않는다. enkephalin+명시 환상체 파츠 등 data-driven recipe를 사용하고 반복 재료 항목은 checked 합산 후 한 번 검증/소비한다. form/사본/장착 compatibility·재료·상태를 후보 전체에서 검증한 뒤 commit한다.
- recipe가 없는 항목은 명시적 unavailable이다. 실제 공개 RON fixture에는 완전한 recipe를 제공해 정상 획득→개화→선택→전투를 입증한다. 본편에 새 숫자가 필요하면 사용자 위임에 따른 **잠정 구현값**으로 RON과 결정 기록에 명시한다. 기존 확정 밸런스나 사용자가 직접 고른 값처럼 기록하지 않는다. 시험 전용 수치와 본편 데이터는 구별한다.
- 분쇄는 exact copy를 지정한다. 장착 중에는 확인되지 않은 요청을 거부/확인 필요로 반환하고, 확인된 요청에서 자동 해제+사본 제거+recipe 산출을 원자적으로 처리한다. 이 확인은 게임 명령의 confirmation 필드/프로토콜이며 개발 중 사용자 응답을 기다리는 절차가 아니다. 현재 count 감소 후 직원 UUID 순으로 임의 해제하는 코드는 제거한다.

### D. 실제 전투·출처·효과 계획·표시

`SkillFragmentLoadout::apply_to_profile`는 현재 definition의 공통 progress로 active skill을 선택한다. `stat_pipeline`의 battle/Certified producer와 `combat_setup/player_spawns.rs`의 compatibility/profile 경로가 모두 이 소비자다.

- 선택된 사본의 form을 resolver로 읽어 active skill을 적용한다. 기본/개화 form을 실제 전투로 실행하고 R06의 amount/trace/effective input과 일치시킨다. UI 표시만 달라지는 구현은 불완전하다.
- source provenance에 필요한 fragment copy/form identity를 명시적으로 전달한다. 파편 사본을 장비 사본으로 가장하거나 직원 UUID로 대체하지 않는다. R05의 이미 구현된 owner/lifetime 경로를 확인하고 해당 binding의 실제 값을 사용한다. 없는 owner type이 필요하면 이 범위의 최소 추가와 검증을 기록한다.
- form 선택/직원 이전은 새 battle entry/profile/관련 fingerprint를 갱신한다. 이미 발사된 delivery의 source/Snapshot identity를 새 form으로 덮어쓰지 않는다. Live도 기존 action 권한과 실제 입력 계약을 유지한다. 비전투 전환으로 옛 lease를 되살리지 않는다.
- 기존 intrinsic baseline modifier는 플레이어 소유 사본으로 위장하지 않는다. 과거 비교 fixture에 필요한 의미와 새 active 파편 모델을 구별한다. 새 passive/channel/basic override 시스템을 만들지 말고 임시 콘텐츠의 지원 여부를 진실하게 표시한다.
- `world/snapshot.rs`와 `behavior.rs`의 목록/진척/장착/호환 DTO를 copy ID, definition, unlocked/selected form, owner, 불가 이유로 전환한다. Core 소비자가 exact copy를 지정할 수 있어야 하며 GUI 작성은 이번 범위가 아니다.

### E. 저장·복원·성능 계량

`world/state.rs`, `world/snapshot.rs`, `world/active_run.rs`, checkpoint validation과 CloneShape를 동시 전환한다. loadout의 copy가 없거나 중복 장착/역결속 불일치/없는 definition·form/미해금 selected form이면 restore 전에 거부한다. 정상 새 process restore는 동일 UUID/form/owner·effective stat·Certified input을 유지한다.

R06의 raw old V7 호환 증거는 보존하고 R07 저장 형식 변화와 별도로 추적한다. 개인 저장 파일을 삭제하거나 자동으로 덮어쓰지 않는다. schema version 변경이 필요하면 구형 DTO에서 **기존 checksum/identity를 먼저 검증**하고 명시적인 compatibility/import 경계에서 변환한다. 구형 count→copy 배정, 공통 개화 상태의 승계, 제거된 강화 진척의 보존/처리, 장착 직원과 copy 대응은 위임 정책으로 확정하고 fixtures에 기대값을 고정한다. 모호한 데이터를 임의 누락하거나 default로 성공시키지 않는다. 비호환이면 원본을 보존한 명시적 거부와 별도 변환 경로를 제공하고, 단순 old save 전부 거부로 저장 통합 완료를 주장하지 않는다.

형태 변경·직원 이전·재배치·attempt 종료·원정 귀환은 다른 사건이다. copy 영속 해금은 단순 장착 변경이나 사망 회수로 초기화하지 않는다. R05 lifetime에 맞는 자원만 보존/종료하며 임의의 전역 초기화/복제를 하지 않는다. 미래 회수품 대여 전체 시스템, 침식 R09, 유물 R14, 새 보상 identity R19를 R07에 끌어오지 않는다.

## 5. R07 필수 수용표

R07 계획은 T11/T18/T20/T27을 아래 actual seam과 연결하고 입력/기대값/실제 selector/원자료를 갖는다. 시험 이름은 구현자가 실제로 만든 이름을 사용한다.

| ID | 필수 검증 |
| --- | --- |
| F01 | 정상 grant로 같은 definition A/B를 획득. UUID 다름, 파생 count2, 실패 grant/재시도는 UUID·재료·영수증 원자성 유지 |
| F02 | A만 개화/형태 선택, B 기본 유지. A를 다른 직원에게 이전해도 동일 UUID/unlock/form. 같은 copy 이중 장착·존재하지 않는 copy 거부 |
| F03 | weapon compatibility, occupied slot, 실패한 이전의 양쪽 roster/inventory 완전 보존. 동일 정의 A/B는 서로 독립 |
| F04 | 개화/선택의 recipe 부족·중복 재료·합산 overflow·잠긴 form·중복 해금·동일 선택·잘못된 관리 상태 경계 |
| F05 | 장착 copy 분쇄 미확인/취소 무변경, 확인 후 그 copy만 해제·제거·산출. 뒤 grant 실패도 전체 rollback. 다른 직원/B 불변 |
| F06 | 사망 회수 후 exact copy owner 해제·unlock 유지; 반복 결과 적용에서도 복제/이중 회수 없음 |
| F07 | 실제 default 공개 RON→grant→equip→Maintenance→base/awakened form 선택→battle의 skill/amount/source/trace 일치. private fixture만으로 대체하지 않음 |
| F08 | Snapshot/Live source-copy, R05 lease/lifetime, form/owner 변경 후 새 입력 fingerprint와 예전 delivery identity 보존 |
| F09 | 새 process save/restore identity·effective stat 일치, checksum 변조·dangling/duplicate binding·잘못된 form 거부/기존 store 무변경 |
| F10 | 원본 old save bytes/정확 checksum 검증·호환 또는 명시 변환의 실제 oracle, 원본 불변. 제거된 강화 경로와 새 ledger가 병행 gameplay truth가 아님 |
| F11 | DTO/preview/explain/실제 command가 동일 exact copy/form과 비용·불가 이유를 표시. query 무변경 |
| F12 | copy/form/cardinality·allocation/retained/clone 한도·취소/실패 후 정상 retry. R06 literal/expr/stat/save/관측/비용 영향 회귀와 전체 기본/feature 시험 |

R06의10 validation dimensions를 R07에도 실제 사례로 매핑한다. unknown field/form/reference, 부정확한 copy, 빈 inventory/허용 개수 경계, 순서 결정성, 실패·재시도, 실제 공개 입력, 저장 재시작을 포함한다. 새 source에는 count/progress/rank를 읽는 옛 authoritative gameplay 경로가 남지 않았음을 호출 그래프와 검색으로 확인한다. 검색 결과0만으로 실제 수용 시험을 대체하지 않는다.

## 6. 두 단계의 검증과 최종 인계

R06의 전체 명령·focused/기본 병렬 정책·release CLI·PC36/R05·원자료 보존·한도 정책을 따른다. R06 경계에서 해당 full baseline을 닫고, R07 최종 source에서 공유 schema/runtime/save의 전체 통합 회귀를 다시 수행한다. 항목마다 full suite를 반복하지 않는다. R07 구조 변경은 R06 비용 근거 재사용을 자동 허용하지 않는다.

PC36의 기존 성공/거부/gameplay oracle을 유지한다. 파편 rank 제거 등 승인된 R07 의미 변경으로 fixture가 영향을 받으면 before→after와 정책 근거를 명시하여 측정 입력을 통제한다. 기존 raw 실패를 삭제하거나 비교 불가능한 workload를 같은 결과로 표시하지 않는다. R05 concurrent peak9,030,188bytes 단독 초과는 계속 경고이고 다른 hard guard는 유지한다. 20분 시험·자동 스킬 취소·새 안전 설정 RON/metrics/실험실·편집기는 포함하지 않는다.

최종 인계에는 R06 수용표, R07 F01–F12/T11·18·20·27, 두 OBS 교정, 신규 정책 결정 목록, R06→R07 변경 분류, 독립 감사 미수행 표시를 포함한다. R06 경계 archive와 최종 통합 archive의 identity·forward/reverse 복원을 확인한다. 추가 source/test/data/config/lock/fixture를 membership에 포함하고 hard-coded 파일/시험 수를 복사하지 않는다.

ROADMAP_STATUS는 실행 중 R06→R07로 현재 위치를 갱신하고 마지막에는 **R06·R07 구현·자체 검증 완료 / 최종 통합 감사 대기**로 표시한다. 구현하지 않은 계획/시험을 PASS로 쓰지 않는다. 최종 target 동결 후 mutation을 멈추고 감사 요청을 기다린다.
