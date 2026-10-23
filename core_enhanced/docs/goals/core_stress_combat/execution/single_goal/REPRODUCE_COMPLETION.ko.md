# U16-4 최종 입력과 재현 안내

`target/u164_current_001/AUDIT_TARGET_RECORD.json`에서 SOURCE/BEFORE archive, source/evidence manifests, DELTA/SCOPED.diff, RESTORE를 찾는다. 기존 실행 증거를 덮어쓰지 않고 새 감사 디렉터리·store·출력명을 사용한다. `test_parts`의 진행기는 기록된 이름에 대해 exclusive 생성하므로 기존 결과 위에서 그대로 재실행하지 않는다. 과거 실패와 승인 receipt는 유지한다.

## 입력·빌드

제품은 `completion001/INTAKE.json`과 감사003 입력과 동일하다. 원래 작업 사본은 `/tmp/u164-final-work`; toolchain은 rustc1.98.1/x86_64 Linux, `CARGO_HOME=/tmp/wp02-cargo-home`이다. 정상·계측 build 명령, compiler, dependency graph, source 목록은 `normal_build002/prepared-build.json`과 `instrumented_build002/prepared-build.json`에 있다. 정상 host와 worker는 `artifacts/normal002`, 계측 worker는 `artifacts/instrumented002`의 파일별 receipt로 식별한다. 재빌드는 `tools/cmva_build.py`의 동일 mode/kind/bin/features를 사용하고 새 out/target 경로를 지정한다.

정상 작성 pack은 `test_parts/normal_inputs002/source`, 확장 최종 pack은 `test_parts/expanded_inputs002/source`다. 각각 `admission.json`의 registry/base-selection/runtime settings 및 이미 기록한 package/calibration/adoption argv는 `runs/*.result.json`에 보존했다. private issuer key는 archive에 포함하지 않는다. 새 환경에서는 지원 host의 `--cmva-calibrate`와 `--cmva-adopt-calibration` 경로로 새 private root와 cache를 만들고 같은 `reviewed_limits.json`을 적용한다. 공용 cache를 쓰는 admission 실행은 순차로 한다.

## 기능·저장

`test_parts/completion_host.py`가 `host.rs` line protocol과 실제 paired IO 실패를 호출한다. `completion_pilot.py`의 Event→소비품/유물→SavePoint가 정상 base 준비다. `completion_flows.py`의 full/extract/retreat/unfinished/unrestrained, `completion_defeat.py`, `completion_savepoint.py`, `completion_expanded.py`가 해당 public 행동과 assertions를 담는다. 새 output/store 이름을 쓴 사본에서 실행한다. 원래 argv/종료 결과와 각 요청/응답은 `runs/completion_host001/<name>/` 및 `completion001/RESULT.json`에 연결했다.

네 경계 probe는 canonical 테스트에 추가하지 않았다. source archive를 별도 tree에 복원한 뒤 `test_parts/COMPLETION_PROBES.rs`를 그 사본의 `core_enhanced/src/game/world/tests/u164_stress.rs` 끝에 추가한다. include_str 절대경로는 같은 고정 normal source를 가리키도록 새 사본에서만 조정한다. 실제 사용한 선택은 다음과 같다.

```sh
CARGO_HOME=/tmp/wp02-cargo-home CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target CARGO_PROFILE_TEST_DEBUG=0 cargo test -p game_core --lib --offline --locked u164_completion_ -- --nocapture
```

cwd는 복원한 사본이다. 원본 로그 `runs/completion-probes005.log`는4pass/0fail/0ignore다. 결과 집계 코드는 `completion_analyze.py`이며, 재집계도 원래 `completion001/RESULT.json`을 덮어쓰지 않도록 사본에서 수행한다.

## 비용

`test_parts/cost_probe`를 고정 제품 경로로 빌드한다. 원래 binary SHA는 `completion001/cost001/RESULT.json`에 있다. `completion_cost.py`는 `normal.trace.jsonl`(prepare001+full002)과 `expanded.trace.jsonl`(expanded006)을 각각3회 재생했다. 개별 읽기 전용 입력의 재현 예시는 다음과 같다.

```sh
.cache/u164-review-dev-target/debug/u164_completion_cost core_enhanced/docs/goals/core_stress_combat/execution/single_goal/test_parts/normal_inputs002/admission.json core_enhanced/docs/goals/core_stress_combat/execution/single_goal/completion001/cost001/normal.trace.jsonl /tmp/u164-audit-cost-normal-new.json
```

확장은 같은 binary에 expanded_inputs002 config와 expanded.trace를 넘긴다. 새 출력명을 사용하고 빌드/host와 겹치지 않게 순차 실행한다. 실제 IO 실패 candidate는 진단 Core 재생에서 제외하고 다음 동일요청을 적용하며, 저장 원자성은 원래 approved host trace로 확인한다. 시간·native work·실제 heap·RSS·drop 회수의 해석 범위는 `COST_REVIEW.json`을 따른다.

최종 비참여 감사자는 고정 입력·원시 증거·재사용 범위와 알려진 서버 선행 fixture 실패를 검토하고 필요한 별도 재현을 수행한다. 기존 자체검증 결과만으로 독립 Complete를 미리 부여하지 않는다.
