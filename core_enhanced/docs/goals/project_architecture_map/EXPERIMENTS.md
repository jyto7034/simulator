# 프로젝트 지도 제작 — 작업 기록

현재 단계: `investigation_only` 지도 조사·작성 및 별도 문서 검토 완료.
산출물은 [project_map.ko.md](../../project_map.ko.md), 최종 기준 대조는 [COMPLETION_AUDIT](COMPLETION_AUDIT.md)다.
아래 문서 준비·범위 보완 기록은 당시 이력이며, 실제 실행은 마지막 항목에 구분한다.

## 2026-09-06 — Goal 문서 준비

### 확인 목적

사용자가 지정한 범용 실행 지침에서 필요한 부분만 가져와, 테스트 실행 없이 진행할 수 있는
설계 검토용 지도 Goal을 작성한다. 실제 지도 실행과 문서 준비를 구분한다.

### 실제 확인

- 지정된 `single_goal_execution_prompt.ko.md`는 직전 검토에서 읽었다. 이번 문서에 필요한 원칙만 반영했다.
- `docs/README.md`, `docs/engineering_workflow.md`, P2G·GEX Reader Contract와 문서 구조 규칙을 확인했다.
- 조상 경로와 docs 하위에서 적용할 `AGENTS.md`를 찾았으며 발견하지 못했다.
- 유물·침식 정책 기록을 확인하고 현재 명세·미결·기존 참고안과 구분할 기준을 작성했다.
- 신규 Goal 경로·예정 지도 경로·입력 문서·서버·데이터 디렉터리의 존재 여부를 확인했다.
- `project_map_documentation`의 PLAN·기록을 읽어 2026-07-07 지도 제작 이력임을 확인했다.
  당시 산출물 `docs/PROJECT_MAP.md`, `docs/maps/`는 현재 없다. 기존 이력은 수정하지 않았다.
- 코어·서버 runtime 추적과 실제 지도 본문 작성은 시작하지 않았다.

### 명령과 도구

핵심 확인 명령:

```text
pwd
git status --short
git rev-parse --show-toplevel HEAD
rg --files -g 'AGENTS.md' -g '*workflow*' -g '*WORKFLOW*' -g '*goal*' -g '*GOAL*' -g 'README.md' -g '*POLICY*' docs .agents .codex
rg --files --hidden -g 'AGENTS.md' -g '!**/.git/**' docs
cat docs/README.md docs/engineering_workflow.md
sed -n '1,240p' docs/goal_execution_pipeline.md
sed -n '1,200p' docs/policy_to_goal_pipeline.md
sed -n '1,115p' docs/goal_execution_pipeline.md
sed -n '95,260p' docs/engineering_workflow.md
cat docs/tmp/run_relic_system_design_notes_2026-08-21.ko.md docs/tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md
sed -n '1,215p' docs/tmp/run_relic_system_design_notes_2026-08-21.ko.md
cat docs/goals/project_map_documentation/PLAN.md
sed -n '1,110p' docs/goals/project_map_documentation/EXPERIMENT_NOTES.md
sed -n '1,150p' docs/goals/project_map_documentation/EXPERIMENTS.md
git status --porcelain=v1 -z --untracked-files=all
```

보조 확인은 Python의 `Path.exists`와 디렉터리 이름 대조로 수행했다.
출력이 길어 일부가 생략된 문서는 적용 부분을 좁혀 다시 읽었다. 읽지 않은 범위를 전체 검토했다고
기록하지 않는다. `rg`의 AGENTS 미발견 exit 1은 조사 결과이며 테스트 실패가 아니다.

Python을 통해 마지막 git status 결과와 revision을
`/tmp/project_architecture_map_goal_before.json`에 보관했다. 이는 문서 준비 전 상태 비교용 임시 기록이며
지도 조사 근거나 테스트 baseline이 아니다. 저장된 상태 행은 16,414개로, 수정 167·삭제 24·미추적
16,223개였다. 해당 수량은 `--untracked-files=all` 기준이며 코드 변경량이나 테스트 수가 아니다.

