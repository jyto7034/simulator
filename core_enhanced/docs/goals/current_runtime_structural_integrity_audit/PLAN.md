# Current Runtime Structural Integrity Audit

## Status

`complete`

감사 조사, `COMPLETION_AUDIT.md`, 최종 기준선 재검증(E-022)과 필수 산출물 교차검사를 완료했다. Goal
Status는 `complete`이며 Runtime Assessment는 별도로 `FAIL`이다.

## Goal Name

`current_runtime_structural_integrity_audit`

## 목표

현재 `core_enhanced` Rust 런타임의 구조적 건전성을 현재 작업 트리에서 독립적으로 감사한다.

감사의 최종 질문은 다음과 같다.

> 현재 런타임은 정책이 변경되더라도 안전하게 이해하고 수정할 수 있으며, 동일한 입력에 안정적인
> 결과를 내고, 실패 시 권위 상태를 보존하며, 향후 Unity 어댑터를 붙일 수 있는 구조인가?

## 해결할 사용자-visible 문제

프로젝트 코드가 대부분 AI로 작성되어 있어 사용자가 현재 런타임의 실제 구조, 신뢰 가능한 부분,
누적된 위험과 수정 우선순위를 알 수 없다.

이 Goal은 이를 구현 변경으로 즉시 고치는 Goal이 아니다. 현재 production runtime의 상태를 재현 가능한
증거로 분류하여 사용자가 후속 remediation과 정책 구현 순서를 결정할 수 있게 한다.

## 감사 Finding ID

없음. 이 Goal 자체가 최초 구조 감사이며 기존 finding에서 유래하지 않았다.

## 감사 기준 Source of Truth

판정 기준과 증거 계약의 source of truth는 다음 문서다.

- `docs/audit/current_runtime_structural_integrity_audit_criteria.ko.md`

이 Goal은 위 문서의 C00~C13, 판정 상태, 심각도, 확신도, 증거 계약과 완료 조건을 완화하지 않는다.

현재 production Rust 코드는 “지금 실제로 무엇이 구현되어 있는가”의 source of truth다. 현재 rulebook,
과거 Goal, 과거 감사 완료 선언은 “구조적으로 올바른가”의 source of truth가 아니다.

## 승인된 감사 정책과 근거

다음 사항은 2026-08-21 사용자 요청으로 승인되었다.

- 과거 `docs/goals/**`는 감사 근거로 사용하지 않는다.
- 현재 rulebook은 게임의 대략적인 용어와 흐름 파악에만 사용할 수 있다.
- 현재 rulebook과 코드의 정책 정합성은 감사하지 않는다.
- 오래된 Unity 구현의 현재 호환성은 감사하지 않는다.
- 현재 Rust runtime code를 중심으로 조사한다.
- 발견사항, 실험 결과, 구조적 판단과 gap은 발견 직후 문서에 기록한다.
- 여러 조사 결과를 기억에 의존해 마지막에 한꺼번에 복원하지 않는다.
- 감사 시작 시 전체 test baseline을 한 번 실행한다.
- baseline이 all-pass이고 production code가 변경되지 않았다면 감사 중 불필요한 test를 반복하지 않는다.

## 현재 Runtime 및 Live Data 근거

현재 기준선은 B-001(HEAD + 210-file aggregate)이며 E-022에서 exact match로 재검증되었다. E-002 최초 broad
suite는 1,131 pass/0 fail/0 ignored, E-004 formatter는 pass, E-005 clippy `-D warnings`는 fail이다.

Runtime entry/state/transaction은 S-003~S-006, A-001~A-005, T-001~T-003에, live publication과 정확한 embedded
bytes는 P-001~P-004/E-013/E-014에, persistence는 PS-001~PS-004에, test 위험 연결은
`TEST_TRACEABILITY.md`에 기록했다.

Live data는 C07 데이터 로딩·검증 경계를 감사하는 데 필요한 범위에서만 읽는다. 콘텐츠 정책이나 밸런스
정합성을 판정하지 않는다.

## Source of Truth와 소유 계층

다음 항목은 현재 코드에서 조사 완료되었다.

- root command/host/admin entry and projection: E-006/E-015/E-016/E-020, S-003/S-005
- run/world/active battle and standalone BattleCore authority: A-001~A-005, E-006/E-008
- event order/RNG/time/physics: E-008/E-017, `DETERMINISM_INVENTORY.md`
- data parse/validate/compile/publication: E-010/E-013/E-014/E-019, `DATA_PUBLICATION_MAP.md`
- durable vs temporary/checkpoint state: E-011/E-012, `PERSISTENCE_MAP.md`
- external DTO/adapter seam: E-015/E-016 and RUNTIME-AUDIT-008

