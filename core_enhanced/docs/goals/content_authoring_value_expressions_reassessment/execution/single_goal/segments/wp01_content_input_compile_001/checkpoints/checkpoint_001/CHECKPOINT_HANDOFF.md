# R06 WP01 단계 checkpoint 001 인계

State: `부분 구현 checkpoint 동결 / 단계검토 대기`

## 경계

이 checkpoint는 실제 skill RON의 bounded expression parse, final skill definition의 공통 kernel compile, immutable `ValueExprCatalog`, authored expression runtime publication 차단까지를 검토한다. 다음 큰 WP01 검증 확장 전에 동결한다.

완료를 주장하지 않는다.

- R06, WP01, Phase, Goal 완료: false
- GEX-20/30 진입: false
- WP02 battle evaluator/runtime cutover: 없음
- production에서 expression 실행 가능 capability: false

## 직접 검증

- `stage_001_value_expr_unit`: 29 passed / 0 failed
- `stage_001_all_targets_check`: exit 0 / 기존 warning 6
- `stage_001_focused_fmt`: exit 0

구현 중 recorder 이전 실패(`cargo check` compile 오류, 첫 신규 시험 23/29, repo-wide fmt baseline 차이)는 `EXPERIMENTS.md`에 별도로 남겼으며 성공 증거로 세지 않는다.

## 단계 reviewer focus

1. RON 0.5.1 custom visitor가 node/member를 owned AST push 전에 거부하고 i64/u64만 i128로 무손실 확장하는가.
2. acquisition scanner가 expression span을 snapshot clone 전에 검사하고 strings/comments/raw strings를 오인하지 않는가.
3. final winning effect source가 compile diagnostic과 catalog key에 유지되는가.
4. literal plan은 runtime publication 가능하고 authored expression만 명시적으로 차단되는가.
5. runtime match의 invariant arm이 fallback이나 빈 executor를 만들지 않는가.

이 checkpoint 승인 여부는 다음 WP01 시험 확장 진행만 통제한다. R06 또는 WP 전체 PASS가 아니다.
