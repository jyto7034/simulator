# R06 WP02 재개 실행 PLAN — action 완료 결과와 release Snapshot

- Goal: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, `WP-CAVER-02`의 잔여 구현.
- 선행 대상: EEP-F01~F03 교정 `target/correction_001/MANIFEST.json` SHA-256 `abe5a96830d8090f77e63fde7c6d7ff79675f701a382437d2492e0c66e9aa60a`; 후속 `review_002` 부분 PASS. `freeze.py verify`에서 source 333개 일치.
- 지침: 현행 `goal_execution_pipeline(1).md`의 기존 적용 `2026-09-13-gex-r7`. 파일의 단계검토·증거 재사용·구현 전후 고정 절에 의미 변경 없음. 이번 사용자 요청은 기존 단계검토를 명시적으로 유지한다.
- 기준선: [entry_001/MANIFEST.json](baseline/entry_001/MANIFEST.json). branch/HEAD, 전체 dirty/untracked 상태와 선행 source 333개 실제 bytes, source/data/test/config/lock 입력을 archive에 보존했다. Git HEAD는 현재 source 기준선이 아니다.

## 실제 연결과 실행 순서

1. `RuntimeFactState::ActiveAction`이 exact `ActionHandle` 및 기존 R04/R05 종료·retention과 함께 완료 결과 witness를 소유한다. `BattleCore`가 현행 `execute_ordered_skill_effects`에서 skill 전체의 필요한 결과 projection을 정하고, 각 effect의 실제 fact 관찰 후 같은 candidate의 witness를 갱신한다. `ActionSite`와 완료 frontier를 별도로 검사한다. `query_action`/마지막 action 결과를 완료 근거로 사용하지 않는다.
2. `execute_ordered_skill_effects`는 effect 전체 대상 평가 직전에 현재 action 완료 witness를 읽고, 기존 command 처리와 fact projection이 성공한 뒤 다음 frontier를 닫는다. 기존 delivery-local `EarlierEffect` 순서·결속은 유지한다. 같은 occurrence를 여러 delivery에서 한 번씩만 합산한다.
3. `build_value_expr_delivery_capture`는 projectile/area의 finite release에서 Snapshot에 필요한 선행 완료 결과를 읽어 기존 `EvalCapture.snapshot_frame`에 고정한다. immediate 경로도 같은 capture API를 사용한다. consumption의 Live는 현재 완료 frontier를 읽는다. release 이전 결과 부재 및 다른 attempt는 typed fault다.
4. 오류는 `execute_ordered_skill_effects` → immediate/projectile/area caller → `runtime_transaction` 또는 timestamp candidate의 `operation?`로 전달한다. witness는 `RuntimeFactState` clone에 들어가므로 실패한 candidate의 facts·witness가 함께 폐기된다. 기존 확정된 앞 timestamp는 보존한다.

## 단계검토 중단 조건

첫 의미 있는 책임이 실제 빌드·영향받는 회귀 시험을 통과하고 전후·남은 WP02 범위를 설명할 수 있을 때 다음 큰 연결 작업 전에 고정·중단한다. 우선 action-wide 완료 결과와 release Snapshot은 동일 witness owner에 직접 의존하므로 함께 연결한다. aggregate·preview/explain 및 WP03/04는 다음 구간이다. 중단 지점은 실제 검증 결과로 확정한다. 생산 expression publication은 열지 않는다.

## 검증 기준

- `cargo check -p game_core --all-targets --offline --locked`, 해당 result/release/candidate 시험, 관련 battle/resource/literal 회귀. 공유 action state와 clone 영향이 넓으면 all-target 제품 시험·기존 Clippy·format·R05 allocation guard와 PC fixture를 확인한다.
- 격리 `CARGO_TARGET_DIR` 및 `/tmp` Cargo cache를 사용한다. 명령·환경·toolchain·selector·로그·exit·실제 시험 수와 실행 source hash를 기록한다.
- 필수 검증을 끝내지 못하면 단계검토 target 완료를 선언하지 않고 실패 및 재개 위치를 기록한다.

## 실행 후 중단 지점

실제 연결은 `ActiveAction`의 선택적 boxed witness, `execute_ordered_skill_effects`의 첫 effect 전 설치·각 effect 후 settle, `build_value_expr_delivery_capture`의 release 읽기와 effect 시작 Live 읽기로 확정했다. finite fact가 ticket을 저장하지 않는 경계는 실행 중 source ticket과 fact action/site로 결속했다. witness는 candidate clone에 포함되고 action settlement에서 해제된다. 초기 inline 저장은 R05 cap을 넘었으므로 일반 action에 payload 공간을 예약하지 않는 선택적 box로 수정하고 재검증했다. 전체 제품·관련 focused·PC·R05 검증 후 D04 aggregate와 D07 preview/explain 연결 전에 고정·중단한다. 실제 명령과 실패·재실행 이력은 `EXPERIMENTS.md` 및 `evidence/`에 기록한다.
