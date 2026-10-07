# R06 WP01 콘텐츠 입력·검증·컴파일 단계검토 인계

State: `부분 구현·검증 종료 / 단계검토 요청 대기`

## 검토 대상

- Target: `target/target_001/`
- Manifest: `target/target_001/MANIFEST.json`
- Detached SHA-256: `target/target_001/MANIFEST.sha256`
- Read-only verifier: `authoring_scripts/r06_wp01_verify_target_001.py`
- Baseline: `baseline/pre_implementation_001/`, manifest SHA-256 `e884bc38014d1ce05dcede78a495c9ae6c888b2e9d99ca10c1d58ba4a22fe6ec`
- Predecessor: `R06_CHECKPOINT_001_CORRECTION_TARGET_002`, manifest SHA-256 `1bedd79cf745bb489db18172b3c29bc3477f728fa9849d2b4e36f357224ff85a`
- Prior partial review: `correction_002_review_001/REVIEW.md`; ValueExpr 계산 커널 부분 PASS일 뿐 이번 target 또는 R06 전체 판정이 아니다.

## 구현된 경계

1. 승인된 `DamageExpr`/`HealExpr`와 `AmountExpr { moment, root }`를 기존 skill RON schema에 연결했다. legacy `Damage`/`Heal` spelling과 직렬화는 유지한다.
2. skill acquisition에서 expression span과 parser nesting을 snapshot receipt clone 전에 제한한다. custom Serde visitor/seed는 recursive AST node와 authored member를 owned allocation 전에 승인한다. compiler의 기존 iterative 비용/retained admission은 다시 적용된다.
3. final winning skill의 literal/expression을 공통 compiler로 immutable `ValueExprCatalog`에 연결한다. unit, rounding, capability/moment, finite grouping, resource/result reference, static evaluation, final damage/heal narrowing 오류는 실제 definition/action/effect/field source로 이동한다.
4. authored expression은 compile 뒤에도 runtime capability가 아니다. 모든 production publication 경로가 `content_capability_unsupported`와 compiled fingerprint를 반환하고 prior generation을 유지한다. runtime match 우회에는 silent zero/literal fallback 대신 invariant failure만 있다.
5. WP02 battle evaluation/capture/resource/result 실행은 구현하지 않았다.

## 단계검토 이력

- `checkpoint_001` 자체검토는 compiler admission 전 AST clone과 bounded parse 실패 뒤 전체 RON diagnostic reparse를 발견해 `CORRECTION_REQUIRED`로 중단했다.
- 교정은 borrowed compile input과 bounded lexical source locator다. continuation-only 재검토 뒤 external pack 경로를 연결했다.
- compact serializer의 익명 tuple에서 source identity가 비는 기존 회귀를 추가로 발견했고, type 이름이 아닌 실제 container/ordinal 경계 locator로 수정했다.
- 이 문서는 재검토 결과를 미리 PASS로 선언하지 않는다.

## 검토 집중점

- `value_expr/parse.rs`: input/depth/node/member limit의 `limit-1 / limit / limit+1`, reject seed가 element/member allocation 전에 동작하는지, thread-local meter cleanup과 RON 0.5 integer 범위.
- `content_pack/snapshot.rs`: expression scan이 `bytes.clone()` 전에 실행되며 file/total/nesting 제한을 우회하지 않는지.
- `value_expr/catalog.rs`: authored AST borrow, final winners only, 기존 compiler/evaluator 재사용, capability 누적, resource/result ordering, static narrowing, immutable source/fingerprint.
- `skill_data.rs`와 `content_pack/raw.rs`: 실패 진단 locator가 pretty named tuple과 compact anonymous tuple 모두에서 source/object/action/effect를 보존하고 두 번째 전체 AST를 만들지 않는지.
- publication 경계: external authoring 및 embedded/test live loader에서 authored expression이 실행 가능 상태로 공개되지 않는지; battle 쪽 expression arm이 fallback을 만들지 않는지.
- legacy compatibility: literal canonical RON/V7 fixture 해시와 15개 runtime, 18개 checkpoint/save 회귀.

## 검증과 알려진 제약

- 직접 실행 결과와 raw logs는 `evidence/validation/`에 있다. final current-source 핵심 결과는 ValueExpr 29, content pipeline 59, authoring 15, skill runtime 15, checkpoint/save 18, all-target check와 baseline allow Clippy exit 0이다.
- `stage_002_v7_checksum_regression`은 0 tests인 잘못된 selector라 무효이며 `_rerun`의 1 passed만 사용한다.
- touched legacy 전체 rustfmt는 baseline 미포맷 영역 때문에 exit 1이다. 새 `value_expr` 6파일 focused rustfmt는 exit 0이며, 새 external test block은 재감사 전에 포맷 교정했다.
- Clippy는 predecessor와 동일한 allow 2개로 exit 0이나 baseline warning 집합에 large-error-return 경로 경고가 추가된다. raw log를 검토 대상으로 남긴다.
- old-build fixture는 덮어쓰지 않았고 target verifier가 SHA-256을 다시 확인한다.

## 금지된 확대와 다음 상태

이 target의 판정 범위는 이번 WP01 구현 구간뿐이다. WP02 본격 전투 실행, 다음 R06 기능, R07+, GEX-20/30, WP/Phase/Goal 완료 선언을 이 인계에서 시작하거나 추론하지 않는다. 재검토자가 target bytes를 판정할 때까지 상태는 `부분 구현·검증 종료 / 단계검토 요청 대기`다.
