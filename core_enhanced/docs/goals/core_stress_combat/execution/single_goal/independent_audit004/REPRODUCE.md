# 감사004 재현

고정대상은 `../target/u164_current_001/AUDIT_TARGET_RECORD.json`이다. 이번 새사본은 `/tmp/u164-independent-audit004`이며 제품원본을 편집하지 않았다.

## 격리 시험

새 빈 폴더에 SOURCE.tar.gz를 복원하고 `../independent_audit001/REPLAY_SUPPORT.tar.gz`에서 없는 build support 파일만 추가한다. 사본의 `core_enhanced/src/game/world/tests/u164_stress.rs` 끝에 다음을 순서대로 추가한다.

1. `../correction001/POSITIVE_REPLAY.rs`
2. `../independent_audit002/PROBES.rs`
3. `../independent_audit002/XP_PROBE.rs`
4. `../test_parts/COMPLETION_PROBES.rs`

probe의 `/mnt/f/work/simulator` 절대경로만 새사본 경로로 바꾼다. 이번 최종시험파일 SHA256은 `a96baace2a5b42611785ec0e4dac132590251e83840cd495612e3f644ac997d9`다.

```sh
env PATH=/home/blast/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  CARGO_HOME=/tmp/wp02-cargo-home \
  CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --manifest-path /tmp/u164-independent-audit004/Cargo.toml \
  -p game_core --lib --offline --locked -- --nocapture \
  experience survival_xp u164_ u14_ \
  reward_claim_failure_does_not_partially_commit_prior_effects \
  combat_terminal_settlement_failure_does_not_partially_apply_rewards_on_retry
```

53pass/0fail/0ignore/exit0이었다. canonical시험수 증가가 아니라 감사사본 probe 포함 수치다.

## Host·비용·검산

`replay_host.py`는 기존 지원 driver를 읽고 새 임시 store로 공개행동을 실행한다. 재실행할 때 `A`를 새 출력 디렉터리로 바꾸고 이 파일의 `E`는 원래 single_goal을 유지한다. 원래 `runs/completion_host001` 아래 감사출력은 exclusive 생성이므로 덮어쓰지 않는다. `PYTHONDONTWRITEBYTECODE=1`로 실행한다. config/issuer가 없으면 `../REPRODUCE_COMPLETION.ko.md`의 정상 calibration/adoption 절차가 필요하다.

비용의 exact argv와 실행종료코드는 `COST_REPLAY.json`에 있다. argv 마지막 출력명을 새 이름으로 바꾼다. 같은 binary의 SHA는 기존 `completion001/cost001/RESULT.json`과 대조한 뒤 정상/확장을 순차실행한다. 새 결과는 `cost-normal.json`/`cost-expanded.json` 및 각각의 trace에 보존했다.

`check_evidence.py`는 제품/원시증거를 읽기 전용으로 검산하고 결과만 감사폴더에 쓴다. 재실행 시 `A` 출력 위치를 별도로 지정하고 `E`/`R`은 고정 입력을 유지한다. 최초 parser 실패는 PLAN.md에 기록했다. SOURCE920/EVIDENCE550 입출구 확인과 새산출물 해시는 ENTRY/EXIT/RESULT.json을 따른다.