### 작성 결과

`apply_patch`로 이 Goal의 PLAN, EXPERIMENTS, EXPERIMENT_NOTES, COMPLETION_AUDIT를 신규 작성했다.
실제 지도와 README 링크는 생성하지 않았다. 코드·데이터·기존 정책·과거 Goal은 변경하지 않았다.

### 검증 상태

- 사용자 보고: 본인이 `all test`를 실행했을 때 모두 통과함.
- 이번 작업의 테스트·빌드·probe 실행: 없음. 사용자 지시로 제외.
- 테스트 실행 수·pass/fail/skip: 해당 없음. 사용자 보고를 이번 실행 결과로 집계하지 않음.
- 새 Goal 문서의 경로·링크·형식 확인: 4개 문서와 로컬 링크 20개 확인, 오류 없음.
- 내용 대조: 문서 준비와 지도 실행의 권한, Unity·테스트 제외, 미결 정책 처리, 원문 분류,
  지도 완료 조건과 미시작 감사 상태를 확인했다. 지도에 관한 사실 검증은 아직 수행하지 않았다.
- 지도 완료 조건 M01~M10: 아직 평가하지 않음.

### 문서 검토 중 확인한 동시 변경

문서 검토 명령 `python3 /tmp/verify_project_architecture_map_goal_docs.py`의 첫 실행에서
4개 문서·19개 로컬 링크와 형식에는 문제가 없었다. 다만 Git 상태를 새 Goal 4개만 추가됐을 것으로
비교한 부분이 별도 작업의 `docs/skills/run_relic_content_design_notes.ko.md` 추가를 발견해 exit 1로
종료했다. 코드나 테스트의 실패가 아니다.

`sed -n '1,90p' docs/skills/run_relic_content_design_notes.ko.md`로 서문과 콘텐츠 방향을 확인했다.
개별 유물은 후속 설계 상태였으며, 이 파일은 보존하고 PLAN의 입력 문서에 연결했다.
상태 비교 도구는 우리 문서의 추가와 범위 밖 동시 변경을 별도로 보고하도록 조정했다.
같은 명령을 다시 실행한 결과 exit 0, 문서 4개·유효한 로컬 링크 20개·형식 오류 없음이었다.
Git 상태에는 우리 Goal 문서 4개와 위 외부 콘텐츠 문서 1개의 추가가 관찰됐고 기존 상태 행의 제거는
없었다. 이 작업의 쓰기는 Goal 문서 4개로 한정했으며 외부 추가 파일은 건드리지 않았다.
예정 지도 `docs/project_map.ko.md`는 생성되지 않았음을 확인했다.

### 다음 위치

문서 준비 결과를 사용자에게 전달한다. 실제 지도 실행 요청이 오면 PLAN의 권한·비범위를 유지하고,
현재 작업 트리 확인과 입력 문서 읽기부터 시작한다.

## 2026-09-06 — 콘텐츠 제작 목표와 후속 검토 경계 보완

### 사용자 확인과 변경

사용자는 커뮤니티 제작자가 엔진이 지원하는 명령·행동을 조합해 보스·장비·파편을 RON으로 만들고,
개발팀이 엔진의 기능을 확장하는 방향을 확인했다. 지도에서는 현재 구조와 근거를 준비하고,
표현력이 충분한지와 필요한 확장은 후속 고성능 AI 검토에서 판단하기로 했다.

- PLAN에 후속 검토에 전달할 목표, 현재 데이터 조합 경계의 조사 항목, M02·M07 근거를 보완했다.
- 엔진 기능·제작 도구·배포 체계 설계는 비범위에 명시했다. 전체 콘텐츠·명령의 전수 조사를 추가하지 않았다.
- EXPERIMENT_NOTES에 결정과 미확정 범위를 기록하고 COMPLETION_AUDIT의 해당 기준을 맞췄다.
- 실제 지도 조사·작성과 완료 감사는 계속 미시작이다. 코드·기존 정책·Unity 계약을 변경하지 않았다.

