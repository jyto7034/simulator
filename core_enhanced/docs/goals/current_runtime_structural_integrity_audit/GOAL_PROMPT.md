# Goal Execution Prompt

아래 내용을 현재 workspace에서 하나의 Goal로 끝까지 수행하라.

## Goal

- `goal_name`: `current_runtime_structural_integrity_audit`
- Goal 디렉터리: `docs/goals/current_runtime_structural_integrity_audit`
- 대상 runtime: `/mnt/f/work/simulator/core_enhanced`
- repository root: `/mnt/f/work/simulator`
- Status 시작값: Goal 문서의 최신 `Status`를 따른다.

현재 `core_enhanced` Rust runtime의 구조적 건전성을 현재 dirty 작업 트리에서 read-only로 감사하라.
Production code를 고치는 것이 아니라 상태 소유권, 불변조건, 원자성, 결정론, 외부 경계, data loading,
persistence, 오류, test 신뢰도, 성능 상한과 AI 생성 누적 부채를 현재 코드에서 독립적으로 증명하고
분류하는 것이 목표다.

## 판정 Source of Truth

다음 문서를 판정 기준과 증거 계약의 source of truth로 사용하라.

- `docs/audit/current_runtime_structural_integrity_audit_criteria.ko.md`

이 문서의 C00~C13, 판정 상태, 심각도, 확신도, 발견사항 증거 계약과 완료 조건을 완화하거나 대체하지
마라.

현재 production Rust 코드는 현재 동작을 확인하는 source of truth다. 코드가 존재한다는 사실을 구조적
정당성으로 간주하지는 마라.

## 시작 절차

이 Goal은 이미 초기화된 기존 Goal이다. Repository-wide code search, runtime 조사, test, formatter 또는
production 파일 접근보다 먼저 다음 문서를 처음부터 끝까지 읽어라.

1. `docs/audit/current_runtime_structural_integrity_audit_criteria.ko.md`
2. `docs/goals/current_runtime_structural_integrity_audit/PLAN.md`
3. `docs/goals/current_runtime_structural_integrity_audit/EXPERIMENTS.md`
4. `docs/goals/current_runtime_structural_integrity_audit/EXPERIMENT_NOTES.md`
5. `docs/goals/current_runtime_structural_integrity_audit/FINDINGS.md`

필수 Goal 문서가 누락되었거나 끝까지 읽을 수 없다면 내용을 추측해 복원하지 말고 안전하게 재개할 수
없는 이유를 보고하라.

문서를 읽은 뒤 최신 Status와 마지막 기록 이후부터 이어서 수행하라. 이미 충분한 증거가 기록된 작업을
의미 없이 반복하지 마라.

## 범위

기본 감사 범위:

- `src/lib.rs`
- `src/game/**`
- `Cargo.toml`의 runtime dependency와 feature
- runtime 증거 확인에 필요한 `tests/**` 및 `src/**` 내부 테스트
- data deserialize, validation, compile, publication 경계
- save, checkpoint, restore와 reconnect 경계
- command, event, snapshot, DTO serialization 경계
- RNG, UUID, time, filesystem과 physics 경계

`game_resources`와 `game_server`는 core runtime의 직접 producer, consumer 또는 persistence boundary를
이해하는 데 반드시 필요하다는 근거가 있을 때만 read-only로 조사하라. 필요성과 확인 범위를 조사 전에
`EXPERIMENT_NOTES.md`에 기록하라.

## 명시적 비범위

- Production code, test, fixture, schema, live data와 canonical 계약 문서 수정
- 리팩터링과 finding remediation
- 현재 rulebook과 runtime의 정책 정합성
- 밸런스, 콘텐츠 완성도와 재미 평가
- 오래된 Unity 구현과 현재 protocol의 호환성 검증
- Unity 기능 감사와 game_server 전체 구조 감사
- 과거 `docs/goals/**` 또는 과거 감사 결과 재사용
- 새 Goal 또는 Master Goal 자동 생성
- 승인되지 않은 compatibility, migration 또는 gameplay 정책 결정

