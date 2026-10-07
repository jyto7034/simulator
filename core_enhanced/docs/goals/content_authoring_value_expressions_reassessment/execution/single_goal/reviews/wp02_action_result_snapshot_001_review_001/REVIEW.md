# R06 WP02 action result / release Snapshot 독립 단계검토

- 일자: 2026-09-26
- 대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, WP02 부분 구현
- 판정: **부분 PASS — 이번 action 완료 결과·release Snapshot 구간은 수용 가능. WP02 전체 완료는 아님.**
- 수정이 필요한 차단 finding: 검토 범위에서 발견하지 못함.
- 제품 소스와 기존 동결 target은 수정하지 않았다. 검토 증거만 이 디렉터리에 추가했다.

## 대상과 currentness

[구현 인계서](../../segments/wp02_action_result_snapshot_001/REVIEW_HANDOFF.ko.md)의 [동결 manifest](../../segments/wp02_action_result_snapshot_001/target/action_result_snapshot_001/MANIFEST.json)를 검토했다.

- Manifest SHA-256: `b16697bfe8c518944f0bddcf800b0f1e005ed639edd6fe915e4d96c726a6e1f4`
- 최종 333개 파일 digest: `60fe6643fa136a16773e93042d9db7d3e5b9398088c495771796abd410708717`
- 선행 correction manifest: `abe5a96830d8090f77e63fde7c6d7ff79675f701a382437d2492e0c66e9aa60a`
- 변경은 `runtime_facts.rs`, `sim.rs`, `value_expr_runtime.rs`, `catalog.rs` 4개 파일이다.

입구와 출구에서 `freeze.py verify`를 실행했다. 현재 333개 파일, evidence와 records, 양방향 patch 복원 검증이 모두 통과했다. 별도의 [verify.py](verify.py)로 현재 파일 및 after archive의 정확한 membership/내용, records 73개와 artifacts 5개, 최종 실행 receipt의 source 전후와 로그 해시를 다시 대조했다. [verification.json](verification.json), [freeze_entry.log](freeze_entry.log), [freeze_exit.log](freeze_exit.log)에 기록했다.

## 코드 검토 결과

1. `ActiveAction`이 선택적 boxed witness를 소유하고, 완료된 effect의 실제 scalar projection을 다음 effect 실행 전에 합산한다. 첫 step의 literal effect도 skill 전체의 요구를 보고 witness를 준비하므로 뒤 step이 요구하는 결과를 보존한다. action의 resource ID 집합을 delivery-local projection에도 전달해 slot 순서를 맞춘다.
2. `CurrentActionPrefix`는 같은 action의 완료 누적값을 읽는다. action/source와 유효 delivery 검증, generation/attempt 검증을 거치며, witness 부재와 유효한 0을 구분한다. `EarlierEffect`의 delivery-local 검사와는 구별된다. compiler의 후속 step 허용은 runtime 완료 여부 검사와 함께 검토했다.
3. Snapshot은 release 시 frame에 읽은 값을 보존하고 Live는 effect 시작 시 다시 읽는다. projectile/area의 capture 생성 및 consumption rebind 경로를 추적했다. rebind가 Snapshot frame을 새 Live frame으로 바꾸지 않는다. focused 시험의 Snapshot 7 / Live 10, 서로 다른 step·delivery의 7+3=10 결과를 직접 재현했다.
4. durable fact source에서 delivery가 제거되는 기존 계약에 맞춰, projection은 현재 실행 source의 delivery와 fact의 action/authored site를 함께 검사한다. 실제 command 처리 범위와 damage/heal/resource 관찰 경로를 확인했다.
5. 실패한 runtime transaction의 witness/facts 복원, 이전에 commit된 prefix 보존, clone 시 active action 소유권 분리, 마지막 delivery 이후 settlement에서 witness 보존량 제거를 확인했다. merge는 모든 덧셈을 검사한 뒤 값을 변경한다.
6. witness allocation 전 retained admission, resource 수집·정렬 및 settle work charge, CloneShape 연결을 확인했다. 기존 cap을 늘리지 않았다. R05 관찰 여유가 작으므로 후속 구간에서도 원 guard를 유지해야 한다.

## 직접 실행한 검증

