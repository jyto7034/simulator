# Run Persistence, Facility, And Monolith Policy Correction Master Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-28

이 문서는 구조적 판단, source-of-truth 충돌, 승인 정책, dirty 상태, 적용하지 않은 대안과 남은 위험을
기록한다. Gameplay/data/save/wire의 durable source of truth가 아니다.

## 1. Master 구조를 선택한 이유

재감사 finding은 네 독립 source-of-truth 경계다.

- Compendium lifetime과 transaction rollback
- Certified power input과 threat projection
- Checkpoint-owned Floor anchor와 active-run save schema
- Monolith stage difficulty/reward와 Unity projection

이를 하나의 giant child로 구현하면 failure 원인과 completion evidence가 섞인다. 반대로 완전히 독립된
Goal 네 개로만 두면 certified power -> anchor -> monolith difficulty의 hard dependency와 save/schema
전달 상태를 상위에서 추적하기 어렵다.

사용자가 기존 `run_persistence_facility_monolith_policy_master`와 같은 하나의 Master + phase 구조를
명시적으로 선택했다. 따라서 Master는 순서와 gate를 소유하고 각 child는 하나의 계약 경계만 수직
완료한다.

## 2. Dependency 구조 판단

### P00 -> P03 transaction seam

Monolith conversion은 공통 `with_state_transaction()`을 사용한다. Stage difficulty와 ledger를 함께
바꾸는 P03 이전에 rollback이 local compendium을 지우지 않는 불변식을 고정해야 conversion 실패 test가
다른 profile 결함과 섞이지 않는다.

### P01 -> P02 power input seam

Floor anchor와 run peak는 certified power를 입력으로 사용한다. Combat eligibility를 나중에 바꾸면 P02의
anchor expectation, power model version과 old-save rejection을 다시 수정해야 하므로 P01이 hard
predecessor다.

### P02 -> P03 persistence seam

P03 typed stage modifier는 `RunMap`/checkpoint/active-run에 저장된다. P02가 current anchor의 checkpoint
ownership과 envelope monotonic metadata를 먼저 정리해야 P03이 안정된 새 save generation 위에 stage
authority를 추가할 수 있다.

### P04 독립 재감사

P04는 child 문서의 완료 표시를 증거로 사용하지 않는다. Runtime mutation, live data, serialized shape,
server ordering, Unity consumer, legacy search와 실제 test 결과를 다시 확인한다.

## 3. Source-Of-Truth 충돌

### Compendium

- 승인 정책: run rollback 밖 monotonic union
- 현재 공통 rollback: default profile 사본을 복원할 수 있음
- 목표: rollback 전후 union과 durable profile이 모든 snapshot 경계에서 수렴

### Certified power

- 최초 decision: combat-eligible roster
- 후속 notes/runtime: all-living roster
- 사용자 승인 목표: combat-eligible roster, run peak는 monotonic

### Floor anchor

- 현재 canonical/runtime: current anchor도 checkpoint 밖 high-water
- 발견된 결과: 과거 Floor load/re-advance가 같은 branch 난이도를 누적 상승
- 사용자 승인 목표: current anchor checkpoint-owned, loads/run peak만 high-water

### Monolith

- 승인/완료 주장: stage 3 actual stat과 reward가 stage 2보다 강함
- 현재 runtime: target threat와 reward만 차이, actual stat/wave 동일
- 사용자 승인 목표: target/HP/attack/defense stage 합성, wave 유지, reward one-pass,
  qualitative Unity tier

## 4. 승인된 정책 질문 상태

현재 미결정 정책은 없다.

- F-01은 기존 P02 monotonic profile 계약을 적용한다.
- F-02/P-03 stage와 reward 합성/표시는 2026-07-28 사용자 승인으로 확정됐다.
- P-01 certified eligibility는 combat-eligible roster로 확정됐다.
- P-02 current anchor는 checkpoint-owned로 확정됐다.
- 개발 단계 구 save/schema compatibility는 제거로 확정됐다.
- Local-install singleton과 dormant corrupted key는 기존 범위를 유지한다.

새 external DTO 의미, balance 수치, save 보존 요구 또는 identity/lifetime이 필요해지면 active child가
정책을 대신 결정하지 않고 중단한다.

## 5. Dirty Worktree 주의사항

Master 신규 경로 확인 직전 다음 명령을 실행했다.

```bash
git status --short
git -C '/mnt/f/unity projects/ark' status --short
```

두 명령 모두 출력이 없었다. 이는 Master 초기화 순간의 기록이며 child 시작 시점 상태를 보장하지
않는다. 각 child는 production 수정 전에 현재 status와 대상 file diff를 새로 확인한다.

임시 감사 메모는 `core_enhanced/tmp` 아래에 있으며 현재 repository status에 나타나지 않았다. 이를
production canonical이나 child 완료 증거로 취급하지 않는다.

