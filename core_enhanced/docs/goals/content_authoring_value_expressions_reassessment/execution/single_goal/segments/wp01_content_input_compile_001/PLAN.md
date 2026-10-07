# R06 WP01 콘텐츠 입력·검증·컴파일 구간 PLAN

Current state: `부분 구현·검증 종료 / 단계검토 요청 대기`

## Identity와 권한

- Exact goal: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`
- Handoff: `HA-CAVER-P2G60-001`
- Applied pipeline: `core_enhanced/docs/goal_execution_pipeline(1).md`, `2026-09-13-gex-r7`
- Run mode: 기존 goal-run의 명시적 후속 구현 구간. 사용자는 2026-09-13 요청에서 구현, 자체 검증, 새 target 동결, 재검토 인계까지만 허용했다.
- Predecessor implementation target: `R06_CHECKPOINT_001_CORRECTION_TARGET_002`, manifest SHA-256 `1bedd79cf745bb489db18172b3c29bc3477f728fa9849d2b4e36f357224ff85a`.
- Predecessor review: `reviews/correction_002_review_001/REVIEW.md`의 계산 커널 부분 범위 PASS. R06 전체 PASS나 다음 구간 자동 승인이 아니다.
- Execution-document successor: 이 디렉터리의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`. 첫 checkpoint 당시 동결된 상위 `PLAN.md`, `CHECKPOINT_REVIEW_HANDOFF.md`, corrections/reviews/evidence는 수정하지 않는다.

## 이번 구현 경계

포함:

- 승인된 `DamageExpr`/`HealExpr` authored RON surface와 `AmountExpr { moment, root }` 의미
- acquisition 단계의 per-expression input byte 제한과 parser nesting 제한
- AST를 만들기 전에 node/depth/member 수를 거부하는 bounded Serde visitor/seed 동등 경로
- 실제 skill RON parse, edit/reference validation, delivery capability validation, 공통 ValueExpr compiler 호출
- legacy Damage/Heal literal도 같은 immutable plan catalog에 lowering
- source-bound compile diagnostic과 중복/missing EffectSite 거부
- runtime consumer가 없는 authored expression의 production publication 차단

제외:

- WP02의 battle `EvalFrame`, Snapshot capture, Live read, resource lease, result prefix, sequential fallible effect execution과 rollback
- expression amount의 실제 Damage/Heal command 실행 및 quantitative usefulness/preview/explain
- WP03 Attack/SpellPower, weapon/copy enhancement, cadence, save schema/fingerprint 전환
- WP04 전체 public 실행 capability와 V01-V21 전체 종료
- 다음 R06 기능, R07+, GEX-20/30, WP/Phase/Goal 완료 선언

## 구현 전 baseline과 currentness

- Baseline: `baseline/pre_implementation_001/MANIFEST.json`, record `R06_WP01_CONTENT_INPUT_COMPILE_BASELINE_001`, SHA-256 `e884bc38014d1ce05dcede78a495c9ae6c888b2e9d99ca10c1d58ba4a22fe6ec`.
- Actual bytes archive: `baseline/pre_implementation_001/SOURCE_BEFORE_WP01_CONTENT_INPUT_COMPILE_001.tar.gz`.
- correction_002 read-only verifier는 baseline 직전 PASS했고 지정 manifest SHA와 일치했다.
- 기존 311-file source scope 전체와 실제 영향 파일 22개를 함께 고정했다. 새 parser/catalog/test/tool/구간 문서 경로 8개는 구현 전 부재로 기록했다.
- old-build V7 fixture SHA-256 `47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9`, literal RON fixture SHA-256 `1bd51a81d6b528ea4a80895bf95348c8417fd7113c77a73c6a2543d7821cf8c0`; 이 구간에서 덮어쓰지 않는다.
- 기존 대규모 dirty/untracked work는 baseline status와 archive에 보존하며 이번 성과로 귀속하지 않는다.

## 상세 설계와 계약 대조

1. RON 0.5.1은 `Deserializer`와 Serde `DeserializeSeed`/visitor 호출은 제공하지만 자체 depth limiter가 없다. 기존 snapshot delimiter scanner를 유지하고, skill catalog acquisition에서 expression span byte 제한을 AST parse와 snapshot string clone보다 먼저 적용한다.
2. `ValueExpr::Deserialize`는 각 node 진입 전에 thread-local parse meter의 depth/node를 승인하고, `FiniteSource::AuthoredMembers`는 각 원소를 push하기 전에 member 한도를 승인한다. meter는 expression마다 초기화되고 host `InputLimits`보다 넓어질 수 없다. compiler의 iterative depth/node/work/retained admission은 그대로 다시 적용한다.
   RON 0.5.1은 `deserialize_i128`을 지원하지 않으므로 authored integer visitor는 실제 parser가 제공하는 signed/unsigned 64-bit integer만 받아 signed i128 kernel 값으로 무손실 확장한다. float나 문자열 숫자, 숨은 narrowing은 받지 않는다.
