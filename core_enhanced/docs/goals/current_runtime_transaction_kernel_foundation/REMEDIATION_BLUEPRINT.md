# Current Runtime Remediation Blueprint

- 상태: `inactive-premature-draft`
- 작성일: 2026-08-22
- 대상: `core_enhanced`
- 역할: 두 감사 결과를 수정 가능한 하나의 의사결정 체계로 통합하는 상위 청사진
- 첫 실행 Goal: `current_runtime_transaction_kernel_foundation`

## 1. 결론

현재 런타임은 전면 재작성하지 않는다.

도메인 엔진과 이미 검증된 자산은 유지하고, 결함이 집중된 application/state/transaction 경계를 단계적으로 교체한다. 첫 단계는 새 기능 개발이나 crate 재편이 아니라, 실패할 수 있는 명령이 권위 상태를 부분 반영하지 못하도록 root-enforced transaction 경계를 만드는 것이다.

이 문서는 감사 원문을 대체하지 않는다. 감사 원문은 증거와 재현 기록으로 보존하고, 이 문서가 감사 결과를 실제 수정 단위와 순서로 변환한다.

## 2. 입력 감사와 권위 순서

### 2.1 입력 감사

1. `docs/goals/current_runtime_structural_integrity_audit/`
2. `docs/goals/current_runtime_architecture_deep_review/`

### 2.2 수정 판단의 권위 순서

충돌이 있을 때 다음 순서를 사용한다.

1. 최신 `docs/game_rulebook.md` 정책
2. 사용자가 이번 Goal에 명시한 결정
3. 이 `REMEDIATION_BLUEPRINT.md`
4. 두 감사의 최종 finding, rewrite decision, preservation/migration map
5. 현재 production code와 테스트가 증명하는 실제 동작
6. stale contract 및 과거 설계 문서

`core_runtime_contract.md`와 Unity contract를 포함한 stale 문서는 현재 정책의 증거가 아니라 비교 자료다. 관련 판단에 필요한 구간만 읽고, 최신 rulebook이나 실제 동작과 충돌하면 충돌을 기록한 뒤 상위 권위를 따른다. 감사 문서 역시 매 작업마다 처음부터 끝까지 재독하지 않는다. 아래 crosswalk가 지정한 finding, 근거, 코드 경로를 우선 읽는다.

## 3. 보존·교체 결정

### 3.1 보존할 자산

- `GameCore` façade와 `PlayerBehavior` 명령 표면
- 전투 시뮬레이션, typed command/event, 결정론 규칙
- movement system과 map/facility 도메인
- skill/battle system과 Boss DSL
- reward/economy의 기존 staged transaction 패턴
- live data loading pipeline
- checkpoint와 determinism 검증 자산
- 기존 save/wire 동작과 대규모 회귀 테스트 자산

보존은 현 구조를 영구 동결한다는 뜻이 아니다. 외부 관찰 동작을 oracle로 삼아 내부 경계를 교체한다는 뜻이다.

### 3.2 단계적으로 교체할 경계

- world application orchestration과 mutation transaction 경계
- active battle session과 event/history 소유 경계
- domain과 protocol DTO/error의 의존 방향
- validated production data construction 경계
- 장기적으로 run phase/state의 typed transition 경계

### 3.3 전면 재작성 판단

전체 core의 big-bang rewrite는 기각한다.

근거는 다음과 같다.

- 결함은 전체 도메인 로직보다 application/state/transaction 경계에 집중되어 있다.
- 보존 가치가 높은 도메인 엔진과 테스트가 이미 존재한다.
- 전체 재작성은 정책 누락과 결정론 회귀 위험을 크게 만든다.
- 출시 전이고 일정 여유가 있으므로, compatibility oracle을 유지한 단계적 boundary rewrite가 더 높은 품질을 낼 수 있다.

단, 각 단계에서 구 경계를 계속 우회하는 코드가 생기거나 단일 권위 전환이 불가능하다는 증거가 나오면 해당 경계 단위의 재작성 범위는 확대할 수 있다. 이는 전체 core 재작성 승인으로 간주하지 않는다.

