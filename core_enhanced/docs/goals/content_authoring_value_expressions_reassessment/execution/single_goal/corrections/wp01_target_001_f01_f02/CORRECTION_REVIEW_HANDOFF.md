# R06 WP01 F01/F02 교정 재검토 인계

Status: `교정·자체 검증 종료 / 교정 타깃 동결 / 재검토 요청 대기`

이 인계는 `WP01-F01`, `WP01-F02`의 교정과 자체 검증 결과다. 자체 테스트 성공을 재검토 PASS로 표시하지 않으며 WP02, 다음 R06 기능, GEX-20/30, WP/Phase/Goal 완료를 시작하거나 선언하지 않는다.

## Before / after identity

- Before target: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp01_content_input_compile_001/target/target_001/MANIFEST.json`
- Before target SHA-256: `6f55cfa0fae0d88971eba6cb3c47e39ab5d12a2e99b2656a40eb2b51e7cf76de`
- Correction baseline: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp01_target_001_f01_f02/baseline/pre_fix_001/MANIFEST.json`
- Correction baseline SHA-256: `a431db07b318c56f572b1799f9ce186903ecc667d3bddb898fbf78f8c7eb635e`
- After correction target: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp01_target_001_f01_f02/target/correction_001/MANIFEST.json`
- After correction target SHA-256: `679336ad025f9dfaa6b44345ca3e494fd694e5903d49786a91d0ff270dbd642f`

기존 target, baseline, review 보고서·probe·실패 evidence는 덮어쓰지 않았다. before/after source actual bytes, 기록/evidence actual bytes와 binary-capable source delta를 새 correction 계보에 보존했다.

## WP01-F01

- 원인: custom `WireValueExpr` enum에 strict unknown-field 설정이 없어 Mul/Ratio/Clamp/Aggregate의 미지원 field가 Serde에서 폐기됐다. `Read`가 사용하는 `ValueRef` struct variant에도 같은 누락이 있었다.
- 변경: `WireValueExpr`와 `ValueRef`에 `deny_unknown_fields`를 적용했다. 이미 strict한 AmountExpr, TypedInteger, AuthoredMember, GroupOwner, ProjectedTile과 상위 effect authoring 타입은 유지했다. 전체 RON 재parse나 사전 AST clone을 추가하지 않았다.
- 검증: 주요 expression struct variant, Stat/HitPoints/Resource/Result/AggregateMember payload, AuthoredMember nested payload와 정상 대조군을 unit regression으로 추가했다. 동일 review probe의 direct parser와 새 외부 pack fixture 모두 typo Mul을 `ron_parse`로 거부하고 source identifiers를 유지했다.

## WP01-F02

- 원인: `validate_result_contract`가 reference set이 비어 있을 때만 평가해 authored source가 직접 공급하는 `AggregateMember`도 dynamic으로 오분류했다.
- 변경: `ResolvedTargets`가 없고 references가 정확히 local `AggregateMember`뿐인 plan을 external-context-free로 분류한다. 해당 plan만 기존 bounded `evaluate`와 기존 Damage/Heal narrowing으로 검증한다. Stat/HitPoints member context, source stat, resource/result/time 및 resolved targets는 static 값으로 대체하지 않는다.
- 검증: authored `-1` Sum Damage와 Heal `i32` 초과는 `value_expr_overflow`, AggregateMember 0 분모는 shared evaluator의 `value_expr_divide_by_zero`로 거부됐다. 정상/중첩 authored aggregate는 통과했고 source stat, member stat, resolved-target aggregate는 compile 후 기존 `content_capability_unsupported` publication gate에 남았다. Damage overflow의 실제 source file/definition/action/effect/field와 외부 pack 식별자를 확인했다.

## 변경 source

- `core_enhanced/src/game/data/content_pack/tests.rs`: `0ebbe53ad25741bd98e6b0359d4e07c0005aa98654be25d225ae170d2b780166` → `965cef0bbe2f75d2495bddaf9326daafaa78c3fc4b3c0d725861a043d2fb4156`
- `core_enhanced/src/game/value_expr/catalog.rs`: `1a9acca92417ad58a0ebb15d485eac2bb7c463528db227493e45be3fcdd4a62c` → `3d86eee5648c02953079b23180a4edd8e0e511bb5edab65b00b1dace6ee9e28e`
- `core_enhanced/src/game/value_expr/mod.rs`: `f2e7b3b834291d38f66334d1556e5e0054fed017e2d175b799fcf1e66ad29fed` → `fb1a52229e6a1d872a122320272f8b26092065ec0bb578d7a9be1a2592cb24f1`
- `core_enhanced/src/game/value_expr/parse.rs`: `387efcb935eb5cb8f84a17a575c8076f5131211285d118bfdbe99d3df9852f9a` → `83dd9a33f90b38723d7b236fbbeb2bc7f0f7b2e2f01cc04a0a47dcf0e75203ba`

## 직접 실행 검증

- 수정 전: build/probe compile PASS, 원 assertion 재현은 1 pass / 2 expected finding failures / 1 ignored, exit 101.
- 수정 후: ValueExpr 36/36, content pipeline 59/59, authoring 15/15, skill runtime 15/15, checkpoint/save 18/18 PASS.
- 동일 review probe 최종 실행: 3 pass / 0 fail / 1 ignored. 별도 depth probe 1/1 PASS.
- all-target check, focused rustfmt PASS.
- Clippy는 predecessor와 동일한 command-line allow `clippy::never_loop`, `clippy::eq_op`만 사용해 exit 0. 기존 warning은 raw log에 보존했다.
- 첫 새 fixture 실행은 부모 디렉터리 setup 오류로 exit 101이었다. invalid attempt로 삭제하지 않고 보존했으며 결과 수에는 포함하지 않았다.

## 재사용 검증

- 교정 진입에서 predecessor target의 구현자 verifier와 reviewer 독립 verifier를 재사용해 source/archive/reference currentness를 확인했다. 441 reference hash, source 313개, records/evidence archive가 일치했다.
- predecessor의 기존 test receipts는 해시로 보존·확인했으나 이번 직접 실행 test 수에 합산하지 않았다.

## 잔여 위험과 경계

- strict regression은 현재 승인된 authored surface를 다루며 전체 미래 RON 입력 공간의 독립 재감사를 대신하지 않는다.
- `AggregateMember` local 분류는 compiler가 보장하는 lexical aggregate scope와 `resolved_targets` requirement에 의존한다. 해당 compiler/fingerprint/cost 의미는 수정하지 않았다.
- WP02 runtime evaluator는 여전히 미연결이며 authored expression publication은 계속 `content_capability_unsupported`로 차단된다. 이는 이번 correction의 의도된 잔여 경계다.

이 target을 fresh 단계 재검토 입력으로 사용한다. 재검토 결과 전에는 correction source·records·evidence·target을 변경하지 않는다.
