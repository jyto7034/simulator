# R08~R12 승인 범위와 위임 결정

선행 감사는 reviews/r06_r07_20260930_001/AUDIT_CLOSURE.json에서 종료했다. 이 문서는 R08~R12 구현 계획이며 독립 감사가 아니다.

## WP08 — 명시 보스 action

StartAction의 skill/target/facing/milestone/timeout/cancel 정책을 검증한다. 권한별 player/boss adapter와 공통 action 실행을 분리한다. state activation이 retained handle을 소유하며 Released/EffectsSettled/거부/소실/timeout을 명시 route에 연결한다. 미방출만 취소하고 방출 payload는 보존한다.

P06: timeout은 명시 CancelUnreleased 또는 Detach; state exit는 CancelUnreleased. 유한 양수 timeout 필수. action ordinal 재생성이나 tick별 전체 fact 스캔을 거부한다.

## WP09 — 침식·강제정지·Breakdown

일반 시전 자격을 지나는 첫 unpaused start gate에서 침식을 한 번 판정한다. exact R07 source/trauma/공명과 최종 보스 제외를 사용한다. 직원 Withdrawn/trauma100·현재 적 특수 death·공명·연출을 원자 commit한다. 필수 death/wave는 유지하고 선택 반응은 억제한다. host elapsed 정지는 사용자 Pause와 독립이다. Breakdown은 identity별 완료 node 잔여 수를 저장한다.

P01~05: 접수 아닌 첫 unpaused 실행을 start로 채택; 준비 중 취소 비용은 기존 그대로. trauma 임계80/연출1000ms는 위임 잠정 live 값이며 RON 명시 양수 검증, 사용자 직접 확정값 아님. 회복 먼저 적용하고 기존 trauma100만 완료 node 차감, 새 진입은2부터. 만료 elapsed의 초과분은 버리고 다음 호출부터 battle 진행. P14: durable 필드 추가는 새 schema와 명시 변환으로만 인수하며 원본을 덮어쓰지 않는다.

## WP10 — 유한 구조화 Flow

Sequence/Branch/Parallel(All)/Wait/유한 Repeat를 공통 compiled flow로 실행한다. 기존 steps는 같은 계획으로 lowering한다. authored 주소와 occurrence/typed result를 분리하고 선행 result ref만 허용한다. self/unit/tile/facing/anchor/retarget을 Instant/Projectile/Area로 전달한다. R08 waiter와 R09 start를 유지한다.

P07: 같은 시각 authored 순서, Parallel All join, 실패시 미방출 sibling 중단/방출분 보존. no-target도 Repeat 기회 소비. 반복 수는 interval과 독립. 시간+1 충돌회피/무한 반복/forward typed ref 금지.

## WP11 — source 효과·channel·overlay

base와 source-owned modifier/lock을 분리하여 종료 시 자기 source만 제거한다. aura membership은 실제 직원 배치/거리/자격을 반영한다. 태아 channel과 Hello/Goodbye 공격 overlay를 정상 fragment 명령에 연결한다. 이미 release된 공격 profile은 보존한다.

P08: 태아 기존 저지 유지, self/Guest/summon/object 제외, 같은 family 비중첩이며 priority 높은 값→동률 creation ordinal 먼저 선택. 기본 공격 피해20%/속도15 points 계약 보존. P09: Hello [start,end), 재입력은 공명 재청구 없이 남은 시간 commit, 만료는 복귀만. 다음 공격 overlay는 release에 소비. P13: channel/overlay/pool은 attempt 밖 저장하지 않는다.

## WP12 — 보호막·생존·제어·HP 비용

개인/공유 pool의 실제 잔량을 단일 source owner에 보관하고 membership 변화로 복제하지 않는다. ordinary damage 감쇄 뒤 shield/HP1/death-prevent를 중재해 HP-loss와 absorbed를 구분한다. parry 첫 strike의 피해/부착 효과만 차단하고 counter 한 번. HP cost는 별도 fact, slow/root/taunt/pull은 명시 capability별로 실행한다.

P10: 개인→공유, 분류 내 authored priority 내림차순→creation ordinal 오름차순. P11: ordinary만 HP1, cost는 기본 HP≥1/명시 lethal opt-in이며 shield/parry 우회, 특수 kill은 모두 우회. P12: 첫 직접 enemy strike만 parry, 반사/self/cost 제외, 공격자 소실은 no-target으로 counter 종료. Black Swan standing passive는 해당 fragment만의 명시 예외이며 전체 passive 활성화 아님.

## WPINT — 통합 검증·동결

A08~A12와 X01~10을 공개 입력/정상 명령/실전/결과/저장까지 실행한다. R06/R07 변경 영향과 PC36/R05를 최종 source에서 검증한다. 실제 before/after membership와 정역 복원/currentness를 확인한 최종 target을 독립 감사 대기로 인계한다.

P13~14: durable identity/counter와 ephemeral effect 소유 분리. 기존 V7→V8 명시 변환을 보존하고 새 schema 변환도 명시적. R05 peak cap9030188 단독 초과 WARNING, 나머지 hard guard 유지. 20분/stress/metrics/차기 원정/GUI/R13 제외.

55개 조건의 정확한 oracle은 preparation/20260930_001/COVERAGE.json을 인수한다. 수치80/1000은 위임 잠정값이며 공식 사용자 확정 수치라고 주장하지 않는다.
