# Docs Index

이 문서는 `core_enhanced/docs`의 입구다. 새 작업자는 먼저 이 파일에서 source-of-truth 순서와 변경 주제별 canonical 문서를 확인한다.

현재 프로젝트 맵은 새 구조로 다시 작성하기 위해 제거된 상태다. 이 인덱스는 존재하지 않는 project-map 문서를 탐색 경로로 사용하지 않는다.

## 이 문서의 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | 탐색 인덱스이자 문서 governance 기준 |
| 소유 범위 | source-of-truth 순서, 문서 등급, 주제별 canonical 위치, archive/goal/decision 문서의 해석 기준 |
| 소유하지 않는 범위 | gameplay 규칙, runtime 불변식, RON schema, Unity wire payload, 개별 goal의 구현 판단 |
| 함께 확인할 문서 | 아래 문서 체계에서 변경 주제에 해당하는 canonical 또는 작업 가이드 |

## Source Of Truth 순서

문서보다 실제 동작이 우선이다.

1. 실제 runtime code
2. live RON/data
3. Unity-facing snapshot/command/WebSocket 계약
4. 최신 durable 정책/계약 문서
5. goal 문서와 과거 작업 기록

문서와 코드가 충돌하면 runtime code와 live data를 먼저 확인한다. 게임 규칙, UX, schema, 밸런스처럼 정책 판단이 필요한 변경은 임의로 확정하지 않고 사용자와 의논한다.

## 현재 문서 체계

| 등급 | 문서 | 소유하는 책임 | 우선 읽는 경우 |
| --- | --- | --- | --- |
| 탐색 인덱스 | [README.md](README.md) | 문서 등급, canonical 위치, 변경 주제별 문서 선택 | 문서 탐색을 시작할 때 |
| 게임 소개 | [game_design_overview.ko.md](game_design_overview.ko.md) | 외부 기획 검토를 위한 게임 정체성, 핵심 재미, 플레이 흐름과 차별점 소개 | 세부 구현보다 게임 전체의 방향을 먼저 이해할 때 |
| Gameplay canonical | [game_rulebook.md](game_rulebook.md) | 플레이어-visible 게임 규칙, 런/노드/전투/보상/성장/실패 정책 | gameplay 의미나 흐름을 바꿀 때 |
| Combat-domain canonical | [skill_target_contract.md](skill_target_contract.md) | 스킬/평타 타겟팅, 타일 범위, 자동 적대 타겟, cast/step/projectile/TileArea 의미 | 스킬·범위·자동 시전·면역 대상 판정을 바꿀 때 |
| Runtime canonical | [core_runtime_contract.md](core_runtime_contract.md) | 결정론/RNG, runtime state, event log/checkpoint, battle record/result, VN runtime, actor identity | core runtime 불변식이나 projection ownership을 바꿀 때 |
| Data canonical | [data_loading_contract.md](data_loading_contract.md) | live RON authoring, embedded loader ownership, fixture, validation, live file 색인 | RON schema/content, loader, validation 경계를 바꿀 때 |
| Evidence canonical | [audit/live_skill_catalog_evidence_contract.md](audit/live_skill_catalog_evidence_contract.md) | live skill manifest가 증명하는 membership/delivery/reference 범위와 update/review lifecycle | live skill/reference/fragment catalog나 coverage 근거를 바꿀 때 |
| 작업 가이드 | [engineering_workflow.md](engineering_workflow.md) | 코드/문서 동기화, 리팩토링 판단, 테스트, goal 운용, Git/file safety | 코드나 문서를 변경하는 모든 작업 |
| 감사 가이드 | [goal_completion_review_guide.md](goal_completion_review_guide.md) | 완료 goal을 runtime/data/DTO/test 근거로 재검토하는 감사 절차 | 완료 체크박스나 master goal을 감사할 때 |
| 실행 색인 | [probe_registry.md](probe_registry.md) | 외부 Unity WebSocket/admin probe의 실제 경로, 실행 gate, 인접 계약 | live server/probe 검증이 필요할 때 |

작업 가이드, 감사 가이드, 실행 색인은 gameplay/runtime/data 정책의 source of truth가 아니다.
특히 `goal_completion_review_guide.md`는 필요할 때만 사용하는 전문 감사 절차다.

## Archived Audit 문서

`docs/archive/audits/*_YYYY-MM-DD.md` 형식의 종료된 시점 감사 보고서는 당시
구현·데이터·문서 상태와 후속 작업의 입력 근거만 보존한다. 일상적인 문서 탐색이나 현재 작업의
필수 읽기 대상이 아니며, gameplay, runtime, data, Unity wire 정책의 source of truth도 아니다.
보고서의 finding은 최신 durable canonical과 현재 runtime/live data로 다시 확인해야 한다.

