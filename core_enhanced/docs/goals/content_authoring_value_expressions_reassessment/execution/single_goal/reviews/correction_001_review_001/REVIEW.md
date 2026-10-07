# R06 correction 001 재감사

검토일: 2026-09-13 · 판정: **수정 필요** · 확정 findings: **3건(P2), 코드 2건 / 보존·검증 1건**.

대상: `R06_CHECKPOINT_001_CORRECTION_TARGET_001`.
Manifest SHA-256: `87feab8cdfc20b43d7c49c2f6238bd1e684cb7e1daf4f993efe2f193f01dc45e`.
Goal: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`.

## 범위·검토 독립성

사용자의 재감사 요청에 따라 correction 인계/기록, before/after manifest와 archive, 4개 수정 source, 변경된 tests와 probes, validation receipts, 이전 CRA D08 및 GEX gex-r7의 전후 고정 경계를 검토했다.

이 reviewer는 이전 checkpoint 검토와 이번 교정 설계 방향 정리에 참여했으나 production 교정 및 구현자 테스트 작성에는 참여하지 않았다. 따라서 완전한 blind review로 표시하지 않는다. 아래 추가 probes는 동결 source 밖의 읽기 전용 진단이며 source/evidence를 교정하지 않는다.

이번 범위는 부분 커널과 교정 기록이다. authored RON/catalog, 실제 battle Snapshot/Live/lease/rollback, stat/save 및 후속 R06 기능을 구현하지 않은 사실 자체는 결함으로 세지 않았다. 이 보고서는 GEX-20/30 또는 WP/Phase/Goal 완료 감사가 아니다.

## 기존 finding의 교정 확인

- F01의 원래 대량 duplicate/긴 owner 복제 재현은 차단된다. raw와 canonical 분리, MemberRef 정렬, payload 사전 admission을 확인했다. 16,384 duplicate 입력은 canonical 평가 전에 거부되며 allocator 추가 live peak는 464 bytes였다. 긴 authored owner는 compile 단계에서 거부된다(peak 500 bytes). 다만 **비용 상한 불일치 R01**이 남아 F01 관련 cost 정합성 전체는 닫지 않는다.
- F02의 넓은 Add는 선거부되고 ContinueAdd로 순차 방문한다. 65,536-child / node limit 4의 추가 live peak는 244 bytes였다. 기존 operand 순서를 바꾼 변경은 발견하지 못했다.
- F03의 작은 runtime stack/실제 nodes/depth 비교 및 push-time 제한은 확인했다. 기존 중첩 stack 단위 테스트도 통과한다. 다만 **새 실행 형태 계산의 native recursion R02**가 별도로 발생한다.
- F04는 resources 검사 후 대상 Stat/HP/TargetEmployee Resource에 moment 조건을 누적 적용한다. source 자원에는 target 조건을 요구하지 않는다. 기존 capability matrix 테스트 및 재현 테스트가 통과한다.
- 기존 11개 단위 테스트 본문을 약화한 변경은 없으며 새 7개가 추가됐다. correction probe의 긴 owner 사례는 새로운 compile 조기 거부를 인정하도록 별도 사본에서 바뀌었고, 원 reviewer probe는 불변이다. 이는 올바른 조기 거부를 수용하는 변경이다.

## R01 — [P2] 정적으로 확정된 authored 집계의 worst_case_work가 실제 비용보다 작다

위치: [compile.rs:1282](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/compile.rs:1282), 같은 파일 1271–1316; [eval.rs:570](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/eval.rs:570).

컴파일 비용은 WholeSet에 member당 scan 2회를 잡지만 실행은 raw admission, identity scan, value payload scan을 별도로 부과한다. identity 정렬 비용도 컴파일은 실제 authored identity 길이 합을 사용하지 않는다. 따라서 외부 frame이나 동적 resolved payload가 전혀 없는 경우에도 상한이 아니다.

[독립 재현](review_probes.rs):

- authored member 1개, 짧은 identity, Magnitude 값 1, WholeSet/Sum: `plan.cost().worst_case_work = 6`, 실제 `usage.work = 7`.
- `max_work = 6`으로 같은 식을 compile하면 승인되지만, 동일 한도로 evaluate하면 `BudgetExceeded: evaluation work 7 exceeds limit 6`로 실패한다.
- authored 32개, identity에 128자 prefix: 계획 148, 실제 963.
- 15개 입력 조합 중 nonempty 12개 모두 계획 상한을 넘었다. 0-member 대조군 3개는 일치했다.

실행 meter는 초과 시 거부하므로 이것을 런타임 무제한 실행으로 부풀리지 않는다. 문제는 **정적으로 계산 가능한 비용을 과소평가해 실행 불가능한 계획을 승인하고, PlanCost를 상한으로 사용하는 후속 admission을 신뢰할 수 없게 하는 것**이다. CRA D08은 checked upper bound를 요구한다. correction notes가 설명한 ‘compile 때 알 수 없는 resolved frame payload’ 예외로 authored 입력의 누락은 설명되지 않는다.

수정 방향:

- compile/evaluate가 같은 raw scan·text granule·정렬·reduction 비용 정의를 사용하게 한다.
- AuthoredMembers에서는 이미 아는 identity/owner/value payload를 반영하고, 중첩 반복 횟수까지 checked 상한으로 합성한다.
- 실제로 미확정인 frame/resolved payload는 구조적 비용과 동적 추가 비용을 명확히 나누되, 최소값을 무조건 worst-case라고 표시하지 않는다.
- meter를 느슨하게 만들지 말고 `actual work <= static bound` 및 동일 예산 compile/evaluate 경계 테스트를 추가한다. 현재 unit의 ‘측정한 usage로 재실행’ 검사는 meter 동작은 검증하지만 compiler bound와의 일치를 검증하지 않는다.

## R02 — [P2] 새 segment_shape 재귀가 허용된 확장 depth에서 프로세스를 abort시킨다

위치: [compile.rs:1779](/mnt/f/work/simulator/core_enhanced/src/game/value_expr/compile.rs:1779), 특히 1811의 자기 재귀 호출.

AST lowering은 반복형이지만 교정에서 추가한 실행 형태 분석은 aggregate segment마다 Rust native call stack을 소비한다. ExprLimits::validate는 양수만 확인하므로 caller가 설정한 depth/node 상한을 통과한 깊은 계획도 이 단계에서 프로세스를 종료시킬 수 있다.

[격리 재현 출력](DEEP_SHAPE_PROBE.log):

- 중첩 aggregate 2,048개, 각 authored member 1개, 맨 안쪽 Const(1).
- worker native stack 2 MiB.
- `max_depth = max_nodes = 2049`, retained 64 MiB, compile_work 16,777,216. 나머지는 기본 한도다.
- 입력 AST는 compile 전에 완성했고 `ManuallyDrop`으로 AST 파괴의 재귀를 배제했다.
- compile 진입 출력 후 반환 출력 없이 `fatal runtime error: stack overflow, aborting`, **exit 134**.
- core dump는 해당 진단 subprocess에서 비활성화했다. repository source나 기본 설정을 변경한 것이 아니다.

**이 문제는 기본 max_depth=64에서 재현한 것이 아니다.** 현재 실제 전투 연결도 아직 없다. 공개 커널 API가 허용하는 확장 limits와 native stack 안전성의 불일치라는 범위로 P2를 부여한다. 모든 기본 workload가 crash한다고 주장하지 않는다.

수정 방향: shape 계산도 제한된 명시적 작업 스택 또는 segment 의존 순서에 따른 bottom-up 계산으로 바꾼다. 해당 scratch storage를 admission한다. 대안으로 지원 가능한 hard cap을 명시적으로 검증할 수 있지만, 계약상 허용 범위를 바꾸므로 근거가 필요하다. worker stack을 크게 만드는 것으로만 우회하지 않는다.

재검증은 기본 depth 경계 외에 허용한 확장 depth를 격리 subprocess에서 수행하여 정상 반환 또는 typed 오류를 요구한다. 이 probe는 일반 suite에서 ignored이며 아래 명령으로 따로 실행한다.

## R03 — [P2] baseline이 고정한 수정 전 PLAN bytes의 보존 근거가 빠졌다

위치: [baseline MANIFEST.json:1638](/mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/checkpoint_001_f01_f04/baseline/pre_fix_001/MANIFEST.json:1638), [baseline 생성기:89](/mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/authoring_scripts/r06_correction_baseline.py:89), [교정 verifier:51](/mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/authoring_scripts/r06_verify_correction_target.py:51).

baseline의 `correction_plan`은 live `corrections/checkpoint_001_f01_f04/PLAN.md`를 4,460 bytes / SHA-256 `661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7`로 고정했다. 현재 파일은 5,640 bytes / `5f32687bb6c22206555d072ad93a34b00cac4ce208dbf5c8f9a23c2e4beb0359`이고 단계검토 결과가 추가돼 있다.

현재 PLAN은 새 target의 해시와 일치하므로 **교정 target 동결 이후 drift라고 판단하지 않는다.** source before/after 311개도 정확히 보존돼 있다. 그러나 baseline archive에는 parent source scope만 들어가며 correction PLAN은 포함되지 않는다. baseline 생성기도 이 PLAN의 hash만 기록하고 원본 bytes를 별도 보존하지 않는다. 제공된 baseline/target 자료와 확인한 격리 pre-fix source에서 이전 PLAN 사본을 찾지 못했다.

구현자 verifier는 baseline manifest 자신의 해시를 확인하지만 그 안의 correction_plan 과거 bytes를 검사하지 않으므로 PASS가 나온다. 이는 모든 전후 자료의 복원 가능성을 확인했다는 뜻이 아니다. 사용자가 요구한 ‘구현 전 계획 고정’에 해당하는 증거 연결이 빠져 있다.

수정 방향:

- 위 이전 hash와 일치하는 실제 사본/버전 기록이 있으면 immutable 별도 경로로 보존하고 successor 보완 기록으로 연결한다.
- 없다면 원본 증거 누락을 명시한다. 현재 문서를 수정 전 문서로 소급 표시하거나 기존 baseline hash를 바꾸지 않는다.
- 이후 baseline 생성은 변하는 실행 문서도 실제 bytes로 보존하고, verifier가 현재 자료와 과거 snapshot을 구분해 확인하도록 한다.

## 실행한 검증과 재사용한 증거

- `cargo build -p game_core --lib --offline --locked`: 성공, 기존 dead_code 경고. 현재 source의 rlib로 외부 probes를 링크했다.
- [value_expr 단위 테스트 재실행](UNIT_RERUN.log): **18 passed**.
- [최종 외부 probes](REVIEW_PROBES_FINAL.log): **8 passed / 2 failed / 1 ignored**, exit 101. 기존 correction probes 7개 모두 통과, 추가 allocation control 1개 통과, R01 독립 테스트 2개 실패. ignored R02는 별도 실행하여 exit 134로 재현했다.
- allocation control은 WholeSet/ByOwner/ByProjectedTile × 0/1/8/64 member 총 12조합에서 성공했고, 추가 live allocation peak가 보고 retained reservation 이하였다. 전 입력 공간이나 allocator metadata/RSS 안전성 증명은 아니다.
- rounding control의 signed 3,096조합과 fingerprint semantics control은 성공했다.
- 구현자의 lib 1,378 + integration 52, all-target check, focused fmt, baseline-allow Clippy, before/after fingerprint 3종 증거는 target의 receipt/output hash를 확인하여 재사용했다. 이번 감사에서 전체 suite나 3종 before build를 다시 돌렸다고 세지 않는다. sandbox EPERM/최초 fingerprint verifier 실패 기록도 보존돼 있다.
- [구현자 verifier 재실행](IMPLEMENTER_VERIFIER_RERUN.log)은 PASS. [독립 currentness 진입 검사](TARGET_CURRENTNESS_ENTRY.json)는 target/current source와 archives/delta는 일치하지만 위 과거 PLAN 참조 1건의 불일치를 확인했다.
- 독립 검사는 source scope의 현재 파일 집합, before/after archive 각 311개의 실제 bytes, exact 0 added/4 modified/0 deleted, 원 reviewer 증거 7개, target 기록 10개/evidence 30개와 manifest/receipt 참조를 대조한다. 총 hash 대조 411회(중복 참조 포함).
- 종료 검사 [TARGET_CURRENTNESS_EXIT.json](TARGET_CURRENTNESS_EXIT.json)에서도 같은 과거 PLAN 참조 외 새 drift가 없음을 확인한다. 모든 새 감사 자료는 본 review 디렉터리에만 둔다.

재현 명령(repository root):

```sh
cargo build -p game_core --lib --offline --locked
rustc --edition=2021 --test core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/correction_001_review_001/review_probes.rs --extern game_core=target/debug/libgame_core.rlib -L dependency=target/debug/deps -o <새 임시 경로>/review_probes
<새 임시 경로>/review_probes --test-threads=1 --nocapture
# 별도 프로세스: 예상 abort. core dump 비활성화 후 실행.
ulimit -c 0
<새 임시 경로>/review_probes --ignored --exact deep_shape_on_normal_worker_stack --test-threads=1 --nocapture
```

## 다음 경계

원래 결함의 주요 경로는 실질적으로 개선됐지만 이번 교정 타깃에는 부분 PASS를 주지 않는다. **R01/R02 교정 및 R03 보존 근거 보완 또는 명시적 evidence-gap 처리 → 새 동결 타깃 → 재검토**가 필요하다.

제품 코드, 기존 계획/target/증거는 수정하지 않았고 다음 R06 기능·GEX-20/30을 시작하지 않았다. 교정이나 다음 구현은 별도 사용자 요청을 기다린다.
