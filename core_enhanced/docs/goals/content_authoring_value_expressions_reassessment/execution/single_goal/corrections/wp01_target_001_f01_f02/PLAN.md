# R06 WP01 F01/F02 교정 PLAN

Current state: `교정·자체 검증 종료 / 교정 타깃 동결 준비 / 재검토 요청 대기`

## Identity와 제한적 교정 권한

- Exact owner: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, `HA-CAVER-P2G60-001`.
- Applied rules: `core_enhanced/docs/goal_execution_pipeline(1).md`, revision `2026-09-13-gex-r7`, §6.6 제한적 교정 및 GEX-10.
- Predecessor target: `R06_WP01_CONTENT_INPUT_COMPILE_TARGET_001`, manifest SHA-256 `6f55cfa0fae0d88971eba6cb3c47e39ab5d12a2e99b2656a40eb2b51e7cf76de`.
- Finding source: `reviews/wp01_content_input_compile_001_review_001/REVIEW.md`의 `WP01-F01`, `WP01-F02`.
- 현재 요청은 두 finding의 최소 교정, 자체 검증, 별도 target 동결과 재검토 인계만 허용한다. WP02, 다음 R06 기능, runtime fallback/연결, GEX-20/30, WP/Phase/Goal 완료는 시작하거나 선언하지 않는다.

## 진입/currentness/baseline

- 기존 reviewer verifier를 읽기 전용으로 실행했고 target manifest·313 source files·before/after/records/evidence archive·441 reference hash가 모두 일치했다.
- 새 correction baseline은 `baseline/pre_fix_001/MANIFEST.json`이다. predecessor source 313개의 actual bytes, 기존 실행 문서, review inputs, dirty 상태를 보존하고 이 correction의 새 기록 경로가 사전 부재였음을 기록한다.
- 기존 target, baseline, review 보고서·probe·실패 evidence는 수정하지 않는다. 원본 `game_resources`는 fixture 출력으로 사용하지 않는다.

## 허용 변경과 설계 경계

1. `WP01-F01`: `WireValueExpr`의 모든 struct variant 및 `Read`가 도달하는 authored reference payload를 조사해 Serde 단계에서 unknown field를 거부한다. 기존 literal 문법, byte/depth/node/member admission, source locator와 진단을 유지하고 전체 RON 재parse나 사전 AST clone을 추가하지 않는다.
2. `WP01-F02`: 참조가 있다는 사실이 아니라 외부 frame 없이 평가 가능한지를 분류한다. authored aggregate가 제공하는 `AggregateMember`만 lexical/local하게 충족되는 plan은 기존 bounded evaluator로 정적 평가하고 Damage `u32` / Heal `i32` narrowing을 적용한다. 외부 stat/resource/result 또는 local binding 밖 member read는 dynamic으로 남긴다. 별도 산술 의미나 임의 기본값을 만들지 않는다.
3. 변경은 ValueExpr parse/catalog와 직접 회귀 테스트·교정 기록/도구에 한정한다. publication gate와 fingerprint 의미를 유지한다.

## 검증·동결

- 수정 전 기존 review assertions를 새 `/tmp` fixture 경로에서 직접 재실행해 두 실패를 보존한다.
- 수정 후 동일 assertions을 통과시키고 주요 struct/reference unknown fields, 정상 식, static/nested aggregate, Damage/Heal narrowing, static fault, 실제 dynamic 대조군과 source identity 회귀를 실행한다.
- ValueExpr, content pipeline과 영향 회귀, `cargo check -p game_core --all-targets --offline --locked`, focused rustfmt, 기존 allow 두 개만 사용한 Clippy를 직접 실행한다.
- 교정 후 actual-byte source/record/evidence archives, binary-capable recoverable delta, manifest와 read-only verifier를 별도 target에 고정한다.

## 종료점

고정 문구는 `교정·자체 검증 종료 / 교정 타깃 동결 / 재검토 요청 대기`다. 자체 검증 성공을 재검토 PASS로 표시하지 않는다.

## 실제 종료 경계

- 제품 source 변경은 `value_expr/parse.rs`, `value_expr/mod.rs`, `value_expr/catalog.rs`, `content_pack/tests.rs` 네 파일뿐이다.
- 수정 전 기대 실패와 수정 후 동일 probe 성공, focused/broader 회귀, all-target check, rustfmt, 동일 baseline allow의 Clippy를 직접 실행했다.
- WP02 publication block은 정상 정적·동적 표현식 대조군 모두에서 `content_capability_unsupported`로 유지됐다.
- 별도 correction target을 생성한 뒤 source·records·evidence mutation을 중단하고 read-only currentness만 실행한다.
