# R06 WP02 runtime frame/capture 구현 계획 — segment 001

## 1. 계보와 진입 currentness

- Goal: `content_authoring_value_expressions_reassessment`
- 승인 기준 target: `corrections/wp01_target_001_f01_f02/target/correction_001/MANIFEST.json`
- 승인 기준 SHA-256: `679336ad025f9dfaa6b44345ca3e494fd694e5903d49786a91d0ff270dbd642f`
- 재감사: `reviews/wp01_f01_f02_correction_001_review_001/REVIEW.md`의 WP01-F01/F02 부분 범위 `PASS`
- mutation 전 correction verifier를 직접 재실행했고 source 313개, change 4개, record 6개, evidence 47개, receipt 23개의 currentness를 확인했다.
- correction source와 이번에 변경할 실행 문서의 실제 bytes는 `baseline/pre_implementation_001/`에 보존했다. baseline manifest SHA-256은 `26b5222aaa514b6622093429bd9994c3bab04f40791e0446bb2c96cd447fe6be`이다.
- R01/R04/R05 과거 closure/target은 계보 증거로만 보존한다. 그 successor verifier가 보고한 차이는 승인된 후속 R06 WP01·거버넌스 변경이므로 과거 target을 갱신하지 않고, 이번 영향 seam을 현재 소스에서 직접 재검증한다.

## 2. 선택한 단계검토 경계

이번 구간은 WP02 전체가 아니라 다음 하나의 책임 경계를 구현한다.

> immutable `CompiledAmountPlan`의 요구 참조만 실제 `BattleCore` 상태에서 읽어, release 시점의 `Snapshot` 또는 consumption/effect-start 시점의 `Live` frame으로 만들고 기존 bounded evaluator와 narrowing에 전달한다.

이 경계는 독립적으로 빌드·검증할 수 있고, 다음의 더 큰 변경인 순차 effect scheduler/result-prefix witness/void callback 오류 전파 전에 검토할 수 있다.

## 3. 구현 판단

1. `BattleCore`는 자신을 만든 `ContentIdentity`를 immutable `Arc`로 보존한다. plan의 generation identity, `EffectSite`, semantic fingerprint는 각각 별도 필드로 비교하며 action/delivery occurrence identity로 대신하지 않는다.
2. capture는 exact `ActionHandle`과 선택적 exact `DeliveryTicket`, source actor/equipment, 확정된 target 집합, release time을 보존한다. `Arc`를 공유하고 전체 전투 상태나 전체 AST를 복제하지 않는다.
3. `Snapshot`은 release에서 한 번 frame을 구성한다. `Live`는 identity만 capture하고 evaluate 직전 실제 `BattleCore`에서 frame을 구성한다. consume time을 release time으로 바꾸지 않는다.
4. 현재 연결 가능한 scalar는 Attack, HP Current/Max/Missing, BattleElapsed/명시적 deadline의 RemainingTime, R05 resource Balance/Reserved/Available이다. WP03이 실제 runtime 값을 추가하기 전의 SpellPower와 AttackSpeedPoints, 미확정 target, inactive/사라진 owner, stale generation/action/delivery는 0이 아닌 typed fault다.
5. resource 정의→actor→owner→scope 선택은 기존 mutation consumer와 공통 read-only resolver로 추출한다. frame read는 기존 `ResourceStore::bind`/`value`만 사용하고 scope/owner/capability를 생성하거나 지출하지 않는다.
6. `ResolvedTargets` aggregate는 owner-domain/primary member 의미와 실제 effect target list의 결합이 확정되는 다음 경계로 남긴다. result reference도 completed-prefix witness가 없으면 typed `NotSettled`/`Unavailable`로 남기며 최근 settled action이나 0을 사용하지 않는다.
7. expression 실행을 현재의 일괄 `build_skill_step_commands`에 삽입하지 않는다. 그렇게 하면 D02의 effect-start 순차 의미보다 먼저 모든 Live 값을 읽게 된다. 이 구간의 adapter가 검토된 뒤 다음 구간에서 sequential effect consumer가 호출한다.
8. `ensure_runtime_publishable()`의 `content_capability_unsupported` 차단은 유지한다.

## 4. 오류와 불변식

- 모든 `ValueExprFault`는 실제 plan의 `EffectSite`를 유지한다.
- generation/fingerprint/site mismatch, missing target/source, inactive owner, unsupported stat/moment/aggregate, different attempt/stale occurrence/not-settled resource scope를 구분한다.
- evaluator의 비용, cancellation, checked arithmetic, rounding, overflow 및 damage/heal narrowing을 복제하지 않는다.
- capture 전후 mutation이 없으며 zero budget/cancellation은 frame read 또는 evaluator 진입에서 mutation 전에 거부된다.

## 5. 검증 계획

- focused unit: Snapshot은 release 뒤 HP/stat/resource 변경에도 유지, Live는 consume 시 변경 반영.
- negative: 미확정 target, owner 소멸, generation/fingerprint/site mismatch, 다른 attempt/stale action/delivery, missing deadline, SpellPower, result 및 resolved aggregate 미연결을 typed fault로 확인.
- resource: 기존 mutation과 read-only resolver의 owner/lifetime 선택이 동일하고 Balance/Reserved/Available 및 stale scope를 확인.
- evaluator: 기존 plan/evaluator/narrowing을 실제 frame adapter가 사용하고 실제 source site가 fault에 유지됨을 확인.
- 회귀: WP01 strict parsing/static aggregate, content pipeline, R04 runtime facts, R05 resources, literal battle behavior와 publication gate.
- 도구: focused rustfmt, `cargo check --all-targets --offline --locked`, correction과 같은 Clippy allow 조건.
- 실행 결과는 새 evidence/receipt에 직접 실행과 과거 증거 재사용을 구분해 고정한다. 실패한 실행도 제외하지 않는다.

## 6. 중단점

위 adapter와 영향 회귀가 통과하면 target을 새로 동결하고 중단한다. sequential effect/result witness/callback·projectile·area 실패 전파, preview/explain/usefulness 연결, publication cutover는 다음 단계검토 이후 작업이며 이번 target은 WP02/Phase/Goal 완료를 선언하지 않는다.
