# 재현 경로

저장소 root `/mnt/f/work/simulator`에서 실행한다. 기존 `single_goal`과 이 디렉터리의 원시 출력·동결 target을 덮어쓰지 않는다. 재실행은 새 sibling execution 디렉터리에 `test_parts/*.py`를 복사하여 새 출력 이름을 사용한다. runner는 기존 결과가 있으면 거절한다. private issuer는 `prepare_host.py`가 `/tmp`에 새로 생성하며 증거에 key를 포함하지 않는다.

Cargo 공통 환경:

```bash
export PATH=/home/blast/.cargo/bin:$PATH
export CARGO_HOME=/tmp/wp02-cargo-home
export CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
cargo test -p game_core --lib --offline --locked
cargo test -p game_core --lib --features runtime-observation --offline --locked u17_
cargo test -p game_server --lib --features cmva-lab-measurement --offline --locked persistence::
```

관련 native 선택은 `runs/native-storage-012.log`, `runs/native-shared-013.log`에 실제 시험 이름이 있다. 이번 실행에서는 동일 test binary에 각각 `u08_continue_owner u08_continue_decode u08_continue_projection game::relic::allocation::tests g11 runtime_safety_checkpoint_restore`, `u08_ u11_return` 필터를 전달했다. 매칭되지 않은 필터를 통과 수로 세지 않으며 실제6/45건만 기록했다. Unix socket 검사는 샌드박스에서 EPERM이 나면 동일 binary의 `--exact game::data::content_pack::snapshot::tests::unix::socket_inputs_return_errors`를 필요한 권한으로 실행한다.

release 빌드의 정확한 command·toolchain·feature·dependency 식별은 `normal001_build/prepared-build.json`, `instrumented001_build/prepared-build.json`에 있다. 새로운 `--out`과 `--target-dir`을 사용해 같은 `tools/cmva_build.py` 명령으로 생성한다. `prepare_host.py`의 artifact source는 사용한 target에 맞춘다.

새 실행 디렉터리를 `E`로 두었을 때:

```bash
python3 "$E/test_parts/prepare_host.py" binaries
python3 "$E/test_parts/run_public.py" normal expanded terminal
cargo build --manifest-path core_enhanced/docs/goals/core_manual_encounter_revival/execution/single_goal/test_parts/cost_probe/Cargo.toml --offline --locked
```

마지막 command의 `debug/{u17_completion_cost,restore_cost}`를 `$E/artifacts/diagnostic001/`에 복사하고 사용 binary 해시·빌드 로그를 기록한다. 다른 Cargo 빌드와 같은 target을 동시에 사용하지 않는다. 비용 실행은 빌드·다른 큰 검사 종료 후 수행한다.

```bash
python3 "$E/test_parts/cost_suite.py" "$E/runs/current001/normal-forward" "$E/runs/current001/expanded-forward"
```

cost runner가 출력한 `/tmp/u17-storage-cost-*`를 `$E/runs/current001/cost`에 보존한다. `test_parts/summarize.py`가 raw assertion·실제 시험 수·12비용process를 검산한다. `finalize.py`는 실제 baseline과 요약이 준비된 최종 입력에서만 실행하며 기존 target을 재생성하지 않는다. 과거 `single_goal` finalizer를 다시 실행하지 않는다.

공개 요청/응답은 각 `runs/current001/*/host` 또는 `hosts` 아래 trace/requests/PROCESS에, IO 거절 지점은 각 `ASSERTIONS.json`에 있다. 성공 전 실패도 `runs/*001.stderr`, `runs/*001.log` 등 원본에 보존한다.
