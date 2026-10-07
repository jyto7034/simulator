# U08 전체 독립 재감사 — u08_current_013 / pass002

판정: **Complete / Confidence High / open findings 0**. 대상은 GOAL-CRNV-01 전체 frozen 범위이며 F01/F02만 검사한 부분 승인이 아니다. source438의 기존 전체 정적 검토와 실제 실행 증거를 동일 bytes로 재사용하고, successor013의 새 입력·실행·문서·복원·현재성을 추가 검증했다. 이전 두 finding은 아래 근거로 폐쇄한다. R05의 기존 peak-only warning은 그대로 남으며 이 판정은 모든 성능 지표의 PASS를 뜻하지 않는다.

감사자는 최초 pass001부터 계획·설계·제품 구현·시험 작성에 참여하지 않은 독립 담당자다. 재감사에서도 제품·성능 시험, Cargo build 또는 finalizer를 실행하지 않았다. 기존 source, 시험, evidence, target, canonical 문서를 수정하지 않았으며 이 감사 디렉터리와 격리 임시 복원 공간만 사용했다. 새 probe/validator 작성에는 참여하지 않았다. 이 감사의 read-only validator 재확인은 기존 실행을 다시 분석한 것으로 새 제품 실행에 해당하지 않는다.

## 정확한 대상과 현재성

- Target: `execution/single_goal/target/u08_current_013/MANIFEST.json`, 336,578 bytes, SHA-256 `f4a1db5be99b390def61fd76224c073430416b2445479753a48458f5233b69a2`.
- Product: 438 files, SHA-256 `4c7bbd317a9a12045267bd306057e771133096ccbccace6a51f484d23a46e2ac`.
- Config: `evidence/finalizer_ready013/CONFIG_READY_013.json`, 14,528,205 bytes, SHA-256 `efcf41fc70d63ccee2759023394b5069e8086eea121b62f42ba5218c661992d8`.
- Entry `2026-10-07T07:14:25.458660Z`, exit `2026-10-07T07:26:50.614066Z`: 고유 현재 파일7,160개, source before577/after617, records6,518, references195 및 내부4개 archive 모두 exact bytes/membership 일치. 별도 MEMBERSHIP7,168행 전체도 일치한다. `ENTRY_IDENTITY.json`, `EXIT_IDENTITY.json`, `ADDITIONAL_INDEPENDENT.json` 참조.
- 독립 격리 복원: 원 before577개에 실제 forward patch 적용 후 after617개와 exact 일치, 실제 reverse 적용 후 before577개로 exact 복귀. workspace에는 reverse를 적용하지 않았다. 외부 U07M4개 및 보존된0124개 archive도 exact membership/bytes 일치. `RESTORE_INDEPENDENT.json` 참조.
- Product438, 원58 selected receipts, execution binding, functional quiescence, performance environments/artifacts, predecessor source delta/191조건은012와 구조·bytes가 같다. 32문장/10차원/3노드/16seam의 기존154 direct claims가 그대로이며 나머지 기존 선행조건 claims와 합한628개 기존 claim도 보존된다. 새27개만 추가되어655개다. frozen 문장을 수정하거나 기존 증거를 새 producer로 바꾸지 않았다.

## F01 폐쇄 — 적용 가능한 실행 경로와 정지 중 접수된 시전

원 pass001의 정상 직원 active AutoCastStart 요구는 감사자의 적용 가능성 가정 오류였다. 이는 단순히 시험을 생략하여 닫은 것이 아니다. `u08_current_012_pass001/ADDENDUM_002_AUTO_APPLICABILITY.ko.md` 및 exact source 검토를 그대로 유지한다. strict 직원 정의는 skill/mode를 등록하지 않으며 정상 직원 생성은 skill=None이다. 정상 fragment 장착은 Manual로 전환하고 form에도 Auto mode authoring이 없다. auto scheduler는 Auto mode를 요구하며 비직원 Auto actor는 직원 유물 binding 소유자가 아니다. Manual의 automatic_unit target selection과 active AutoCastStart를 구별한다. 기존 basic Auto/Triggered는 실제 양성 시험으로 확인되어 있다.

