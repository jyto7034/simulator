# Docs Index

**개발을 재개하려면 [로드맵 개발 현황](roadmaps/content_authoring/ROADMAP_STATUS.md)에서 시작한다.**
현재 작업·다음 작업·전체 진행표는 그 문서 하나에서 안내한다.
활성 개발 범위는 [Core 우선 통합 로드맵](roadmaps/core_first/ROADMAP.ko.md)이며,
[2026-10-04 연속 실행 요청](roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md)에 따라 P2G/GEX를 적용한다. 이 인덱스는 주제별 문서 탐색을 맡는다.

이 문서는 `core_enhanced/docs`의 탐색 입구다. 새 정책을 Goal로 변환하는 작업은
[Policy-To-Goal Pipeline](policy_to_goal_pipeline_revised_v2.md)의 Reader Contract에서 시작하고, P2G-60 Handoff 이후
Goal 구현·검증·감사는 [Goal Execution Pipeline](goal_execution_pipeline%281%29.md)의 Reader Contract에서 시작한다.

의미 있는 부분 구현 후 검토하는 작업의 지시서는
[단계검토 구현 지시서 템플릿](templates/IMPLEMENTATION_CHECKPOINT_REQUEST.template.ko.md)을 사용한다.
작성한 지시서를 구현 대화에 전달할 때는
[구현 시작 요청 템플릿](templates/IMPLEMENTATION_START_PROMPT.template.ko.md)을 사용한다.
GEX의 구현·기록·검증 원칙을 적용하는 작성 도구이며, 현재 개발 위치는 위 로드맵 개발 현황에서 확인한다.

현재 프로젝트 맵은 새 구조로 다시 작성하기 위해 제거된 상태다. 이 인덱스는 존재하지 않는 project-map 문서를 탐색 경로로 사용하지 않는다.

## 이 문서의 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | 탐색 인덱스 |
| 소유 범위 | 문서 등급, 주제별 canonical 위치와 archive/goal/decision 문서의 탐색 경로 |
| 소유하지 않는 범위 | P2G/GEX 실행 규칙, gameplay/runtime/data/Unity 계약, 개별 Goal 판단 |
| 함께 확인할 문서 | [policy_to_goal_pipeline_revised_v2.md](policy_to_goal_pipeline_revised_v2.md), [Goal Execution Pipeline](goal_execution_pipeline%281%29.md), 아래 변경 주제의 canonical |

## Source Of Truth 순서

질문의 종류에 맞는 가장 가까운 canonical을 아래 표에서 고른다. Policy Topic, approved target, Final Freeze,
Phase와 Goal authoring 경계는 Policy-To-Goal Pipeline이 소유하고, selected Goal의 current-tree 실행 evidence,
구현·검증과 완료 감사 경계는 Goal Execution Pipeline이 소유한다. Workspace/released/deployed actual은 해당
source identity와 current direct evidence로 확인한다. Goal과 audit 문서는 실행 history이며
gameplay/runtime/data 계약을 대신하지 않는다.

게임 정책은 최신 사용자 합의 → `game_rulebook.md`와 연결 상세 규칙 순으로 읽는다.
`archive/game_rulebook_legacy_2026-10-04.md`의 파일 수정 시각이나 과거 canonical 표기로 이 순서를 뒤집지 않는다.
[2026-10-04 룰북 혼입 교정](roadmaps/core_first/execution/rulebook_authority_correction_20261004_001/RECORD.ko.md)을 반영했다.

[2026-10-04 경로 변경 대응표](roadmaps/core_first/execution/rulebook_path_migration_20261004_001/RECORD.ko.md)는 과거 동결 기록의 원래 경로·hash 해석을 설명한다.

## 현재 문서 체계

