# D07-B F03 — C04/C05 동적 amount replay 단계검토 인계

2026-09-28 · C04/C05 부분 자체 검증과 해당 C16의 인계. **F03·D07-B 전체 완료나 독립 검토 PASS가 아니다.**

## 범위와 변경

[F02/C02 사본 부분 PASS](../../reviews/wp02_d07_b_f02_correction_001_review_001/REVIEW.md) 후
사용자 재개 요청으로 [기존 교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md) §5의
실제 입력 변화→소비→기록→두 query를 보강했다. 두 구현 템플릿 revision `2026-09-27-v1`을 따른다.
현재 위치/target은 [PLAN](PLAN.md), 시도·실패는 [EXPERIMENTS](EXPERIMENTS.md), 판단은 [NOTES](EXPERIMENT_NOTES.md)가 소유한다.

제품 실행 코드는 바뀌지 않았다. `value_expr_runtime.rs`의 기존 `#[cfg(test)] mod tests` 아래
`tests/amount_replay.rs`를 연결했다. 기존 test helper는 resource catalog를 builder.build 뒤에 덮어쓰던 것을
`with_resources`→`build`로 고쳤다. 자원을 읽는 RON이 실제 compiler 검증을 통과하도록 준비 순서를 바로잡은 것이다.
public authored expression 차단·admission·계량·소유권·정책은 변경하지 않았다.

새 fixture는 관측을 켜고 실제 ObservationKey와 action을 bind한다. release 시 finite delivery를 예약하고
`build_value_expr_delivery_capture`가 만든 carrier를 `DamageSourceSnapshot.value_expr_capture`에 넣는다.
그 뒤 입력을 바꾸고 actual `execute_ordered_skill_effects`를 transaction 안에서 실행한다.
실제 projectile caller처럼 runtime_fact_source도 template의 delivery source로 바꿨다가 복원한다.
Snapshot record는 release frame과 동일 Arc이며 Live capture는 snapshot frame 없이 소비 시점 frame을 만든다.

## 신규 관측 시험 조건표

모든 시험의 접두사는 `game::battle::core::value_expr_runtime::tests::amount_replay::d07_f03_`다.
공통으로 실제 HP/완료 fact, record의 독립 기대 수치·frame 입력·입력을 포함한 trace,
preview/explain의 기록 수치·trace·ordinal·release/consume time을 대조한다.
query를 반복해 출력 bytes와 usage가 같고 gameplay/queue/facts/record Arc/ordinal이 그대로임을 확인한다.
조회 후 다시 입력을 바꾸고 같은 기록의 두 문서가 바뀌지 않음을 확인한다.

| 조건 / 원 요구 | 시험 suffix | 입력과 독립 기대값 |
| --- | --- | --- |
| C04/C05 · B02–B04 | `c04_c05_attack_release_consume_and_later_queries_keep_distinct_moments` | 공격력 release10→consume20→query 후30. True Damage의 실제 HP 감소, record와 query는 Snapshot10 / Live20 |
| C04/C05 · B02–B04 | `c04_c05_target_hp_release_and_consumption_are_frozen_in_record` | 대상HP release80→consume60→query 후30. HP/4 식은 Snapshot20 / Live15. frame에는80/60 |
| C04/C05 · B02–B04 | `c04_c05_resource_mutations_use_release_or_consumption_balance_in_queries` | Employee Attempt meter 초기7→실제 Grant4로11→소비/조회 후 Grant5로16. 단위를 지킨 ratio×Magnitude 식의 Damage는 Snapshot7 / Live11 |
| C04/C05 · B02–B04 | `c04_c05_completed_action_results_keep_the_release_frontier_or_live_frontier` | 실제 첫 Damage7 완료→release→다음 Damage3 완료→Heal 소비. CurrentActionPrefix 피해 합계는 Snapshot7 / Live10. 이후 실제 피해3 추가로 합계13이 되어도 기존 query 동일 |
| C05 · B03–B04 | `c05_live_reads_prior_effect_hp_resource_and_actual_mitigated_result` | 대상HP60/max80, defense75. 앞 Physical7의 실제 피해는4, 자원 Grant4. 뒤 Live Heal은 MissingHP24 / EarlierEffect의 완료 피해4 / 자원 잔액11. 최종HP80/60/67, 기록 순번2. 이후HP30·자원 추가 변경에도 기존 query 동일 |

