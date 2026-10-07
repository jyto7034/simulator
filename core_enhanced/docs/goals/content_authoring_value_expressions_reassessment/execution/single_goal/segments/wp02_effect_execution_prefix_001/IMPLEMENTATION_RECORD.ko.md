# R06 WP02 순차 effect/result-prefix 구현 기록 — segment 001

## 1. 동결 경계

이번 구간은 **한 delivery consumption 안의 effect 순차 실행, 동일 action/delivery의 완료 prefix witness, immediate/projectile/area 오류 반환과 현재 candidate rollback**까지만 구현한다. WP02 전체, WP03, WP04, public expression publication 완료를 주장하지 않는다.

선행 승인 타깃은 `cap_01_02_correction_002/MANIFEST.json` SHA-256 `6ee8ea5a889428eef4a0986cd83037ef8c51de6c751868ef9657adb11d3ce789`이다. 구현 전 승인 source 333개가 현재 actual bytes와 일치함을 확인했고, `baseline/pre_implementation_001/`에 source·기준 문서·status를 보존했다. baseline manifest SHA-256은 `8a97eeb8432b1eea83128f11370c0ebf0dac3377de15e0979846ce8e31ee1b13`이다.

## 2. 실행 구조

`BattleCore::execute_ordered_skill_effects`가 다음 순서를 소유한다.

1. executor가 확정한 source/action/delivery와 기존 순서의 target slice를 받는다.
2. expression effect이면 어떤 command도 적용하기 전에 그 effect의 모든 target amount를 release capture와 effect-start frame으로 평가·검증한다.
3. 해당 effect만 기존 `BattleCommand` damage/heal/resource/stat/buff/interrupt/extra-attack 경로로 적용한다.
4. command가 기록한 실제 damage HP loss, heal delta, typed resource delta, damaging strike/count projection을 확정한다.
5. exact prefix frontier에 effect 결과를 settle한 뒤 다음 effect로 이동한다.

따라서 같은 effect의 뒤 target은 앞 target mutation을 읽지 않고, 다음 effect의 Live frame은 앞 effect가 적용된 상태를 읽는다. `ModifyDamage`는 기존처럼 step 전체 fold이고 mitigation·crit·strike·death 책임은 기존 damage executor에 남는다. evaluator는 amount narrowing만 담당한다.

## 3. witness와 수명

- `ValueExprDeliveryCapture` row는 authored effect 순서와 정확히 정렬하고, fixed release target이 있으면 target별 capture를 둔다. plan/frame은 `Arc`로 공유하며 tick/impact마다 deep clone하지 않는다.
- `EffectResultPrefix`는 exact `ActionHandle`, delivery ticket, `ActionSite`, 현재 authored effect index에 결속한다. active action snapshot이나 전역 최근 fact를 witness로 사용하지 않는다.
- 없는 effect/result, 미완료, 자기/미래 effect, 다른 attempt/action/delivery는 typed fault다. 완료 row가 존재하는 값 0만 유효한 0이다.
- full fact history를 복제하지 않는다. authored effect 수만큼의 optional projection과 expression이 요구한 resource id vector만 보유한다.
- Snapshot frame은 release에서 만들고 Live frame은 effect-start에서 만든다. delayed projectile/area carrier는 실제 completion delivery로 다시 결속하되 immutable plan/snapshot frame은 공유한다. released Snapshot projectile는 source 퇴장 뒤에도 release 값을 유지하고, departed source를 요구하는 Live는 `MissingContext`로 거부한다.

## 4. 비용 소유권

