# A05 — 보스 제작·task 포트·진행 권한

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

원래 논리 설계와 승인 목표는 유지하고 새 R owner/현재 코드 전환 경계를 반영했다. 새 문법·타입·포트 이름은 실제 구현 존재의 증거가 아니다.

## 1. 유지할 구조

기존 Encounter→ActorBehavior→Pattern/State→Task→Result/Transition 구조, typed parameter/call frame, cooldown/priority·loop 검증, bounded microstep와 state activation을 재사용한다. 보스 전체를 플레이어 skill Flow에 흡수하거나 보스 이름별 실행 분기를 추가하지 않는다.

## 2. 조기 최소 호출 R08

raw BossTask·compile·공통 내부 Action admission·result mapping·target/facing·state exit cancel을 함께 추가한다. Player 전용 ManualSkill 요청을 그대로 호출하거나 void invoke로 실패/자격을 우회하지 않는다. Accepted/Released/EffectsSettled의 waiter를 구별하고 모든 정상 실패·취소·timeout을 graph에서 처리한다. release 뒤 payload는 기존 수명에 따른다.

## 3. 고급 확장 R18

R18은 상태 profile/가역 modifier·targetability/거리 조건·HP 연속성·임계치 skip, 실제 Priority/Weighted 선택자, 여러 actor·field-work signal을 확장한다. 선택자의 enum만 추가하고 FixedSequence로 fallback하지 않는다. cooldown은 후보 검토와 실제 admission의 소비 시점을 구별한다.

## 4. Blackboard·signal·권한

변수/신호는 타입·scope·writer를 선언한다. RON signal이 무제한 world mutation이나 보상 authority를 가지지 않는다. field_work의 RepairProtectedUnit-only 완료를 확장할 때 raw/validator/executor/boss 구독을 동시 전환한다. wave activation/required-for-victory/loot authority는 별도 owner다.

## 5. 소환과 실패

공통 SpawnKernel의 atomic preflight·idempotent request·placement를 재사용한다. 현재 boss/opponent 상위 adapter를 일반 actor/object 생성 포트와 분리한다. recursive spawn·zero-time graph·stale task completion은 유한성/세대 검사로 제어한다. 잘못된 task 요청은 정상 성공/fallback state로 숨기지 않는다.

## 6. 인수

R08의 기본 시전과 R18의 C03/C07를 별도 검증한다. C03은 HP/actor identity를 재생성하지 않고 phase skip/변신 중 death를 검사한다. C07은 작업완료→typed signal→억제와 Route/wave를 검사한다. 두 경우 모두 R21에서 실제 최신 공식 RON, R22에서 다른 작성자의 새 보스 조합으로 연결한다.

원문 설계 근거: [S084](../../evidence/SOURCES.ko.md#s084). 현재 코드 근거와 각 단계의 상세 동시 전환은 [DEPENDENCIES](../DEPENDENCIES.ko.md) 및 [조사 결과](../../report/REVIEW.ko.md)를 따른다.