| 문서 | 역사적 역할 | 현재 상태 확인 위치 |
| --- | --- | --- |
| `docs/archive/audits/documentation_code_alignment_audit_2026-07-10.md` | 52개 문서–코드 정합성 finding의 원본 감사 근거 | `docs/goals/cross_layer_contract_hardening_master/PLAN.md`, `STATUS.md` |
| `docs/archive/audits/comprehensive_project_review_2026-07-10.md` | 15개 프로젝트 종합 finding의 감사 스냅샷 | 같은 master의 CPR alias/unique finding registry |

감사 보고서의 원문 판정을 현재 정책처럼 고쳐 쓰지 않는다. 후속 disposition과 canonical
ownership을 보고서 상단에 덧붙이고, 실제 정책 변경은 가장 가까운 durable 문서에만 반영한다.

## 외부 Unity Canonical 문서

Unity-facing 계약과 클라이언트 구현 문서는 이 저장소가 아니라 아래 외부 Unity 프로젝트가 canonical이다.

```text
F:\unity projects\ark\docs
/mnt/f/unity projects/ark/docs
```

| 외부 문서 | 소유하는 책임 |
| --- | --- |
| `unity_core_contract.md` | `/game` WebSocket envelope, command/result, root snapshot, non-battle DTO |
| `core_unity_battle_transport_contract.md` | battle setup/update/event/checkpoint/resync의 JSON과 소비 순서 |
| `unity_client_implementation_goal.md` | Unity 클라이언트 구현 작업 지침과 검증 순서; durable 정책은 아님 |

이 저장소 안에는 같은 이름의 Unity 계약 copy를 두지 않는다. Battle transport의 exact payload와 message order는 통합 transport 문서가 canonical이며, 과거 setup/update 분리 문서는 migration context로만 읽는다.

## 콘텐츠 문서

| 문서 | 역할 |
| --- | --- |
| `docs/skills/lobotomy_content_catalog.md` | 확정 환상체/도구 로스터와 콘텐츠 분류·설계 정책 |
| `docs/skills/abnormality_skill_design_notes.ko.md` | 환상체별 조우 기믹, counterplay, 보상·파편 설계 노트 |
| `docs/skills/skill_fragment_system.md` | 스킬 파편의 상위 게임 정책과 특수 파편 체인 |
| `docs/skills/live_content_index.md` | 현재 live placeholder E.G.O 장비와 스킬 파편의 통합 파생 색인 |

콘텐츠 문서는 정책과 탐색용 색인을 제공한다. 실제 구현 여부, ID, schema/field shape와 현재 효과는 live RON/data 및 runtime validation을 먼저 확인한다.

### 후반 콘텐츠 개발 시 함께 수행할 작업: 모노리스 숨겨진 보스

숨겨진 보스 콘텐츠는 현재 구현 대상이 아니며, 현재
`run_persistence_facility_monolith_policy_master`의 완료 조건에서도 제외됐다. 실제 보스 콘텐츠의
설계와 제작을 시작할 때 별도 후반 콘텐츠 Goal을 만들고 다음 작업을 하나의 수직 범위로 수행한다.

- 고유 보스 actor, skill, encounter, battlefield/wave와 reward를 확정하고 live content로 저작한다.
- 일반 FinalBoss pool 재분류, 변질된 모노리스 소비와 기존 Endless 완료 interception을 구현한다.
- 관리자 Visual Novel, 전용 single-terminal capstone과 typed special completion을 연결한다.
- 숨겨진 보스 discovered/defeated를 로컬 도감에 기록하고 checkpoint/active-run 상태를 확장한다.
- Core runtime·live RON validation·game_server·Unity reader/presentation/resource·canonical 문서와
  focused/broad 검증을 같은 콘텐츠 작업에서 함께 완료한다.
- 이 후반 Goal은 현재 master의 P07 감사나 완료를 막지 않는다.

이 항목은 후반 작업을 잊지 않기 위한 일정·범위 표식이며, 보스 기믹·밸런스·서사·asset의 승인이나
현재 구현 완료를 뜻하지 않는다. 콘텐츠 정책 확정과 사용자의 명시적 재개 전에는 후반 Goal이나
placeholder 보스, 기존 Der Freischutz 재사용, FinalBoss 분류 및 Endless 완료 흐름 변경을 진행하지
않는다. 범위의 역사적 결정과 현재 분리 disposition은
[현재 master 기록](goals/run_persistence_facility_monolith_policy_master/PLAN.md)과
[모노리스·숨겨진 보스 결정 기록](decisions/elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md)을 따른다.

