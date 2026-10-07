# DefenseRoute 전술 지휘 구현 Master Goal — Experiment Notes

Status: complete

## 초기 구조 판단

- 사용자는 2026-08-06 각 구현 사항을 순차 Phase로 취급하는 Master Goal 구조와
  `defense_route_tactical_command_master` 이름을 명시적으로 승인했다.
- Master는 여러 독립 source-of-truth 경계를 직접 구현하는 거대 단일 Goal이 아니다.
- Master는 순서, 공통 불변식, coverage, Phase 완료 감사와 최종 완료만 조정한다.
- 각 Phase는 활성화될 때 필수 문서 3개를 가진 별도 단일 Goal로 초기화한다.
- 한 번에 하나의 Phase만 활성화하며, 이전 Phase 완료 감사 전에는 다음 Phase를 시작하지 않는다.

## Source-of-truth 판단

- Goal 작성, 단일/Master/Phase 선택과 완료 감사 절차는
  `core_enhanced/docs/goal_authoring_and_execution_guide.md`를 따른다.
- 현재 runtime 사실은 Core/game_server code와 production live data가 우선한다.
- 확정 목표 정책은 현재 임시 조사 문서의 번호별 확정 절에 보존되어 있다.
- Phase 0에서 최종 정책문을 decision record로 분리하되, 구현되지 않은 미래 정책을 현재 runtime
  canonical처럼 표현하지 않는다.
- 각 canonical 문서는 관련 Phase가 구현될 때 실제 contract와 함께 갱신한다.
- Master와 Phase Goal은 실행 기록이지 durable gameplay/data/wire source of truth가 아니다.

## 적용하지 않은 대안

### 모든 Phase를 하나의 단일 구현 Goal에 기록

적용하지 않는다. 전술 자원, control capability, 무기 data, 태세, 이동, targeting과 현장 작업은
각각 독립적으로 구현·검증 가능한 계약 경계다. 하나의 실행 기록에 합치면 scope 판단, baseline,
migration과 완료 증거가 혼합된다.

### Master 없이 Phase Goal만 독립 실행

적용하지 않는다. Phase 사이에 명확한 선행 관계가 있고 `DR-01`~`DR-17` 전체 coverage와 최종
교차 검증을 한 곳에서 추적할 필요가 있다.

### 미래 Phase state/schema 선행 추가

적용하지 않는다. 불완전한 중간 runtime과 사용되지 않는 placeholder가 생기며 Phase 완료 감사를
왜곡한다.

## 외부 시스템 의존성과 미검증 소비자

- 외부 Unity canonical과 Unity runtime이 존재하는 경로는 알려져 있다.
- 사용자가 Unity에는 구현된 것이 없으므로 앞으로 읽을 필요가 없다고 명시했다.
- Unity 문서, code, reader, store, presentation과 tests는 이 Master에서 접근하거나 검증하지 않는다.
- Master 완료는 Unity consumer 완료를 뜻하지 않으며, 필요해지면 사용자 승인 아래 별도 후속
  consumer Goal이 필요하다.

## 초기 dirty worktree

Master 초기화 직전 `git status --short` 결과:

```text
 M core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md
?? core_enhanced/docs/tmp/
```

- `core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md`는 기존 사용자 수정으로 간주한다.
- `core_enhanced/docs/tmp/`에는 이번 정책 검토 임시 문서가 있으며 Phase 0 입력이다.
- 위 항목을 임의로 되돌리거나 정리하지 않는다.
- 각 Phase 시작 시 관련 저장소, 인접 저장소와 수정 대상 파일의 diff를 다시 확인한다.

## 경로 충돌

- 초기화 전 `core_enhanced/docs/goals/defense_route_tactical_command_master`는 존재하지 않았다.
- 필수 문서 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`도 존재하지 않았다.
- Phase 0 child Goal 후보 경로는 활성화 직전에 충돌이 없음을 확인했다.
- Phase 0~9 child path를 순서대로 생성했고 각 predecessor `Complete / High` 뒤에만 다음 Phase를 활성화했다.
- 모든 child path와 completion audit이 존재하며 활성 Phase는 남아 있지 않다.

## 최종 구조 판단

- Core owns battle state, command eligibility, object/activity/time state and atomic mutation.
- Production live RON owns exact resource/weapon/skill/encounter tuning and fails closed on removed/unknown shapes.
- game_server forwards current Core behavior/result/setup/update/checkpoint DTOs and does not reconstruct tactical rules.
- Final strict versions are active-run Core `3`, skill catalog `3`, setup `2`, battle event log `31`; active battle has no
  durable save reader or migration.
- Canonical gameplay/runtime/data documents describe the implemented contract; Goal documents remain execution evidence only.

## 생략한 검증과 근거

- Unity runtime/contract/UI/tests: 사용자가 전체 Master의 명시적 비범위로 승인했다.
- Strict `-D warnings` clippy: child audit에서 기록된 existing tree-wide lint debt이며 required completion gate가 아니다.
  Final compile/test/format/diff gates are clean.

## 범위 밖 후속 후보

- Unity tactical command consumer, UI와 presentation
- `PlayerTile`을 사용하는 첫 live skill content
- 추가 FieldWork 종류와 범용화
- 전술 이동과 별개의 보행·pathfinding·reservation

후속 후보는 이 Master에서 자동으로 Goal을 만들거나 구현하지 않는다.

## 남은 위험

- Unity 제외로 인해 Core/game_server 계약의 외부 presentation consumer adoption은 이 Master가 증명하지 않는다.
- PlayerTile 첫 live content, 추가 FieldWork kind/generic framework와 walking/pathfinding은 의도적으로 구현하지 않았다.
- 위 항목은 승인된 비범위/후속 후보이며 Master correctness gap이 아니다.

## 최종 판정

`Complete / High`. `DR-01`~`DR-17`, Phase 0~9, final Core/live data/game_server shapes, broad validation and canonical
alignment are all evidenced in [COMPLETION_AUDIT.md](COMPLETION_AUDIT.md). No required action remains inside this Master.
