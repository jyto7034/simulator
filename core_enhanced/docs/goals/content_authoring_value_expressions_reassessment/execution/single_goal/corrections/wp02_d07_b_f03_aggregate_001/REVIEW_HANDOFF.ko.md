# D07-B F03 C06 — 집계 관계·필터·복수 node 실제 소비와 replay

2026-09-28 · **C06 부분 자체 검증·단계검토 인계**. F03/D07-B 전체 완료 또는 독립 검토 PASS가 아니다.

## 범위와 구현

[C03 runtime 사본 PASS](../../reviews/wp02_d07_b_f03_legacy_001_review_001/REVIEW.md) 후 사용자 재개 요청으로
[교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)의 C06/B05 및 이번 C16을 진행했다.
템플릿 revision `2026-09-27-v1`을 적용하며 [PLAN](PLAN.md), [EXPERIMENTS](EXPERIMENTS.md), [NOTES](EXPERIMENT_NOTES.md)를 갱신했다.
C03-OBS01의 RON i32::MIN 역직렬화 실패는 선행 검토의 probe와 함께 **C15 미해결 의무로 유지**한다.

제품 실행 코드는 변경하지 않았다. `value_expr_runtime.rs`의 cfg(test) 아래 `tests/aggregate_replay.rs`만 연결했다.
기존 owner_fixture와 C03의 실제 command tap을 재사용한다. 실제 catalog의 authored plan으로 action/ObservationKey를 bind하고,
finite delivery를 예약하여 release 시 `build_value_expr_delivery_capture`가 만든 carrier를 source template에 보관한다.
그 뒤 상태를 바꾸고 같은 delivery source를 runtime_fact_source에 연결해 transaction 안의 실제 ordered executor로 소비한다.
release frame을 소비 직전에 다시 만들거나 record/frame을 시험이 조립하지 않는다.

여기서 실제 소비는 ordered executor와 command/settlement 경계다. 투사체 이동·충돌 queue 전체나 장비 교체 UI 흐름을 새로 검증한 것은 아니다.
장비 슬롯·공격력의 시점 변화는 private fixture에서 수행한다. authored expression의 production publication 차단은 유지한다.

## 신규 관측 시험과 기대값

세 시험의 접두사는 `game::battle::core::value_expr_runtime::tests::aggregate_replay::d07_f03_c06_`다.

| 시험 suffix / 사례 | 실제 입력과 독립 기대 |
| --- | --- |
| `four_owner_views_preserve_snapshot_live_and_recorded_node_member_trace` / 4사례 | ByType(Weapon/Armor) 및 Any3(0/1) 각각 Snapshot/Live. 직원A 공격3·무기X/방어구Y, 직원B 공격5·무기Z, 소환물11, 적7. 같은 식의 네 node: Side Max→Sum18, IsActualEmployee/Employee8, 직원+첫슬롯 필터/장비사본8, 직원+둘째슬롯 필터/장비사본3. 합계37 |
| 위 시험의 release 후 변화 | 공격4/9/13/6, B 첫슬롯 제거. Live는19+13+4+4=40. Snapshot은37. 실제 target4명 각각 command amount37/40 및 HP63/60, 최신 record의 ordinal4·마지막 target·delivery·release10/consume20 확인 |
| `filtered_empty_owner_view_records_zero_without_damage_commands` / 2사례 | Snapshot/Live 각각 IsActualEmployee+HasEquippedSlot(Accessory)로 모두 제외. Sum/outer Sum은0. resolved target4명은 유지하고 damage command 없음·HP100, 평가 record/ordinal4·빈 node view·단일0 trace 확인 |
| `ineligible_relations_refuse_and_preserve_prior_record_and_gameplay` / 4사례 | 직원+소환물의 무필터 Employee=InvalidOwner, B 빈 Armor=InvalidOwner, 소환물의 Weapon layout 부재=MissingContext, HasEquippedSlot을 명시했어도 실제 runtime item ID가 nil이면 InvalidOwner. 필터가 손상 identity를 조용히 제외하지 않음 |

복수 node 시험에서는 aggregate 필터가 **피해 대상 자체를 줄이지 않음**을 actual command4건 및 네 유닛 HP로 확인한다.
Snapshot frame은 실제 release capture와 같은 Arc이고 Live carrier에는 snapshot frame이 없다.
각 node의 member identity·owner domain·직원/장비 사본 key·공격력·canonical 순서를 독립 기대값으로 검사한다.
Snapshot의 node별 member 수는4/2/2/1, Live는4/2/1/1이다. 같은 유닛도 node별 관계를 따로 보존한다.

필터 predicate도 canonical node ID를 소비한다. 이 식의 aggregate/read 번호쌍은 `(1,2)/(3,5)/(6,9)/(10,13)`이다.
시험은 각 member Read, aggregate 결과, 마지막 Add를 포함한 **전체 trace 벡터**를 명시적 기대 순서·수치와 대조한다.
Snapshot trace14개 / Live13개다. query 값끼리만 비교해 같은 잘못된 계산을 통과시키는 방식이 아니다.