| 등급 | 문서 | 소유하는 책임 | 우선 읽는 경우 |
| --- | --- | --- | --- |
| 탐색 인덱스 | [README.md](README.md) | 문서 등급, canonical 위치, 변경 주제별 문서 선택 | 문서 탐색을 시작할 때 |
| 정책→Goal 파이프라인 | [policy_to_goal_pipeline_revised_v2.md](policy_to_goal_pipeline_revised_v2.md) | 정책 요소 검토, code-informed review, Final Freeze, Phase Plan, Goal Set과 P2G-60 Handoff | 정책을 검토하고 구현 Goal로 변환할 때 |
| Goal 실행 파이프라인 | [Goal Execution Pipeline](goal_execution_pipeline%281%29.md) | Handoff 인수, Single/Phase Goal 구현·검증, Phase 완료 감사와 Master 최종 통합 감사 | P2G-60 이후 Goal을 실행하거나 감사할 때 |
| 게임 소개 | [game_design_overview.ko.md](game_design_overview.ko.md) | 외부 기획 검토를 위한 게임 정체성, 핵심 재미, 플레이 흐름과 차별점 소개 | 세부 구현보다 게임 전체의 방향을 먼저 이해할 때 |
| Gameplay canonical | [game_rulebook.md](game_rulebook.md)·연결된 `game_rules/` | 최신 게임 정책, 상세 조건·수치·예외와 미정. 이후 명시적 사용자 합의가 우선하며 구현 완료와는 구분 | gameplay 의미나 흐름을 검토·변경할 때 |
| 과거 룰북 | [과거 게임 룰북](archive/game_rulebook_legacy_2026-10-04.md) | 역사적 비교와 최신 정책이 명시적으로 계승한 공통 규칙. 새 정책·구현 결과의 동기화 대상 아님 | 과거 정책과 변경 배경을 확인할 때 |
| 전장 이미지 제작 가이드 | [전장 컨셉아트 이미지 생성 가이드](ui-layout/battlefield-image-generation-guide.ko.md) | 1타일 규격, 폐허 분위기, 경로 오버레이 제외와 이미지 저장 위치 | 전장 배경 컨셉아트를 생성할 때 |
| 전장 설계 참고 | [명일방주 전장 구조와 난이도 설계 참고](battlefields/arknights_difficulty_design_reference.ko.md) | 실제 스테이지 사례·출처, 난도 발생 원리의 해석과 일반 침식체 전장 적용 후보 | 전장 풀의 동선·배치 영역·지원 범위와 난도 차이를 설계할 때 |
| 시설 노드 구성 검토 | [공용 구간 방별 노드 구성 검토안](ui-layout/md/facility-upper-node-composition-v1.ko.md) | 지도 시안 1~3층의 방별 예약·배정 후보·일반전 수요 계산. 새 수치는 검토안이며 gameplay canonical을 대체하지 않음 | 전장 풀 수량을 산정하기 전에 공용 구간의 활동 구성을 검토할 때 |
| 시설 심층 구성 검토 | [심층과 최종방 노드 구성 검토안](ui-layout/md/facility-lower-node-composition-v1.ko.md) | 4~7층 51개 탐사 방·최종방 4개, 예약·전조·일반전 기대량과 격파 후 탐사 | 심층 활동 구성과 전체 전장 수요를 검토할 때 |
| Combat-domain canonical | [skill_target_contract.md](skill_target_contract.md) | 스킬/평타 타겟팅, 타일 범위, 자동 적대 타겟, cast/step/projectile/TileArea 의미 | 스킬·범위·자동 시전·면역 대상 판정을 바꿀 때 |
| Runtime canonical | [core_runtime_contract.md](core_runtime_contract.md) | 결정론/RNG, runtime state, event log/checkpoint, battle record/result, VN runtime, actor identity | core runtime 불변식이나 projection ownership을 바꿀 때 |
| Data canonical | [data_loading_contract.md](data_loading_contract.md) | live RON authoring, embedded loader ownership, fixture, validation, live file 색인 | RON schema/content, loader, validation 경계를 바꿀 때 |
| Evidence canonical | [audit/live_skill_catalog_evidence_contract.md](audit/live_skill_catalog_evidence_contract.md) | live skill manifest가 증명하는 membership/delivery/reference 범위와 update/review lifecycle | live skill/reference/fragment catalog나 coverage 근거를 바꿀 때 |
| 작업 가이드 | [engineering_workflow.md](engineering_workflow.md) | 일반 code/document sync, refactor 분류, test와 Git/file safety | 코드나 문서를 변경할 때 |
| 실행 색인 | [probe_registry.md](probe_registry.md) | 외부 Unity WebSocket/admin probe의 실제 경로, 실행 gate, 인접 계약 | live server/probe 검증이 필요할 때 |

정책 검토와 Goal 작성은 P2G Reader Contract, Goal 실행과 완료 감사는 GEX Reader Contract에서 현재 요청의
단계 하나만 선택해 읽는다.

## Policy-To-Goal과 Goal Execution Pipeline

정책 결정부터 Goal 작성까지는 [Policy-To-Goal Pipeline](policy_to_goal_pipeline_revised_v2.md), 실제 Goal 실행과 완료
판정은 [Goal Execution Pipeline](goal_execution_pipeline%281%29.md)이 각각 소유한다.

