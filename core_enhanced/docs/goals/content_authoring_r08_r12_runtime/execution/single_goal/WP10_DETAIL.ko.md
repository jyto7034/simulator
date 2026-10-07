# WP10 상세 설계 — R09 중간 source 이후

현재: 기능 구현·최종 자체 검증 자료 동결, 독립 감사 요청 대기. 통합55조건 중54 자체 검증/X09 환경 제한1. 최신 판정은 COVERAGE.json / REVIEW_HANDOFF.md가 소유한다. 아래 단계별 메모는 당시 이력이며 독립 감사 판정이 아니다.

2026-09-30. 근거: ability.rs::SkillDef/SkillStepDef, data/skill_data.rs::RawSkillDef/bind_definition_sources, core/sim.rs::invoke_ability_with_accepted_target_tile/resolve_skill_step/execute_skill_step, skill_runtime/cast.rs::finalize_skill_step/resolve_skill_step_delivery. R09 lib regression002는1544개 통과/Unix socket fixture1개 환경 EPERM 제외이며 전체 broad PASS 아님.

## 작성 입력과 compile

기존 steps는 안정된 authored action/effect 정의로 보존한다. 선택적 flow AST는 Sequence/Parallel(All)/Branch/Repeat/Wait/Action을 제공하고 Action은 step_id를 참조한다. 반복을 Vec로 전개하지 않는다. legacy steps도 동일 compiled flow로 lowering하며 별도 flat executor를 남기지 않는다. compiled plan은 static node table과 index를 가지며 raw node/action/effect identity와 runtime occurrence를 구분한다.

Flow result는 producer node ID, metric, typed literal 단위를 검증한다. Sequence의 선행 EffectsSettled 결과만 다음 child에서 읽을 수 있다. Parallel sibling의 미완료 결과/forward reference/분기 내부에서만 생성되는 결과는 외부에서 읽지 못한다. Released leaf 결과는 미완료값0으로 투영하지 않는다. count와 HP 단위는 분리한다. R06 ActionSite/EffectSite 출처와 amount compile/consumer 경계는 유지한다.

위임 기술 한도는 raw depth64/node4096/repeat1,000,000회이며 모두 기존 native/authoring hard guards와 함께 적용한다. zero count는 거부하고 zero interval은 유한 work를 charge하며 허용한다. Repeat interval_ms는 이전 body milestone 완료 뒤 다음 기회까지의 간격이며 마지막 회차 뒤에는 기다리지 않는다. no-target도 기회를 소비한다. 합성 horizon과 base-time 더하기 overflow를 거부한다. 자기 호출/함수/재귀 문법은 없다.

## 실행

cast/action은 하나로 유지하고 flow scheduler의 bounded continuation이 leaf occurrence를 발행한다. 기존 Instant/Projectile/Area kernel과 step result/delivery 종료 callback을 재사용한다. occurrence key는 authored step index와 구분하며 stale wake는 exact cast/frame activation에 맞아야 한다. Sequence/Parallel은 OS 병렬화가 아니고 authored 순서의 deterministic scheduling이다. 종료한 반복 occurrence의 보관량을 회수한다.

전체 action Released와 leaf delivery completion은 별개다. flow가 끝날 때 control을 닫고 이미 방출된 delivery는 ticket/출처를 유지해 EffectsSettled까지 기다린다. 실패·취소는 미방출 continuation만 닫으며 released payload를 지우지 않는다. R08 milestone waiter와 R09 actual start/freeze를 그대로 연결해 재시험한다.

## 검증

A10-01~09: 공개 RON 광역3명→self-heal1회, authored/occurrence 주소, Repeat 중간 무대상/interval-only 변경, Branch/Sequence/Parallel join, delivery별 대상·anchor·facing·retarget, 취소와 stale wake, typed result 진단·overflow·depth/반복 제한, native/authoring fault rollback, R06 실제 HP-loss/Snapshot/Live/D04-F01/관통 만료 회귀. 구현하면서 실제 심볼·실패·교정을 현재 PLAN/EXPERIMENTS/NOTES에 갱신한다.