## 4. 감사 finding 통합 crosswalk

| 통합 단위 | Structural audit | Architecture review | 결정 | 단계 |
|---|---|---|---|---|
| `REM-TX-001` 명령별 transaction 분류와 root dispatcher | `RUNTIME-AUDIT-005`, `006`; `RC-02` | `ARCH-REVIEW-004`, `005`; `RC-A` | 모든 `ActionKind`에 mutation/authority 분류를 부여하고 수정 명령의 실행 경계를 root가 강제 | 현재 Goal |
| `REM-TX-002` live battle command 원자화 | `RUNTIME-AUDIT-005` | `ARCH-REVIEW-004` | pause/resume/speed/focus/activation 여섯 명령을 staged projection 후 publish로 전환 | 현재 Goal |
| `REM-TX-003` StartNewGame 원자화 | `RUNTIME-AUDIT-006` | `ARCH-REVIEW-005` | full-session 변경과 fallible RunStart trigger를 하나의 성공/실패 단위로 전환 | 현재 Goal |
| `REM-TX-004` rejection equality와 retry conformance | `RUNTIME-AUDIT-005`, `006` | `ARCH-REVIEW-004`, `005` | 실패 전후 권위 상태 동일성과 동일 요청 재시도 가능성을 테스트로 고정 | 현재 Goal |
| `REM-BATTLE-001` active battle/history 소유 경계 | `RUNTIME-AUDIT-003`, `004`, `009`; `RC-03` | `ARCH-REVIEW-002`, `003`; `RC-C` | battle authority, history/archive, projection을 분리 | 후속 Goal |
| `REM-DATA-001` validated production data | `RUNTIME-AUDIT-007`; `RC-01` | `ARCH-REVIEW-006`; `RC-B` | production loader만 만들 수 있는 validated type과 fixture builder 분리 | 후속 Goal |
| `REM-PROTOCOL-001` protocol DTO/error 경계 | `RUNTIME-AUDIT-008`; `RC-03` | `RC-B` | domain error와 wire schema 분리 및 version 정책 수립 | 후속 Goal |
| `REM-STATE-001` typed state transition | `RUNTIME-AUDIT-001`; `RC-01` | `ARCH-REVIEW-001`; `RC-A` | broad root mutation과 debug/test surface를 축소 | 후속 Goal |

`RUNTIME-AUDIT-002` shop overflow finding은 감사에서 기각되었으므로 수정 backlog에 넣지 않는다. 새 증거 없이 되살리지 않는다.

## 5. 목표 transaction architecture

### 5.1 실행 흐름

```text
transport context
  -> action descriptor / authority selection
  -> immutable preflight
  -> staged state에서 decide + validate
  -> staged state로 response/projection 생성
  -> PreparedChange
  -> infallible authority publish
  -> CommitReceipt / response
```

핵심은 "write 뒤에 실패하지 않는다"가 아니다. 성공 여부를 결정할 수 있는 모든 fallible 작업을 authority publish 전에 끝내는 것이다. publish 이후에 남는 작업은 infallible이어야 한다.

### 5.2 명령 transaction 분류

최종 이름은 구현 중 코드와 충돌하지 않는 범위에서 조정할 수 있으나 의미는 다음을 유지한다.

- `ReadOnly`: 권위 상태를 변경하지 않는 query
- `BattleMutation`: active battle authority와 그 projection을 함께 다루는 변경
- `RunMutation`: 비전투 run/session 권위 변경
- `ProfileMutation`: profile 영속 권위 변경
- `FullSessionMutation`: 여러 root와 trigger/projection을 하나로 전환하는 변경
- `HostOwned`: core가 직접 commit하지 않는 host recovery/continuation 작업

분류 enum을 추가하는 것만으로 Goal을 완료할 수 없다. dispatcher 또는 동등한 root 실행 경로가 해당 분류를 실제로 강제해야 한다.

### 5.3 transaction 불변식

