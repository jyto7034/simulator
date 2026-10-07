# D07-B F03 C07 — 최근 기록 교체·순번·0값과 무대상 구별

2026-09-29 · **C07 부분 자체 검증·단계검토 인계**. 독립 검토 PASS 또는 F03/D07-B 전체 완료가 아니다.

[교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)의 C07/B06/B09 및 이번 C16을 진행했다.
직접 선행은 [C06 사본 부분 PASS](../../reviews/wp02_d07_b_f03_aggregate_001_review_001/REVIEW.md)다.
템플릿 `2026-09-27-v1`에 따라 [현재 위치](PLAN.md), [시도 이력](EXPERIMENTS.md), [기술 판단](EXPERIMENT_NOTES.md)을 작성했다.

## 변경과 실제 경로

제품 실행 코드 변경은 없다. 두 cfg(test) parent와 신규 시험 파일 두 개만 바뀌었다.

- `src/authoring/trial.rs`: 기존 공개 amount 시험의 준비 코드를 `public_amount_trial_fixture`로 추출했다.
  기존 시험은 no-op configure로 같은 준비 문서·실행·assertion을 사용한다. 새 시험만 외부 RON fixture를 변경한다.
- `src/authoring/trial/tests/amount_order.rs`: 실제 loader → 공개 TrialSession prepare/advance → 공개 preview_amount/explain_amount → production selector 경로.
  record를 주입하거나 `encode_record_for_test`로 selector를 우회하지 않는다.
- `src/game/battle/core/mod.rs`: cfg(test) 하위 `mod amount_order;` 한 줄.
- `src/game/battle/core/tests/amount_order.rs`: 기존 runtime fixture·실제 ordered executor·command tap 및 projectile event queue를 사용한다.

동적 authored expression의 production publication 차단은 유지한다. 공개 사례는 legacy Damage/Heal이다.
Snapshot/Live 입력과 집계 trace의 선행 범위는 C04/C05/C06 PASS를 이어받으며, 이번에 그 전체를 다시 주장하지 않는다.

## 조건별 시험과 기대 결과

시험 접두사는 `d07_f03_c07_`다.

| suffix | 입력·경로·독립 기대값 |
| --- | --- |
| `public_replacement_rejects_actual_old_exact_and_retains_zero` | 기존 공개 스킬 준비 fixture의 첫 step을 Instant Damage1, 100ms 뒤 두 번째 step을 Heal0으로 구성. 공개 clock으로 두 경계를 분리. 처음에는 not_yet_evaluated, A ordinal1/Damage1, B ordinal2/Heal0. 같은 battle의 실제 A Exact는 B 실행 후 not_retained이며 B의 identity/수치를 반환하지 않는다. Latest와 B Exact는 B를 반환한다. |
| `multieffect_multitarget_same_timestamp_has_execution_order_ordinals` | 같은 10ms, 대상 순서 `[target,caster,ally]`와 `[ally,target,caster]`. 각각 Damage2, Damage2→Heal1, Damage2→Heal1→Damage0 prefix를 실제로 소비. 총6실행/36평가. 누적 ordinal3/9/18/21/27/36과 최종 effect index0/1/2·마지막 대상·amount가 일치한다. damage/heal command 전체 순서와 세 유닛 HP92를 독립 대조한다. 0은 command가 없어도 평가 순번에 포함된다. |
| `piercing_hits_replace_zero_record_but_empty_expiry_preserves_it` | 피해0/2 각각 무제한 관통 만료와 max_pierces=1의 두 번째 terminal hit, 총4사례. 실제 invoke→release→advance→impact queue로 대상 둘을 차례로 맞힘. ordinal1→2, 마지막 대상·amount·소비시각·event의 completion delivery identity와 record 일치. 피해0은 command 없이 HP100, 피해2는 command2개/각 HP98. |
| `d04_f01_empty_dynamic_impact_keeps_record_and_runs_following_resource` | 기존 D04-F01 fixture가 실제 DamageExpr/HealExpr projectile을 release하고 대상을 제거한 뒤 empty impact를 enqueue. impact 소비 전 관측을 켠다. amount command 없음·record 없음·ordinal0 유지, 뒤 SourceEmployee resource Grant3 실행, step/cast/queue/delivery 완료. release 시점의 관측 on 비용 검증은 아니다. |

공개 preview/explain은 **7쌍**이다: 초기 Latest, A Latest, A Exact, B Latest, 교체된 A Exact, B Exact, B Latest 재조회.
각 조회를 반복하여 문서 byte가 같고 accepted work·다음 step index·영수증이 변하지 않는지 확인한다.
A/B site와 typed amount·trace·kind 및 consume 시각 차이100ms를 검사한다.
이 사례는 같은 active battle의 실제 A→B 교체다. terminal 교체·손상·권한·한도·취소 전체 범위의 증거가 아니다.

