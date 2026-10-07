# A03 — 실행·상태·수명·실제 결과

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.

## 1. Owner와 수명

StateOwner와 Lifetime은 직교한다. 직원/장비사본/파편사본/유물사본/activation/object가 소유하는 상태가 Run/Attempt/Deployment/Activation/Delivery/Chain 중 어느 종료를 따르는지 별도 선언한다. 공유 pool은 하나의 owner가 소유하고 수혜자 membership만 여러 개다. 정의 ID나 transient unit ID로 모든 상태를 묶지 않는다.

## 2. Action과 continuation

request admission, Accepted, Released, FlowFinished, EffectsSettled, Canceled/Failed를 구별한다. R04는 현재 carrier의 handle부터 제공하고 R08 boss adapter와 R10 structured Flow가 이를 소비한다. stale state/task generation의 결과가 새 state를 전이시키면 안 된다. 미방출 cancel과 이미 방출된 projectile/fixed area의 독립 수명을 구별한다.

## 3. 실제 사실과 반응

Damage/Heal/Cost request와 actual HP/pool/회복/처치 receipt는 다르다. pre-death 관찰이 필요한 상태는 정의한 시점의 snapshot을 남기고, mandatory death/wave bookkeeping과 optional OnDeath/OnKill/proc dispatch를 분리한다. 기본공격 OnAttack/OnHit의 현 의미를 일반 Damage로 무단 확대하지 않는다.

## 4. Modifier·membership·channel

R11은 source별 ledger/lease를 합성하고 종료 시 자기 항목만 제거한다. 반대 숫자를 더하거나 옛 stat 전체를 덮어 복원하지 않는다. channel과 기본 공격 overlay는 행동 lock/수혜자 membership/재입력/timeout을 명시한다. 이미 release한 payload snapshot은 이후 profile 변경과 구별한다.

## 5. 원자성·예산·관찰

현재 공개 operation의 후보 상태·resource·queue·RNG·ID·log가 실패에서 일관되게 commit/abort되어야 한다. 이미 완료된 이전 timestamp의 모든 상태를 무조건 되감는다는 계약은 만들지 않는다. 연쇄/소환/반복/graph microstep/원문 크기마다 도입 시점부터 bounded budget과 typed failure를 둔다. 최종 R23에서 현실 부하를 측정한다. 로그 retention/출력 여부는 새 인과 state의 authority가 아니다.

## 6. 저장

Run 상태는 도입 R이 checkpoint/entry/result와 함께 소유한다. live-process reconnect와 durable save를 구별한다. compiled index·pointer를 저장하지 않는다. R20은 여러 상태를 섞은 결과/후퇴/복원의 통합 인수다.

원문 설계 근거: [S082](../../evidence/SOURCES.ko.md#s082). 현재 코드 근거와 각 단계의 상세 동시 전환은 [DEPENDENCIES](../DEPENDENCIES.ko.md) 및 [조사 결과](../../report/REVIEW.ko.md)를 따른다.