Root의 `evidence/finalizer_ready013/AUTO_APPLICABILITY_BINDING.json`은 이 기존 지원 소비자 해석을 계획·실험 snapshot,10개 exact source,기존 계약과 결속한다. 새 직원 Auto API나 내부 주입을 U08 수용 조건으로 추가하지 않는다. 원 frozen 기본·자동·명령 소비 요구는 지원되는 basic Auto와 Manual 명령 경로, 비직원의 직원 유물 기여 비적용을 구분하여 해석한다. 원 F01의 자동스킬 양성 요구는 철회된 부분이며, 별도 정책 변경이나 미지원 기능의 구현 완료를 인정한 것이 아니다.

남아 있던 실제 paused manual 소비 공백은 새 `evidence/audit_f01_paused_probe001/attempt003`의 공개-port 실행으로 닫힌다. 전체 main.rs, driver, runner, RON 입력, ORACLE, 잠금 의존성, 실패한001/002의 원인과002→003 diff, verifier 전체 및 tamper controls를 읽었다.

- 정상 RON loader→StartNewGame→정상 직원 선택→grant/equip→정상 encounter→deployment→host tick readiness→PauseBattle→BeginSkillTargeting/ActivateSkill 두 건→ResumeBattle를 거친다. admin은 시험용 relic/fragment/catalyst 공급만 한다. battle 내부 주입이나 제품 변경은 없다.
- 서로 다른 두 직원이 같은 `u08_trial_damage`를 정지 시각5,000ms에 접수한다. 실제 ManualCastStart seq16/18과 command identity가 일치한다. 접수 전후 HP/time와6개 damage counter/witness가 불변이다. resonance 소비20→22는 정상 접수 비용이므로 전체 접수 FactSummary 불변을 요구하지 않는다.
- 정지 중100ms host tick5회는 실제 raw에서 모두 null이며, command checkpoint의 HP/time도 불변이다. 매 tick의 full RuntimeFactView equality는 exact 실행 source의 fail-closed assertion 및 exit0에 의해 입증된다. 매 tick의 full facts가 별도 raw JSON으로 저장된 것처럼 주장하지 않는다.
- Resume 뒤1ms에서 Corrupt210(seq24, HP1,000,000→999,790), 이어 Normal144(seq26, HP999,790→999,646)가 실제 적용된다. 기대값은 제품 evaluator가 아니라 authored literal100과 고정 flat/percent의 `(100+40)*150/100=210`, `(100+20)*120/100=144`로 독립 계산했다. clamp 없는 실제 HP 감소와 정확한 segment가 같다.
- actor/employee/copy/effect/variant/catalog generation, 원 fragment copy/form/SkillId, 원 authored file/site, 각 cast/action identity, ManualCastStart→AbilityCast→AbilityStepTriggered→HpChanged 부모 관계를 raw와 transcript에서 다시 계산했다. source binding은 서로 다르며 다른 modifier 기여는 없다.
- retained Arc 확인 대상은 employee0 Normal hit이며 두 hit가 발생한 뒤 확보한 것이다. 추가1ms 후 unchanged assertion은 유효하지만 이를 시간상 첫 hit의 동시 처리 보존 증거라고 확대하지 않는다. active reconnect query는 socket detach/reattach 시험이 아니다. 기존 실제 Host detach/attach 증거는 별도로 유지된다.

실제 producer receipt는05:48:49.253222Z 시작,62.686023초,exit0이며 84 fail-closed checks,20 commands,57 ticks,3 admin 공급을 남겼다. raw86 JSON documents 중80개 operation이 TRANSCRIPT와 exact 순서/내용 일치하고 raw completion이 SUMMARY와 같다. 직접 재계산한 결과는 `SUCCESSOR_INDEPENDENT.json`에 있다.

Prepared manifest가 실제 명령의 hash와 같고 primary35/base1,322개 입력이 각 before/after snapshot과 archive에 exact 일치하며 모두 실제 실행 시간창을 감싼다. 각 시점 writer0이고 별도 union1,356개로 결속된다. 독립 crate의140개 lock package 중 probe 제외139개의 name/version/source/checksum이 원 Cargo.lock basis에 있다. raw의 실제 `cargo build --offline --locked` dev build와 소비된342,503,992-byte binary SHA `676769dc06bfa653ba57e0bf4d767ef5ed97d306354735e91c273bbd5bc866a9`를 현재 bytes와 확인했다. 역사적 release binary를 이 probe의 실행파일로 오인하지 않는다.

