# U16-4 독립 감사 재현

현재 준비된 실행 사본은 `/tmp/u164-independent-audit001`이다. 원본 제품은 수정하지 않았고 이 사본의 `core_enhanced/src/game/world/tests/u164_stress.rs`에만 `PROBES.rs`와 `DIAGNOSTIC_PROBE.rs`를 붙였다. 이전 `/tmp/u164-build`도 보존했다.

현재 사본에서 최종 정산 실패를 재현하는 명령:

```bash
cd /tmp/u164-independent-audit001
env PATH=/home/blast/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  CARGO_HOME=/tmp/wp02-cargo-home \
  CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p game_core --lib u164_independent_cumulative_authored_rewards_are_payable --offline --locked -- --nocapture
```

예상 결과는 실제 정산 `InvalidAction`, 시험 실패/exit101이다. 같은 옵션에서 selector를 `u164_independent_reproduction_fifth_round_atomic_failure`로 바꾸면5라운드 오류·재시도 실패·자산 원자성·후퇴 가능성을 확인하는 진단1개가 통과한다. 이 통과는 결함 재현을 뜻한다.

처음13개 선택은 `cargo test -p game_core --lib u164_ --offline --locked`였으며 당시에는 DIAGNOSTIC_PROBE.rs를 붙이기 전이다. 현재 사본에는 추가 진단1개가 더 있으므로 같은 selector를 다시 실행하면14개다. 기존 원시 실행 수를14개로 소급 바꾸지 않는다.

사본이 사라졌다면 새로운 빈 디렉터리에 `AUDIT_INPUT.tar.gz`, 이어 `REPLAY_SUPPORT.tar.gz`를 풀고, 해당 사본의 u164_stress.rs 끝에 PROBES.rs 및 DIAGNOSTIC_PROBE.rs 내용을 순서대로 붙인다. 전자는 감사 시점 제품/시험/데이터이고 후자는 기존 격리 workspace의 Cargo 구성·컴파일 참조 문서·선행 fixture 보충분166개다. REPLAY_SUPPORT는 제품 source를 덮어쓰지 않는다. manifest별 SHA-256으로 복원 결과를 확인한다. root 저장소에는 archive/patch를 재적용하지 않는다. 의존성 cache와 Rust toolchain은 별도 환경 입력이다.

공개 작성 입력 승인 재현은 저장소 root에서 다음 형태로 실행한다. 출력 디렉터리는 존재하지 않는 새 이름을 사용한다.

```bash
.cache/u164-review-dev-target/debug/package_official_content \
  core_enhanced/docs/goals/core_stress_combat/execution/single_goal/independent_audit001/cumulative_reward_source \
  /tmp/u164-audit-cumulative-pack-replay
```

기존 실행은 `cumulative_reward_pack`을 생성했고 exit0이었다. 바이너리 identity는 PUBLIC_PACKAGE_INPUT.json에 있다. 이후 cache 바이너리가 다른 소스로 다시 빌드됐다면 현재 제품 결과라고 혼용하지 말고 고정 소스로 다시 빌드한다. 지원 작성 RON의 다섯 라운드 엔케팔린 수량 외에는 공식 입력을 바꾸지 않았다.
