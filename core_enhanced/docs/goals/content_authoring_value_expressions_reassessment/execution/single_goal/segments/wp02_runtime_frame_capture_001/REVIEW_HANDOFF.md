# R06 WP02 runtime frame/capture segment 001 재검토 인계

Status: `부분 구현·검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기`

이 인계는 WP02 전체 완료가 아니라 첫 의미 있는 단계검토 경계의 부분 구현과 자체 검증 결과다. 자체 테스트 성공을 재검토 PASS로 표시하지 않으며 다음 구간, GEX-20/30, WP/Phase/Goal 완료를 시작하거나 선언하지 않는다.

## Before / after identity

- 승인 predecessor target: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp01_target_001_f01_f02/target/correction_001/MANIFEST.json`
- predecessor SHA-256: `679336ad025f9dfaa6b44345ca3e494fd694e5903d49786a91d0ff270dbd642f`
- pre-implementation baseline: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp02_runtime_frame_capture_001/baseline/pre_implementation_001/MANIFEST.json`
- baseline SHA-256: `26b5222aaa514b6622093429bd9994c3bab04f40791e0446bb2c96cd447fe6be`
- after target: `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp02_runtime_frame_capture_001/target/target_001/MANIFEST.json`
- after target SHA-256: `d3dbf3ba7c3c3be5a584db3092e7059c116d92b1ae812740f1fb68ccb05c2fc3`

기존 correction target, baseline, 감사 보고서와 그 evidence를 덮어쓰지 않았다. 새 target에는 source actual bytes, 실행 기록 actual bytes, evidence 37개 actual bytes와 binary-capable 복원 delta를 별도로 보존했다.

## 구현 책임과 결과

- generation/capture: production `World`의 exact immutable content identity를 `BattleCore`가 공유하며, current catalog의 exact plan만 capture한다. capture는 action attempt, delivery occurrence, authored source/site/fingerprint를 별도 검증한다.
- Snapshot/Live: Snapshot은 release frame을 한 번 보존하고 Live는 consumption/effect-start 인자의 target/time으로 실제 core state를 읽는다. plan/source/frame은 `Arc` 공유이고 전체 state/AST deep clone은 없다.
- 실제 scalar: Attack, HP Current/Max/Missing, BattleElapsed, explicit-deadline RemainingTime과 typed resource Balance/Reserved/Available을 연결했다.
- 계산: 기존 admitted plan limits, bounded evaluator, rounding/checked arithmetic, Damage/Heal narrowing만 사용한다. 별도 계산 의미는 없다.
- resource 권한: mutation consumer와 공통 exact lease resolver를 사용하되 read는 `bind`/`value`뿐이며 지출 capability, reservation, revision을 만들거나 변경하지 않는다. 공개 `apply_operations` 계약도 유지했다.
- typed refusal: 미확정 target, 사라진 owner/unit, generation/action/delivery mismatch, missing deadline, 아직 미연결인 stat/result/aggregate는 0/default 없이 typed fault이고 실제 source `EffectSite`를 유지한다.

## 변경 source