Rulebook은 게임 용어와 대략적인 흐름 파악에만 사용할 수 있다. Rulebook과 코드가 다르다는 이유만으로
finding을 만들지 마라. 과거 Goal의 finding은 현재 finding으로 가져오지 마라.

## Dirty Worktree와 기준선

기존 변경과 untracked 파일은 모두 사용자 소유다. 다음을 하지 마라.

- reset, restore, checkout, clean 또는 임의 stash
- 관련 없는 파일 수정, 포맷, 이동 또는 삭제
- 기존 변경을 감사 변경으로 표현
- 서로 다른 기준선의 증거 혼합

Runtime code를 읽기 시작하기 전에 현재 dirty 기준선을 재현 가능하게 식별하라.

최소 기록:

- 감사 시작 시각과 timezone
- `git rev-parse HEAD`
- `git status --short`
- 감사 대상 runtime file의 content manifest 또는 동등한 식별자
- `rustc --version`
- `cargo --version`
- 적용 feature와 실행 환경

Commit ID만으로 dirty tree를 식별했다고 표현하지 마라. 기준선 결과를 확인한 즉시 다음 조사 전에 Goal
문서에 기록하라.

감사 도중 audited production file, live data, dependency 또는 실행 환경이 바뀌면 즉시 기록하고 조사를
일시 중단하라. 기존 기준선에서 끝낼지 새 기준선으로 재시작할지 사용자 결정이 필요하다.

## 최초 전체 Test Baseline

기준선 식별 직후, runtime code 감사를 시작하기 전에 다음 명령을 정확히 한 번 실행하라.

```bash
cargo test --all-targets --all-features
```

실행 위치:

```text
/mnt/f/work/simulator/core_enhanced
```

명령이 끝나면 다른 조사를 시작하기 전에 `EXPERIMENTS.md`에 다음을 기록하라.

- 정확한 명령과 실행 위치
- 실제 실행된 test binary
- runner가 실제 보고한 test 수
- pass/fail/skip 수
- compile 또는 runner 오류
- baseline 판정

Runner가 전체 합계를 제공하지 않으면 추측하지 마라. Binary별 출력에서 안전하게 합산 가능한 경우만
계산하고 계산 근거를 남겨라.

### All-pass 이후 Test 금지 원칙

최초 suite가 all-pass이고 production code, live data, test, dependency와 실행 환경이 바뀌지 않았다면 다음을
수행하지 마라.

- 같은 full suite 재실행
- criterion마다 기존 test 반복
- 단순 확인을 위한 focused test
- 감사 문서만 바꾼 뒤 종료 regression test

전체 test 통과는 C00의 baseline 증거일 뿐 C01~C13의 합격 증거가 아니다. All-pass를 이유로 코드 감사를
축소하거나 종료하지 마라.

추가 test 또는 runtime probe는 정적 증거만으로 특정 finding을 증명하거나 반증할 수 없을 때만 허용한다.
실행 전에 다음을 `EXPERIMENT_NOTES.md`에 기록하라.

- 확인할 구체적인 가설
- 최초 baseline으로 충분하지 않은 이유
- 새 실행이 제공할 중복되지 않는 증거

실행 직후 정확한 명령과 결과를 `EXPERIMENTS.md`에 기록하라. Production test 파일은 수정하지 마라.

## 즉시 기록 규율

발견사항과 증거를 여러 파일이나 subsystem을 조사한 뒤 한꺼번에 취합하지 마라. 기억과 요약 과정에서
정보가 오염되지 않도록 하나의 bounded investigation마다 다음 순서를 지켜라.

1. 조사할 criterion, 코드 표면 또는 call path와 가설을 정한다.
2. 하나의 bounded call path 또는 작은 구조 표면만 조사한다.
3. 다음 표면으로 이동하기 전에 raw observation과 실제 결과를 문서에 기록한다.
4. authority 또는 surface 정보가 생기면 해당 inventory 문서를 즉시 갱신한다.
5. 결함 가능성이 보이면 확정까지 기다리지 말고 `FINDINGS.md`에 새 ID와 `Hypothesis`로 즉시 등록한다.
6. 그 후에만 다음 조사로 이동한다.

