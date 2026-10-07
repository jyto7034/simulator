# R06 WP02 D07-A 정적 compiled amount 설명 — 단계검토 인계

2026-09-27 · 상태: **부분 구현·자체 검증 종료 / 단계검토 target 동결 준비 / 독립 검토 대기**

## 1. 범위와 시작점

- 구현 범위는 `WP02_D07_IMPLEMENTATION_REQUEST_20260927.ko.md`의 D07-A뿐이다.
- 시작 manifest SHA-256은
  `58505bbcd34089b9c1ec3d70f4d9df4ca32a2397e1115c68386364e44514e549`이다.
- 시작 337개 source digest는
  `8aa03d6bd04f8d524fa61b465a9bd7a4171062a1190f615b794e20f915190f04`,
  제품 333개 digest는
  `8f6a58059254d4db4dc726bc8aeb2a6c6ec348e9aef685052837df903fd94bcd`다.
- 구현 전에 predecessor `freeze.py verify`의 currentness와 격리 양방향 복원을 확인하고,
  `baseline/entry_001/SOURCE_BEFORE.tar.gz`에 실제 337개 바이트와 신규 파일 부재를 보존했다.

## 2. 구현 결과

### 실제 publication의 amount plan 설명

- 기존 `ExplainQuery::Effect`가 실제 winning `ValueExprCatalog`의 `EffectSite` entry를 조회한다.
- Damage/Heal 및 내부 staged DamageExpr/HealExpr는 borrowed `CompiledAmountExplanation`으로
  representation, kind, compiler version, provenance, moment, result unit, 실제 source,
  semantic fingerprint, 전체 requirements, PlanCost, compiler 순서의 segment/index/ops,
  evaluation context와 publication 상태를 직렬화한다.
- 정상 비금액 effect만 `value_expression:null`이다. 금액 effect plan 부재는
  `authoring_amount_plan_unavailable`, site/kind/source 불일치는
  `authoring_amount_plan_inconsistent`로 거부한다.
- 기존 effect literal은 그대로 남는다. descriptor에는 `amount`, `trace`, `final_damage`가 없고
  `evaluation.performed=false`다.

### 공통 문맥 분류와 bounded query

- catalog validation이 쓰던 `can_evaluate_without_external_context`를 crate 내부에서 공유했다.
  AuthoredMembers의 bare AggregateMember는 context-free이고 Source/Target stat, resolved member,
  resource/time/result는 runtime context 필요로 분류된다.
- segment view는 `0..segment_count`를 순서대로 순회하며 각 borrowed op slice를 직렬화한다.
  aggregate member segment를 전개하거나 plan/requirements/source/AST를 clone하지 않는다.
- catalog lookup 문자열 비교 상한, segment/ops, requirements/source/cost, 64자리 fingerprint를
  기존 Inspection work/metadata/result/cancellation 계량에 연결했다. overflow와 빠진 segment는
  성공 또는 빈 값으로 바꾸지 않는다.
- `describe`에는 static effect explain 지원, numeric preview 미지원, authored expression
  production publication 차단 유지가 additive 정보로 추가됐다.
- `TrialSession::explain`은 외부 official publication에서 direct explain과 동일한 문서와 usage를
  반환함을 확인했다. runtime frame/evaluator/action/RNG/resource/target state는 만들거나 읽지 않는다.

### 변경 파일

- 추가: `core_enhanced/src/authoring/amount.rs`
- 수정:
  - `core_enhanced/src/authoring/mod.rs`
  - `core_enhanced/src/authoring/query.rs`
  - `core_enhanced/src/authoring/tests.rs`
  - `core_enhanced/src/authoring/trial.rs`
  - `core_enhanced/src/game/value_expr/catalog.rs`
  - `core_enhanced/src/game/value_expr/mod.rs`
- 최종 확장 source는 338개, digest는
  `5279b8333c6be8f90992ac94a47f09beb083bf82bfcdf02ebbe3e0d179c8f530`다.
- 신규 제품 모듈을 포함한 제품 source는 334개, digest는
  `17e41fa7625d001242fcfcec65834ab533fea43ea74e3f89f7ce78f3bae5acfd`다.

## 3. A01–A10 대응

| 기준 | 고정된 검증 |
| --- | --- |
| A01 | `public_effect_explain_describes_actual_legacy_damage_heal_and_non_amount_effects`: 실제 외부 legacy Damage/Heal의 kind/unit/moment/source/fingerprint/cost/segments와 catalog를 대조 |
| A02 | 같은 시험에서 비금액 null, 금액 plan 부재 typed 오류를 분리. 기존 missing site 오류 회귀는 전체 authoring 시험에서 유지 |
| A03 | `static_explanation_context_classification_uses_the_catalog_rule_without_evaluating`: Const, AuthoredMembers와 모든 요구 runtime reference 분류 및 performed=false; i128 최대값 typed serializer 원문 보존 |
| A04 | `static_explanation_preserves_relations_filters_and_member_segment_indices`: Side/ActualEmployee/EquippedSlot, 실제 직원·장비 슬롯 predicate, 여러 aggregate와 중첩 member segment index를 정확히 대조 |
| A05 | `effect_explain_follows_same_id_winners_and_keeps_old_generation_immutable`: 실제 loader의 두 same-ID winner generation, source/fingerprint/identity 차이와 옛 문서 불변 확인 |
| A06 | public query의 work/metadata/result 1 미만 typed 거부와 cancellation, 내부 512-member long plan의 work/metadata/cancellation 거부 |
| A07 | 반복 문서·usage 결정성, 조회 전후 publication identity와 plan fingerprint 불변 확인 |
| A08 | `authored_amount_publication_remains_blocked_while_static_support_is_truthful`: public DamageExpr가 계속 `content_capability_unsupported`; describe 지원 문구도 대조 |
| A09 | 실제 `author_content explain` 성공, `TrialSession::explain` direct 위임 동일성, 전체 target에서 기존 compiled boss graph와 공간 preview 회귀 통과 |
| A10 | public 시험과 실제 CLI에서 numeric evaluation/trace/final damage 부재, `performed=false`, 기존 literal effect 보존 확인 |

