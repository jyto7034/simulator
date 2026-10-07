# U01 / R08–R10 F01 교정 실행

2026-10-04 · 현재: GEX-10 교정·자체 검증 완료 / fresh GEX-20 인계 준비. 감사 판정은 별도.

## 기준·권한

- Exact Goal: `GS-CARX-001 / GOAL-CARX-01`, Handoff `HA-CARX-001`.
- 원 기준: [PLAN_HANDOFF](../../../../PLAN_HANDOFF.ko.md), [55조건](../../COVERAGE.json),
  [F01 독립 감사](../../../../post_audit/20261003_001/COMPLETION_AUDIT.md).
- 상위 요청: [Core 연속 실행](../../../../../../roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md).
  정책 owner 반환·GEX 교정·독립 재감사·선행 통과 후 후속 작업을 포함한다. 사용자 단계검토 정지는 없다.
- [U00](../../../../../../roadmaps/core_first/execution/u00_20261004_001/INTAKE.ko.md)에서 동일한 387개 대상을 대조했다.
  386개 동일, 현황 문서만 변경. 기존 records 1,435·archives/patches 9개 hash 일치. 제품 교정 successor 없음.
- 첫 동결 `baseline/`은 이 pass의 실제 mutation 전 파일과 membership을 보존한다. 기존 dirty/untracked는 사용자 변경이다.
- 현재 F01 Not Complete를 제한적으로 교정한다. 새 원정·공명·침식·구저장 제거·도구 정책은 제품 변경에서 제외한다.
  이번 U00의 차기 정책 입력은 이 Goal의 현행 R08–R12 acceptance를 바꾸지 않는다. 새 P2G 초기화는 불필요하다.

## 상세 설계와 검증할 경계

현재 `cancel_current_state`/timeout → `detach_boss_action`은 pending 준비 중인 동일 action에만
actor 전체 `cancel_boss_owned_work`를 적용한다. 준비 종료 뒤 `active_skill_casts`와 Flow/wake가 남는다.

교정은 actor ID와 opaque ActionHandle이 모두 일치하는 pending cast/active cast만 대상으로 한다.
pending 준비의 소유 locks·취소 기록을 정리하고, 해당 cast의 미방출 SkillStep/FlowWake와 continuation을
회수한다. 이미 방출한 projectile/area의 독립 source·resource lease·종료 event는 보존한다.
actor 전체 이동/basic attack/다른 action을 취소하는 우회는 사용하지 않는다. 명시 Detach와 정상 Released
완료는 기존 detach만 수행한다. stale handle/wake와 이후 state/action은 영향받지 않아야 한다.

큐 스캔/변경은 기존 work accounting·후보 transaction rollback에 포함한다. 스캔 실패를 성공으로 삼키지 않는다.
공개 RON·정상 command의 재현과 좁은 내부 경계 시험을 구분한다. 새 gameplay·schema·wire 의미는 추가하지 않는다.

## 자체 검증과 감사 인계

1. 원 감사의 정상 외부 팩 5개 oracle을 새 release 실행으로 확인한다: Cancel50/Cancel1500/Cancel500/state-exit/Detach1500.
2. 실제 RON 회귀: 준비 중/후 timeout, 강제 이탈, 방출물 보존, 후속 area 차단, 명시 Detach/Released 보존.
3. 정확 action 격리·새 state·stale/같은 시각·예약 회수, 취소 중 fault rollback/retry·resource/provenance 회귀.
4. 기본/feature all-target 및 관련 check/release/clippy/fmt를 현재 target에서 확인한다. 과거 PASS 재사용 범위와 새 실행은 분리한다.
   공유 queue/action 변경의 영향이 넓으면 55조건·R06/R07 선택자와 공개 CLI/PC36/R05의 영향 근거도 갱신한다.
   20분 시험은 제외하며 기존 concurrent peak 단독 합의 경고의 기준을 완화하지 않는다.
5. 별도 after target·manifest·정/역 복원·검증 원자료를 고정한다. 기존 감사/target은 덮어쓰지 않는다.
6. 구현 비참여 GPT-6 Astra/High 에이전트가 fresh GEX-20에서 F01과 영향 조건을 판정한다.
   실패하면 새 correction pass로 교정하고 재감사한다. 통과 후 상위 요청에 따라 U02를 선택한다.

실행 원자료는 `EXPERIMENTS.md`/`evidence/`, 판단은 `EXPERIMENT_NOTES.md`, 감사 인계는 종료 시 별도 작성한다.
자체 시험 성공은 독립 Complete/High 판정이 아니다.