기록 위치:

- 명령, test, probe와 실제 결과: `EXPERIMENTS.md`
- 구조적 판단, source-of-truth, 접근 범위, 대안, 생략 근거와 후속 후보: `EXPERIMENT_NOTES.md`
- candidate, active, superseded, rejected finding: `FINDINGS.md`
- runtime surface와 entry point: `SURFACE_INVENTORY.md`
- state owner와 mutation entry: `STATE_AUTHORITY_MAP.md`
- criterion별 증거와 판정: `EVIDENCE_MATRIX.md`
- 미확인 영역과 후속 조사 방법: `GAP_REGISTER.md`

추가 산출물은 첫 관련 증거가 생길 때 생성하라. 마지막에 기억으로 처음부터 작성하지 마라.

Finding ID는 `RUNTIME-AUDIT-###` 형식을 사용하라. 후보가 반증되어도 삭제하지 말고 `Rejected` 상태와
반증 근거를 남겨라. Finding을 통합하거나 대체해도 기존 ID와 관계를 보존하라. 새 증거에 맞추기 위해
과거 실행 결과를 다시 쓰지 마라. 필요한 경우 정정 기록을 추가하라.

## 감사 수행

파일 순서대로 피상적으로 읽지 말고 production entry point에서 authority mutation과 외부 projection까지
이어지는 vertical call path를 우선하라.

다음 결과 범위를 모두 다뤄라.

1. C00 실행 기준선과 기본 건전성
2. C01 상태 소유권
3. C02 상태 전이와 불변조건
4. C03 원자성과 실패 안전성
5. C04 결정론, event ordering과 시간
6. C05 의존 방향과 변경 용이성
7. C06 외부 DTO와 향후 Unity adapter seam
8. C07 data loading, validation과 publication
9. C08 save, checkpoint와 restore
10. C09 production 실행 경로의 완결성
11. C10 오류, log, trace와 non-authoritative I/O
12. C11 기존 test의 신뢰도와 증거 범위
13. C12 성능 복잡도와 unbounded growth
14. C13 AI 생성 누적 부채와 중복 의미

큰 파일, 긴 함수, `clone`, `unwrap`, `panic!`, 많은 public field와 같은 항목은 조사 신호로만 사용하라.
실제 production 호출 경로, 깨지는 불변조건, 실패 결과 또는 변경 증폭 없이 finding으로 확정하지 마라.

각 criterion을 `PASS`, `PARTIAL`, `FAIL`, `UNKNOWN`, `N/A` 중 하나로 판정하라. 문제를 발견하지 못했다는
이유만으로 `PASS`를 주지 마라. 조사 범위와 합격 증거가 부족하면 `PARTIAL` 또는 `UNKNOWN`이다.

## Finding 증거

모든 최종 `FAIL` finding은 감사 기준 문서의 증거 계약을 만족해야 한다.

최소한 다음을 제시하라.

- 현재 dirty 기준선
- 관련 criterion ID
- 심각도와 확신도
- 영향받는 불변조건
- 파일과 코드 위치
- production entry부터 문제 지점까지의 호출 경로
- raw observation과 현재 동작
- 상태 또는 외부 결과에 미치는 영향
- 반대 증거와 남은 불확실성
- 근본 원인과 연관 finding
- 별도 remediation 방향과 필요한 회귀 테스트

동적 재현이 필요하지 않은 구조 finding은 값을 쓰고 읽는 모든 authority 경로와 compiler, visibility 또는
validation이 문제를 막지 못한다는 정적 증거를 제시하라.

심각도는 스타일이 아니라 영향, 도달 가능성, 범위, 복구성과 탐지 가능성으로 정하라. `Hypothesis`를 확정
finding 수에 포함하지 마라.

## 범위 밖 문제

현재 Goal 밖의 독립적인 문제를 발견해도 새 Goal이나 Master Goal을 생성하거나 구현하지 마라.

다음을 `EXPERIMENT_NOTES.md`의 범위 밖 후속 후보에 즉시 기록하라.