## C07-OBS01 — 사용자 결정으로 C09에 이관한 미해결 사항

관통 projectile이 적중 이력을 가진 채 만료될 때, 기존 코드는 추가 empty impact 없이 native carrier만 닫는다.
이번 실제0/2 사례에서 native facts는500ms settled/outstanding delivery0인데 **active_skill_casts1개와 step pending_delivery_count1이 남았다**.
기존 `native_action_piercing_empty_final_expiry_closes_carrier_without_releasing_gate`도 damage gate 대기를 유지하는 계약을 가진다.
이 결과를 정상적인 시전 정리나 전체 수명 PASS로 간주하지 않는다.

사용자는 **“C09 수명 검증에 명시적으로 넘기고 C07을 먼저 동결한다”**를 선택했다.
따라서 이번 시험은 이 경로에서 최근 amount Arc/ordinal2가 보존되고 carrier가 닫힘을 확인하며, 알려진 pending cast1도 명시적으로 확인한다.
정상 terminal 대조인 max_pierces=1은 두 번째 hit의156ms에 settled/pending cast0이다.
미해결 재현과 소유 단계는 [C07-OBS01](evidence/C07_OBS01.json)에 기록했다. **F03 전체 완료 전에 교정해야 한다.**

C07-OBS01을 숨기려고 기존 실패를 지우거나 제품 cleanup을 우회하지 않았다. 첫 expiry cleanup assertion 실패 source도 보존했다.
C03-OBS01의 RON i32::MIN 역직렬화 실패는 그대로 C15 의무다.

## 검증과 실패 이력

최종 수치·명령·실행 시간·source 결속의 원본은 [FINAL_VALIDATION](evidence/FINAL_VALIDATION.json)이다.
[집중 영수증](evidence/c07_order_007.json)과 [로그](evidence/c07_order_007.log), [원자료 판정표](evidence/ORDER_MATRIX.json)를 연결한다.
최종347개 source에서 신규4개·authoring113개·value_expr104개·전체1560개(132.3초)가 통과했고 check/Clippy/format은 exit0이다.
전체 suite는 기본 병렬, 출력 대조용 집중 시험만 단일 스레드다. 긴 전투 시험은 추가하지 않았다.

실패 시도001–006과 각 source archive를 모두 보존했다.
001은 새 시험의 Option key/QueuedEvent view 타입 사용 오류, 002는 RON 교체 괄호 오류와80event 조기 중단,
003은 1ms projectile queue가77ms까지만 소비된 사실 확인, 004는 적중 후 만료에도 empty impact가 생긴다는 잘못된 기대,
005는 실제 시전 잔존 C07-OBS01, 006은 자동 정산 후 회수된 action의 handle query를 잘못 선택한 오류다.
007은 해당 fixture 계약에 따라 latest_settled_action의 ordinal을 대조하고, 사용자 승인된 C09 이관 범위를 명시하여 통과했다.
판정 범위와 기대값을 바꾼 이유는 [EXPERIMENTS](EXPERIMENTS.md)에 시간순으로 남겼다.

PC36/R05는 [재사용 판정](evidence/EVIDENCE_REUSE.json)에 따라 **F02 동결 측정을 재사용**한다.
변경한 두 parent의 production prefix byte가 같고, 기존345개 중343개가 완전히 동일하다. 추가한 두 파일은 cfg(test) 모듈이다.
PC36 성공31/typed 거부5, R05 peak9,049,037 bytes와 cap 초과18,849 bytes의 단일 경고를 유지한다.
새 비용 측정·관측 on 전체 비용/수명 보장이 아니다. 별도 시연 `tests/battle_showcase.rs`도 predecessor와 같은 byte로 유지했다.

## 동결과 다음 작업

[Successor manifest](target/d07_b_f03_order_001/MANIFEST.json)는 before345/after347개,
변경2개·신규2개, 전후 source archive와 정·역 patch, 지시서·검토·실패·최종 실행 원자료를 결속한다.
[격리 복원](target/d07_b_f03_order_001/RESTORATION_VERIFICATION.json)과 `python3 freeze.py verify`로 currentness 및 양방향 byte 복원을 확인한다.
다음은 사본 대화의 **C07 부분 범위 독립 검토**다. C08–C15, C09 소유 C07-OBS01과 C15 소유 C03-OBS01은 남는다.

**부분 구현·자체 검증 종료 / 단계검토 target 동결 / 검토 요청 대기**