### 확인과 도구

이 Goal의 네 문서를 읽어 현재 범위와 기록을 복원했다. 적용 지침은 직전 문서 작성 때 확인한 것을
재사용했다. `git status --short -- docs/goals/project_architecture_map`에서 기존 미추적 Goal 폴더를
확인했고, docs 하위 AGENTS 검색 결과는 없었다.

Python으로 네 문서의 수정 전 내용을 `/tmp/project_architecture_map_before_authoring_scope.json`에
보관한 뒤 `apply_patch`로 필요한 부분만 수정했다. 이번 보완에서는 runtime 코드를 조사하지 않았다.
문서 diff·로컬 링크·형식 확인 결과는 아래에 기록한다. 테스트·빌드·probe는 실행하지 않는다.

- Python의 `difflib.unified_diff`로 수정 전 기록과 대조해 목표·조사 경계·완료 기준 및 이번 기록의
  변경을 확인했다. 수정한 Goal 문서는 4개다.
- `Path.exists`와 Markdown 링크 추출로 로컬 링크 20개를 확인했다. 줄 끝 공백·마지막 개행·코드 블록
  짝 확인을 포함해 오류 없이 종료했다(exit 0).
- PLAN과 완료 감사의 M02·M07, 엔진 확장 평가의 비범위, 기존 테스트·Unity 제외 조건을 대조했다.
- 지도 산출물은 여전히 미생성이고 완료 감사는 미시작이다. 테스트·빌드·probe 실행은 없다.

## 2026-09-06 — 데이터 기반 제작 대상에 유물 추가

사용자 추가 요청으로 제작 대상을 **보스·장비·파편·유물**로 명시했다. PLAN의 목표·조합·공유 기능
조사 항목, EXPERIMENT_NOTES의 결정 요약, COMPLETION_AUDIT의 검토 범위를 맞췄다.
유물의 변경 목표를 현재 구현과 구분하는 기존 원칙을 유지했다. 변경은 Goal 문서에 한정하며
지도 실행과 엔진 확장 설계는 시작하지 않았다. 테스트·빌드·probe도 실행하지 않았다.

## 2026-09-06 — 콘텐츠 설계 상태와 검토 자료 전달 방식 확인

사용자는 장비·파편 설계를 모두 완료했고 유물을 설계 중이라고 확인했다. 검토 AI는 압축된 프로젝트
코드 또는 GitHub 링크와 지도를 함께 받아 원문 코드에 직접 접근할 예정이다.

PLAN과 EXPERIMENT_NOTES에 이 상태를 반영하고, 기존의 '전체 코드 전달 없이'라는 전제를 고쳤다.
지도는 구조·영향·원문 읽기 순서를 안내하며, 코드 발췌를 최소화한다. 기준 commit·미커밋 상태와
원문 경로를 기록하도록 전달 안내와 COMPLETION_AUDIT의 확인 항목도 맞췄다.
실제 압축·업로드·공개를 새 완료 조건으로 추가하지 않았다. 이번 변경은 Goal 문서 보완이며
runtime 조사·지도 실행·테스트·빌드·probe는 수행하지 않았다.

## 2026-09-06 — 전면 재작성도 후속 검토 선택지로 기록

사용자는 필요에 따라 전면 재작성도 고려한다고 밝혔다. PLAN에 후속 AI가 현재 구조 유지·부분
재설계·전면 재작성을 목표 충족과 근거·효과·전환 비용·위험·재사용 관점에서 비교할 수 있음을 기록했다.
EXPERIMENT_NOTES와 COMPLETION_AUDIT에도 같은 역할 구분을 반영했다.
재작성 결정이나 실행 승인으로 해석하지 않았으며, 지도 제작의 산출물은 현재 구조와 근거로 유지했다.
이 변경은 문서 보완이다. 코드·데이터 수정, 지도 실행, 테스트·빌드·probe는 수행하지 않았다.