Final criterion verdicts are one PASS, one PARTIAL and twelve FAIL with no UNKNOWN/N/A (`EVIDENCE_MATRIX.md`).

## 유지할 불변식

### 감사 수행 불변식

- 기존 사용자 변경을 수정, 삭제, 되돌리거나 포맷하지 않는다.
- production code, live data, schema, DTO, test, fixture, canonical 정책 문서를 수정하지 않는다.
- 현재 기준선의 코드와 감사자가 만든 문서 변경을 구분한다.
- 문서, 주석, 테스트 이름만으로 `PASS` 또는 `FAIL`을 판정하지 않는다.
- 큰 파일, `clone`, `unwrap`, `panic!`, public field 같은 신호를 증거 없이 결함으로 승격하지 않는다.
- 정책에 의존하는 gameplay 의미를 임의로 결정하지 않는다.
- 테스트 전체 통과를 C01~C13의 자동 합격으로 사용하지 않는다.
- 감사 finding이 있어도 현재 Goal 안에서 수정하지 않는다.
- 범위 밖 문제를 새 Goal이나 Master Goal로 자동 생성하거나 구현하지 않는다.

### 증거 기록 불변식

- 명령 또는 probe 결과는 다음 조사로 넘어가기 전에 `EXPERIMENTS.md`에 기록한다.
- 구조적 판단, authority map 변화, 정책 의존성, 접근 불가와 생략 근거는 다음 조사로 넘어가기 전에
  `EXPERIMENT_NOTES.md`에 기록한다.
- 결함 가능성을 처음 식별하면 확정될 때까지 기다리지 않고 `FINDINGS.md`에 `Hypothesis`로 즉시 등록한다.
- finding이 강화되면 기존 항목을 `Strong inference` 또는 `Confirmed`로 승격하고 새 증거를 추가한다.
- 반증된 후보는 삭제하지 않고 `Rejected`로 이동하여 반증 근거를 남긴다.
- 뒤늦게 수정한 기록은 원래 관찰과 정정 이유를 모두 보존한다.
- test 수와 pass/fail/skip 수는 runner가 실제 보고한 값만 기록한다.

## 범위

### 포함 범위

- `core_enhanced/src/lib.rs`
- `core_enhanced/src/game/**`
- `core_enhanced/Cargo.toml`의 runtime dependency와 feature 구성
- runtime 구조를 증명하거나 반증하는 `core_enhanced/tests/**`와 `src/**` 내부 테스트
- RON loading과 validation 경계를 판단하는 데 필요한 loader와 data type
- runtime의 command, event, snapshot, DTO serialization 경계
- runtime이 직접 사용하는 RNG, UUID, time, filesystem, physics 경계
- current dirty tree에서 위 범위에 이미 존재하는 사용자 변경

### 조사 전용 인접 범위

다음은 core runtime의 producer 또는 persistence boundary를 이해하는 데 반드시 필요한 경우에만 read-only로
확인할 수 있다.

- `game_resources`의 production live-data 사용 여부와 실제 loader entry
- `game_server`가 core DTO를 직접 생산·변형하는지 여부

인접 범위를 확인하더라도 server나 Unity의 기능 완결성 또는 기존 wire compatibility를 감사하지 않는다.
필요성이 증명되지 않으면 접근하지 않고 생략 근거를 기록한다.

## 명시적 비범위

- 모든 production 수정과 리팩터링
- 최신 정책과 runtime 구현의 의미 정합성
- 밸런스, 콘텐츠 품질, 재미 평가
- 과거 `docs/goals/**`와 과거 감사의 재사용 또는 재검증
- 과거 Goal finding을 현재 finding으로 가져오는 작업
- 오래된 Unity reader, store, command, presentation 구현 감사
- 기존 Unity와 현재 wire protocol의 호환성 인증
- game_server 전체 구조 감사
- 정책 문서 또는 canonical runtime 계약 갱신
- finding별 remediation 계획의 상세 설계와 구현
- 새 Goal 또는 Master Goal 자동 생성

## 이 Goal의 단일 책임

이 Goal의 단일 결과는 “현재 Rust runtime의 구조적 위험에 대한 증거 기반 감사 catalog”다.

