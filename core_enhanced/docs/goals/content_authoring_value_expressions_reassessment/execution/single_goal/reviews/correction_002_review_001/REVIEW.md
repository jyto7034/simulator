# R06 correction 002 재감사

검토일: 2026-09-13 · **부분 범위 PASS** · 새 확정 findings: **0건**.
이전 재감사 R01/R02/R03은 아래 근거로 현재 correction target에서 해소한다.

대상: `R06_CHECKPOINT_001_CORRECTION_TARGET_002`.
Manifest SHA-256: `1bedd79cf745bb489db18172b3c29bc3477f728fa9849d2b4e36f357224ff85a`.
Before baseline SHA-256: `96eb47695ea346627b5c0c6aedbd75953cf496353ad00b03b04d37bd80232bda`.
Goal: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`.

## 범위와 독립성

사용자의 correction_002 재감사 요청에 따라 인계서·교정 기록·실험 기록, 변경 source 4파일의 교정 경로와 테스트, snapshot/보존 도구, 이전 재감사 및 GEX gex-r7의 부분검토·전후 고정 규칙을 확인했다.

이 reviewer는 이전 검토와 교정 방향·인수인계 지시서 작성에 참여했으나 이번 제품 코드 교정과 구현자 테스트 작성에는 참여하지 않았다. 완전한 blind review로 표시하지 않는다. 추가 검증 코드는 동결 target 밖의 진단 파일이다.

이번 PASS는 checkpoint 001 계산 커널의 누적 교정과 R01–R03 보완에 한정한다. R06 전체 구현, authored RON/parser/catalog, 실제 battle Snapshot/Live·resource lease·rollback, stat/weapon/save 및 public integration 완료를 승인하지 않는다.

## R01 — 비용 상한 불일치: 해소

[공통 비용 helper](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/mod.rs:644)를 compiler와 evaluator가 함께 사용한다. AuthoredMembers의 raw 수, identity/owner/value payload, 정렬과 reduction 비용을 계산하고, member 실행 비용을 중첩 횟수에 따라 checked arithmetic으로 합성한다.

PlanCost의 계약도 명시됐다.

- `runtime_dynamic_work=false`: `static_work == worst_case_work`, 알려진 입력의 실행 비용 상한.
- `runtime_dynamic_work=true`: static_work는 구조적으로 산정한 부분이며, worst_case_work는 실제 비용 예측값이 아니라 승인된 max_work ceiling. 실제 frame/resolved payload 추가 비용은 runtime meter로 제한한다.
- 정적 식과 무관한 frame 내용을 순회해 비용을 부과하지 않도록 frame admission을 필요한 reference에 한정한다.

기존 [실패 재현을 수정 없이 재실행](REVIEW_PROBES.log)한 결과:

- 1-member 사례: 계획 7 / 실제 7, 같은 예산 7로 compile/evaluate 성공.
- count 0/1/2/8/32 × identity prefix 0/32/128의 15조합 모두 bound와 actual 일치. 이전 32×128 사례도 963/963이다.
- 추가한 seeded 정적 표현식 200개에서 독립 계산 결과 oracle, 실제 work ≤ static bound, 동일 work 예산 compile/evaluate, 중첩 stack 한도 검사를 모두 통과했다. Add/Scalar-Mul 및 유한 Sum/Max·WholeSet/Owner/Tile 그룹·중첩을 조합한다. 전 입력 공간의 완전한 증명은 아니다.
- 구현자 unit의 empty/grouped/nested/resource payload 및 bound−1/bound/bound+1, dynamic flag 검증도 재실행해 통과했다.

## R02 — native recursion: 해소

[execution_shape](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/compile.rs:1845)는 child segment가 parent보다 뒤에 생성되는 구조를 이용해 역순 bottom-up으로 계산한다. 자기 재귀 호출이 없어졌고, 참조할 shape가 없으면 typed error를 반환한다. Option<ExecutionShape> scratch 공간은 생성 전에 admission한다. 새 hard cap이나 기본 예산 확대는 없다.

[기존 deep probe의 격리 재실행](DEEP_SHAPE_PROBE.log):

- nesting 2,048 / worker native stack 2 MiB / AST ManuallyDrop 조건을 그대로 유지.
- 정상 compile 반환, exit 0.
- nodes=2049, max_depth=2049, max_stack=1, stack_storage_slots=2049, max_task_frames=4097, static_work=worst_case_work=12289.
- 기본 depth 64/65 및 확장 512의 compile/evaluate와 scratch 경계는 현재 unit 재실행으로 확인했다.

이 재현은 컴파일 보조 분석의 native stack 안전성에 관한 것이다. 무제한 깊이의 외부 입력 parsing 또는 caller 소유 AST의 recursive Drop까지 검증했다는 뜻은 아니다.

## R03 — 과거 PLAN 원본 보존: 현재 증거 공백 해소

복구된 `HISTORICAL_PLAN_PRE_FIX_001.md`의 실제 bytes를 확인했다.

- 크기: 4,460 bytes.
- SHA-256: `661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7`.
- 보존된 session apply-patch event의 해당 file content와 byte-identical.
- predecessor baseline의 correction_plan hash/크기와 일치.

확인은 제공된 raw provenance event에서 해당 파일 내용만 추출·대조하는 방식이다. 다른 세션 전체를 추가 탐색하지 않았다. 기존 baseline manifest는 불변이며, 생성 당시 실제 bytes가 archive에 빠졌던 역사적 결함을 소급 PASS로 바꾸지 않았다. successor 복구 기록으로 당시 원본에 접근할 수 있게 보완했다.

이번 baseline/target은 변경할 실행 문서 3개도 before/after archive로 각각 보존한다. 독립 검사에서 source뿐 아니라 이 문서들의 archive 내부 bytes와 exact delta도 확인했다. R03은 '당시 절차가 올바랐다'가 아니라 **현재는 원본을 정확히 복원·대조할 수 있다**는 의미로 해소한다.

## 기존 F01–F04 회귀 확인

이전 correction probes 7개를 포함한 외부 suite를 현재 library에 다시 링크했다.

- 65,536-child Add / node limit 4: 조기 거부, 추가 live heap peak 244 bytes.
- 16,384 duplicate raw member / work limit 32: canonical 평가 전에 거부, peak 464 bytes.
- 긴 authored owner: compile 단계의 memory admission 거부.
- 더 작은 runtime stack과 TargetEmployee Snapshot capability 거부가 유지된다.
- signed rounding 3,096조합과 fingerprint 의미 대조군 통과.
- WholeSet/Owner/Tile × 0/1/8/64 members의 12개 allocation control에서 성공 경로의 추가 live allocation peak가 보고 retained reservation 이하였다.

측정은 reviewer의 직렬 global allocator layout 기반이며 allocator metadata나 RSS를 포함하는 전체 메모리 보장은 아니다.

## 실행·재사용한 검증

직접 실행:

- `cargo build -p game_core --lib --offline --locked`: PASS, 기존 dead_code 경고 5건. 현재 source library로 probes를 링크했다.
- [value_expr unit](UNIT_RERUN.log): **22/22 PASS**.
- [외부 재감사 suite](REVIEW_PROBES.log): **11 PASS / 0 FAIL / 1 ignored**. 기존 재감사 10개 + 추가 200-case matrix 테스트 1개.
- ignored deep probe는 별도 subprocess에서 **1/1 PASS**.
- [진입 currentness](TARGET_CURRENTNESS_ENTRY.json)와 [종료 currentness](TARGET_CURRENTNESS_EXIT.json): PASS.
- 구현자 read-only verifier 실행에 더해 independent archive/delta 검사를 수행했다. source 311, 변경 문서 3, evidence 38, 실행 기록 8, validation receipts 17 및 참조 해시 461회(중복 포함)를 확인했다.
- archive 7개 내부 원본: source before/after 각 311, documents before/after 각 3, evidence 38, predecessor records 10, prior review inputs 11. 빠진/추가/중복 member 없이 일치한다.
- exact source delta는 0 added / 4 modified / 0 deleted. 문서 delta는 3 modified.

현재 target에 결속된 receipt/output hash를 확인해 재사용:

- all-target test lib 1,382 + integration 52.
- all-target check, focused rustfmt.
- Clippy with 기존 `clippy::never_loop`, `clippy::eq_op` allows.
- 별도 before build와 비교한 fingerprint 3종 byte-identical 결과.

이 전체 검증을 이번 reviewer가 다시 실행했다고 세지 않는다. pre-fix R01 실패, R02 SIGABRT, 구현 중 invalid ResourceId fixture 실패도 증거에 유지돼 있다. 기존 production tests를 약화한 변경은 발견하지 못했다.

재현(repository root, output은 새 임시 경로 사용):

```sh
cargo build -p game_core --lib --offline --locked
rustc --edition=2021 --test core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/correction_002_review_001/review_probes.rs --extern game_core=target/debug/libgame_core.rlib -L dependency=target/debug/deps -o <임시 경로>/review_probes
<임시 경로>/review_probes --test-threads=1 --nocapture
ulimit -c 0
<임시 경로>/review_probes --ignored --exact deep_shape_on_normal_worker_stack --test-threads=1 --nocapture
```

## 잔여 경계와 다음 단계

현재 감사 범위에서 추가 교정을 요구할 확정 결함은 발견하지 못했다. 이는 모든 입력·환경에 대한 무결함 증명이나 전체 R06의 안전성 보증이 아니다. 후속 consumer는 static_work와 runtime_dynamic_work/worst_case_work ceiling의 차이를 지켜야 하며 실제 runtime 통합은 별도로 검증해야 한다.

**부분 범위 PASS는 다음 구현 자동 승인과 다르다.** 새 사용자 구현 요청이 오면 currentness와 남은 Goal 범위를 확인하고 새 구간의 baseline을 고정한 뒤 진행한다. 마지막 전체 구현 후 GEX-20/필요한 Phase 감사/GEX-30은 면제되지 않는다.

제품 source·기존 실행 기록·동결 target·기존 evidence는 변경하지 않았다. 이번 review directory 외 제품 mutation, 다음 R06 기능, GEX-20/30 또는 WP/Phase/Goal 완료 선언은 수행하지 않았다.