1. 거부된 명령은 명시된 monotonic telemetry 예외를 제외하고 authority를 변경하지 않는다.
2. 실패 응답을 만들 수 있는 projection/DTO 계산은 publish 전에 완료한다.
3. 한 명령의 authority writer는 정확히 하나다.
4. 구 경로와 신 경로의 dual-write 또는 silent fallback을 허용하지 않는다.
5. 동일 초기 상태와 동일 입력은 동일 결과와 동일 state delta를 만든다.
6. 실패 후 같은 요청을 재시도할 수 있다.
7. transport serialization 실패를 gameplay rollback 대상으로 잘못 포함하지 않는다. 다만 serialization 이전 response construction이 gameplay 상태에 의존해 실패할 수 있다면 publish 전에 수행한다.
8. 성능은 같은 runner/fixture 기준 현재 baseline보다 나빠지지 않아야 한다. 불가피한 비용은 측정 결과와 후속 최적화 계획 없이 승인하지 않는다.

## 6. 단계별 roadmap

### Phase 0 — Transaction Kernel Foundation

현재 Goal이다.

- 현재 동작 oracle 동결
- 전체 `ActionKind` transaction inventory 작성
- root-enforced transaction descriptor/runner 도입
- 여섯 live battle command 원자화
- `StartNewGame` 원자화
- rejection equality, retry, success parity 검증

### Phase 1 — Active Battle Session and History Boundary

- battle authority surface 축소
- live session, durable archive, client cursor/projection 분리
- full clone/history 비용 측정과 교체
- float-bearing event ordering premise 제거 또는 명시적 key 도입

### Phase 2 — Validated Data and Domain Boundary

- `ValidatedGameData` 또는 동등한 validated construction
- fixture/debug builder 분리
- battle kernel public mutation surface 축소

### Phase 3 — Typed Runtime State

- broad `GameCore` root mutation 축소
- run/battle/event transition의 typed state 검토
- test-only/debug surface 격리

### Phase 4 — Protocol Boundary

- domain result/error와 protocol DTO 분리
- schema/version/compatibility 정책 수립
- game server와 Unity adapter는 필요 시 소규모 재작성 가능

### Phase 5 — New Feature Integration

Relic, corrosion, handoff fragment, boss egg 등 최근 정책 기능은 위 경계가 안정된 뒤 통합한다. 현재 Goal에 새 기능을 섞지 않는다.

## 7. 현재 Goal의 고정 범위

### 포함

- 두 감사의 transaction 관련 finding 통합
- `ActionKind::ALL` 전수 transaction inventory
- transaction classification과 root enforcement
- `PauseBattle`
- `ResumeBattle`
- `SetBattleSpeed`
- `BeginSkillTargeting`
- `CancelSkillTargeting`
- `ActivateSkill`
- `StartNewGame`
- 해당 명령의 성공 parity, 실패 state equality, retry 테스트
- 필요한 최소 문서와 migration ledger 갱신

### 제외

- RunPhase 전면 재설계
- protocol v2 또는 DTO crate 추출
- battle history/archive 교체
- `ValidatedGameData` migration
- Relic/corrosion/boss egg 등 새 feature
- 대규모 module/crate split
- game server 및 Unity 변경
- 무관한 lint/formatting 정리

범위 밖 결함이 현재 Goal을 막으면 증거와 최소 선행 조건을 기록하고 멈춘다. 편의상 함께 고치지 않는다.

## 8. 완료 판단

현재 Goal은 코드가 더 보기 좋아졌다는 이유로 완료되지 않는다. 다음이 모두 만족되어야 한다.

- 모든 `ActionKind`가 transaction inventory에서 누락 없이 분류됨
- 대상 일곱 명령이 root-enforced path만 사용함
- 대상 실패 케이스의 authority equality와 retry가 테스트로 고정됨
- 성공 결과와 기존 외부 관찰 동작이 유지됨
- 새 dual-write/fallback이 없음
- baseline 대비 성능/테스트 회귀가 없거나 명시적으로 승인됨
- `ACCEPTANCE_GATES.md`의 모든 필수 gate가 통과함
- `MIGRATION_LEDGER.md`가 실제 전환 상태와 일치함
