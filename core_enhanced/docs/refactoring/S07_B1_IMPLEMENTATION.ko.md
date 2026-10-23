# S07 B1 보상 수량 일관성 교정

작성일: 2026-10-16 · 상태: B1 교정·관련 회귀 검증·분리된 완료 검토 완료

후속 [R1 공통 후처리 통합](S07_COMBAT_BOUNDARY_REVIEW.ko.md#r1-구현과-완료-검토)도 완료했다. 아래 검토 대상·다음 재개 절은 B1 완료 당시 기록이며,
현재 다음 작업인 S01 상세 조사는 [출발점](README.md)을 따른다.

## 범위와 판단 기준

[출발점](README.md)의 다음 행동과 사용자 재개 요청에 따라 B1 오류 교정·관련 회귀 검증·별도 완료 검토를 수행한다.
종료점은 B1 완료 검토와 진행 현황 갱신이다. R1 공통 후처리 통합은 다음 작업으로 유지한다.
[기존 상세 조사](S07_COMBAT_BOUNDARY_REVIEW.ko.md)는 교정 전 원인과 재현의 기록으로 보존한다.

규칙과 기대값은 다음 현재 문서의 관련 절을 대조했다. 룰북 전체·모든 정책을 감사했다는 의미는 아니다.

| 근거 | 이번 교정에서 보존하는 의미 |
| --- | --- |
| [룰북의 원정 규칙](../game_rulebook.md) | 무억제 승리의 엔케팔린·장비 가루·환상체 파츠·XP는 기존 총량의 2배. 파편·알·장비·유물·선택 횟수·비전투 보상은 제외 |
| [무억제 전투 보상](../game_rules/growth-and-rewards.md#무억제-전투-보상) | 기본·달성 보너스·승리 생존 XP의 기존 대상별 수량에 한 번 적용. 패배 생존 XP는 기존량, 후퇴는 미지급 |
| [무억제 강화와 보상](../game_rules/expedition-and-deterrence.md#무억제-강화와-보상) | 최초 입장 비용 후 진입 시 배율 확정. 봉인 승리 후 억제력 보충으로 소급 변경하지 않음 |
| [런타임 계약](../core_runtime_contract.md)과 [결과·저장](../game_rules/results-and-save.md) | 종료 시 정산·저장, 결과 확인 시 재지급 금지, 선택 후보·지급 문맥과 Continue의 수명 유지 |

B1은 코드의 생산·표시·복원 불일치다. 테스트 추가만으로 해결되지 않으며 관련 조합의 회귀 공백도 함께 보강한다.
선택형 재현 입력은 지원 데이터 형태를 사용한 통제 fixture다. 일반전의 공식 보상을 선택형으로 바꾸는 정책 변경이 아니다.

## 인수와 변경 전 재현

- [관련 입력 기준선](evidence/s07_b1_001/baseline.json)과 `evidence/s07_b1_001/before/`의 파일 사본은 이번 편집 전 내용을 보존한다.
  기존 사용자 변경을 포함한 작업 트리를 기준으로 하며 commit ID를 깨끗한 기준선으로 간주하지 않는다.
- 보존된 공개 API probe를 같은 offline dependency cache·debug profile에서 다시 실행했다.
  [변경 전 출력](evidence/s07_b1_001/before-probe.log): 배율 1의 결과·ACK·claim 뒤 Continue는 성공,
  배율 2는 표시 17·지급 후보 34이며 세 시점 모두 `pending reward differs from retained content`로 실패했다.
  종료 코드 0은 관측 종료이며 기능 통과 판정이 아니다.

## 교정 설계와 구현

1. `transition_finished_live_battle_to_result`에서 부활 등 authored 후보 확정 직후 승리 수량 배율을 적용한다.
   이후 결과 표시·자동 지급·선택형 지급 후보는 같은 최종 옵션을 소비한다. snapshot에는 별도의 계산을 추가하지 않는다.
2. victory planner의 옵션 배율 적용을 제거한다. encounter·달성 보너스 효과와 생존 XP의 별도 지급 경로는 유지한다.
3. `combat_setup/rewards.rs::matches_retained_combat_reward`에서 기존 원본/난이도 percent 적용본과
   각각의 합법적인 무억제 변환본을 기존 산술 함수로 비교한다. percent 반올림 후 수량 배율 순서를 유지한다.
   전체 option의 동등성·identity 및 summary/entitlement 비교를 유지하며 임의 수량을 허용하지 않는다.
4. 진입 snapshot과 reader가 `BattleDeterrenceSnapshot::quantity_multiplier`의 같은 배율 정의를 사용한다.
   복원 시 현재 억제력으로 진입 배율을 역추론하지 않는다. overflow인 대안은 합법적인 producer 값이 아니므로
   다른 표현 가능한 대안의 일치를 막지 않으며 실제 생산의 checked overflow는 유지한다.

DTO·저장 버전·서버 저장 알고리즘·보상 밸런스를 변경하지 않는다. 기존 불일치 저장을 재작성하는 migration은 포함하지 않는다.

## 검증 설계와 실행

- 새 `world/tests/combat_reward_quantity.rs`는 실제 공개 start·초기 선택·보상 활동·입장·소진 후 계속하기·시간 승리·
  결과 확인·보상 선택/수령·Continue를 사용한다. 저장 bytes나 실행 상태를 직접 수정하지 않는다.
- 1배/2배 × ChooseOne/ClaimAll × 난이도 보너스 없음/125%의 8개 입력을 두 테스트에서 실행한다.
  17/3/11은 percent 반올림 후 21/4/14가 되고, 무억제에서는 각각 42/8/28이 된다. 장비 사본은 한 개다.
  참가자를 배치하지 않아 이 테스트의 XP는 명시적인 AliveRoster 지급만 확인하며 생존·달성 보너스는 기존 U14 검증으로 연결한다.
- 결과·ACK·선택·claim·Exit 뒤 새 CoreApplication 인스턴스에서 원본 저장을 읽고 다음 행동을 이어간다.
  중복 ACK/claim과 미선택 claim/미수령 Exit 거절, 표시·후보·실제 자산과 XP 일치를 확인한다.
  같은 프로세스의 새 인스턴스 복원이며 OS 프로세스 재시작·호스트 crash recovery의 대체 증거는 아니다.
- 공통 비교 함수는 percent 반올림과 quantity 적용 순서, overflow 대안과 표현 가능한 원본의 공존을 별도로 검사한다.
- 기존 ChooseOne 빈 후보·terminal pending choice, 정산 실패 rollback, 승패·후퇴·소비품, 봉인 보충·보너스,
  스트레스 누적 보상과 부활 선택, 준비된 명령 폐기·commit 경계를 영향 회귀로 선택한다.

관련 입력을 고정한 채 다음 검사를 수행했다. 각 실행은 종료 코드 0으로 끝났고 실행 중인 명령은 없다.

| 실행 | 실제 결과와 범위 | 원시 근거 |
| --- | --- | --- |
| 새 공개 경로 집중 검사 | 2개 테스트 통과, 내부 대조 입력 8개. unit-test target 빌드 50.57초, 검사 20.87초 | [focused001.log](evidence/s07_b1_001/focused001.log) |
| 기존 영향 경로와 산술 경계 | 52개 통과. 새 산술 테스트 2개 포함. 빌드 36.92초, 검사 20.13초 | [regression001.log](evidence/s07_b1_001/regression001.log) |
| terminal 빈 후보 | 1개 통과, 5.44초. 앞 실행의 `u04_terminal_empty` 필터는 실제 이름과 달라 선택되지 않아 정확한 이름으로 추가 실행 | [terminal-zero.log](evidence/s07_b1_001/terminal-zero.log) |
| 원래 probe 재실행 | 6건 모두 Continue 성공. 일반 표시/후보 17, 무억제 표시/후보 34, 지급 뒤 각각 517/534. library·도구 빌드 39.08초 | [after-probe.log](evidence/s07_b1_001/after-probe.log), [전후 대조](evidence/s07_b1_001/probe-comparison.json) |

합계 **55개 테스트 통과, 실패·ignore 0개**다. 별도 probe 6건을 테스트 수에 합산하지 않는다.
기존 warning은 unit-test target 33개, library 36개이며 이번 작업에서 무관한 warning 정리는 하지 않았다.
[검증 요약](evidence/s07_b1_001/verification.json)은 실행 건수와 미실행 범위를 식별한다.

동일 선택을 한 번에 재현하는 명령(저장소 루트, 기존 offline cache 사용):

```bash
env PATH=/home/blast/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin CARGO_HOME=/tmp/wp02-cargo-home CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p game_core --lib --offline --locked -- s07_ game::combat_setup::rewards:: u04_choose_one u04_terminal_choice u04_terminal_zero u04_live_terminal u04_continue_unfinished u04_prepared combat_terminal_settlement_failure combat_result_commits_attempt_conditions combat_defeat_commits consumable_modifier_survives u14_ game::world::tests::u164_stress:: game::world::tests::u17_public::
```

probe 명령은 [원래 재현 설명](S07_COMBAT_BOUNDARY_REVIEW.ko.md)의 manifest·명령을 그대로 사용했다.
전체 Core unit/integration suite, release build, 실제 OS 프로세스 재시작, 서버 crash recovery·Unity 소비자 검사는 수행하지 않았다.
이번 변경은 최종 보상 옵션의 생성 시점과 제한된 수량 비교에 국한되고, 해당 지급·복원 경로와 인접 분기를 위 검사로 확인했다.
전체 codec·호스트·공식 콘텐츠 전수 검증으로 확대해 보고하지 않는다. 성능 개선이나 전수 회귀 부재를 주장하지 않는다.

## 분리된 완료 검토

구현·검증 종료 뒤 제품 입력을 편집하지 않는 별도 검토 pass에서 현재 소스·규칙·원시 결과를 대조했다.
같은 실행자의 시점을 분리한 검토이며 비참여 외부 감사가 아니다.
[검토 대상](evidence/s07_b1_001/review-target.json)은 제품 4개 파일·테스트 등록 1개·새 테스트 1개의 식별 정보,
[이번 변경 diff](evidence/s07_b1_001/source.diff)는 편집 전 사본과의 차이다. 기존 사용자 변경 전체를 이번 성과로 귀속하지 않는다.

| 수용 조건 | 검토 결과 |
| --- | --- |
| 생산·표시·지급 후보의 같은 수량 | authored 후보 확정 뒤, summary 생성 전에 최종 수량을 계산. planner는 같은 옵션을 소비하고 재배율하지 않음. 8개 입력과 원래 probe 일치 |
| 정상 writer의 Continue 수용 | 원본/난이도 값에 같은 quantity 함수를 적용하여 정확히 비교. 전체 option 비교와 summary/entitlement 검사를 유지. 현재 억제력으로 역추론하지 않음 |
| 배율·대상·시점 정책 보존 | percent 반올림 후 2배, singular 사본·후보 수 유지. 봉인 보충·생존/보너스 XP·스트레스 누적·부활 지급 회귀 통과 |
| ACK·재시작·수령의 1회 처리 | 새 인스턴스 Continue 뒤 후속 명령과 중복 거절, 빈 후보·terminal pending choice·최종 정리 회귀 통과 |
| 실패와 authority 경계 보존 | 기존 complete application draft 경계를 유지. checked overflow, 정산 실패 rollback, 준비된 명령 폐기/commit·fatal fault 관련 회귀 통과 |
| 범위와 기존 작업 보존 | DTO·codec version·서버·게임 데이터 미변경. 제품 변경은 대상 4개 파일의 국소 수정. R1의 세 단계 중복 조립은 그대로 남아 있음 |

**판정: B1 완료.** 현재 범위의 필수 실패·미확인은 남지 않았다. 교정 전 summary/entitlement가 이미 다른 저장의
복구 migration은 구현하지 않았다. 일부 과거 ACK 이후 저장이 새 비교에 맞더라도 과거 저장 전체 복구를 보장하지 않는다.

## 다음 재개 위치

다음 작업은 **R1 승패 공통 후처리 조립 통합·검증·완료 검토**다.
`world/combat.rs::stage_committed_combat_resolution`과 `plan_player_victory_combat_result_completion`의
capture → 직원 후처리 → 소비품 기간 감소 세 단계를 대조하고, 승리 경로를 기존 helper 호출로 통합한다.
node completion·도감 검증 뒤라는 호출 위치, admission·clone·오류 순서와 지급 의미를 보존한다.
이번 55개 결과는 B1 완료 근거이며, R1 변경 뒤 영향받는 condition·XP·소비품·rollback 검증은 다시 수행한다.
S07 전체·다른 시스템의 완료로 확대하지 않는다.