- 관찰 사실
- 현재 Goal과 독립적인 이유
- 예상되는 사용자 영향
- 권장 조사 또는 실행 순서

현재 Goal의 증거 수집에 직접 필요하지 않으면 더 깊게 조사하지 마라.

## Status와 중단 조건

사용할 Status:

- `active`
- `scope_decision_required`
- `policy_decision_required`
- `externally_blocked`
- `complete`

여러 독립 감사 범위를 안전하게 하나의 catalog로 취합할 수 없거나 core 밖 전체 감사가 필수가 되면
`scope_decision_required`로 변경하라.

정책 없이는 구조 결함인지 구분할 수 없고 해당 항목을 `UNKNOWN`이나 비범위로 격리할 수 없다면
`policy_decision_required`로 변경하라.

필수 source, dependency, 권한, runner 또는 재현 가능한 dirty baseline이 없어 완료할 수 없다면 안전한
대안을 먼저 확인한 뒤 `externally_blocked`로 변경하라.

Status를 바꾸면 근거, 현재까지 완료한 안전한 조사, 남은 범위와 필요한 사용자 입력을 Goal 문서에 즉시
기록하고 완료라고 표현하지 마라.

## 필수 산출물

감사 완료 시 최소한 다음 문서가 현재 증거와 일치해야 한다.

- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `FINDINGS.md`
- `SURFACE_INVENTORY.md`
- `STATE_AUTHORITY_MAP.md`
- `EVIDENCE_MATRIX.md`
- `GAP_REGISTER.md`
- `COMPLETION_AUDIT.md`

최종 취합 단계에서는 이미 기록된 finding을 root cause에 연결하고 criterion 판정을 정리하라. 이 단계에서
이전 조사에 없던 raw observation, 명령 결과 또는 finding을 기억으로 새로 만들지 마라. 새 사실을 발견하면
해당 조사 단계로 돌아가 즉시 기록 절차를 적용하라.

## 완료 조건

`PLAN.md`와 감사 기준 문서의 완료 조건을 모두 실제 evidence ID와 산출물에 연결한 뒤에만 Status를
`complete`로 변경하라.

다음을 명확히 구분하라.

```text
Goal Status: 감사 수행과 증거 기록의 완료 여부
Runtime Assessment: 현재 runtime의 구조적 합격 여부
```

S0 또는 S1 finding이 남아 있어도 승인된 감사 범위와 증거 계약이 모두 완결되면 Goal Status는
`complete`일 수 있다. 반대로 finding이 없어도 조사 표면이나 증거가 부족하면 complete가 아니다.

완료 전에 `COMPLETION_AUDIT.md`에 다음을 포함하라.

- 기준선과 최초 full-suite 결과
- C00~C13의 최종 판정과 evidence 연결
- 확정 finding, candidate와 rejected candidate
- root-cause 통합 결과
- 모든 `UNKNOWN`과 접근 불가 범위
- 생략한 검증과 근거
- 범위 밖 후속 후보
- dirty worktree 보존 확인
- Goal Status와 Runtime Assessment의 분리된 결론

## 최종 보고

사용자에게 다음을 보고하라.

- 최종 Goal Status
- 최종 Runtime Assessment
- 감사 기준선
- 최초 전체 test 명령과 실제 pass/fail/skip 수
- C00~C13 판정 요약
- 심각도·확신도별 finding 수와 핵심 root cause
- `UNKNOWN`과 접근하지 못한 영역
- 생략했거나 추가로 실행한 검증과 이유
- 범위 밖 후속 후보
- production 파일을 수정하지 않았다는 확인
- 기존 dirty worktree 보존 결과
- 생성·갱신한 감사 산출물
- 사용자 결정이 필요한 미완료 항목

Status가 `scope_decision_required`, `policy_decision_required` 또는 `externally_blocked`이면 완료라고 표현하지
말고 정확한 blocker와 필요한 사용자 입력을 먼저 제시하라.

계획만 제시하고 멈추지 마라. 유효한 중단 조건이 발생하지 않는 한 Goal의 감사와 문서화를 실제로 끝까지
수행하라.

