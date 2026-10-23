# F-U164-IA002 공통 XP 교정 결과

2026-10-15 사용자의 “그럼 교정해줄래?” 요청에 따라 별도 GEX-10 교정 pass를 수행했다. **XP 소실 결함의 교정과 영향 검증을 완료했다. U16-4 Goal 전체는 Not Complete이며 독립 재감사는 아직 인수하지 않았다.** 이전 independent_audit002·실패 원문은 보존했다. 이번 실행자는 교정에 참여했으므로 이 기록은 비참여 독립 감사가 아니다.

`Employee::add_experience_with_policy`가 기존 잔여XP와 이번 지급량을 u64 임시 합계에 정확히 더하고, 기존 레벨 비용을 차감한 뒤 잔여를 u32 필드에 저장하도록 수정했다. 두 입력이 각각u32이므로 합계는u64에 들어가고, 반복 종료 시 잔여는 검증된 양수u32 비용보다 작아 저장도 정확하다. 보상량·대상·레벨곡선·마지막 비용 반복·level 최댓값 증가 의미·공개/저장 필드는 유지했다.

canonical 변경은 `src/game/employee.rs`의 계산과 영구회귀3개, `docs/core_runtime_contract.md`의 해당 동작 설명이다. 공통 grant나 스트레스 전용 우회 처리를 추가하지 않았다. 상위규칙의 XP 총량/잔여 보존을 구현한 기술교정이며, 과거 saturating-add 동작을 설명했던 계약 문장을 현재 구현에 맞췄다.

## 검증

| 실행 | 결과 | 확인 범위 |
| --- | --- | --- |
| red001 | 1통과/2실패/ignore0, exit101 | 기존 코드가 생존XP 뒤 큰 보상과 두 큰 보상에서 XP를 소실함을 새 영구회귀가 검출. 작은 가변곡선/경계 대조는 통과 |
| green001 | 49통과/0실패/ignore0, exit0 | 영구회귀3·기존 성장/대상별 grant/결과diff/생존XP/정산 실패 원자성·U14·U16-4 및 감사 probe |

감사 당시의 동일 XP probe는 정상 총2,147,483,657 및 무억제 총4,294,967,314를 출전직원 두 명 모두에서 확인했다. 무억제 결과는 레벨5/잔여XP294,967,314로 정확하며, 비출전 직원은XP0이다. 원래 실패의 기대값이나 보상량을 낮추지 않았다.

영구회귀는20+(u32::MAX-1)의 합계, 거의u32최댓값인 잔여에 다시u32최댓값을 지급하는 경우, 비용[7,11,13]에서0/경계/다중레벨/마지막비용반복을 확인한다. 기존 대상별 전액지급·선택직원·사망포함/제외·중복UUID제거·빈대상/누락context·level포화 의미도 유지됐다. U16-4의 정상25/무억제50XP, 통화·가루·파츠 경계와 유물1개 유지, 후퇴·패배 대조를 함께 재실행했다.

빌드34.60초/검사19.33초는 개발 실행시간이며 제품 비용 합격선이 아니다. 이49개를 기존2222개 전체 재실행, 정상 승인host/paired 저장/SavePoint 또는 성능PASS로 확대하지 않는다. 로더·저장형식은 바꾸지 않았으므로 해당 전체 검사를 교정만을 이유로 반복하지 않았다.

## 증거와 재현

`before/`, `INTAKE.json`, `CHANGES.patch`, `AFTER.tar.gz`, `GREEN_INPUT.json`, `RESULT.json`이 수정 전/후 코드와 원시로그를 연결한다. 현재2파일은 실제 시험 사본과 해시가 같고 archive도 검증했다. 별도 scratch에서 patch 정방향/역방향 적용 후 각 원문의 해시 일치를 확인했다(`RESTORE_CHECK.json`). 이는 교정2파일의 보관·복원 검증이며 Goal 전체 최종 동결/정역복원을 대신하지 않는다.

현재 실행 사본은 `/tmp/u164-correction002`다. 이전 감사사본과 원문 probe는 수정하지 않았다. 다음 명령으로 같은49개를 선택한다.

```bash
cd /tmp/u164-correction002
env PATH=/home/blast/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  CARGO_HOME=/tmp/wp02-cargo-home \
  CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p game_core --lib --offline --locked -- --nocapture \
  experience survival_xp u164_ u14_ \
  reward_claim_failure_does_not_partially_commit_prior_effects \
  combat_terminal_settlement_failure_does_not_partially_apply_rewards_on_retry
```

사본이 사라졌다면 independent_audit002/REPRODUCE.md에 따라 새 디렉터리에 감사 최종사본을 복원하고 이번 AFTER.tar.gz의2파일을 덮어쓴다. 원본 저장소에 기록용 patch를 재적용하지 않는다. root에서 실행하는 영구회귀 selector는 `experience_accumulation`이며 감사 probe가 없는 root와 위 사본의 선택 시험 수는 다르다.

이번 교정은 종료했으며 실행 중 세션은 없다. F-U164-IA002는 구현자 기준 교정·영향검증 완료/별도 재감사 미인수다. 기존 정상host·실제저장·소비품/직원손실·동시각·비용·최종화 조건은 그대로 남고 U17 이후로 진행하지 않았다.