## 정책·설계 결정 기록

`docs/decisions/`는 승인 배경, 검토한 대안과 당시 영향도를 보존하는 정식 decision/design
record 위치다. Goal의 실행 일지와 분리하지만 현재 gameplay/runtime/data/wire 정책의 canonical
대체물은 아니다. 현재 동작은 record가 링크한 durable canonical과 runtime/live data를 따른다.

| 문서 | 역할 | 현재 정책 위치 |
| --- | --- | --- |
| `docs/decisions/employee_domain_policy_2026-07-17.md` | 직원 도메인 정책 승인 배경과 구현 난이도 기록 | `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md` |
| `docs/decisions/event_employee_check_dice_policy_2026-07-17.md` | Event aptitude·DC·장비 보정·choice key 주사위 정책의 승인 배경과 구현 disposition | `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`, 외부 `unity_core_contract.md` |
| `docs/decisions/pre_run_global_visual_novel_core_2026-07-09.md` | top-level VN과 pre-run briefing 설계 결정 기록 | `core_runtime_contract.md`, `data_loading_contract.md`, 외부 `unity_core_contract.md` |
| `docs/decisions/save_point_trauma_recovery_policy_draft_2026-07-25.md` | SavePoint·Trauma 회복, active-run 이어하기와 실패 사전 선택의 구현 이력 | implemented decision record |
| `docs/decisions/local_encounter_compendium_persistence_policy_draft_2026-07-25.md` | P02에 구현된 run 밖 로컬 영구 도감 profile 정책의 역사적 입력 | implemented decision record |
| `docs/decisions/facility_map_and_node_manifestation_policy_2026-07-25_26.md` | 시설 template·node 배정·navigation과 선택형 보너스 node 발현 구현 이력 | implemented decision record |
| `docs/decisions/combat_power_floor_difficulty_policy_draft_2026-07-26.md` | certified power, Floor anchor와 node/role 난이도 분포 구현 이력 | implemented decision record |
| `docs/decisions/elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md` | P05 모노리스 구현 이력과 별도 후반 숨겨진 보스 정책 입력 | P05 implemented / hidden-boss future content record |

결정 기록의 과거 추천이나 open question을 현재 미완료 작업으로 재해석하지 않는다. 후속 구현이
완료됐다면 record 상단의 disposition과 연결된 completion evidence를 먼저 확인한다.

## Goal 문서

`docs/goals/<goal_name>/`은 진행 중 작업의 기억장치이며 durable 정책 문서가 아니다.

일반적인 구성:

| 파일 | 역할 |
| --- | --- |
| `PLAN.md` | 목표, 범위, 완료/중단 조건, 검증 명령 |
| `EXPERIMENTS.md` | 시도와 실패/성공 결과, 수정과 재검증 기록 |
| `EXPERIMENT_NOTES.md` | 판단, source-of-truth 충돌, 정책 질문, 후속 후보 |

goal 디렉터리를 이 인덱스에 정적으로 열거하지 않는다. master campaign이 있으면 그
`PLAN.md` child/status table과 goal `Status` header가 현재 active work의 권위다. 완료 goal은
실행/검토 evidence를 보존하는 history이며, 유지해야 할 정책만 가장 가까운 durable 계약으로
옮긴다. 완료 goal 문서 자체는 live gameplay/wire 정책의 유일한 source가 될 수 없다.

## 정리 원칙

- 최상위 durable 문서는 `문서 등급`, `소유 범위`, `소유하지 않는 범위`, `함께 확인할 문서`를 서문에서 명시한다.
- 하나의 정책은 하나의 canonical 문서가 소유하고 다른 문서는 짧은 seam과 링크만 둔다.
- 비소유 문서에 경계 설명을 보존할 수는 있지만, 세부 계약을 변경할 때는 소유 문서를 함께 갱신하고 소유 문서의 판정을 우선한다.
- gameplay 의미, runtime 생산자 불변식, data authoring/loading, Unity wire 계약을 한 문서에 섞지 않는다.
- 삭제된 문서, goal 기록, debug output, logs, tmp를 현재 source of truth로 사용하지 않는다.
- 새 정책과 충돌하는 레거시는 compatibility layer나 dual schema로 보존하지 않는다.
- 문서 변경도 실제 runtime code, live data, DTO, test/probe와 같은 내용을 말하는지 확인해야 완료된다.