- capture row/cardinality, effect id/resource id, per-target capture와 frame retained extent는 allocation 전에 checked admission한다.
- delayed completion rebind는 기존 carrier retained extent에 새 completion source extent × capture 수를 더한 보수적 상한을 clone 전에 admission하고, 저장되는 retained extent는 old capture extent를 new extent로 치환한 값이다.
- `DamageSourceSnapshot` clone shape/retained scan에 carrier를 포함한다. carrier pointee는 per-owner 보수 비용으로 계산하고 실제 clone은 `Arc` handle 공유다.
- prefix projection vector/resource vector도 authored effect와 필요한 projection 수를 기준으로 선청구한다. 실패·rollback·delivery 종료 시 candidate/owner와 함께 drop된다.
- 원본 36조건 PC fixture 직접 재실행 결과는 31 completion/기존 128-Read typed refusal 5건으로 동일하다. 새 `runtime_effect_projection: Option<_>` 슬롯 때문에 native retained admission은 private transaction에서 +96 bytes, literal timestamp workload에서 최대 +576 bytes였고, literal requested allocation 합계는 6조건에서 +480 bytes였다. 이는 requested sum/native admission이며 RSS나 동시 live peak가 아니다.
- 별도 R05 allocation-observer guard의 동시 live-peak 최대는 8,920,677 bytes로 기존 cap 9,030,188 bytes 이내(headroom 109,511 bytes)다. 상한과 128-Read 정책을 변경하지 않았다.

## 5. 오류·복원 경계

- `GameError::ValueExpressionFault`가 typed top-level error를 운반한다.
- instant step, projectile dispatch/advance/impact, persistent area register/tick/expire와 event dispatch callback은 `Result`를 반환한다. capture/evaluation/rebind/close 오류를 성공처럼 삼키지 않는다.
- instant resource delivery는 operation 성공·실패 모두 prior source를 먼저 복원하고 poll/close를 실행한다. queued event dispatch도 prior fact source를 복원한 뒤 오류를 반환한다.
- effect-local strike scope와 result accumulator는 성공·실패 모두 원래 context로 복원된다.
- 오류는 기존 `runtime_transaction`/timestamp candidate 경계까지 올라가 HP, active/death state, queue, RNG/UUID sequence, log, runtime fact, carrier/prefix를 함께 폐기한다. 이전 timestamp commit을 되돌리는 새 정책은 없다.

## 6. 직접 검증

최종 source bytes에서 다음을 직접 실행했다.

- focused runtime 23/23: Live 순차 상태, same-effect target freeze, damage 7→heal 7, defense 후 actual loss 4→heal 4, valid zero/missing/unsettled/future/self/other attempt, action–cast/resource/Snapshot 수명, 예산·취소·재시도.
- delivery error 3/3: immediate event queue rollback, projectile impact event rollback, area tick rollback.
- projectile Snapshot source-withdrawal 1/1.
- runtime fact 13/13, resource 37/37, literal integration 15/15.
- sandbox 밖 all-target: lib 1,432 + integration 52 = 1,484 PASS.
- all-target check PASS.
- Clippy PASS: 기존 command-line allow `clippy::never_loop`, `clippy::eq_op`만 사용.
- changed-region rustfmt PASS: preserved baseline의 기존 formatting debt multiset 외 신규/수정 영역 edit group 0.
- 원본 PC 36조건과 R05 allocation guard 직접 실행 PASS.

상세 command, source identity, raw output은 `evidence/validation_final/`과 `evidence/performance/`에 있다. 최초 sandbox all-target의 Unix socket 1건 실패와 PC sandbox identity 거부 등은 지우지 않고 `DEVELOPMENT_FAILURES.json`에 구분했다.

## 7. 남은 WP02 범위

- 현재 prefix는 **한 delivery consumption** 내부 완료 effect만 보유한다. 다른 step/delivery에서 완료된 동일 action prefix를 이어 붙이는 action-level witness carrier는 아직 없다.
- release 전 이미 완료된 action-level result를 Snapshot에 캡처하는 cross-step 경로는 아직 없다. 같은 release 안에서 아직 완료되지 않은 result Snapshot은 의도대로 `NotSettled`다.
- `ResolvedTargets` aggregate는 executor의 확정 target slice를 새 frame aggregate에 투영하지 않았으며 기존 typed unsupported 경계를 유지한다.
- release 시 fixed target이 없는 delayed Snapshot target-dependent 조합을 새 targeting 정책으로 확장하지 않았다.
- public expression publication 차단은 유지한다.

이 남은 항목 때문에 WP02/R06/Phase/Goal 완료를 선언하지 않는다.
