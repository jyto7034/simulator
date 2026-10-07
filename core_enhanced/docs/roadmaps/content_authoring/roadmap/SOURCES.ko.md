# 출처·확인 범위·현재 코드 조사 입구

## S00 — 이번 v2 작성의 기준

날짜는 **2026-09-09**다. 기존 v1 파일은 2026-09-08로 표시되어 있어 새 개정일과 구분한다.

`Codex with ChatGPT · simulator`의 workspace_info를 이번에 호출했다. simulator / Rust / branch `cleanup/simulator-dirty-worktree-20260803` / HEAD `f318414` / dirty=true였다. 전체 dirty diff fingerprint를 캡처한 것은 아니다. HEAD만 체크아웃한 작업 트리를 이 기준과 같다고 보지 않는다.

이번에는 기존 대화 첨부 단일 Markdown과 ZIP의 내용·작업 목록을 읽어 v2 문서를 작성했다. 프로젝트 쓰기·빌드·테스트·benchmark는 수행하지 않았다. 로컬 문서 파일·JSON·링크·의존성·해시 점검은 별도 문서 정합성 검사다.

## S01 — 직접 재확인한 게시 경계

`core_enhanced/src/game/data/mod.rs:803–822`, `load_live_sources`: parse→into_validated→private candidate→generated combat preview validation→ValidatedGameData 반환을 확인했다. 이는 외부 팩 시스템 구현 완료 증거가 아니라 재사용할 현재 publication 경계다.

## S02 — 직접 재확인한 보스 작성 task

`core_enhanced/src/game/data/boss_runtime_data.rs:164–230`: BossPatternStateData와 raw BossTaskData/ID 접근을 확인했다. raw enum은 RouteCombat·AcquireTarget·MoveToTarget·RejoinForwardRoute·Wait·Summon·EmitSignal·RepathToProtectedObjective를 포함한다. **raw enum 존재는 실행 지원의 증거가 아니다.** compiler/runtime은 CA16에서 대조해야 한다. 새 CastSkill/ActionHandle/blackboard 등은 v2의 제안이다.

## S03 — 기존 v1에서 이어받은 조사 입구

아래는 이번에 모두 재조회한 파일 목록이 아니라 v1과 앞선 검토의 source map이다. 구현 AI는 선택한 CA의 실제 현재 내용을 읽어 가정을 검증한다.

| 경계 | 현재 파일·심볼 후보 |
| --- | --- |
| 효과 작성 | `core_enhanced/src/game/ability.rs::SkillDef, SkillStepDef, SkillEffectDef`; `data/skill_data.rs` |
| loading | `data/live_ron.rs::GameplayRonSourceSet, ParsedGameplayRon, LiveRonError`; `data/validation.rs` |
| 사본·장착 | `skill_fragment.rs`; `data/skill_fragment_data.rs`; `resources/inventory.rs`; `resources/equipment_loadout.rs` |
| 산술 | `stats.rs`; `battle/types.rs::UnitCombatProfile, scale_attack_interval_ms`; `battle/stat_pipeline.rs`; `battle/damage.rs` |
| 사건·상태 | `battle/core/types.rs`; `ids.rs`; `build.rs`; `damage_runtime.rs`; `triggers.rs`; `death_runtime.rs`; `commands.rs` |
| flow·전달 | `battle/core/sim.rs`; `skill_runtime/{cast,area,projectile}.rs`; `basic_attack.rs`; `basic_attack_projectile.rs` |
| 보스 | `data/boss_runtime_data.rs`; `battle/core/boss_runtime.rs::admit_state_tasks, run_boss_runtime_boundary, boss_runtime_allows_combat_action`; `combat_setup/boss_runtime.rs` |
| 소환·wave·작업 | `data/summoned_unit_data.rs`; `battle/core/{spawn_kernel,summon_runtime,wave_runtime,field_work}.rs`; `data/pve_data.rs` |
| 런·결과·저장 | `world/state.rs`; `world/combat.rs`; `world/active_run.rs`; `world/application.rs`; `world/snapshot.rs`; `core_enhanced/src/application.rs` |
| 등록·평가 | `reward.rs`; `world/grant_state.rs`; `encounter_progression.rs`; `combat_power.rs`; `world/node_flow.rs`; `world/shop.rs` |

첫 행 이외의 `data/`, `battle/`, `world/` 경로는 `core_enhanced/src/game/` 기준이다. 위치가 바뀌었으면 심볼과 호출 관계로 찾는다. A 문서의 새 모듈명은 실제 디렉터리 존재 주장이 아니다.