여러 runtime source-of-truth 경계를 조사하지만 서로 다른 구현을 하나의 변경으로 묶지 않는다. 이 Goal은
read-only 조사와 분류만 수행한다. 각 경계의 수정은 사용자가 별도로 승인한 후속 Goal로 분리한다.

감사 자체를 현재 범위에서 완결할 수 없을 정도로 독립적인 조사 영역이 새로 발견되면 근거를 기록하고
`scope_decision_required`로 전환한다. 새 Goal을 자동 생성하지 않는다.

## 미결정 정책

현재 구조 감사 완료에 필요한 gameplay 정책 결정은 없다.

정책 없이는 판정할 수 없는 항목은 다음처럼 처리한다.

- 구조적 사실까지는 조사한다.
- gameplay 의미의 정오 판정은 하지 않는다.
- 해당 세부 항목을 `UNKNOWN`, `N/A` 또는 후속 Policy Alignment 후보로 기록한다.

정책 선택이 필요한 production 변경은 이 Goal의 비범위다.

## 구현 범위

Production 구현 없음.

허용되는 변경은 이 Goal의 감사 문서와 감사 기준을 참조하는 read-only 산출물뿐이다.

예정 산출물:

- `GOAL_PROMPT.md`
- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `FINDINGS.md`
- `SURFACE_INVENTORY.md`
- `STATE_AUTHORITY_MAP.md`
- `EVIDENCE_MATRIX.md`
- `GAP_REGISTER.md`
- `COMPLETION_AUDIT.md`

필요한 문서는 처음 관련 증거가 생길 때 생성하고 즉시 갱신한다. 마지막 취합 단계에서 기억에 의존해
처음부터 복원하지 않는다.

## DTO Migration 여부

없음. DTO migration은 비범위다.

## Data 또는 Live RON Migration 여부

없음. Data와 live RON은 loading·validation 증거 확인을 위해서만 읽는다.

## Save Migration 여부

없음. Save format과 restore 경계는 감사하지만 변경하지 않는다.

## Baseline 계획

### Baseline 식별

감사 코드 조사를 시작하기 전에 다음을 기록한다.

- 감사 시작 시각과 timezone
- `git rev-parse HEAD`
- `git status --short`
- 감사 대상 runtime 파일의 content manifest 또는 동등한 dirty-tree 식별자
- `rustc --version`
- `cargo --version`
- 실제 사용한 feature 조합

현재 repository root는 `/mnt/f/work/simulator`이고 Goal 초기화 시점부터 대규모 dirty 상태다. commit ID만으로
기준선을 식별하지 않는다.

### 최초 전체 Test Sweep

Goal 문서 초기화 후 runtime code 조사 전에 다음 baseline을 정확히 한 번 실행한다.

```bash
cargo test --all-targets --all-features
```

실행 위치는 `/mnt/f/work/simulator/core_enhanced`다.

Runner가 보고한 test binary별 수치와 전체 pass/fail/skip 수를 `EXPERIMENTS.md`에 즉시 기록한다. 출력이
전체 합계를 제공하지 않으면 실제 출력에 근거해 안전하게 합산 가능한 경우만 합산하고, 그렇지 않으면
runner가 전체 합계를 보고하지 않았다고 기록한다.

### Baseline 이후 Test 원칙

최초 전체 sweep이 all-pass이고 production code, live data, test, dependency 또는 실행 환경이 변경되지
않았다면 다음을 수행하지 않는다.

- 같은 전체 test command 반복
- 단순 안심을 위한 focused test 반복
- 이미 baseline이 포함한 테스트를 criterion별로 재실행
- 감사 문서만 변경한 뒤 최종 regression test 재실행

다음 경우에만 추가 test 또는 runtime probe를 허용한다.

- 정적 호출 경로만으로 finding을 증명하거나 반증할 수 없음
- baseline test가 실제로 다루는 계약인지 확인하기 위해 개별 test 관측이 필요함
- 감사 도중 외부에서 baseline 대상 파일이나 실행 환경이 변경됨
- 최초 baseline이 실패하거나 결과 수가 불명확함

추가 실행 전 `EXPERIMENT_NOTES.md`에 필요성, 기존 baseline으로 충분하지 않은 이유와 예상 증거를 먼저
기록한다. 실행 직후 정확한 명령과 결과를 `EXPERIMENTS.md`에 기록한다.

