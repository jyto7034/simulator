# R06 correction 001 R01–R03 실험 기록

State: `검증 완료 / target 동결 입력`

모든 command, exit code, 원시 output, 실행 시각과 output hash는 `evidence/validation/<label>.json|.log`에 보존한다.

## 이번 pass에서 실행한 검사

| 단계 | label | 결과 |
|---|---|---|
| 수정 전 build | `pre_fix_build` | PASS |
| R01/R02 재현 compile | `pre_fix_reaudit_probe_compile` | PASS |
| R01 재현 | `pre_fix_reaudit_probe` | expected FAIL: 8 pass / 2 fail / 1 ignored; 1-member 6/7 및 12 authored 조합 underbound |
| R02 격리 재현 | `pre_fix_deep_shape_probe` | expected SIGABRT(-6): depth 2,048, 2 MiB worker, AST drop disabled |
| 구현 중 fixture 검사 | `intermediate_unit_invalid_resource_fixture` | FAIL: test fixture `ResourceId`가 128-byte ID 계약 초과; 100 bytes로 수정 후 재실행 성공 |
| 수정 후 build | `post_fix_build` | PASS |
| value_expr unit | `post_fix_value_expr_unit` | PASS: 22/22 |
| predecessor correction probe compile/run | `post_fix_correction_probe_compile`, `post_fix_correction_probe` | PASS: 7/7 |
| 재감사 probe compile/run | `post_fix_reaudit_probe_compile`, `post_fix_reaudit_probe` | PASS: 10/10, deep 1 ignored |
| R02 격리 후검사 | `post_fix_deep_shape_probe` | PASS: depth 2,048, 2 MiB worker, 정상 `PlanCost` 반환 |
| fingerprint | `post_fix_fingerprint_compat` | PASS: 3/3 byte-identical |
| all-target check | `post_fix_all_targets_check` | PASS; 기존 dead-code warning 5건 |
| focused rustfmt | `post_fix_focused_fmt` | PASS: 변경 source 4개 |
| Clippy | `post_fix_clippy_with_baseline_allows` | PASS: `-A clippy::never_loop -A clippy::eq_op`; 기존 baseline allow를 명시 |
| all-target test | `post_fix_all_targets_test` | PASS: lib 1,382 + integration 52, failures 0 |

## R01 경계 결과

- reviewer의 1-member 사례: `static_work=7`, actual `usage.work=7`; 같은 7 meter에서 성공하고 6에서 거부한다.
- count `0/1/2/8/32` × identity length `0/32/128`의 15개 조합에서 정적 bound와 실제 work가 일치했다. 대표 최대 조합은 32×128에서 `963/963`이다.
- empty aggregate, long identity 120, owner identity 96, grouped whole/owner/tile, resource payload 100, nested aggregate를 unit 경계에 포함했다.
- 정적 work `bound-1 / bound / bound+1`에서 compiler 및 evaluator의 거부/승인을 검사했다. 실제 usage를 다시 한도로 넣는 검사와 별도로 `usage.work <= static_work`를 모든 정적 사례에서 확인했다.
- resolved target 및 frame read는 `runtime_dynamic_work=true`; literal/authored-only는 false임을 확인했다.

## R02 경계 결과

- 기본 `max_depth=64`: depth 64 승인, 65 typed `BudgetExceeded`.
- 확장 depth: aggregate nesting 512(depth 513)을 compile/evaluate하고 결과 1, `max_task_frames=1025`를 확인했다.
- shape scratch retained limit의 `required-1 / required / required+1` compile 경계를 검사했다.
- 재감사 격리 probe: nesting 2,048, worker stack 2,097,152 bytes, `ManuallyDrop` 유지. 수정 전 native stack abort에서 수정 후 정상 `PlanCost { nodes: 2049, max_depth: 2049, max_stack: 1, stack_storage_slots: 2049, max_task_frames: 4097, static_work: 12289, worst_case_work: 12289 }` 반환으로 바뀌었다.

## 호환성과 재사용 증거

이번 pass에서 직접 실행한 fingerprint 검사는 predecessor의 frozen pre-fix source archive를 별도 compile해 비교했다. 세 값은 다음과 같이 동일했다.

- constant: `39615ba8f80ab6d23c5e3b7456f417e27454bbf848bfc76bd65866765fb13caa`
- dynamic_add: `143e8af8e4716e6aaddf21cbdef6275a6d6543cbfda7791df39eb81ad14be2ae`
- authored_aggregate: `30673c776d40fef4e594b3f43d32043379440dbd19fbc23e1e4ec521df75b308`

predecessor target의 source/record/evidence/archive hash 검증과 correction 001의 Unix socket EPERM 분류는 기존 evidence를 재사용했다. 기존 correction probe와 전체 테스트는 현재 source에서 다시 실행했으므로 재사용 항목으로 세지 않는다.

정상 그룹 순서와 계산 순서, signed rounding 전 부호 조합, checked overflow, unit error, legacy literal lowering/narrowing은 기존 unit·correction probe·전체 suite 재실행으로 확인했다.
