# R06 ValueExpr·집계·Attack/SpellPower와 무기 성장 — Working Policy

2026-09-13 · P2G-10 complete; P2G-20 pending. 이 문서는 mutable Working Revision view이며 정책·구현·감사 완료 증거가 아니다.

현재 요청 범위는 [새 인수](REQUEST_INTAKE_002.ko.md)의 `single-r-v4-plan-only`다. 과거 P2G-00/10 payload의 실행 범위 표기는 당시 원본으로 보존하며, 현재 P2G-60 종료 경계와 구현 위임 철회가 우선한다. 이번 문서 수정은 P2G-20 실행·완료가 아니다.

## Metadata와 승인 경계

- Working Policy Document ID: `WPD-CAVE-20260913`
- Policy Topic: R06 ValueExpr·집계·Attack/SpellPower와 무기 성장
- Target Documentation Root: `core_enhanced/docs`
- Policy Directory Name: `content_authoring_value_expressions`
- Physical Path: `core_enhanced/docs/goals/content_authoring_value_expressions/WORKING_POLICY.ko.md`
- Current Working Revision / Checkpoint: `WR-CAVE-P2G10-001`; predecessor `WR-CAVE-P2G00-001`
- Policy Intake Snapshot: `PIS-CAVE-P2G00-001`
- Current Stage: P2G-10 completed; P2G-20 pending
- P2G-10 Completion Predicate: true, proven by [15 exit checks](P2G_10_VALIDATION_001.json). Current 9 elements/9 Decisions; 10 candidate dispositions; 0 user questions. No element is `code_reviewed` yet
- Requested Result / Progression: R06 `bounded_pipeline`; P2G-60 필수 검증과 상세 설계·기준선·계획 인계를 마치면 정상 종료. R 구현·감사 완료는 이번 종료점에 포함하지 않음
- Stage Selection Basis: 사용자의 계획 후 모델 교체 지시와 [새 인수](REQUEST_INTAKE_002.ko.md), AUTORUN `single-r-v4-plan-only` §§1/3/6/9/10. 마지막 formal checkpoint는 `WR-CAVE-P2G10-001`로 보존
- Allowed Actions: 재개 요청 시 R06 P2G artifact·source 검토·설계·문서 검증·인계 기록. GEX-00/activation/구현/감사는 별도 사용자 요청 필요. 이번 턴의 실행은 요청 경계 문서 수정·검증에 한정

## User-approved initial scope

I01–I05는 계획에 추적할 기능·최종 수용 범위다. 게임 구현·시험·감사의 실행 위임은 현재 요청에 포함되지 않는다. 아래 I06/O05는 새 사용자 지시를 반영한 mutable view이며 기존 initial/decided artifact를 소급 수정하지 않는다. 다음 P2G owner는 새 요청 경계를 인수하고 formal 요청 metadata 갱신이 필요하면 정상 successor 절차를 적용한다.

| ID | Disposition | Scope |
| --- | --- | --- |
| I01 | in | 유한 typed ValueExpr AST/bytecode와 상수·stat·HP·time·resource·result 참조, 합·곱·비율·clamp 및 유한 Count/Sum/Max, 단위·범위·checked overflow/0나눗셈·반올림·empty-set 검증, snapshot/live 평가 시점. |
| I02 | in | Attack/SpellPower와 직원·무기·modifier 성장/profile 합성을 한 번만 계산하고 Magic damage type과 SpellPower를 분리. 공격속도 point와 interval을 구분하며 기존 `scale_attack_interval_ms` 의미를 보존. |
| I03 | in | AT06의 stat/profile, inventory/player-spawn, preview/explain, combat-power input/cache, snapshot/save projection을 최소 동시 전환하고 새 상태가 있으면 owner/lifetime/restore/failure/cancel까지 함께 소유. |
| I04 | in | 새 RON/schema/opcode의 public parser→validation→compiler→executor와 negative fixture, 실제 resource/result/source-copy 입력, unsupported publication의 명시적 거부와 유한 실행 예산. |
| I05 | in | T06 전체와 R06에 적용되는 T18/T27, 직접 소비자·선행 계약 currentness, deterministic/cost/clone/fault 검증과 fresh 감사. R19의 Certified 최종 가중 모델은 구현하지 않되 truthful input 및 cache invalidation seam은 연결. |
| I06 | in | R06에 필요한 P2G-00→60, 상세 구현 설계와 exact Goal/Handoff 작성·검증, 계획 종료 기준선과 인계 보고까지. 완료 후 즉시 종료하고 GEX는 새 구현 요청 대기. 통상적 정책은 위임 조건과 exact 결정을 owner에 기록. |
| O01 | out | 서버·Unity/client, 외부 서비스/remote 배포, RON 조합기 UI, 전역 설치·설정. |
| O02 | out | 사용자 선택이 필요한 새 gameplay/balance/UX/저장손실/호환 결정, 근거 없는 Replace 우선순위·공식 수치·평가 weight·safety cap의 gameplay cap 승격. |
| O03 | out | R07–R24의 신규 기능. 특히 파편 사본화/구형 rank 제거, 전체 Flow/유물/보스/공식 콘텐츠와 Certified 최종 모델은 필요한 R06 seam 밖에서 미완료로 유지. |
| O04 | out | 개인 저장 삭제·자동 변환, 콘텐츠 이름 전용 executor, 외부 code eval, 빈 executor·fixture-only 성공·silent fallback, 미지원 기능의 허위 publication. |
| O05 | out | P2G 실행 중 AUTORUN/P2G/GEX/validator/AGENTS/runner/global 설정 수정 또는 artifact가 새 권한을 만든다는 주장. 이번 사용자가 별도로 승인한 요청 경계 문서 개정은 REQUEST_INTAKE_002의 한정 범위로 구별. |

## Current policy와 다음 owner

[P2G-00 initial artifact](P2G_00_INITIAL_001.json)는 사용자 요청과 exact repository reference를 의미 판정 없이 고정한다. [P2G-10 policy](P2G_10_POLICY_001.json)는 E01 typed unit/arithmetic, E02 snapshot/live, E03 finite aggregate, E04 Attack/SpellPower/weapon growth authority, E05 HP/time/resource/result reference, E06 attack-speed conversion, E07 AT06 projection/save/cache/explain, E08 public authoring/budget/fault, E09 compatibility/acceptance/later-owner boundary의 9개 current `decided` revision을 선택한다.

E02/E03/E05/E07은 [triage](P2G_10_TRIAGE_001.json)에 AUTORUN §3과 P2G §8.6의 다섯 조건을 각각 연결한 AI 위임 통상적 결정이다. 나머지는 사용자가 채택한 R06 카드의 직접 의미다. loss가 가능한 integral division은 rounding mode를 명시하고, Max(empty)는 typed error, Count/Sum(empty)는 0, resource 조회는 read-only, result는 같은 attempt의 settled current/earlier fact만 허용한다. final Certified weight·공식 수치·미정 Replace conflict는 결정하지 않고 later owner/out-of-scope로 유지한다.

[제한적 사실 확인](P2G_10_FACTS_001.json)은 현재 ValueExpr/SpellPower 부재, fixed amount schema, stat pipeline/weapon/growth, damage snapshot, R05 resource lease와 R04 action result, current Certified model의 존재를 current dirty bytes에서 확인했다. 이는 P2G-20 Code Review Binding이 아니다. 다음 owner는 P2G-20이며 parser→compiler→runtime→preview/explain/save/cache의 실제 consumer와 직접 predecessor output을 exact current source에서 검토한다.