동결 `SOURCE_AFTER.tar.gz`를 `/tmp/r06-action-review-qlag__yf`에 풀고 독립 build 디렉터리로 실행했다. 컴파일에 필요한 기존 문서 입력 2개는 [config.json](config.json)에 별도 해시로 기록했다. 도구는 Rust/Cargo 1.98.1, offline/locked 조건이다.

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| 동결 소스의 `value_expr_runtime` focused 시험 | 30/30 PASS | [focused_001.log](focused_001.log), [receipt](focused_001.json) |
| 실제 Resources effect의 cap 적용·signed change 누적 | PASS: Grant 요청 10 → 실제 +5, Decay −3, prefix heal 5와 2, 최종 HP 67 | [probes.rs](probes.rs), [probes_003.log](probes_003.log) |
| terminal 후 남은 delivery 동안 witness 유지 및 clone 분리 | PASS | 같은 probe 로그 |
| witness 설치 후 settlement의 retained extent 회수 | PASS: witness 미설치 대조 clone과 종료 후 보존량 동일 | 같은 probe 로그 |

추가 probe는 **3/3 PASS**다. 격리 사본에 test module hook 하나만 추가했고 제품 동작 코드는 바꾸지 않았다. [test_only_hook.patch](test_only_hook.patch)와 [probe_binding.json](probe_binding.json)이 소스·probe·receipt·로그를 연결한다. 모든 실행 전후 작업공간의 대상 333개 파일은 동일했다.

초기 검토 probe 실패도 보존했다. `probes_001`은 fixture가 resource 정의를 등록하기 전에 skill validation을 실행해서 실패했다. `probes_002`는 검토자가 `Consume`을 `ResourceChangeDelta`에 포함한다고 잘못 가정해 예상 HP 67 / 실제 70으로 실패했다. 기존 계약상 Consume은 Cost fact이고 ResourceChange와 분리된다. 최종 probe는 음수 ResourceChange를 만드는 `Decay`로 수정했다. 두 실패를 제품 결함이나 성공으로 재분류하지 않는다. 당시 probe 소스는 `probes_attempt_001.rs`, `probes_attempt_002.rs`에 남겼다.

## 기존 실행 증거의 재사용

아래 결과는 이번 검토에서 전부 재실행한 것이 아니라, 최종 source identity와 로그 결속을 확인해 재사용했다. 이름에 `final`이 붙은 일부 기록은 중간 inline revision의 digest이므로 최종 증거로 채택하지 않았다.

| 항목 | 채택한 최종 receipt | 확인 결과 |
| --- | --- | --- |
| 전체 대상 빌드 | `check_after_heap_001.json` | exit 0 |
| focused | `focused_after_heap_001.json` | 30/30; 위에서 독립 재실행도 수행 |
| 전체 제품 시험 | `all_targets_box_001.json` | 1,491/1,491 |
| Clippy | `clippy_box_001.json` | 기존 allow 2개 조건, exit 0 |
| 변경 영역 format | `format_after_heap_001.json` | exit 0 |
| PC | `pc_isolate_002.json`, `pc_box_001.json`, `PC_SUMMARY_BOX.json` | 36조건: 성공 31 / typed refusal 5, 기존 gameplay oracle 및 완료·거부 code 유지 |
| R05 | `r05_guard_box_001.json`, `r05_guard_box_raw.json` | peak 9,019,233 bytes < 원 cap 9,030,188 bytes |

PC는 원본→copied 333개 identity와 test hook 적용 후 built identity→실제 실행 source identity가 연결됨을 확인했다. R05 headroom은 10,955 bytes다. 해당 raw receipt는 `allocation_gate_applicable=true`, `latency_gate_applicable=false`이므로 이를 새 latency 보증으로 확대하지 않는다. 중간 inline revision의 guard 실패 기록은 그대로 남아 있다.

## 다음 구간과 판정 한계

이번 PASS는 action 완료 결과와 release Snapshot의 단계검토에 한정한다. 모든 delayed delivery 조합에 대한 새 end-to-end 시험이나 전체 WP02 지원 범위 완료를 주장하지 않는다. 기존 계획의 D04 실제 대상 집계와 D07 preview/explain은 미완료다. WP03/04, GEX-20/30 및 production expression publication도 완료·해제하지 않는다.

다음 구현 지시서는 **D04 실제 resolved target 집계**를 다음 검토 단위로 구체화하고, 이어 D07을 다루는 방향이 적절하다. 이때 이번 독립 probe의 실제 resource effect 및 settlement 경계 사례를 제품 회귀 시험에 편입하는 것도 권장한다. 설계·검토는 원 대화, 제품 구현은 별도 대화라는 작업 분담을 유지한다.
