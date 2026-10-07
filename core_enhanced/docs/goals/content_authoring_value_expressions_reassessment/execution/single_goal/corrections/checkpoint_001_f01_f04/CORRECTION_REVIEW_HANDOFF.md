# R06 checkpoint 001 F01–F04 교정 재검토 인계

State: **교정본 재검토 준비 완료 / 재검토 요청 대기**

## 검토 대상

- Handoff / Goal: `HA-CAVER-P2G60-001 / GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`
- 적용 pipeline: `2026-09-13-gex-r7`
- 원 target: `R06_STAGE_REVIEW_TARGET_001`, manifest SHA-256 `e62085bbcbdb8cfe9108f640c7d61859a0e12c29aae8171fa62d64b98474bfe1`
- 원 review: `reviews/checkpoint_001_review_001/REVIEW.md`, SHA-256 `5239bf47318462c214feaa94d091e9b4e397ac36d2a5168e26c6411ffc8d8f31`
- 수정 전 baseline: `corrections/checkpoint_001_f01_f04/baseline/pre_fix_001/MANIFEST.json`, SHA-256 `2dacf1988cfa7bc5b30bbdf39190b4690a6dff79435407dfa1791eb8915fdee9`
- 새 target: `corrections/checkpoint_001_f01_f04/target/correction_001/MANIFEST.json`

원 checkpoint와 review 디렉터리는 수정하지 않았다. 수정 전 source 311개의 실제 bytes는 `SOURCE_BEFORE_FIX_001.tar.gz`에, 수정 후 bytes는 새 target의 `SOURCE_AFTER_CORRECTION_001.tar.gz`에 보존한다. 두 archive와 `CORRECTION_DELTA.patch`가 복구 및 비교 기준이다.

## 단계검토와 변경 판단

수정 전에 parent manifest/detached hash, source 311개, review 증거 7개의 currentness를 확인했다. 원 unit 11건은 통과했고 원 review probe는 2 pass / 5 fail로 보고서의 F01–F04를 재현했다. 이후 네 finding이 공유하는 admission/cost 경계를 구현하고, 경계·할당·회귀 검증이 모두 통과한 지점에서 mutation을 중단했다.

핵심 설계인 typed AST → compiled immutable plan → evaluation, 단위 검사, signed rounding, checked overflow, literal 호환, 정상 operand/group 순서와 fingerprint 의미는 유지했다. 설계 핵심 원칙을 바꿀 사유는 발생하지 않았다.

| Finding | 교정 내용 | 재검토 초점 |
|---|---|---|
| F01 | compile/evaluate가 raw cardinality와 가변 payload work를 복제·정렬 전에 checked admission한다. compile work를 runtime work와 분리하고 plan/task/stack/group/trace의 새 저장 공간을 allocation 전에 보수적으로 계측한다. raw count와 canonical member count를 별도 usage로 기록한다. | 긴 identifier, 대량 duplicate, limit−1/limit/limit+1, bounded diagnostic, 계측값과 allocator peak가 함께 안전한지 확인 |
| F02 | Add 직접 자식 수가 남은 node budget을 넘으면 즉시 거부하고 `ContinueAdd` frame으로 한 자식씩 source order로 방문한다. fingerprint는 임시 byte Vec 없이 streaming SHA-256으로 계산한다. | 65,536-wide Add 조기 거부 시 peak allocation, left-to-right overflow 및 fingerprint 유지 확인 |
| F03 | plan에 nested simultaneous `max_stack`, `stack_storage_slots`, `max_task_frames`, 실제 nodes/depth를 기록한다. 평가 입구와 매 push에서 더 작은 runtime meter를 적용하고 task/group는 별도 retained/structural limit으로 보호한다. | parent가 값을 보유한 nested aggregate, stack 4/task frame 3의 경계와 runtime peak 일치 확인 |
| F04 | Resource는 항상 `resources`를 요구하고, TargetEmployee는 Snapshot/Live에 따라 target-at-release/consume을 추가로 요구한다. source resource에는 target 조건을 붙이지 않는다. | source/target × Snapshot/Live × capability matrix와 실행 시 값·unit 검사 유지 확인 |

