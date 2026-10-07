# R06 WP01 checkpoint 001 단계검토

Verdict: `CORRECTION_REQUIRED`

이 판정은 구현자 자체 단계검토이며 독립 재검토 PASS가 아니다. checkpoint 001 target의 detached hash와 archive/current bytes 검증은 PASS했지만, 다음 WP01 검증 확장 전에 교정해야 할 보호장치 2건을 찾았다.

## Currentness

- Target: `R06_WP01_CONTENT_INPUT_CHECKPOINT_001`
- Manifest SHA-256: `8a707d4bbfd7cc65ba0b37a7e712a1e5596ff57511cc3766a007ee3e6bca0bbb`
- `stage_001_target_verify_entry`: exit 0, source 313, added 2 / modified 14 / deleted 0, frozen validation receipts 3
- source archive, baseline, records, evidence, old V7/literal fixtures 모두 target row와 일치했다.

## SR01 — compile admission 전 expression AST clone

Severity: `blocking`

`ValueExprCatalog::compile_skills`가 `DamageExpr/HealExpr.amount.clone()`으로 전체 AST를 먼저 복제한 뒤 공통 `compile`을 호출한다. 이는 F01과 이번 요청의 “복제·정렬·할당 전에 비용 승인”을 위반한다. parser input/node limit이 있더라도 compiler의 retained/compile-work admission보다 앞선 kernel-owned deep clone을 정당화하지 못한다.

Required correction: authored expression은 원본을 borrow해 compile하고, 고정 크기 legacy lowering만 local owned value로 유지한다. clone 제거를 구조/probe로 검증한다.

## SR02 — bounded parse 실패 뒤 일반 diagnostic reparse

Severity: `blocking`

`enrich_parse_error`는 primary bounded parse 오류 뒤 `ron::de::from_str::<Locations>(input)`을 실행한다. shadow type이 `op`를 `IgnoredAny`로 건너뛰므로 custom `ValueExpr` node/depth visitor를 다시 사용하지 않으며, 전체 skills/steps/effects Vec도 별도 count admission 없이 만든다. 따라서 bounded rejection 뒤 allocation/recursive skip 우회 경로가 생긴다.

Required correction: 이 full diagnostic reparse를 제거한다. primary serde path와 source file은 그대로 유지하고, definition/action/effect identity가 필요하면 allocation-bounded lexical locator 또는 primary bounded parse 자체의 진행 정보에서 얻는다. hostile depth/node/input 실패가 secondary deserialize를 호출하지 않는 regression을 둔다.

## 계속 진행 조건

SR01/SR02를 successor source에서 교정하고 focused tests/build를 통과하기 전 external pack/publication 회귀 확장으로 넘어가지 않는다. checkpoint 001 자체는 수정하지 않는다. 이 단계검토는 R06/WP/Phase/Goal 완료나 GEX 진입을 승인하지 않는다.

Raw locator: `evidence/validation/stage_001_review_probe.{json,log}`.