3. authored form은 `DamageExpr { amount: AmountExpr, damage_type }`, `HealExpr { amount: AmountExpr }`이며 `AmountExpr`은 `moment`와 `root`만 입력받는다. provenance는 author가 쓰지 못하고 `AuthoredExpression`으로 설정된다. 기존 `Damage`/`Heal` spelling과 serialization은 변경하지 않는다.
4. 최종 winning skill definitions에 source를 bind한 뒤, action/effect 순서로 literal과 expression 모두 공통 `compile`을 호출해 immutable `ValueExprCatalog`를 만든다. constant plan은 공통 evaluator+narrowing으로 compile-time final range까지 검사한다. 별도 산술 검증기는 만들지 않는다.
5. delivery capability는 현재 schema의 target/delivery facts에서 보수적으로 산출한다. 이는 WP02 runtime capability 발급이 아니며 모든 authored expression은 compile 후 publication boundary에서 `content_capability_unsupported`로 명시적으로 차단한다. 공식 literal-only generation은 catalog를 보유하되 기존 runtime literal path를 유지한다.
6. 새 enum variant가 publication 우회 fixture를 통해 runtime에 들어오면 기존 literal이나 0으로 대체하지 않고 invariant failure로 중단한다. production loader의 차단이 주 안전 경계다.

## 단계검토 중단 조건

checkpoint 위치·개수·파일 수는 사전 완료 단위로 고정하지 않는다. 구현 중 아래를 처음 함께 만족하면 다음 큰 WP02 연결 전에 멈춘다.

1. 실제 RON acquisition→bounded parse→skill validation→공통 compile→immutable catalog/publication 차단을 하나의 책임으로 검토할 수 있다.
2. crate build, 해당 경로의 boundary/negative/legacy tests와 영향받은 회귀가 통과한다.
3. parse/compile 가능한 expression과 runtime 실행 가능 capability가 명시적으로 분리된다.
4. 미지원 expression은 production publication이나 silent fallback으로 노출되지 않는다.
5. baseline/target actual bytes, recoverable delta, 명령·exit·원시 출력과 실패 이력을 고정할 수 있다.

핵심 설계 의미, compatibility, Phase/Goal 경계를 바꿔야 하거나 설명되지 않은 drift/overlap이 생기면 target 완료를 주장하지 않고 근거·대안과 함께 중단한다.

## 검증 계획

- actual RON: 정상 expression compile plan, legacy literal plan/bytes, invalid unit/reference/moment/rounding/syntax와 exact source/action/effect/field diagnostic
- limits: input bytes, parser nesting, expression depth/node/member의 `limit-1 / limit / limit+1`; 깊고 넓고 긴 payload의 pre-allocation rejection 경로
- publication: compiled but runtime-unconnected expression의 explicit rejection, prior accepted generation 불변
- kernel: current `game::value_expr` units와 correction/re-audit regression
- affected content/authoring/skill runtime/save fixture regression과 `cargo check -p game_core --all-targets`
- focused rustfmt; Clippy는 correction_002가 고정한 baseline `clippy::never_loop`, `clippy::eq_op`만 command-line allow하고 새 lint는 허용하지 않는다.

각 직접 실행 명령은 cwd, exit, nonzero test 수, 기대/실제, raw log와 receipt를 보존한다. correction_002 target/review evidence 재사용은 직접 재실행과 분리한다.

## 종료점

위 중단 조건을 만족한 source·test·record·evidence를 새 read-only target으로 고정하고 verifier로 확인한다. 이후 mutation을 멈추고 `부분 구현·검증 종료 / 단계검토 요청 대기`로 인계한다. 재검토 PASS나 후속 구현은 이 pass에서 수행하지 않는다.

## 실제 단계검토 이력과 최종 중단

- 의미 있는 첫 경계는 `checkpoints/checkpoint_001/`에 동결했다. 이 target과 당시 PLAN/checkpoint/evidence는 후속 수정하지 않았다.
- checkpoint 자체검토 `reviews/checkpoint_001_self_review/REVIEW.md`는 AST clone 선행과 전체 RON 진단 재parse 우회 2건을 찾아 `CORRECTION_REQUIRED`로 중단했다.
- 후속 교정은 authored AST borrow와 bounded lexical source locator로 두 우회를 제거했다. `CORRECTION_REVIEW.md`의 수용은 WP01 계속 진행만 허용하며 R06/WP PASS가 아니다.
- external pack winner/source relocation, production publication 차단, literal/content/save 회귀까지 완료한 현재 경계가 이 구현 구간의 새 동결점이다. WP02 실행 연결은 시작하지 않았다.
- 검토 진입 문서는 `REVIEW_HANDOFF.md`, 동결 bytes는 `target/target_001/`이 소유한다. 이 상태에서 mutation과 다음 구현을 중단한다.