## 6. 외부 시스템과 직접 의존성

- Unity 프로젝트: `/mnt/f/unity projects/ark`
- Unity MCP instance는 pre-Goal 감사에서 접근 가능했지만 child 시작 시 현재 instance/readiness를 다시
  확인해야 한다.
- P00/P02는 game_server actor와 실제 filesystem persistence test가 필요하다.
- P03은 Core DTO producer, game_server mapping과 Unity strict reader/presentation을 함께 변경한다.

접근 가능한 파일을 읽었다는 사실을 Unity EditMode/PlayMode나 target filesystem behavior를 실행 검증한
것으로 표현하지 않는다.

## 7. 적용하지 않은 대안

### 네 finding을 하나의 child patch로 구현

적용하지 않는다. Transaction, formula, save와 battle/Unity 실패 표면이 달라 rollback과 regression
증거를 분리할 수 없다.

### 독립 Master 네 개 생성

적용하지 않는다. 사용자가 하나의 Master 아래 phase 관리를 명시적으로 선택했고 P01->P02->P03의
hard dependency를 한 registry에서 관리해야 한다.

### P02와 P03 schema를 미래 통합 형태로 한 번에 선행 설계

적용하지 않는다. P02는 checkpoint anchor ownership만 완결하고 P03은 실제 stage consumer가 필요할 때
typed modifier를 추가한다. 현재 live 경로에 없는 미래 추상화를 P02에서 만들지 않는다.

### 구 save migration 또는 compatibility reader

적용하지 않는다. 사용자가 개발 단계 legacy 제거를 승인했다. Version bump, explicit rejection과 개발
save reset만 허용한다.

### Potential power와 deployable power 이중 authority

적용하지 않는다. 승인된 단일 certified formula와 monotonic run peak로 사용자-visible 문제를 해결할 수
있고 이중 version/persistence 의미가 필요하지 않다.

### Stage multiplier를 wave에도 적용

적용하지 않는다. `stat multiplier` 의미와 P04 role-owned wave budget을 위반하고 stage 3 난이도를
개체 수와 개별 stat 양쪽에서 중복 상승시킨다.

## 8. 범위 밖 후속 후보

- 계정/게임 내부 다중 profile persistence
- P06 hidden boss와 corrupted-monolith consumer
- provisional power coefficient의 실제 balance calibration/playtest
- 현재 finding과 무관한 map/content/transport/UI 리팩토링

이 항목은 child Goal을 자동 생성하거나 현재 campaign에서 구현하지 않는다.

## 9. 접근하거나 검증하지 못한 소비자

Master 초기화 단계에서는 production consumer를 새로 조사하지 않았다. Pre-Goal 감사에서 workspace Core,
server, live data와 외부 Unity source/MCP에 접근했지만 각 child의 current baseline으로 재검증해야 한다.

존재 여부가 알려지지 않은 배포 build, 실제 사용자 save와 계정/cloud consumer는 현재 제품 범위 밖이다.

## 10. 생략한 검증과 근거

- Master 초기화는 문서 작업이므로 cargo/Unity test를 실행하지 않았다.
- Production baseline은 active child 생성 뒤 현재 dirty 상태에서 확보해야 변경 전후 비교가 유효하다.
- Pre-Goal broad test 결과는 감사 근거 포인터이며 새 child baseline이나 완료 증거로 재사용하지 않는다.

## 11. 남은 위험

- Child 시작 전 코드나 live data가 바뀌면 pre-Goal finding 경로를 다시 확인해야 한다.
- P01 power model bump, P02 envelope bump와 P03 internal difficulty schema 변경이 연속되므로 각 phase에서
  구 version field/fixture 제거와 current version rejection을 명시적으로 감사해야 한다.
- P03 reward one-pass 합성은 작은 quantity의 실제 결과를 바꾸므로 경계값 test 없이 snapshot만
  갱신하면 안 된다.
- Unity qualitative reward tier의 exact DTO 이름과 표시 문구가 승인 의미를 약화하지 않는 범위에서
  technical naming으로 결정 가능한지 child가 확인해야 한다. 새 UX 의미가 필요하면 정책 질문이다.
- P04가 material gap을 발견하면 Master를 완료하지 않고 해당 owner child를 재개하거나 범위 밖 후보로
  보고해야 한다.

## 12. Final Correction Assessment

- 모든 correction finding은 current authority, strict serialization, canonical과 current regression에
  연결됐다.
- Legacy 문자열은 internal policy 또는 negative strictness evidence만 남고 compatibility path는 없다.
- D-01은 current header와 correction note만 수정해 history를 보존했다.
- P04가 확인한 explicit live Maintenance failure는 correction source-of-truth와 독립적인 범위 밖 후보다.
  전체 live opt-in green을 주장하지 않으며 P04 completion review에 공개했다.
- Correction Master verdict는 `Complete / High`다.