## S08 — 공식 콘텐츠와 15축

`core_enhanced/docs/project_map.ko.md`는 탐색 입구다. `docs/skills/run_relic_content_design_notes.ko.md`의 효과 단위·단계/분기·15축·태그/속성/사건 구분을 유지한다. 이전 검토의 267–378 같은 줄 번호는 위치 안내이며 현재 heading을 우선한다.

파편·장비는 `core_enhanced/docs/skills/abnormality_skill_fragments_design_notes.md`, 보스 조우는 `core_enhanced/docs/skills/abnormality_design_notes.ko.md`의 최신 해당 절을 따른다. CA00에서 전체 합의된 기계적 요구를 읽고 목록화한다. 이번 v2 개정에서 이 장문들을 다시 전수 감사했다고 주장하지 않는다.

C01–C07 선정과 미정 상태는 v1 추적표에서 이어받았다. 이름이 같은 보스와 파편은 서로 다른 콘텐츠다. 실제 구현에서는 latest approval와 기본형·개화형·탱/딜 분기를 다시 대조한다.

## S09 — 승인 정책과 현재 canonical

`core_enhanced/docs/tmp/run_relic_system_design_notes_2026-08-21.ko.md`와 `ego_corrosion_system_design_notes_2026-08-19.ko.md`의 승인된 목표, 이 대화의 외부 팩·최신 규격·제작자 권한·개별 개화 결정을 함께 사용한다. 과거 코드의 종류별 개화가 최신 대화의 사본별 개화보다 우선하는 것이 아니다.

`core_enhanced/docs/{game_rulebook,core_runtime_contract,skill_target_contract,data_loading_contract}.md`는 current canonical과 전환 영향 확인 대상이다. 권장 architecture를 작성했다고 자동으로 canonical이나 구현이 변경되지는 않는다. 실제 CA에서 승인된 변경의 owner 문서를 갱신한다.

## S10 — 테스트·작업 절차

v1에서 확인한 테스트 입구는 `core_enhanced/tests/{ron_loading,skill_runtime_contract,live_item_skill_activation,live_skill_catalog_audit}.rs` 등이다. package·feature·target 이름은 구현 직전 Cargo.toml과 대조한다. 이전 테스트 기록은 현재 실행 결과가 아니다.

현재 프로젝트의 `engineering_workflow.md`, `policy_to_goal_pipeline.md`, `goal_execution_pipeline.md`는 구현 AI가 실행 범위에 맞게 확인한다. 이 패키지가 새로운 실제 Goal을 생성하거나 완료한 것은 아니다.

## S11 — 이전 문서와 대화

이전 첨부 `simulator_RON_content_authoring_roadmap.ko.md`와 `simulator_RON_content_authoring_roadmap_package.zip`에서 범위·CA00–CA23·PG00–PG09·C01–C07·T00–T24를 이어받았다. v2는 목적 중심 작업에 구조 선택·조사 질문·L 설계·DG·delta와 보스 포트 설계를 추가한다.

기존 첨부 파일은 보존했고 v2를 새 파일로 만들었다. source 코드 수정 없이 문서 개정만 수행했다.

## S12 — v2.1 순서 근거·운영 인수 명료화

2026-09-09 v2.1은 기존 v2 Markdown·ZIP, 현황 v1.0·운영 규칙, P2G r2 및 운영안의 관련 절을 근거로 한 **문서 개정**이다. 위 S00–S11은 과거 조사·개정의 출처 기록으로 보존했다. 이번에는 simulator 코드·의존성 graph·Phase 중간 상태를 새로 전수 조사하거나 빌드·테스트·GEX 감사를 수행하지 않았다.

수정한 것은 후보 순서의 표기, 실제 P2G-20/40 의존성 확인과 GEX 준비 확인의 구분, CA00의 주요 공유 경계 1차 조사, 각 작업의 관련 근거 요구다. v2의 edge 목록과 권장 architecture·게임 정책은 보존했다. 실제 Project/CA 상태는 운영 문서 인수 전 미확인이다.

P2G r2의 P2G-40 dependency edge reason, Intermediate-State Audit와 기존 변경·권한 절차를 재사용한다. 본 개정이 신규 Policy Topic·Freeze·Phase Plan을 만들거나 그 의미 검토를 통과시킨 것은 아니다. 문서 정적 검사 결과는 별도 `DOCUMENT_CHECKS.json`에 기록한다.