```text
임시 정책 입력 → code-informed Final Policy Freeze → Work Package/Atomic Group/Phase 설계
→ Single 또는 Master/Phase Goal Set → P2G-60 계획 인계·종료 / 별도 구현 요청 대기
→ GEX-00 Goal 선택 → GEX-10 전체 구현·자체 검증·target 고정·종료 / 별도 감사 요청 대기
→ GEX-20 Phase 완료 감사
→ 다음 Phase 반복 → GEX-30 Master 최종 통합 감사
```

두 파이프라인 모두 요청 mode·범위·종료점을 먼저 판정한다. P2G `bounded_pipeline`과 GEX `goal-run`은
승인된 내부 단계를 별도 pass로 이어갈 수 있지만 P2G-60 계획 인계와 기본 구현 후 감사 인계에서는 종료한다.
“단계검토 과정을 포함해줘”처럼 명시적으로 요청한 경우에만 빌드 가능한 중간 checkpoint에서도 전후 기준선·검증을
고정하고 멈춘다. 사전 중단 조건은 확인하되 실제 지점은 의미 있는 부분 진전·빌드·관련 검증 결과로 구현 중 선택한다.
사전 후보는 선택 사항이며 구현 전 baseline 고정은 생략하지 않는다. 검토 PASS·모델 변경은 다음 구현 승인이 아니다. 감사까지 명시 위임한 예외, 부분 검토와 최종 감사의
구분, 재개 권한은 [GEX Reader Contract](goal_execution_pipeline%281%29.md#기본-구현-인계와-선택적-단계검토)를 따른다.
이 규칙은 특정 Goal이나 R에 한정하지 않으며 전체 로드맵 자동 실행을 승인하지 않는다.

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
| [Run 유물 콘텐츠 설계 노트](skills/run_relic_content_design_notes.ko.md) | 유물의 재미, 강화·전문화·연결·전환 분류와 설계 순서. 미래 콘텐츠 방향이며 개별 효과·수치는 후속 설계 |
| `docs/skills/abnormality_skill_design_notes.ko.md` | 환상체별 조우 기믹, counterplay, 보상·파편 설계 노트 |
| `docs/skills/skill_fragment_system.md` | 스킬 파편의 상위 게임 정책과 특수 파편 체인 |
| `docs/skills/live_content_index.md` | 현재 live placeholder E.G.O 장비와 스킬 파편의 통합 파생 색인 |

콘텐츠 문서는 승인된 정책과 의미를 소유하고 탐색용 색인을 제공한다. Live RON/data와 runtime
validation은 현재 구현 여부, ID와 field shape를 확인하는 evidence다. 문서와 다르면 구현 drift로
기록하고 승인 문서에 맞게 수정한다.

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
| `docs/decisions/employee_domain_policy_2026-07-17.md` | 직원 도메인 정책 승인 배경과 구현 난이도 기록 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, `data_loading_contract.md` |
| `docs/decisions/event_employee_check_dice_policy_2026-07-17.md` | Event aptitude·DC·장비 보정·choice key 주사위 정책의 승인 배경과 구현 disposition | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, `data_loading_contract.md`, 외부 `unity_core_contract.md` |
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
| `COMPLETION_AUDIT.md` | 구현 pass와 분리된 Single/Phase 또는 Master 완료 감사와 verdict |

goal 디렉터리를 이 인덱스에 정적으로 열거하지 않는다. exact 실행 대상 선택, current dirty/collision 확인,
구현·검증, correction과 completion audit은 Goal Execution Pipeline에서 현재 요청에 해당하는 GEX 단계가
판정한다. 완료 Goal은 history이며, 유지할 정책 의미는 가장 가까운 durable canonical에 반영한다.

## 정리 원칙

- 최상위 durable 문서는 `문서 등급`, `소유 범위`, `소유하지 않는 범위`, `함께 확인할 문서`를 서문에서 명시한다.
- 하나의 정책은 하나의 canonical 문서가 소유하고 다른 문서는 짧은 seam과 링크만 둔다.
- 비소유 문서에 경계 설명을 보존할 수는 있지만, 세부 계약을 변경할 때는 소유 문서를 함께 갱신하고 소유 문서의 판정을 우선한다.
- gameplay 의미, runtime 생산자 불변식, data authoring/loading, Unity wire 계약을 한 문서에 섞지 않는다.
- 삭제된 문서, goal 기록, debug output, logs, tmp를 현재 source of truth로 사용하지 않는다.
- 새 정책과 충돌하는 레거시는 compatibility layer나 dual schema로 보존하지 않는다.
- 문서 변경도 실제 runtime code, live data, DTO, test/probe와 같은 내용을 말하는지 확인해야 완료된다.