새 evidence는 별도 producer와 actual validator receipt/raw/result로 결속되며 원58역할에 가짜 selector로 삽입되지 않는다. actual validator013 exit0 및 명령/원시로그/source/boundary를 확인했고, source를 읽은 verifier를 read-only로 재사용한 결과도 성공했다. 이 재사용 결과는 독립 raw 재계산과 구분해 `REVIEWED_VERIFIER_RECHECK.json`에 보존했다.10개 tamper rejection은 source/raw/oracle/damage/누락field/actor/causality/dependency/시간창을 대상으로 한다. 새27 claim은 기존 주장의 보강이며 새 withdrawal·소켓·자동스킬·성능 실행으로 확대되지 않는다.

## F02 폐쇄 — successor의 현재 canonical index와 복원

현재 `core_enhanced/docs/README.md`는18,658 bytes, SHA `64e33ca0632cefbbb14b1b4e88764f5be19080f811b4ff08703daa2824993472`이다. 원012에서 드리프트한18,356-byte 버전은 보존되며 Root035 `readme_delta002/REVIEW.json`은 현재 upper 링크 경로 교정과 lower 계획 색인 행을 인수한다. upper/lower immutable reviewed snapshot의 적용 범위·활동 배정·후속 검토 문구를 직접 확인했다. 새 수량/가중치는 지도·활동 구성 초안이며 U08 유물 계약의 canonical 변경이 아니다. 개별 최종 보스 구현 완료로 확대하지 않는다.

이 README의 현재 bytes가013 source archive/canonical references/membership 및 entry/exit와 일치한다. 다른 작업을 되돌리거나 과거012 target을 덮어쓰지 않았다. 따라서 원 F02의 전체 target currentness 불일치는 successor013에서 해소됐다.

## 전체 Goal 재검토와 증거 범위

`CONDITION_REVIEW.json`은16군/32문장,10차원,3노드,16seam,191 선행조건(187 affected/4 reviewed_nonimpact),별도R06 31행 전체를 기록한다. 아래 범위는 원 pass001의 직접 source/actual assertion 검토를 동일 product/source/evidence에서 재사용하고013 보강을 합한 판단이다. 모든 과거 시험 줄을 새로 실행하거나 모두 독립 증명했다는 주장은 하지 않는다.

| 범위 | 검토된 핵심 경계 및 판정 |
| --- | --- |
| 01–03 사본·슬롯·이전 | UUID 사본/immutable 정의 분리,3슬롯·선검증 이전·대칭 비호환, forged slot graph Continue 거절, 실제 lethal settlement에서 슬롯 회수와 copy 보존. 충족. |
| 04 획득·분쇄·판매 | 정상 grant/reward, 특정 사유 판매 거절, 취소 불변, dust overflow 이전 refusal, 마지막 copy 삭제 뒤 UUID stream 보존. 충족. |
| 05–06 조건·수치 | same-effect tag AND와 exact equipment/fragment/form,checked i128 flat합→percent합→한 번 적용,6slot permutation, 실제 mitigation와 raw attack 보존. 충족. |
| 07–08 소비·출처 | Manual literal/Expr, observation-off 두 직원, basic Auto/Triggered, released Snapshot projectile/fixed area, follow 취소, Live missing-context atomic failure, catalog/generation/action/provider guard 및013 paused144/210. 적용 가능성은 위 F01 절. 충족. |
| 09–11 침식·원자성 | authored Corrupt 효과/negative penalty,유료1촉매와무료Event 분리, exact scene/node/history authority,once/idempotence,late output admission 실패의 모든 root rollback. 충족. |
| 12–13 복원·수명 | current/checkpoint/entry/Continue 동시 전환,generation/graph typed failure,실제 Host detach/attach pause/resume,fresh process/retreat/reentry/redeploy,3회 Weak owner/capture 회수. 충족. 전체 process heap0 주장은 없음. |
| 14 공개 작성 | 독립 새 ID RON loader→grant→Event/decline→실제 Normal180/Corrupt210→save/fresh process,비권위 prospective query,추가 공개 paused144/210. 충족. |
| 15 예산·호환 | 실제 one-short/exact admission,cumulative retained seed/rollback,empty legacy owner 무추가 계량,관측 modes,allocator/PC,typed HostTick fault/복구. 아래 warning/측정 한계를 유지하여 frozen 범위 충족. |
| 16 증거·독립 완료 | 원58실행과 별도 supplemental,원시 assertion/입력·시간창 결속,fulltarget 독립 forward/reverse 및 entry/exit exact. 본 전체 독립 Complete/High/open0 판정으로 충족. |

