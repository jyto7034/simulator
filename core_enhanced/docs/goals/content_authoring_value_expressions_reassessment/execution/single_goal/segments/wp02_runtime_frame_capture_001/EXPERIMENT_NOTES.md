# R06 WP02 runtime frame/capture 구현 메모

## 진입 조사

- 현재 expression plan은 generation-local `Arc<CompiledAmountPlan>`이고 external publication은 명시적으로 차단되어 있다.
- 현재 skill step command builder는 모든 effect를 한 번에 생성하고 `DamageExpr`/`HealExpr`를 `unreachable!`로 둔다. 여기서 expression을 즉시 평가하면 effect-start 순차 의미를 위반한다.
- timestamp transaction은 candidate clone 뒤 `Result`가 성공할 때만 commit하므로 상위 rollback 경계는 이미 있다. 반면 instant callback 및 일부 projectile/area 경로는 오류를 삼키므로 이는 별도 D08 변경이 필요하다.
- R04 action identity는 Arc token exactness, attempt identity, delivery ordinal/capability로 이미 구분된다. active action query는 settled result witness가 아니다.
- R05 resource store는 read-only `bind`/`value`와 typed stale owner/scope/lease 오류를 제공한다. 기존 consumer의 definition/owner/lifetime 선택을 공통화할 수 있다.

## 범위 메모

- 이번 구간은 frame/capture/evaluate seam까지 연결한다.
- sequential effect와 result-prefix publication은 의도적으로 미연결이다.
- authored expression capability publication 차단은 의도적으로 유지한다.

## 구현 결과

- `ValidatedGameData`의 content identity를 immutable `Arc`로 공유하고 production `World`가 만든 `BattleCore`에 전달한다. pre-publication feasibility처럼 generation authority가 없는 core는 capture를 만들 수 없다.
- catalog entry capture는 site/fingerprint의 값 비교에 그치지 않고 해당 core catalog의 exact plan `Arc`인지 확인한다. capture 이후에는 content generation, plan site/fingerprint, action attempt와 delivery occurrence를 각각 재검증한다.
- Snapshot은 release에서 요구 reference만 `EvalFrame`에 고정하고 Live는 consumption/effect-start 인자로 받은 시각과 target으로 다시 읽는다. capture clone은 plan/source/frame `Arc`를 공유하며 전투 상태나 AST를 복제하지 않는다.
- Attack, HP Current/Max/Missing, BattleElapsed, 명시적 deadline의 RemainingTime 및 R05 typed resource Balance/Reserved/Available을 실제 core 상태에서 읽는다. resource read는 공통 exact lease resolver와 `value`만 사용하며 revision/reservation을 변경하지 않는다.
- evaluator는 `CompiledAmountPlan`에 admission된 기존 limits와 기존 `evaluate`, `narrow_damage`, `narrow_heal`을 사용한다. 실제 node와 `EffectSite` fault를 유지하고 별도 계산기를 만들지 않았다.
- 기존 공개 `ResourceStore::apply_operations` 계약은 유지했다. battle mutation adapter만 새 crate-private leased batch를 사용하며 transaction clone 안에서 lease를 다시 검증한다.

## 자체 검토 결과

- release 뒤 source attack/target HP 변경에서 Snapshot 90 유지, Live 50 반영을 확인했다. 이후 source/target 제거 시 Snapshot은 고정 frame을 사용하고 Live는 `MissingContext`로 거부된다.
- target 미확정 Snapshot, consume-before-release, 다른 fixed target, missing deadline, 다른 content generation/exact catalog plan, 다른 attempt, 닫힌 delivery를 typed fault로 확인했다.
- Damage 음수와 Heal `i32` 초과는 기존 Narrow phase overflow이며 실제 effect site를 유지한다.
- source/target employee resource가 alias되지 않고 Balance/Reserved/Available read 전후 store revision과 reservation count가 불변임을 확인했다.
- 미연결 SpellPower, result prefix, ResolvedTargets aggregate는 0/default 없이 각각 typed `MissingContext`, `NotSettled`, `UnsupportedMoment`다.
- tight authoring work budget과 cancellation은 capture allocation 전에 거부되고 transaction debug snapshot은 변하지 않는다.
- 전체 직접 검증 결과는 `EXPERIMENTS.md`와 `evidence/validation/VALIDATION_SUMMARY.json`에 기록했다.

## 보존한 개발 실패

1. 잘못된 cwd-relative inspection path: exit 2, 검증 아님.
2. 테스트 삽입 중 우발적 `+` marker compile failure: exit 101, assertion 변경 없이 교정.
3. R05 test selector zero-match: exit 0이지만 검증으로 계산하지 않고 정확한 selector 17개를 재실행.
4. 첫 fresh pack fixture 부모 누락: probe exit 101, 실패 receipt/log 보존 후 setup만 고쳐 동일 assertion 재실행.

## 잔여 경계와 위험

- 새 adapter는 production content identity와 실제 `BattleCore` state를 받지만 아직 skill effect sequential consumer가 호출하지 않는다. 따라서 non-test build의 dead-code warning은 예상되며 lint allow로 숨기지 않았다.
- completed effect-prefix result witness, `ResolvedTargets` member frame, sequential effect execution, callback/projectile/area 오류의 top-level transaction 전파는 미구현이다. 후속 effect 실패 전체 rollback도 이 다음 경계에서 검증해야 한다.
- SpellPower/AttackSpeedPoints의 실제 runtime producer는 WP03 이후 의존성이므로 이번 구간에서 임의 값을 만들지 않았다.
- preview/explain/usefulness와 publication atomic cutover는 미연결이다. `content_capability_unsupported` 차단은 직접 probe로 유지됨을 확인했다.
- 자체 검증 성공은 단계 재검토 PASS나 WP02/Phase/Goal 완료가 아니다.