추가 test는 기존 suite의 재확인이 아니라 특정 감사 가설을 확인하는 최소 probe여야 한다. Production
test 파일은 수정하지 않는다.

## 필수 Focused 검증

이 Goal은 구현 Goal이 아니므로 모든 criterion마다 test를 새로 실행하지 않는다.

Focused 검증은 다음에 한정한다.

- finding을 `Confirmed`로 승격하는 데 동적 재현이 반드시 필요한 경우
- 원자성, 결정론, persistence 행동 동등성을 정적 증명만으로 확정할 수 없는 경우
- 실제 production entry point 도달 가능성을 확인해야 하는 경우

가능하면 기존 테스트 코드와 production call path를 읽어 증거를 확보한다. 새 test 실행이 baseline과
중복되면 생략한다.

## 필수 Broad 검증

- 감사 시작 전 `cargo test --all-targets --all-features` 1회
- 기준선, 실행 환경과 실제 결과 기록

감사 중 production 파일을 변경하지 않으므로 all-pass baseline이 유지되는 동안 별도 종료 test는 없다.

## 생략할 검증과 근거

- Unity test: Unity 구현은 장기간 갱신되지 않았고 현재 호환성은 비범위다.
- game_server 전체 test: server 구조와 transport 구현은 기본 감사 범위가 아니다.
- 정책 alignment test: 현재 rulebook과 runtime 정책이 의도적으로 불일치한다.
- remediation regression test: 이 Goal은 production 코드를 수정하지 않는다.
- 반복 full-suite test: all-pass baseline 이후 코드가 변하지 않으면 새 정보를 제공하지 않는다.

## 감사 단계

구체적인 조사 순서는 runtime 구조와 비용에 따라 조정할 수 있다. 단, 아래 결과 범위는 모두 다룬다.

1. 기준선과 최초 전체 test sweep
2. runtime surface와 production entry point inventory
3. state authority와 public mutation entry point mapping
4. 상태 전이, 불변조건, 원자성과 실패 경계 감사
5. 결정론, 시간, event ordering, physics와 identity 감사
6. dependency direction, change amplification과 adapter seam 감사
7. data loading, validation과 publication 경계 감사
8. save, checkpoint, restore와 temporary-state 경계 감사
9. 실행 경로 완결성, error, log와 non-authoritative I/O 감사
10. 기존 test의 증거 신뢰도, 성능 상한과 AI 생성 누적 부채 감사
11. finding root-cause 통합과 gap register 정리
12. criterion별 판정과 완료 감사

각 단계는 여러 파일을 읽은 뒤 몰아서 기록하지 않는다. 의미 있는 관찰 하나 또는 하나의 bounded call path
조사가 끝날 때마다 관련 문서를 먼저 갱신한다.

## 즉시 기록 프로토콜

조사 단위마다 다음 순서를 지킨다.

1. 조사 대상과 가설을 정한다.
2. 하나의 bounded call path 또는 코드 표면을 읽거나 probe한다.
3. 다음 코드 표면으로 이동하기 전에 실제 결과를 문서화한다.
4. finding 후보가 있으면 `FINDINGS.md`에 즉시 ID를 부여한다.
5. authority·surface inventory가 바뀌면 해당 inventory 문서를 즉시 갱신한다.
6. 그 후 다음 조사로 이동한다.

금지 사항:

- 여러 날 또는 여러 subsystem의 발견을 기억에 의존해 마지막에 취합
- 확정되지 않았다는 이유로 finding 후보를 문서에 남기지 않음
- 반증된 후보를 기록 없이 삭제
- 새 증거에 맞추기 위해 과거 실험 결과를 성공적으로 보이게 다시 작성
- 실제로 실행하지 않은 명령을 계획과 결과 사이에서 혼동

각 기록은 가능한 한 시각, 기준선, criterion ID, 코드 위치와 다음 판단을 포함한다.

## 사용자-visible 완료 조건

다음을 모두 충족해야 한다.