source scope의 수정 파일은 다음 네 개다.

- `core_enhanced/src/game/value_expr/compile.rs`
- `core_enhanced/src/game/value_expr/eval.rs`
- `core_enhanced/src/game/value_expr/mod.rs`
- `core_enhanced/src/game/value_expr/tests.rs`

## 검증 인계

| 검증 | 결과 | 증거 |
|---|---:|---|
| 수정 전 value_expr unit | 11 pass | `pre_fix_original_unit.{json,log}` |
| 수정 전 원 review probe | 2 pass / 5 expected fail | `pre_fix_review_probe.{json,log}` |
| 수정 후 value_expr unit | 18 pass | `post_fix_value_expr_unit.{json,log}` |
| 수정 후 correction allocator/repro probe | 7 pass | `post_fix_correction_probe.{json,log}` |
| `cargo test -p game_core --all-targets --offline --locked` | lib 1,378 pass + integration 52 pass | `post_fix_all_targets_test.{json,log}` |
| all-target check | pass | `post_fix_all_targets_check.{json,log}` |
| focused rustfmt check | pass | `post_fix_focused_fmt.{json,log}` |
| Clippy with checkpoint baseline allows | pass; repository baseline warnings only | `post_fix_clippy_with_baseline_allows.{json,log}` |
| frozen-before/current fingerprint comparison | 3/3 exact byte match | `post_fix_fingerprint_compat.{json,log}` |

Correction probe의 주요 관측은 wide Add 65,536개를 node limit 4 / retained limit 1,024에서 `BudgetExceeded`로 거부하면서 추가 live heap peak가 244 bytes였고, duplicate raw member 16,384개는 canonical count를 올리거나 index buffer를 만들기 전에 work 16,385에서 거부하면서 peak가 464 bytes였다는 것이다. 긴 owner도 short-plan exact retained limit에서 compile 전에 거부되며 peak 500 bytes였다. 이는 결정적 예산 계측 테스트를 특정 allocator의 정확한 바이트 수에 종속시키지 않으면서 기존 probe와 함께 확인한 결과다.

샌드박스 all-target 첫 실행은 Unix socket 생성 테스트에서 OS `EPERM`이 발생해 1,377 pass / 1 fail이었다. 같은 source와 같은 명령을 승인된 환경에서 재실행한 영수증은 lib 1,378건 및 integration 52건 전부 통과를 기록한다. 실패 receipt도 환경 차이를 숨기지 않기 위해 `post_fix_all_targets_test_sandbox`로 보존했다.

Fingerprint verifier 최초 시도는 격리한 이전 workspace에서 lockfile workspace membership을 바꾼 뒤 `--locked`를 사용한 verifier 결함으로 종료됐다. 제품 fingerprint 비교에 도달하기 전 실패였으며, old build에서만 부적합한 flag를 제거한 뒤 세 표본이 모두 일치했다. 최초 receipt는 `post_fix_fingerprint_compat_attempt_001`로 함께 보존했다.

## 남은 제약과 중단 경계

retained 모델은 process RSS나 allocator metadata의 정확한 측정 계약이 아니라 커널이 실행 중 새로 확보하는 저장 공간의 보수적 누적 admission이다. 외부 frame/plan이 이미 가진 메모리는 구분하며, 그 payload를 검사하는 work는 실행 시 부과한다. 실제 resource lease, authored RON/catalog, battle Snapshot/Live 연결, 저장 구조 변경은 후속 통합 책임이다.

이 인계는 **F01–F04 교정본의 재검토 요청**이다. 다음 R06 기능, GEX-20/30, WP/Phase/Goal 완료를 선언하거나 시작하지 않는다. 재검토 결과가 나오기 전에는 correction source·record·evidence를 변경하지 않는다.