## 4. 최종 검증 원자료

모든 최종 제품 명령의 source before/after digest는
`5279b8333c6be8f90992ac94a47f09beb083bf82bfcdf02ebbe3e0d179c8f530`로 같다.

| 영수증 | 명령/결과 |
| --- | --- |
| `format_final_004.json` | 변경 영역과 신규 모듈 rustfmt 검사, exit 0 |
| `check_all_final_001.json` | `cargo check -p game_core --all-targets --offline --locked`, exit 0, 14.5초 |
| `authoring_focused_final_001.json` | `cargo test -p game_core --lib authoring --offline --locked`, 110 passed / 0 failed, 133.8초 |
| `value_expr_final_001.json` | `cargo test -p game_core --lib value_expr --offline --locked`, 88 passed / 0 failed, 7.8초 |
| `all_targets_parallel_final_001.json` | 지시서 원문의 기본 병렬 `cargo test -p game_core --all-targets --offline --locked`, **1,533 passed / 0 failed**, 106.6초 |
| `clippy_final_001.json` | 지정 allow 두 개를 적용한 all-target clippy, exit 0, 33.5초. 기존 warning은 log에 보존 |
| `package_current_cli_fixture_001.json` | 현재 contract `catalog_schema=3, execution=2` 공식 fixture 생성, exit 0 |
| `cli_effect_explain_current_final_001.json` | 실제 `author_content explain TRUSTED_ROOT SELECTION REQUEST`, exit 0, 3.6초 |
| `trial_delegation_002.json` | 교정된 TrialSession 위임 exact 시험 1/1 |
| `probe_static_explain_usage_final_004.json` | 별도 target의 공개 API cost probe, exit 0 |

사용자가 별도로 확인한 기본 병렬 시간 원자료와 첫 sandbox EPERM은
`reviews/wp02_d07_test_timing_001/`에 있다. 구현자의 최종 전체 시험은 그 보고서와 별개로 다시 실행했다.
SIGBUS는 어느 보존 로그에서도 관찰되지 않았다. 상세 실패·복구는 `FAILURE_HISTORY.md`에 있다.

## 5. 비용 원자료

현재 공식 fixture의 `unknown_distortion_strike/strike/effect_01` public 설명에서:

- query usage: work `25,828`, metadata peak `1,153` bytes, result peak/document `2,995` bytes,
  나머지 pending/bucket/effect/geometry/entity/definition/clone peak는 모두 0.
- 1 미만 경계:
  - work limit `25,827` → `authoring_work_limit`, requested `25,828`
  - metadata limit `1,152` → `authoring_metadata_limit`, requested `1,153`
  - result limit `2,994` → `authoring_result_limit`, requested `2,995`
- 동일 descriptor의 plan cost: nodes 1, max depth/stack/task frames 1, stack slots 1,
  static/worst work 1, dynamic false, compile work 49, compile reserved 2,318 bytes,
  retained 1,382 bytes, authored members 0.
- semantic fingerprint:
  `2d43b2f0fe9675a2b0ad5309aeda5d85e29a64390bd0039fb708d1ccc3e8143c`.

원문은 `probe_static_explain_usage_final_004.log`와
`cli_effect_explain_current_final_001.log`에 있다. probe는
`.cache/d07-static-explain-probe-target`, 제품 검증은
`.cache/d07-static-explain-product-target`을 사용했다.

## 6. 검토 시 확인점과 잔여 범위

- target의 currentness와 forward/reverse patch가 신규 파일의 부재↔존재까지 byte 단위로 복원하는지 확인한다.
- 실제 catalog entry와 effect source/kind/site 검증, borrowed segment 직렬화, 공통 context 분류,
  lookup/descriptor 계량 및 낮은 한도 실패를 우선 검토한다.
- D07-B actual frame 숫자/trace 관측, D07-C/WP03 저장·최종 입력, WP04 전체 vertical slice는 남아 있다.
- authored expression production publication 차단은 유지된다.
- PC36/R05는 runtime 제품 경로와 소유/저장 구조를 바꾸지 않아 이번 D07-A에서 재측정하지 않았다.
- 안전 초과 시 스킬 자동 취소와 20분 시험은 범위 밖이며 수행하지 않았다.
- 독립 검토 PASS, D07 전체 완료, WP02/R06 전체 완료를 주장하지 않는다.