소비 뒤 공격력을40/90/130/60으로 바꾸고 A의 두 슬롯도 비운다. 보관 frame/trace와 preview/explain 문서는 그대로다.
두 encoder를 각각 반복 호출해 bytes와 usage가 동일하고 gameplay·queue·facts·event log·record Arc/순번이 보존됨을 확인한다.

오류4사례는 먼저 실제 Damage1의 성공 record A를 만든다. 다음 transaction에서 첫 effect의 Damage1 command가 실제로 소비된 뒤
두 번째 관계 평가가 typed 거부되는 것을 tap으로 확인한다. accepted gameplay/queue/facts/event log와 같은 Arc A·ordinal1 및
두 query가 전부 보존된다. 이는 C06 관계 오류 경계의 증거이며 C10 전체 실패·취소 요구를 대신하지 않는다.

## 실행·원자료와 실패 이력

- [집중 영수증](evidence/c06_aggregate_final_001.json), [실행 로그](evidence/c06_aggregate_final_001.log): 3개 시험, 정상6/거부4의10개 시나리오.
- [출력 대조표](evidence/AGGREGATE_MATRIX.json): 원문 `C06_REPLAY` 18행의 preview/explain18쌍과 `C06_REFUSAL`4행.
  각 문서의 수치·node 결과·member 순서와 소비 후/거부 후 동일 bytes를 재대조했다.
- `c06_aggregate_001`: 2 pass/1 fail. 복수 식의 actual command/HP는 통과했으나 시험이 predicate ID를 빼고 view 번호를1/3/5/7로 가정했다.
  compiler의 canonical 번호 계약을 대조해 기대 번호만 교정했다. 당시 source archive와 실패 로그를 보존했다.
- 사용자 중단 시 최초 시험은 이미 exit101로 종료된 상태였다. 재개 때 실행 중 Cargo/rustc/시험 프로세스가 없음을 확인했다.
- 첫 원자료 요약 스크립트는 `empty_slot` 오류 사례를 정상 빈 집계와 같은 접두사로 분류해 assertion 실패했다.
  정확한 사례 이름으로 교정했다. 제품/시험 source나 기대 수치를 변경하거나 시험을 다시 돌린 결과가 아니다.

작업 중 별도 `core_enhanced/tests/battle_showcase.rs`가 추가되어 최초 전체 실행에 통합시험1개가 더 포함됐다.
이 C06 작업이 작성하거나 수정한 파일이 아니다. [동시 입력 기록](evidence/WORKSPACE_ADDITION.json)에 원본 byte와 시작 시 부재 근거를 보존했다.
최초 실행은 계획된344개만 source digest에 포함했으므로 최종 전체 입력 결속으로 인정하지 않고 모두 재실행했다.
최종 실행·동결은 이 외부 입력도 포함한345개 기준이며, C06의 신규 시험3개와 별도 통합시험1개를 구분한다.
시작 before343개 자료를 소급 수정하지 않았고 after archive/patch에 해당 파일을 보존만 한다.

[최종 판정표](evidence/FINAL_VALIDATION.json)는 최종 source의 check, authoring/value_expr, 기본 병렬 전체 target,
Clippy·변경 format 명령/건수/시간/source digest의 원본이다. 신규3개·authoring112개·value_expr104개·전체1556개(약107초)가 통과했고 check/Clippy/format은 exit0이다. 경고와 실패 원문도 보존한다.

PC36/R05는 **재실행하지 않았다**. [재사용 근거](evidence/EVIDENCE_REUSE.json)에 직전343개 중342개 byte 동일,
변경 parent에서 새 cfg(test) module 한 줄만 제거하면 before byte와 동일한 것, 선행 측정/도구/환경 대조를 기록했다.
PC36 성공31/typed거부5 및 R05 peak9,049,037 bytes / cap 초과18,849 bytes 단일 경고를 유지한다.
관측 on 비용·수명·취소 한도를 새로 입증한 측정은 아니다.

## 범위와 검토 인계

신규 관측 증거는 위 C06 조건, 기존 runtime 회귀는 최종 suite 실행, source/비용 근거는 재사용 판정으로 구분한다.
private `encode_record_for_test`는 selector를 건너뛰므로 public Latest/Exact·권한·battle/terminal 교체를 검증했다고 표시하지 않는다.
C07–C15, C03-OBS01 및 F03/D07-B/WP02/R06 전체는 미완료다.

[successor manifest](target/d07_b_f03_aggregate_001/MANIFEST.json)에 before343/after345개, C06 변경1개·추가1개 및 동시 입력 통합시험1개와
전후 archive·정/역 patch·지시서/검토/실행/실패 원자료를 결속한다.
[격리 복원](target/d07_b_f03_aggregate_001/RESTORATION_VERIFICATION.json) 및 `python3 freeze.py verify`로
source membership/currentness와 양방향 byte 복원을 확인한다. 최초 D07-B 대비 누적 delta는 최종 판정표에 있다.
다음은 사본 대화의 **C06 독립 단계검토**다. 후속 구현은 사용자 재개 요청 뒤 진행한다.

**부분 구현·자체 검증 종료 / 단계검토 target 동결 / 검토 요청 대기**
