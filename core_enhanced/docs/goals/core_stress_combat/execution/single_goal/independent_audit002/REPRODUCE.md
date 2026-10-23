# 독립 재감사 재현

준비된 사본은 `/tmp/u164-independent-audit002`이다. 제품 원본은 바꾸지 않았으며 사본의 `u164_stress.rs`에만 감사 검사를 덧붙였다.

```bash
cd /tmp/u164-independent-audit002
env PATH=/home/blast/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  CARGO_HOME=/tmp/wp02-cargo-home \
  CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p game_core --lib u164_audit2_ --offline --locked -- --nocapture
```

현재 기대 결과는 신규2개 중 가루/파츠/유물1통과, XP보존1실패, exit101이다. 실패는 무억제XP 기대4,294,967,314/실제4,294,967,295다. exact selector `u164_audit2_xp_quantity_preserves_survival_and_authored_grants`로 XP만 재현할 수 있다. 실패 기대값을 제품 합격으로 해석하지 않는다.

사본이 없다면 새 빈 디렉터리에 이번 `AUDIT_INPUT.tar.gz`와 이전 `../independent_audit001/REPLAY_SUPPORT.tar.gz`를 순서대로 복원한다. 감사 입력 manifest와 사본 digest를 대조한 뒤 사본의 `core_enhanced/src/game/world/tests/u164_stress.rs` 끝에 다음 세 파일을 순서대로 붙인다.

1. `../correction001/POSITIVE_REPLAY.rs`
2. `PROBES.rs`
3. `XP_PROBE.rs`

원본 저장소에 archive/patch를 재적용하지 않는다. 의존성 cache/toolchain은 위 환경을 따른다. 현재 전체 `u164_` selector는18개를 선택하며, 첫 실행 기록의17개와 다르다. 첫 실행을 그대로 복원하려면 `PROBES.before_fixture_correction.rs`를 사용하고 XP_PROBE를 붙이지 않는다. 이 경우1개는 유물목록 준비 오류로 실패한다.

원시 로그·종료 코드·시험 이름은 RESULT.json, 실행 전 오라클과 fixture 교정 이유는 PROBE_PLAN.md에 있다. 이 검사는 Core와 정상 작성 로더를 사용하며 실제 영속 host/fresh process 검증을 대신하지 않는다.
