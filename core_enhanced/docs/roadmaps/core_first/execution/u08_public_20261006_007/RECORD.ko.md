# U08 E1 보상 교정과 Continue 실패 — 2026-10-06/007

E001 타입 연결(EventId constructor), E002 authored skill RON의 i128 재직렬화 fixture 오류, E003 정상 보상 허용목록의 제품 누락을 각각 구분하고 실패 원본을 보존했다. E003에 대해 Sol은 mission_policy.rs의 ALLOWED_COMBAT_REWARD_KINDS에 Relic/RelicCorruptionCatalyst 두 항목만 추가했다. 기존 PE-CRNV-04의 정상 Core 지급·보상 범위이며 정책 확장이 아니다. featured reward 조건과 기존 공급목록은 유지했다. evidence/product_combat_reward_kinds_fix001 및 relic_combat_reward_kinds_fmt001을 따른다.

Astra E004는 source434/683f1d6d41d8baced21d5623d348dbefd09a332421c9354445a1f428258e199e 불변, exit101, 47.355571초다. 정상 보상→장착→SavePoint→Event 선택/decline→Load rollback 이후 fresh OS child의 CoreApplication Continue가 battle_runtime_retained_byte_limit/site relic_continue_value로 거부됐다(limit8,388,608/requested11,527,716). E1은 실패이며 그 뒤의 actual battle/trace oracle은 아직 검증되지 않았다. public_trial_cli004에는 고정 content/save/child stdout/stderr가 남아 있다. save.json 전체 포장과 구분한 실제 checkpoint compact UTF8는49,775bytes다.

같은 source에서 relic_reward_policy_regression001 3/3 PASS(exit0/4.436988초), relic_reward_authored_regression001 1/1 PASS(exit0/8.308059초). root는 세 receipt의 source 불변 및 raw SHA를 실제 대조했다. 기존 focused25와 이 보상회귀를 전체 U08 완료로 합산하지 않는다.

game/relic/allocation.rs의 lexical shape bound는 typed input decode에도 JSON Value/정렬 준비용 상계를 적용한다. continuation.rs의 restore는 decode 이후 canonical SHA 확인에도 admission을 수행한다. 11.5MB의 원인이 개별 bound인지 누적인지, 실제 필요한 allocation을 얼마나 초과하는지는 Sol이 분석 중이다. 아직 guard 수정 방법을 확정하지 않았다. 8MiB 상한·동일 fixture·회귀 oracle을 유지한다. Astra는 기존 allocation2/durable3/Eventowner4와 신규 Continue one-short atomic 경계 acceptance를 독립 검토 중이다. 모든 writer/runner HOLD, 제품 분석만 열려 있다. 프로세스 조회는 sandbox namespace에 한정되며 담당자 보고 runner0과 구분한다.

이후 제품 원인 교정→E1 재실행/관련 회귀→남은 carrier/literal/observer-off 증거→통합 검증 순서로 계속한다. cfg ActionKind 소유 test66→75 정정도 안전경계에 남아 있다. 기존 역할002의 자율 집중 실행 계약을 유지하며 별도 독립 전체 U08 감사는 최종 동결 후 진행한다.