5개 시험이 11개 시나리오를 실행한다. focused 명령과 원문 preview/explain은
[evidence/f03_replay_004.json](evidence/f03_replay_004.json), [실행 로그](evidence/f03_replay_004.log)에 보존했다.
JSON 로그의 `D07_F03_REPLAY` 행에는 기대 입력·결과와 실제 두 문서를 출력한다.

완료 result의 **조회 후 변화** 사례는 이미 확보한 record를 교체하지 않도록 private fixture에서
수치 관측만 잠시 끈 채 다음 실제 damage/settlement를 수행한 뒤 다시 켠다. 최초 관측·소비·record 생성은 켜진 상태다.
이 조작은 query가 현재 action 결과를 다시 읽지 않는지 확인하는 장치이며, 일반 관측 전환 API나 최근 기록 교체 정책의 증거가 아니다.

## 증거 종류와 한계

- **신규 관측 시험:** 위 C04/C05. 실제 release carrier와 실제 command 소비·commit record에서 시작한다. 시험이 frame/trace를 가짜로 조립하지 않는다.
- **기존 runtime 회귀:** 최종 source의 value_expr/authoring/전체 target 실행. 기존 F01/C01·F02/C02 회귀 포함.
- **source 근거·측정 재사용:** [EVIDENCE_REUSE](evidence/EVIDENCE_REUSE.json). 기존 source340개 byte 동일,
  바뀐 parent의 cfg(test) 이전 부분도 byte 동일, 신규 module도 test 안에서만 연결. 선행 manifest에 묶인 probe·원자료·로그와 toolchain을 대조했다.
- **미검증/후속:** C03, C06–C15 및 F03 전체. 직접 `encode_record_for_test`를 부르므로 public Latest/Exact selector,
  권한, terminal/battle 교체, 취소/실패 복원, memory 한도 및 긴 trace/i128 원문까지 새로 검증한 것으로 주장하지 않는다.
  query에 Exact를 넣은 것은 문서 입력을 명시한 것이며 selector를 통과했다는 뜻이 아니다.

## 시도와 검증

최초 grant helper 타입 오류(compile0건), RON 필수 필드/자원 fixture 준비 오류(2 pass/3 fail),
호출자의 delivery source scope 누락(4 pass/1 fail)과 진단 단일 실패를 모두 보존했다.
마지막 경우 실제 피해4는 정상이고 EarlierEffect를 읽는 Heal이0이었다. carrier만 넘기고 command의 delivery scope를
연결하지 않은 **시험 오류**였으며, 실제 caller 계약을 재현해 해결했다. 결과 matching을 느슨하게 바꾸지 않았다.
당시 source archive와 각 실행의 전후 digest·실패 원문은 evidence에 있다.

[최종 판정표](evidence/FINAL_VALIDATION.json)가 모든 최종 명령·시험 수·source digest·소요 시간의 원본이다.
전체 시험은 기본 병렬이다. check/Clippy/변경 파일 format 결과와 경고 원문도 보존한다.
PC36 및 R05는 **재실행하지 않았다**. 테스트 밖 코드·입력·도구가 보존되어 직전 F02 측정을 재사용한다.
PC36 성공31/typed거부5·gameplay 차이0, R05 peak9,049,037 bytes의 cap 초과18,849 bytes 단일 경고를 유지한다.
새 관측 on 비용이나 이번 실행의 새로운 메모리 측정값으로 해석하지 않는다. 20분 시험은 없다.

## 동결·후속 검토

[successor manifest](target/d07_b_f03_replay_001/MANIFEST.json)에 before341/after342개, 변경1개·추가1개,
전후 archive와 정·역 patch, 기록/실패/실행/검토/지시서 archive를 결속한다.
[격리 복원](target/d07_b_f03_replay_001/RESTORATION_VERIFICATION.json) 및 `python3 freeze.py verify`로
currentness·추가 파일 membership·정/역 byte 복원을 확인한다. 제품 작업 트리는 verify가 수정하지 않는다.
최초 D07-B 대비 누적 변경은 최종 판정표의 `initial_d07_b_cumulative_delta`에 기록한다.

다음은 **사본 대화에서 이 C04/C05 successor를 검토**하는 것이다. C03·C06–C15의 다음 구현은 사용자 재개 요청 뒤 진행한다.

**부분 교정·자체 검증 종료 / successor target 동결 / 단계검토 요청 대기**