- `core_enhanced/src/game/battle/core/mod.rs`: `modified`, `75ef06ea9d01648baaad247c3bcd472af8f0dbe44a5e1f718ef345b638057a15` → `9f06122e0df071023b2377f8e246f1a2fe6f69f1c69de06ece1f758ea9a1ac1f`
- `core_enhanced/src/game/battle/core/resource_runtime.rs`: `modified`, `e657614f2dc6abfae4f549c1ae6dd3c1df16cfa8b5d36870f6b985d11750144a` → `25d3c36b1d672cf0285393525db33659e35bec3d8e0c203f2480fbe5c17dec75`
- `core_enhanced/src/game/battle/core/runtime_facts.rs`: `modified`, `2a66485ccbf2331f6999973441c29c78ef7f0761786a4b594fa9264d3a7fddf9` → `f1b5399087da9c7d970f6d7d103861c2d029fe9543abb558670e1af8ed9f6ae7`
- `core_enhanced/src/game/battle/core/value_expr_runtime.rs`: `added`, `None` → `bbe0227f8843334c1f4ee20bce4d13bae47e0621fb4ed246cc1d750a504f3b25`
- `core_enhanced/src/game/data/content_pack/mod.rs`: `modified`, `68038ec567d81f35520a35d9d56b6ed02634626f04567f1d87862adf80a41ac9` → `2859927e6f6c9c4f198c842fdc6271566b7788a5b31414e2c785ff82c9f49a2e`
- `core_enhanced/src/game/data/mod.rs`: `modified`, `1b9edba85f9b447fc245ca99908eafa25ca870e861bffefaa4306dd8ead24365` → `4859474527232acfbbf1336f0b95803a340d59df94f3c80c88182e6c763f7d1a`
- `core_enhanced/src/game/events/combat.rs`: `modified`, `995c4255be46c04cdde86c92407feda59db9d6ba999dc74244e7856073f390e9` → `c28d5f9b51c02d86f1d175707aadde326884f95e4651c70992ccc8997be08466`
- `core_enhanced/src/game/resource/store.rs`: `modified`, `236b5c03afcb6efaba90eff75c0a66b821c5cb873e8286bc9ffc64154074e64b` → `f884dcc0c1c29a0a41408235cf72244741d966e16990b3ee879be2ead6c9a446`
- `core_enhanced/src/game/value_expr/compile.rs`: `modified`, `d4bb639c4abc749ababd983c24c723296ed95d553084de283ac751a5c9af9a79` → `f1b25fba8b3eb797cc6e91556b957e0ce9357941ed92342725d2f7ad3cdea832`
- `core_enhanced/src/game/world/combat.rs`: `modified`, `4c7c63e1f74292dcb00664b1666ff3f917b5dcc002ee80d0c2d48e62c760ea7f` → `96bc69bad5bc802a96840a36a9b56468c2969c928204a9700699db43791d467a`

## 직접 실행 검증

- focused adapter 7/7, R04 runtime facts 11/11, R05 resource store 37/37, R05 lifetime 17/17 PASS.
- WP01 ValueExpr 36/36, content pipeline 59/59, authoring 15/15, checkpoint/save 18/18, literal skill runtime 15/15 PASS.
- 같은 WP01 review probe는 새 임시 external pack에서 3 pass / 0 fail / 1 ignored, depth probe는 1/1 PASS. `typo_parameter`, authored negative aggregate, source diagnostics와 publication gate assertion을 유지했다.
- all-target check와 focused rustfmt는 exit 0. Clippy는 predecessor와 같은 command-line allow `clippy::never_loop`, `clippy::eq_op`만 사용해 exit 0이며 기존 warning과 아직 consumer 미연결에 따른 dead-code warning을 raw log에 보존했다.
- tight work budget 및 cancellation은 capture allocation/mutation 전에 거부되고 core transaction snapshot이 불변임을 확인했다.
- 첫 external fixture 실행의 부모 디렉터리 setup 오류(exit 101)는 무효 시도로 삭제하지 않고 보존했다. assertion 변경 없이 setup만 교정한 별도 receipt를 성공 근거로 사용했다.

## 재사용 증거

- predecessor target SHA와 WP01-F01/F02 부분 PASS 감사 보고서는 진입 currentness/계보로만 재사용했다.
- R01/R04/R05 과거 보고서는 계약 조사에만 사용했고 성공 수에 합산하지 않았다. 관련 seam은 이번 source identity에서 직접 재실행했다.

## 잔여 위험과 다음 검토 경계

- adapter는 actual `BattleCore` state에 연결됐지만 production skill effect consumer는 아직 호출하지 않는다. 따라서 authored expression publication은 계속 `content_capability_unsupported`다.
- sequential effect execution, exact completed-prefix result witness, `ResolvedTargets` member frame, callback/instant/projectile/area 실패 전파와 후속 effect 실패 시 candidate 전체 rollback은 미구현이다.
- SpellPower/AttackSpeedPoints runtime producer, preview/explain/usefulness, atomic publication cutover는 미구현이다.
- 이번 target은 위 책임 경계만 재검토한다. 재검토 결과 전에는 이 source·records·evidence·handoff·target을 변경하지 않는다.
