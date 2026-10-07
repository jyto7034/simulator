# Acceptance Gates — Transaction Kernel Foundation

- 상태: `prepared`
- 판정 시점: PLAN Phase 5–6
- 원칙: 모든 필수 gate가 증거와 함께 PASS여야 완료 가능

## 1. 판정 형식

실행 중 각 gate에 다음을 기록한다.

- 상태: `NOT_RUN`, `PASS`, `FAIL`, `BLOCKED`
- 실행 명령 또는 review procedure
- 환경/feature/profile
- 결과 요약
- artifact 또는 test path
- baseline 대비 차이
- 판정자와 시각

기억에 의존한 PASS, “전체 테스트가 대체로 통과함”, 관련 없는 suite 결과는 증거가 아니다.

## G0 — Reproducible baseline

필수 조건:

- [ ] 시작 commit과 dirty worktree 기록
- [ ] Rust/Cargo toolchain과 feature 조건 기록
- [ ] full test baseline 1회
- [ ] fmt check baseline 1회
- [ ] strict clippy baseline 1회
- [ ] 관련 performance baseline
- [ ] 기존 실패와 Goal로 생긴 실패를 구분할 exact output/summary

판정: `NOT_RUN`

## G1 — Exhaustive command classification

필수 조건:

- [ ] `COMMAND_TRANSACTION_INVENTORY.md`와 `ActionKind::ALL` exact-set 일치
- [ ] 모든 action에 transaction class와 authority root 존재
- [ ] descriptor match가 exhaustive
- [ ] 새 action을 추가하고 transaction classification을 빠뜨리면 compile 또는 test가 실패
- [ ] Query/Recovery/Gameplay 분류가 기존 route policy와 모순되지 않음

권장 자동화:

- `ActionKind::ALL`을 순회해 descriptor/transaction metadata를 검증하는 table test
- inventory-derived assertion 또는 사람이 확인 가능한 generated diagnostic

판정: `NOT_RUN`

## G2 — Root enforcement and single writer

필수 조건:

- [ ] target gameplay mutation은 execute root의 transaction selection을 통과
- [ ] target handler를 호출하는 별도 production bypass가 없음
- [ ] old path와 new path dual-write 없음
- [ ] error 시 fallback으로 legacy mutation을 재실행하지 않음
- [ ] publish 전 fallible 작업과 publish 후 infallible 작업의 경계가 code review로 식별 가능
- [ ] metadata만 추가되고 handler semantics가 그대로인 허위 완료가 아님

증거:

- call-site search
- target route tests
- migration ledger old writer removal entry

판정: `NOT_RUN`

## G3 — Live battle rejection atomicity

각 action에 대해 future cursor 또는 동등한 post-decision failure를 주입한다.

- [ ] `PauseBattle`
- [ ] `ResumeBattle`
- [ ] `SetBattleSpeed`
- [ ] `BeginSkillTargeting`
- [ ] `CancelSkillTargeting`
- [ ] `ActivateSkill`

각 테스트는 다음을 모두 검증한다.

- [ ] expected error/result
- [ ] active battle authority before == after
- [ ] revision/cursor/event history before == after
- [ ] focus/activation/playback 관련 state before == after
- [ ] 승인된 monotonic exception 외 변화 없음
- [ ] 같은 initial state에서 valid retry 성공

response error만 검증하는 테스트는 이 gate를 통과시키지 못한다.

판정: `NOT_RUN`

## G4 — StartNewGame rejection atomicity

필수 조건:

- [ ] invalid/missing RunStart trigger 또는 실제 fallible 경계를 재현
- [ ] player/run/map/event/battle/profile 관련 authoritative roots의 before == after
- [ ] partial initial inventory/resource/phase가 남지 않음
- [ ] save/checkpoint-visible state가 변하지 않음
- [ ] 실패 후 corrected 또는 identical-valid retry가 성공
- [ ] 성공 publish 전에 trigger/response construction이 완료됨

판정: `NOT_RUN`

## G5 — Success parity and determinism

필수 조건:

- [ ] 대상 일곱 명령의 success response shape/meaning 유지
- [ ] state delta, revision, event/projection delta 유지
- [ ] battle activation deterministic outcome 유지
- [ ] checkpoint/save/new-game success behavior 유지
- [ ] existing staged tactical commands 회귀 없음
- [ ] 같은 initial state/input 반복 결과 동일

wire byte-for-byte equality가 기존 contract상 필요한지, semantic equality면 충분한지는 각 surface별로 기록한다. 임의로 완화하지 않는다.

판정: `NOT_RUN`

## G6 — Failure/retry conformance matrix

대상 각 action row에 다음 case가 연결되어야 한다.

| Case | Required |
|---|---|
| invalid precondition before staging | yes |
| failure during decide/validate | applicable |
| failure during response/projection construction | yes for audited defect |
| authority equality after failure | yes |
| corrected retry | yes |
| success after prior rejection | yes |
| duplicate/expected authority guard behavior | applicable |

- [ ] 일곱 action 모두 matrix complete
- [ ] test names와 source paths 기록
- [ ] ignored/flaky test 없음

판정: `NOT_RUN`

## G7 — Non-regression and code quality

필수 조건:

- [ ] 관련 narrow suites PASS
- [ ] full test가 baseline 대비 새 실패 없이 PASS
- [ ] fmt check가 baseline 대비 악화 없음
- [ ] strict clippy가 baseline 대비 새 warning/error 없음
- [ ] 새 dependency 없음
- [ ] save/protocol shape 의도치 않은 변경 없음
- [ ] production TODO/fallback/debug branch 없음

baseline에 기존 clippy 실패가 있으면 이 Goal이 그 전체를 고칠 필요는 없다. 대신 동일 명령의 exact delta로 새 문제 없음을 증명한다.

판정: `NOT_RUN`

## G8 — Performance non-regression

필수 조건:

- [ ] before/after가 같은 runner/profile/fixture/repetition 조건
- [ ] 대상 success path p50 또는 median regression <= 10%
- [ ] 대상 success path p95 regression <= 15%
- [ ] allocation/clone 변화 기록
- [ ] 모든 gameplay action에 full `GameCore` clone이 추가되지 않음
- [ ] noise가 threshold 근처면 재측정

threshold 초과는 사용자의 명시적 승인 없이 PASS로 바꿀 수 없다.

판정: `NOT_RUN`

## G9 — Migration integrity and recovery

필수 조건:

- [ ] `MIGRATION_LEDGER.md`가 실제 code route와 일치
- [ ] 각 target action의 old writer가 removed/retired로 기록
- [ ] 단위별 narrow verification 증거 존재
- [ ] 승인된 경우 atomic commit 경계가 계획과 일치
- [ ] 승인되지 않은 경우 변경이 동일한 commit-ready 단위로 설명됨
- [ ] 사용자 기존 변경이 섞이거나 되돌려지지 않음
- [ ] `COMPLETION_AUDIT.md`에 잔여 위험과 후속 Goal이 기록됨
- [ ] push는 명시적 권한이 있을 때만 수행됨

판정: `NOT_RUN`

## 2. Mandatory completion table

실행 종료 시 아래 표를 갱신한다.

| Gate | Status | Evidence summary |
|---|---|---|
| G0 Baseline | NOT_RUN | |
| G1 Classification | NOT_RUN | |
| G2 Root enforcement | NOT_RUN | |
| G3 Battle rejection atomicity | NOT_RUN | |
| G4 StartNewGame atomicity | NOT_RUN | |
| G5 Success parity | NOT_RUN | |
| G6 Retry conformance | NOT_RUN | |
| G7 Non-regression | NOT_RUN | |
| G8 Performance | NOT_RUN | |
| G9 Migration integrity | NOT_RUN | |

## 3. Completion rule

- G0–G9 중 하나라도 `FAIL`, `BLOCKED`, `NOT_RUN`이면 Goal은 완료가 아니다.
- scope 변경으로 gate가 불필요해졌다면 삭제하지 말고 사용자 승인과 근거를 남긴 뒤 `N/A-approved`로 표시한다.
- 테스트 통과만으로 architecture gate(G1, G2, G9)를 대신할 수 없다.