선행조건 delta102개와 compiler publication 교정 영향,4개 비영향 문서의 source 근거는 그대로 유효하다. R06 mixed Resource A/B static/dynamic Sum2건은 actual raw compile `value_expr_unit_mismatch`의 expected/actual unit을,같은-resource positive2건은 actual validated를 다시 비교했다. assertion 단순 PASS 요약으로 대신하지 않았다.

`RECEIPTS_INDEPENDENT.json`은58 exact command/raw/source/exit,655 claim-selector,249 formatted assertions 및426 artifact checks를 검증했다. 별도 `SUCCESSOR_INDEPENDENT.json`은 format 생략 assertion까지 포함한7,265 pointer assertion 및1,466 unique refs,새 raw/인과/수치/입력/lock을 독립 검사했다. Core default/fixtures1967/1967,observation1986;Host94/105/124;heap1/1,PC1의 기존 실제 결과가 그대로다.57역할 exit0,R05만101이며 supplemental은 별도exit0이다.

원 composite007/008/focus005/009/010의17/7/1/24/9 역할과1,958 union은 role별 timely before/after/dependency/flat archive와 gap으로 결속된다. PC/R05 functional projection50개가 원 nested receipts와 같고 기능 종료→quiescence관측→측정 시작의 순서를 재검사했다. **새 paused probe는 이50개 또는 과거 성능 정숙구간에 소급 포함되지 않는다.** 제품 bytes와 성능 입력이 변하지 않았으므로 새 시험 기록을 만들기 위한 성능 재실행은 요구하지 않는다.

## 유지되는 경고·한계와 정정

PC576행은 **immediate U07M→현재U08에서 시간 외 모든 값이 exact 동일**하다. 재귀 비교에서 차이는 measurement elapsed_ns576개와setup elapsed_ns192개뿐이다. 원 pass001의 +200 bytes를 immediate U07M 증가라고 적은 부분은 감사자 오류였고 `ADDENDUM_001_PC_BASELINE.ko.md`가 정정했다. +200은 더 오래된 starter-tactics baseline→현재 비교의 native retained/owner extent/refusal requested 차이다. 두 경계를 혼동하지 않는다.31성공5typed refusal,기존17 negative controls 및 불변 cap을 보존한다.

R05 peak **9,683,689 > cap9,030,188**,초과653,501 bytes,raw101은 그대로다. immediate U07M9,669,160 대비14,529 bytes 증가이며 다른 hard guard 실패는 없다. retained4,196,928은 cap와 같고 operations370,928≤389,240이다. frozen16 범위가 기존 peak-only warning 보존을 명시하므로 신규 open finding으로 중복 계산하지 않는다. cap 인상이나 증가 없음·전체 성능PASS를 승인하지 않는다. shared-host latency는 gate 불가이며 admitted extent/allocator live/요청량/RSS를 구별한다.

역사 native observation-on binary SHA 미보존(null)을 그대로 기록한다. 새 off/aggregate와 source/input/build/raw의 확인은 존재하지 않는 옛 binary hash를 대체하지 않는다. WebSocket007 no-fault 실패는 보존되며 successor005의 정상 RON delayed wave,8800ms 후 실제 HostTick work1148>cap1000,sticky fault/거절/reconnect/replacement/quit/Continue는 기존 검토대로 유효하다.20분 stress는 frozen U08 범위 밖이다.

현재6개 정책의 dealer-first/첫게임 뒤 gambit/양수 반복/무이자 부분 담보/나가기 완료,stress XP1회·공통 보상은 후속 U16/U13/U14/U26 책임이다. 기존4개 정책 bytes의 archive+recorded diff 재구성과2개 current-only review라는 한계를 유지한다. 새 README 계획 색인도 U08 사본3슬롯/수치/once corruption/atomic 계약을 바꾸지 않는다. 이 감사는 후속 콘텐츠 기능을 구현 완료로 선언하지 않는다.

본 verdict는 위 exact target과 source,검사 시점의 현재성에 한정된다. Root의 canonical 완료 기록/roadmap handoff는 이 외부 감사 산출물을 참조하여 진행할 수 있다. 감사자는 기존 target의 pending 표기를 덮어쓰거나 후속 U09 작업을 수행하지 않았다. 원 pass001 report와두 addenda는 역사 기록으로 보존하며 현재 verdict는 이 pass002가 대체한다.

감사 보조 parser의 초기 JSON-line format 분기 오류는 감사 도구 준비 오류였으며 수정 후 전체7,265 assertion 재검사가 성공했다. 기존 제품/증거 실패를 삭제하거나 성공으로 바꾼 것이 아니다.
