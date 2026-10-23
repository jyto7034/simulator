# 독립 재감사003 재현

준비된 사본은 `/tmp/u164-independent-audit003`이다.

```bash
cd /tmp/u164-independent-audit003
env PATH=/home/blast/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  CARGO_HOME=/tmp/wp02-cargo-home \
  CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p game_core --lib --offline --locked -- --nocapture \
  experience survival_xp u164_ u14_ \
  reward_claim_failure_does_not_partially_commit_prior_effects \
  combat_terminal_settlement_failure_does_not_partially_apply_rewards_on_retry
```

기록된 결과는49통과/0실패/0ignore, exit0이다. XP 원래 실패만 선택하려면 `u164_audit2_xp_quantity_preserves_survival_and_authored_grants`를 selector로 사용한다. 정상/무억제 두 모드와 출전 두 직원 모두 검사한다.

사본이 없다면 새 빈 디렉터리에 이번 `AUDIT_INPUT.tar.gz`와 `../independent_audit001/REPLAY_SUPPORT.tar.gz`를 순서대로 복원한다. 이번 manifest753파일의 해시를 대조한 뒤 사본의 `core_enhanced/src/game/world/tests/u164_stress.rs`에 다음 파일을 각각 줄바꿈 하나와 함께 순서대로 덧붙인다.

1. `../correction001/POSITIVE_REPLAY.rs`
2. `../independent_audit002/PROBES.rs`
3. `../independent_audit002/XP_PROBE.rs`

최종 stress 시험 파일의 SHA-256은 `ce6e76c7051f35809e2d43ff92d5d43d92a45309a14acf11decd4b1493ea2ce6`이다. 제품 원본과 과거 감사사본에는 archive/patch를 적용하지 않는다. 원시 실행은 `replay001.log`, 실행 결과와 해시는 `RESULT.json`에 있다. 정상 host/영속 저장/비용 측정 결과가 아니다.