- 현재 dirty runtime 기준선이 재현 가능하게 식별되어 있다.
- 최초 전체 test sweep의 정확한 명령과 실제 결과가 기록되어 있다.
- C00~C13 각각이 `PASS`, `PARTIAL`, `FAIL`, `UNKNOWN`, `N/A` 중 하나로 판정되어 있다.
- production runtime의 주요 entry point와 state owner가 목록화되어 있다.
- 주요 mutation 경로의 검증, mutation, event, projection과 실패 순서가 조사되어 있다.
- 결정론 입력, event ordering, UUID, collection iteration, time과 physics 경계가 조사되어 있다.
- data publication, persistence·restore와 external DTO seam이 조사되어 있다.
- 기존 test가 어떤 위험을 증명하고 무엇을 증명하지 않는지 연결되어 있다.
- 모든 `FAIL` finding이 감사 기준 문서의 증거 계약을 만족한다.
- 모든 `UNKNOWN`과 접근 불가 영역이 이유와 후속 조사 방법을 가진다.
- 후보와 반증된 항목을 삭제하지 않고 판정 이력이 남아 있다.
- 발견사항이 조사 직후 기록되었음을 실행 기록에서 추적할 수 있다.
- 증상 중복이 root cause 단위로 통합되어 있다.
- 범위 밖 remediation 후보가 구현되지 않고 별도로 구분되어 있다.
- `COMPLETION_AUDIT.md`가 감사 완료와 runtime 합격을 구분한다.

S0 또는 S1 finding이 남아 있어도 감사 자체는 `complete`가 될 수 있다. `complete`는 런타임 합격이 아니라
승인된 범위의 감사와 증거 기록이 완료되었다는 뜻이다.

## 중단 조건

다음 경우 현재 Status를 변경하고 선택에 따라 달라지는 조사를 중단한다.

### `scope_decision_required`

- 현재 단일 감사 결과로 안전하게 취합할 수 없는 독립 감사 범위가 발견됨
- core runtime 범위를 넘어 server 또는 Unity 전체 감사가 완료에 필수로 드러남
- 사용자가 승인하지 않은 정책 alignment 감사 없이는 범위를 정의할 수 없음

### `policy_decision_required`

- 구조적 결함인지 gameplay 정책 차이인지 분리할 수 없고 그 항목이 전체 감사 완료에 필수임

정책 의존 항목을 `UNKNOWN` 또는 비범위로 격리할 수 있다면 전체 Goal을 중단하지 않는다.

### `externally_blocked`

- 필수 runtime source, dependency, test runner 또는 권한에 접근할 수 없음
- 현재 dirty baseline을 재현 가능하게 식별할 수 없음
- 필수 전체 test baseline을 실행하지 못하고 대체 증거도 없음

### 기준선 변경

감사 도중 사용자가 audited production 파일, live data, dependency 또는 실행 환경을 변경하면 조사를
일시 중단한다. 기존 기준선에서 마칠지 새 기준선으로 재시작할지 사용자에게 확인한다. 서로 다른 기준선의
증거를 조용히 섞지 않는다.

## Compatibility

Compatibility 구현과 migration은 비범위다. 기존 Unity, server, payload나 save version에 대한 compatibility
정책을 임의로 만들지 않는다.

현재 core boundary가 versionable한지 감사할 수는 있지만 구체적인 dual-read, fallback, alias 또는 migration
설계를 승인하거나 구현하지 않는다.

## Canonical 문서

이 Goal은 gameplay 또는 runtime canonical 계약을 갱신하지 않는다.

- 감사 판정 기준: `docs/audit/current_runtime_structural_integrity_audit_criteria.ko.md`
- 현재 동작 증거: 현재 production Rust runtime
- 실행 history: 이 Goal 디렉터리의 문서

Rulebook과 기존 canonical 문서는 용어 탐색 외에 합격 근거로 사용하지 않는다. 감사 finding을 고치는 후속
Goal에서 승인된 정책과 실제 구현이 안정된 뒤 해당 canonical 문서를 갱신한다.

## 완료 판정

완료 전에 `COMPLETION_AUDIT.md`에서 사용자-visible 완료 조건 각각을 실제 산출물과 evidence ID에 연결한다.

다음은 완료 증거가 아니다.

- 전체 test가 통과했다는 사실만으로 구조 전체를 합격 처리
- 파일 크기, lint 수 또는 `unwrap` 수만으로 결함 처리
- 과거 Goal의 finding이나 완료 선언 재사용
- 문서에 적힌 의도를 현재 실행 동작처럼 표현
- 접근하지 않은 Unity 또는 server consumer를 검증했다고 표현
- finding 후보를 삭제하여 gap이 없는 것처럼 보이는 최종 catalog

Status가 `complete`여도 최종 Runtime Assessment는 `PASS`, `PARTIAL`, `FAIL` 또는 `UNKNOWN`일 수 있다.