## 2026-09-06 — 승인된 지도 조사·작성 실행 및 별도 문서 검토

### 실행 입력과 기준

- 사용자 첨부 `pasted-text-1.txt`와 기존 Goal 네 문서를 읽고, 실제 실행 승인·다섯 산출물만 쓰기·제외 조건을 복원했다.
- PLAN의 문서 준비 단계 표시를 실행 상태로 갱신했다. 기존 준비 이력과 다른 Goal은 보존했다.
- Git root `/mnt/f/work/simulator`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`와 다수의 기존 dirty 경로를 확인했다.
  원문 기준은 HEAD 단독이 아니라 당시 미커밋 작업 트리다.
- 조상 경로와 조사 대상의 `AGENTS.md` 탐색에서 적용 파일을 발견하지 못했다. 별도 에이전트는 사용하지 않았다.
- 룰북·core runtime·skill target·data loading 계약의 관련 절, 유물/성장·침식 정책 기록,
  파편·장비 및 유물 콘텐츠 노트의 상태와 대표 조우 맥락을 읽었다. Unity 계약·클라이언트 구현은 읽지 않았다.

### 실제 조사한 연결

다음 표는 단순 검색 결과가 아니라 선언·호출자·구현 본문·상태 변경·소비자를 연결해 읽은 범위다.
세부 파일 링크와 심볼은 지도의 해당 절에 두며 전체 명령·콘텐츠를 전수 감사한 것으로 해석하지 않는다.

| 경로 | 직접 읽은 주요 원문과 확인 결과 | 지도 |
| --- | --- | --- |
| 데이터 → 실행 | `data/live_ron.rs`, `data/mod.rs`, skill/equipment/fragment/boss/summoned 정의·검증, 대표 RON, `combat_setup/boss_runtime.rs`와 battle boss runtime. embedded 24 sources·compile·generated preview 검증 뒤 publication; schema vocabulary와 실행 지원 구분 | 2.2, 4.1 |
| 요청 → 결과 | 서버 `session.rs`·actor `handlers.rs`·`state.rs`·`main.rs`, 코어 `application.rs`, `world.rs`, `world/application.rs`·capability. 현재 공개 API 단절 D01; draft 안의 도메인 결과와 commit 뒤 공개 result projection 구분 | 2.1, 4.2 |
| 장비·파편 → 전투 | `world/maintenance.rs`·helpers, `resources/equipment_loadout.rs`, inventory/employee/skill_fragment, player_spawns/stat_pipeline/build. 양방향 장착·fragment ID별 progress·자동 형태 선택·새 runtime 구성; live 방어구 강화 recipe 존재 | 3, 4.3 |
| 시전 → 효과·처치 | `world/combat.rs`, battle sim/commands/basic_attack/damage_runtime/death_runtime/triggers, skill_runtime cast/area/projectile, damage/buff 정의. Pause 중 준비 cast와 실제 ability 호출·공명 소비, delivery 결과·후속 trigger 연결 | 4.4 |
| 직원·attempt | world state/helpers/combat, employee, battle build. attempt roster의 자격 검사·runtime ID별 전투불능 반영, 철수 정리와 재배치, 결과 반영·패배 재시도·Retreat 복원의 차이 | 3, 4.5 |
| 사망 → 결과·진행 | death_runtime/wave_runtime/sim/boss_runtime/summon_runtime, world combat/node_flow/grant_state, reward/encounter_progression. death epoch·required group·terminal 우선순위, CombatResult와 reward/node commit 분리 | 4.6 |
| 저장·복원·재접속 | world state/active_run/node_flow/facility/snapshot, event_log, public application, 서버 actor tick/attach/detach와 active_run/profile store. entry rollback·완료 SavePoint·live projection·영구 profile 구분; core payload V5와 host envelope V4 구분 | 3, 4.7 |

공통 기능/전용 처리의 대조에는 `suppress_little_red` → `little_red_reference_encounter`의 actor binding과 공통 task 실행,
`RepathToProtectedObjective`의 compile 거부, `cwbs_reference_summon_cycle`의 비production 표시,
Monolith shop의 domain 분기를 포함했다. 콘텐츠 ID lookup만으로 전용 행동 코드라고 판정하지 않았다.
유물은 현재 catalog·owner·장착·실행·저장 경로와 목표를 비교했으며, 신규 aggregate의 부재를 검색어 미발견만으로 결론내리지 않았다.

### 명령·도구와 처리

- `git status --short`, `git rev-parse --show-toplevel HEAD`, `git ls-files -z --full-name --cached --others --exclude-standard`,
  `git status --porcelain=v1 -z --untracked-files=all`로 경로·기준과 기존 변경을 확인했다.
- `rg --files`, `rg -n`으로 경로·선언·호출을 찾고 `cat`, `sed -n '<구간>p' <파일>`로 관련 본문을 읽었다.
  출력이 생략된 경우 좁은 구간으로 다시 읽었다. 추측한 roster 분리 파일은 없었고 실제 `employee.rs`의 선언·구현을 확인했다.
- 수정은 `apply_patch`로 승인된 다섯 문서에만 수행했다. 조사 중 anchor 불일치로 적용되지 않은 patch는
  현재 문구를 확인하고 다시 적용했다. 기존 source나 정책 내용을 맞춰 고치지 않았다.
- Python inline 읽기 전용 문서 확인으로 상대 경로·링크·backtick 탐색어·Markdown 구조와 SHA-256을 대조했다.
  검증 스크립트·임시 문서·fixture 파일은 새로 만들지 않았다. 앞선 준비 단계의 `/tmp` 기록 생성과 구별한다.
- 큰 상태 출력이나 완료 출력을 확보하지 못한 최초 hash 시도는 검증 증거로 사용하지 않았다.
  완료 출력을 확보한 아래 기준/대조만 증거로 기록한다.

### 핵심 조사 결과

지도의 D01~D07로 분류했다. D01은 서버가 이전 코어 공개 표면을 사용하는 실제 소스 단절,
D02는 룰북 Breakdown node countdown과 현재 condition/policy/checkpoint 사이 차이,
D03은 data loading 계약의 core payload V4 기술과 현재 V5 writer/V4 importer 사이 불일치다.
D04는 설계 완료와 live placeholder·기존 강화의 구분, D05는 조사 경로에서 확인한 신규 유물/침식 기능 부재,
D06은 runtime proc identity/수명과 목표 사본 수명의 차이, D07은 schema와 실행 지원의 차이다.
이들은 수정·정책 결정이나 runtime 실패 재현 결과가 아니다.

### 지도 작성 후 별도 읽기 pass

완성한 지도 1~7절을 다시 읽고 PLAN M01~M10, 필수 흐름 7개, 정책 관점 11개를 대조했다.
정책 표의 16개 행이 해당 관점을 담고 있는지 확인했다. 특히 다음 본문을 재확인했다.

- `invoke_ability`의 cast 생성과 step 예약, TileArea/Projectile의 impact→공통 effect→delivery 결과 소비.
- `ManualCastEnd`의 실제 ability 호출 뒤 공명 소모, `UnitDied` command의 무처리와 HP 경로의 사망 확정.
- `execute_application`의 draft/root commit과 `ApplicationResult::from_behavior`의 commit 후 projection.
- `RunCheckpointPayload`·entry restore와 profile/peak 보존, 서버 Attach/Detach·저장 실패 처리의 의도 및 현재 포트 단절.
- boss microstep/task 실행, wave death epoch와 terminal, 직원 condition의 자격 및 node 완료 시점.
- 강화·개화 정의/소비, live `standard_armor`, 계약 두 곳의 codec 버전, 현재 Breakdown 관련 owner·policy·완료 경로.

문서 검토에서 수정한 내용:

- 파편의 ID별 progress와 유물 독립 사본을 비교하던 문장에서 유물 자체 성장 상태가 있는 듯한 표현을 제거했다.
- 상태 표의 `active_battle`을 실제 `Option<ActiveBattleSession>`로 맞추고 Critical/Dead/Breakdown 투입 제한을 명시했다.
- `UnitDied` 명령만으로 kill을 수행할 수 없음을 보완했다.
- transaction 그림에 draft 생성 전 lifecycle 거부 경로를 추가했다.
- 유물의 전투 효과 범위·자체 강화 없음·방어구/액세서리 자체 효과 허용과 **한 연쇄 경로**의 순환 제한을 명확히 했다.

지도 로컬 링크 157개(서로 다른 원문 파일 86개)는 모두 존재한다. Mermaid 3개의 호출 의존/데이터 변환/transaction
의미를 본문과 수동 대조했으며 fence 표식 6개(블록 3개), 줄 끝 공백 없음·마지막 개행을 확인했다.
심볼은 원문 선언/분기와 주요 호출·소비 본문을 재대조했다. 추가 문자 검색은 보조 수단이며,
문자열 일치만으로 동작·컴파일 성공을 증명하지 않는다. 다이어그램 renderer·패키지를 설치하거나 실행하지 않았다.
기록 반영 후 다섯 산출물의 링크·표 열 구조·fence 짝·공백·마지막 개행을 확인했으며 오류가 없었다.

### 기존 변경 보존 대조

조사 기준과 지도 작성 후 재확인에서 다음 값이 동일했다.

| 대조 대상 | 결과 |
| --- | --- |
| HEAD | `f318414fb0156a67e04c5479000c2a2683fc0c09` 유지 |
| Git 목록의 코어 src·서버 src·live data | 233경로: core 191, server 17, resources 25; 경로별 bytes SHA-256 또는 MISSING을 정렬하여 만든 목록이 동일 |
| 위 목록 JSON의 SHA-256 | `3f3f1539fa0debc5762271619a6416d49cc9495b790d173663c5d1c9338fb6f1` |
| 다섯 산출물을 제외한 porcelain 상태 | 16,415행, 정렬·NUL 결합 SHA-256 `75d5959d4debdc3669612daa02e6a3851f9119eafa80ddaff85435884132b69e` 유지 |

이는 Git 목록의 해당 source/data bytes와 다른 경로의 Git 상태 대조다. 무시된 파일 전체의 내용 동일성이나
기존 모든 문서의 byte 단위 동결을 증명하는 것으로 확대하지 않는다. 이 작업의 실제 쓰기 명령은 다섯 산출물에 한정됐다.
Goal 네 파일은 작업 시작부터 미추적이었고 지도도 미추적 상태다. commit·stage·기존 변경 되돌리기는 수행하지 않았다.

### 검증 상태와 결과 전달 위치

- 테스트·빌드·check·clippy·probe·benchmark·서버 실행·fixture 생성·패키지 설치: 없음.
- 사용자 `all test` 통과 보고만 유지했다. AI 실행 수·pass/fail/skip은 해당 없음이다.
- 필수 원문 접근 불가나 미조사로 남긴 완료 조건은 없다. 실제 실행/배포/Unity/전체 콘텐츠 표현력과
  보류된 gameplay 결정은 지도의 미확인·미결정·비범위로 남겼다.
- 산출물은 [지도](../../project_map.ko.md)와 기존 Goal 네 문서다. 최종 M01~M10 판정은 [완료 검토](COMPLETION_AUDIT.md)에 기록한다.
- 후속 위치는 원문과 같은 기준 상태의 지도 전달 및 설계 검토다. 이 작업에서 압축·업로드·공개나 신규 구현 Goal을 만들지 않는다.
