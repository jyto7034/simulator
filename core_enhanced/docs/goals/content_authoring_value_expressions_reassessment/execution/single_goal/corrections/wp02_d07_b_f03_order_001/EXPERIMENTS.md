# 수행 이력

## 시작 대조 · 2026-09-29
지시서·템플릿·C06 검토를 읽고 amount commit/select, 기존 multi-target·D04-F01·piercing·공개 TrialSession 시험을 확인했다.
선행345개 source 일치 및 양방향 복원 통과: evidence/ENTRY_PREDECESSOR_VERIFY.json.
기존 A Exact 시험은 미래 ordinal을 조회하므로 실제 교체된 A의 not_retained 증거가 아니다.
복수 target 순번은 마지막 값만 확인했고 piercing/miss 시험은 새 amount 관측을 켜지 않았다.
원본 before345개와 신규2개 파일 부재를 보존했다. 새 runtime 정책은 필요하지 않다.

## c07_order_001 · compile exit101
시험 구현의 Option<BattleKey> 해제 누락 및 QueuedEvent에 없는 Deref 사용으로 컴파일 실패(0개 실행). 로그/영수증과 실패 source archive 보존. 실제 view()와 명시적 key expect로 교정한다. AmountPlanKind의 serde snake_case도 코드 대조해 기대값 수정.

## c07_order_002 · 2 pass / 2 fail
첫 타입 오류 수정 후 runtime 복수순번 및 D04-F01 empty 후속 resource 시험 통과. 공개 fixture RON 교체에서 effect tuple 닫는 괄호까지 삭제해 로더가 거부했다. ModifyStats 닫는 괄호만 교체하도록 범위 교정. piercing은 두번째 적중 부재 assertion 실패하여 queue 잔여/투사체 좌표를 추가 관측한다(원인 미확정). 중단 후 재개 시 실행중 프로세스 없음, 정상 종료 영수증 존재. 실패 source/로그 보존.

## c07_order_003 · 3 pass / 1 fail
공개 loader→TrialSession→Latest/Exact A→B 교체 시험 통과. 관통 실패 원인은 시험의 80 event 임의 중단: 실제 재평가가 1ms 단위여서 77ms까지만 소비했고 queue에78ms 이벤트가 남았다. source/log의 C07_DEBUG가 lifetime500ms·현재좌표2.27·다음78ms를 보여준다. 제품 시간/속도/순서는 그대로 두고 500ms의 이벤트와 완료 이벤트를 소진할 충분한 event-count bound(500+16)로 변경하며 queue empty 및 time<=500을 강제한다.

## c07_order_004 · 3 pass / 1 fail
500ms queue를 끝까지 소비하자 두 적중·ordinal1/2는 통과했지만 마지막 empty impact 기대1 vs 실제0으로 실패. `advance_skill_projectile_inner`는 expired && hit_unit_ids.is_empty()일 때만 empty impact를 발행한다. 적중 이력이 있는 관통 만료는 carrier만 닫고 새 impact를 만들지 않는 기존 계약이다. C07은 이 경로의 마지막 record 유지와 실제 D04-F01 무대상 impact를 별개로 검증한다. 제품 동작 변경 없이 잘못된 시험 기대를 코드 계약에 맞춘다.

## c07_order_005 · 3 pass / 1 fail · C07-OBS01
두 적중의 실제 command/amount·순번1/2 및 만료 후 동일 Arc/ordinal2 보존 통과.
하지만 `active_skill_casts.is_empty()`가 실패했다. 만료 분기는 carrier만 닫고 `resolve_skill_step_delivery(... terminal=true)`를 호출하지 않아 pending_delivery_count가 남는다.
기존 `native_action_piercing_empty_final_expiry_closes_carrier_without_releasing_gate`도 zero의 damage gate를 계속 Pending으로 기대한다.
이 완료/후속 효과 정책을 임의로 바꾸지 않기 위해 C09 수명 단계에 명시 이관(권고) vs 이번 교정을 사용자에게 질문했다.
최종 PASS 근거가 아니라 실패 source/archive/log와 재현할 obligation으로 남긴다. 답을 기다리는 동안 다른 원자료/도구를 준비한다.

## 사용자 범위 결정
C07-OBS01은 C09 수명 검증에 명시 이관하고 C07 먼저 동결하기로 선택했다. 무제한 관통 만료의 record 보존·native carrier 종료를 검증하되 pending cast1은 알려진 미해결 상태로 표시한다. max_pierces=1의 정상 두번째 terminal hit 대조 사례도 추가한다. 0/2 damage 각각 expiry·hit_limit의4사례. 제품 코드는 그대로이며 F03 전체 완료 전에 교정 필수.

## c07_order_006 · 3 pass / 1 fail
C09 이관 후 만료·hit_limit 대조 확장. 내부 invoke_ability의 자동 정산 action을 public handle query로 조회하여 Unavailable로 실패했다. 종료 후 회수되는 이 fixture의 기존 계약에 따라 `runtime_fact_snapshot.latest_settled_action`을 조회하고 action ordinal을 대조한다. query에서 실패한 것은 C07-OBS01의 추가 증상이 아니라 시험 조회 경계 선택 오류다.

## c07_order_007 · 4 pass / 0 fail
최종 집중 시험에서 공개 query7쌍, runtime ordinal3/9/18/21/27/36, piercing0/2×expiry/hit_limit4사례와 D04-F01 empty resource3·완료를 확인했다. 무제한 관통은 native facts settled500ms·delivery0이지만 pending cast1이며 C07-OBS01 그대로다. hit_limit은156ms settled·pending cast0. check/authoring/value_expr/전체기본병렬/Clippy/format을 동일 source에서 시작했다.

## 최종 검증 / 판정표 교정
동일347개 source에서 check·authoring113·value_expr104·전체1560(132.3초)·Clippy·format 통과. 판정표 스크립트 첫 실행은 TraceEntry의 member_identity:null을 기대 벡터에 빼서 assertion 실패했다. 실제 TraceEntry 타입과 원문 대조 후 필드를 명시했다. script 원본과 오류 요약을 evidence에 보존하며 제품/시험 source와 실행 근거는 바꾸지 않는다.
