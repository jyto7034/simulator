# R06 WP02 순차 effect/result-prefix 구현 계획 — segment 001

## 1. 진입 기준과 currentness

- 직접 선행 승인 타깃은 `corrections/wp02_cap_01_02_001/target/cap_01_02_correction_002/MANIFEST.json`이며 SHA-256은 `6ee8ea5a889428eef4a0986cd83037ef8c51de6c751868ef9657adb11d3ce789`이다.
- `reviews/wp02_cap_01_02_correction_002_review_001/REVIEW.md`의 판정은 CAP-01/CAP-02 교정 구간 한정 PASS이며 WP02 전체 완료가 아니다.
- mutation 전에 선행 verifier를 직접 실행해 source 333개와 복원 delta의 currentness를 확인했다. 현재 333개 source actual bytes는 승인 타깃과 일치하며 예상 밖 source drift는 없다.
- 현재 actual bytes, 전역/범위 status, 기준 문서는 `baseline/pre_implementation_001/`에 별도로 보존한다. 선행 타깃과 검토 기록은 변경하지 않는다.

## 2. 선택한 단계검토 경계

이번 구간은 다음 하나의 실행 경계를 구현하고 검증한 뒤 동결한다.

> 한 delivery consumption 안에서 authored effect를 순서대로 `evaluate all targets -> apply -> settle projected actual facts` 처리하고, 완료된 동일 action/delivery prefix만 다음 effect의 ValueExpr가 읽게 하며, immediate/projectile/area 오류를 timestamp transaction까지 반환한다.

WP03/WP04, public expression publication, preview/explain, 새 targeting 정책은 범위 밖이다.

## 3. 구현 불변식

1. 같은 effect의 모든 target amount는 어떤 command도 적용하기 전에 계산·검증한다. 다음 effect의 Live frame은 앞 effect가 적용된 상태를 읽는다.
2. amount만 ValueExpr evaluator가 계산한다. mitigation, crit, strike, ModifyDamage 및 기존 damage/heal command authority는 재사용한다.
3. prefix witness는 exact action capability, action site, delivery occurrence, authored effect frontier에 결속한다. 완료되지 않은 effect, 자기/미래 effect, 다른 action/attempt는 읽지 않는다.
4. witness는 요청된 scalar/resource projection만 보유하고 authored effect 수와 projection 수를 기준으로 allocation 전에 admission한다. full fact history를 새로 보존하지 않는다.
5. Snapshot carrier는 release에서 capture하고 Live는 effect-start에서 읽는다. 구현되지 않은 resolved-target aggregate는 기존 typed unsupported 경계를 유지한다.
6. 오류 시 임시 source/strike/fact-recording context를 복원하고 `GameError`를 바깥 timestamp transaction으로 전달한다. 기존 timestamp 이전 commit은 되돌리지 않는다.
7. expression publication 차단, 기존 budget 상한, 누적 admission 미환급, R04/R05 수명 의미를 유지한다.

## 4. 검증과 중단

- 순차 Live, same-effect multi-target freeze, requested damage와 actual HP loss result 차이, 유효 0 witness와 missing/future/self/other occurrence 거부를 focused test로 검증한다.
- effect 0 mutation 뒤 effect 1 ValueExpr 오류가 현재 candidate의 HP/queue/RNG/UUID/log/witness를 복원하는지 immediate와 변경한 delayed path에서 확인한다.
- literal/action-cast/resource/CAP 회귀, all-target check/test, 기존 allow 조건 Clippy, changed-region rustfmt를 실행한다.
- carrier/clone 비용 변화가 있으므로 기존 PC fixture와 R05 allocation guard를 직접 재검증하고 상한을 변경하지 않는다.
- 빌드 가능하고 검증 가능한 위 경계에 도달하면 before/after archive, 복원 delta, 실행 로그, manifest/hash와 인계서를 새 경로에 동결하고 `부분 구현·자체 검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기` 상태로 중단한다.

