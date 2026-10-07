# U09 코드 검토와 구현 인계

상태: P2G-20 review body complete; current publication은 `P2G_20_RESULT_001.json`의 전체 reference/coverage 검증 결과가 소유한다. 정책 입력 basis는 `WR-CRSA-002`, source plan은 `RSP-CRSA-002`다. Root Astra/High가 RQ01/03/05/06 및 전체 설계를 소유하고 Sol 제품 담당자의 RQ02/05, Astra 시험 담당자의 RQ04 검토를 결합했다. 제품·시험 변경과 신규 실행은 이 검토에서 하지 않았다. 독립 완료 감사가 아니다.

## 승인 목표와 관찰 조건

`P2G_10_POLICY_001.json`의 17개 decided revision과 각 2개 Observable Policy Acceptance가 원본이다. 이 검토는 문장·정책 의미를 수정하지 않는다. 유한 seeded 활성, typed property/구조 recipe, 개인 plan, 포큐버스·태아·풍운승려의 실제 실행, 정상 외부 RON/조회, 소유·저장·예산·회귀·최종 독립 감사를 인계한다. 상세 규칙은 `WORKING_POLICY.ko.md` PE01–17을 따른다. 전체 공식 조우·밸런스, U10 반응계, Unity, 최종보스 고유 구현은 포함하지 않는다.

## Current Actual — RQ01

- `src/game/data/relic_data.rs`의 `RelicDefinition`/corruption 효과는 `Vec<RelicDamageEffect>`이며 selector는 damage target/type, 같은 효과의 all_tags, equipment/fragment/form이다. `validate_contract`와 `validate_references`는 정의 ID/효과 ID/배타 조건/형태 참조와 양수 flat/percent의 기존 계약을 검사한다. grant·closure·typed recipe schema가 현재 있다고 주장할 수 없다.
- `src/game/ability/authoring.rs`는 ActionSite/EffectSite와 source를 구분한다. `EffectPredicate::matches`는 같은 효과의 tag와 literal property를 AND한다. `EFFECT_SUPPORT`는 Instant literal Damage/Heal의 Amount만 편집 대상으로 표시하고 DamageType은 읽기 전용, Target/Delivery는 action 소유로 제외한다.
- `src/game/data/skill_data.rs:640`의 selector와 `set_effect_amount`는 게시 전 candidate definition의 제한된 literal 수정이다. 개인 effective plan을 만들거나 지원되지 않는 delivery/property를 변경하는 API가 아니다. step 공용 target/delivery를 effect write로 오인하지 않는다.
- `src/game/data/mod.rs:1084`는 유물 계약/참조 검사 후 resource/skill 참조·ValueExpr catalog compile를 하고 database를 반환한다. `src/game/data/content_pack/mod.rs:155`는 최종 winning definitions의 semantic identity를 계산하고 assemble→validated database→runtime publishable amount plans→generated combat preview 검증을 마친 뒤 generation을 게시한다. `raw.rs`는 Relic을 독립 string ID 정의로 취급하고 실제 pack source로 진단을 재배치한다. 새 schema도 이 실제 경로에 연결해야 한다.
- `src/game/value_expr/catalog.rs:92`의 compile은 SkillDatabase의 모든 step/effect를 순회하고 compiled Flow의 settled-prefix와 effect source를 사용한다. Damage/Heal/Shield expression을 source EffectSite에 결속하며 AmountPlanKind·capability·참조·결과 계약·중복 주소를 검증한다. 새 generated effect를 실제로 실행하려면 generated 주소·source와 compiled amount/Flow 관계도 유효해야 한다. 원 catalog에 존재하지 않는 주소를 넣기만 해서는 실행할 수 없다.

## Current Actual — RQ03

- `src/game/relic.rs:243`의 loadout 검증은 3슬롯·한 사본 정체성과 중복/상호배타 정의를 검사한다. `equip`은 대상과 이전 소유자의 후보 슬롯을 검사한 뒤 admission/poll을 거쳐 commit한다. structural write conflict나 활성 closure 검증은 현재 없다. corruption도 variant 변경이며 새 plan 성공을 먼저 보장하는 경계는 추가해야 한다.
- `src/game/world/relic_runtime.rs`의 query는 정상 employee draft/profile/최종 stat를 준비하고 missing weapon은 provider-unavailable로 표시한다. `src/game/relic/plan.rs`는 Damage/DamageExpr와 basic damage만 열거하고 legacy selector/flat/percent를 다시 계산하는 읽기 전용 prospective DTO다. actor/action/epoch 실행 권한은 갖지 않는다. generation·equipment persistent copy·fragment snapshot과 active/dormant 이유를 이미 표시하므로 새 plan query의 호환 기반으로 쓸 수 있다.
- World equip/unequip/corrupt/dismantle는 허용된 비전투 context를 검사한다. `src/game/world/application.rs:400`의 staged draft가 operation→death/relic/resource/lifecycle reconciliation→Continue refresh→final validation/poll→PreparedApplicationCommit을 소유한다. catalyst 비용과 variant 또는 슬롯의 늦은 실패도 이 외부 원자 경계를 보존해야 한다. 새 compiler를 성공한 query 뒤에만 배치하면 장착 명령의 원자 거절을 충족하지 못한다.
- `src/game/world/combat.rs:480`는 현재 inventory/database/generation을 capture한 RelicNumericOwner를 combat setup에 전달한다. 동일 파일의 `handle_preview_skill_target`은 실제 battle의 `validate_manual_skill_activation`을 사용한다. `tactical_preview.rs`는 이동/swap과 runtime range preview를 호출하므로 footprint의 표시도 effect-aware 소비자와 일치해야 한다. 별도 query 계산만으로 readiness/preview/실행 일치를 주장할 수 없다.
- `src/game/world/state.rs:560`의 RunCheckpointPayload는 relic inventory/variant/slots, crafting resources, UUID stream을 capture/restore하며 immutable plan은 저장하지 않는다. `node_flow.rs:1365,1604`는 exact content identity와 relic ownership graph를 검증한다. `active_run/continuation.rs:712`는 version/hash/전체 payload 검증을 마친 뒤 current와 SavePoint를 복원한다. plan은 현재 validated 구성에서 재구성하는 것이 이 소유 계약과 맞는다. inventory만 검증하고 구조 충돌 graph를 신뢰하는 복원은 허용하지 않는다.
- `src/application.rs:1328`은 새 generation 설치를 StartNewGame에 한정한다. `game_server/src/game/player_game_actor/handlers.rs:270`은 Continue를 durable store snapshot에서 복원하고 일반 mutation은 `prepare_command`→durable publication, query는 application execute 경로로 처리한다. host 전용 구조 권한이나 두 번째 유물 owner는 필요하지 않다. 새 DTO를 기존 command envelope에 연결하고 actual adapter serialization을 검증해야 한다.

## Current Actual — RQ05/06

- `src/game/relic/allocation.rs`는 continuation JSON Value/decode/normalization key/buffer/Arc와 profile/DTO projection을 실제 producer 전에 나누어 계량한다. 이 serialization bound를 native plan clone/retained의 대체값으로 쓰지 않는다.
- `src/authoring/clone_shape.rs:27`의 controlled_clone은 native preflight와 authoring CloneMeter를 clone 전에 실행한다. Arc/collection의 actual 소유 형태에 맞는 visitor가 필요하며 clone하면서 크기를 재는 방식은 안 된다. `src/authoring/control.rs:586`의 admit는 current+additional checked 요청과 limit/peak를 기록한다.
- `src/game/battle/core/mod.rs:549`의 authoring runtime count는 authoring이 꺼지면 반환하며 materialized entity/record 수를 합한다. native byte/storage admission과 별개다. 새 plan/closure/candidate/source/payload를 둘 중 한 곳만 추가하면 충분하지 않다. runtime owner·해제 상세는 Sol RQ05 검토를 함께 사용한다.
- 정책 사실은 P2G-00 exact intake와 P2G-10의 한정 policy evidence archive를 재사용한다. 선행 U08은 독립 audit013 Complete/High/open0 및 `u08_closure_20261007_001/PUBLICATION_RECORD.json`의 현황문서 delta로 인수했다. 새 검토는 현재 workspace의 후속 구현 delta이지 선행 완료를 소급 취소하는 발견이 아니다. U08 32+선행191 및 별도 R06 31의 실제 ID·영향은 RQ04의 mapping과 이후 GEX evidence selection이 소유한다.

## Proposed Implementation — TD01 유한 활성과 typed authoring

제안이며 현존 API나 확정 RON 문법이 아니다. 기존 legacy damage 효과를 호환 입력으로 유지하고 typed selector/condition/grant/modifier/recipe를 명시적인 variant 또는 새 optional 필드로 확장한다. parser가 불명확한 untagged fallback으로 unknown 필드를 버리게 하지 않는다. authored definition, validated compiled recipe, employee/provider effective plan, activation runtime state를 서로 구분한다.

한 구성에서 잠재 generated identity는 유물 사본·authored grant site·원 provider/effect site의 유한 조합으로 먼저 bound한다. grant activation이 새 grant definition을 무한 복제하지 않는다. 현재 원 효과와 unconditional grant를 seed로, 아직 활성화하지 않은 유한 항목만 추가하는 worklist로 최소 집합을 구한다. 이전 plan/과거 활성은 seed가 아니다. 효과 단위 AND와 별도 exact identity/form을 유지한다. 슬롯/UUID 순서는 provenance 정렬에만 쓰며 활성·값·충돌 선택의 우선순위가 아니다.

긍정 순환은 seed 유무로 평가하고 일괄 금지하지 않는다. NOT/속성 제거/자기 변경 의존은 정적 의존 분석으로 안정성이 입증되는 범위만 허용하며 부정 순환·무한 identity·입증 불가능한 nonmonotone graph를 typed 진단으로 게시 전에 거절한다. 전체 효과를 먼저 전부 활성화한 뒤 필터하는 방법은 자기 bootstrap 위험 때문에 채택하지 않는다. 고정 횟수 loop로 부분 결과를 성공 반환하지 않는다. work budget 실패는 gameplay cap이 아니라 기존 typed fault다.

typed capability는 현재 실행 가능한 effect/action domain과 단위, Add/Scale/Replace 여부, recipe 전후 support를 명시한다. property presence selector와 편집 권한을 하나의 문자열 field path로 합치지 않는다. 최종 구조에서 제거된 area property는 더 이상 modifier 대상이 아니다. 표현 방식·enum 이름·dependency proof의 내부 자료구조는 Sol 상세 설계 재량이지만 지원 descriptor/validation/query/실행 의미와 finite proof는 필수다.

## Proposed Implementation — TD02 결정적 합성과 구조 검증

같은 typed 수치 property의 Replace 동의값은 한 기준값으로 합치고 모든 provenance를 남긴다. 다른 Replace 또는 겹치는 비호환 structural write set은 순서와 무관한 typed conflict다. 기준값→동일 단위 Add 합→같은 종류 Scale 증분 합→한 번 scale/정수 절삭을 checked 연산한다. 기존 numeric damage flat→percent와 후속 전투 저항 순서를 보존한다. Shield 단일2S는 원 source 초기 계산의 결과를 한 번2배로 하는 recipe이지 receiver modifier나 임의 범용 percent와 순서를 뒤섞는 규칙이 아니다.

recipe는 effect writes와 carrier/qualification/source/channel/action/cost/time/resource preserve를 선언한다. 필요한 효과 묶음을 후보에 원자 적용한 후 effect source/attached-to/Flow/amount catalog/target capability를 검증한다. Monk의 damage footprint를 step 공용 target/delivery 치환으로 구현하면 같은 carrier의 heal이 변할 수 있으므로 effect-local footprint 또는 그와 관찰상 동등한 검증된 lowering이 필요하다. lowering을 고르면 원 effect 주소·settled metrics·1회 heal·비용·시각·queued ownership을 증명해야 한다. 최종 타입·함수 배치는 구현자가 제안하고 시험 담당자가 같은 의미를 독립 입력으로 검증한다.

## Proposed Implementation — TD03 개인 plan과 원자 전환

shared immutable catalog를 수정하지 않고 순수한 plan builder가 현재 configuration으로 effective graph+diagnostic+provenance를 만든다. key에는 generation, employee/provider, 원 skill/equipment persistent copy/fragment copy+form, relic copy/variant 집합이 포함된다. 같은 SkillId만으로 cache하지 않는다. no-relic fast path는 기존 원형 경로와 수치를 유지한다.

query/prospective validation과 battle materialization은 이 공통 compiler 결과를 소비한다. 장착·이전은 영향을 받는 두 직원, corruption은 변형 사본의 장착 직원, equipment/form/fragment 교체는 해당 직원의 후보 구성을 외부 staged draft 안에서 검증한다. conflict·overflow·admission failure에서 저장/슬롯/비용/UUID/RNG/기존 plan에 부분 commit이 없어야 한다. 조건 부재는 정상 dormancy이고 missing weapon은 기존 provider-unavailable이며, 불법 write conflict를 dormancy로 숨기지 않는다.

정상 supported consumer의 readiness/preview/cast/basic과 paused command가 선택한 plan은 실제 queued/released projectile/area/delayed owner까지 전달한다. 새 provenance가 실행 capability를 만들지 않으며 원 source/provider/action/epoch 검증과 에고 침식 억제를 유지한다. Snapshot/Live는 기존 평가 시점 의미를 보존한다. battle-local owner는 허용된 release가 끝날 때까지 살고 마지막 참조가 사라지면 회수한다. 재구성된 현재 plan에 stale owner를 다시 seed로 넣지 않는다.

## Proposed Implementation — TD04 저장·공개 조회·호환

RunCheckpoint/entry/Continue는 현재 사본·variant·slot·비용을 권위로 보존하며 compiled plan/closure index를 신뢰된 저장 입력으로 추가하지 않는다. restore의 generation/ownership 확인 후 새 plan 유효성을 검증하고 현재 콘텐츠에서 재구성한다. 원 SavePoint/current/entry의 동시 복원과 fresh Core 경로를 실제 실행으로 확인한다. 새 상태가 live battle에 필요하면 그 native clone·host detach/attach lifetime을 포함한다.

기존 relic query/result envelope의 호환성을 유지하는 확장 DTO 또는 별도 typed query의 구체 선택은 Sol이 제안한다. 반드시 active/dormant 이유, generated/원 effect 주소, 전후 typed 값·support·source를 읽기 전용으로 설명하고 정상 command/serialization path로 노출한다. 외부 RON의 새 ID로 load→validate/compile→register→정상 reward/equip→실제 cast→query가 닫혀야 한다. 내부 fixture 주입은 집중 검증에 쓸 수 있으나 정상 경로의 대체 증거로 세지 않는다. 세 대표 구조의 S/기간/비용/시험용 penalty는 representative fixture임을 표시하고 미정 공식 밸런스로 승격하지 않는다.

## Proposed Implementation — TD05 예산·전환·실행 범위

closure frontier/candidate graph/compiled plan/source strings/DTO/payload와 retained owner의 실제 allocation 전에 admission한다. 누적 temporary peak와 최종 retained를 구분하고 Arc 공유/소유 clone/rollback을 각 actual 경계에 계량한다. observation off/on/saturated가 gameplay 결과나 safety를 바꾸지 않는다. 실패 후보는 drop되고 이전 상태가 보존되어야 한다. 구조를 지원한다는 이유로 기존 PC36/R05 workload/cap 또는 oracle를 줄이지 않는다.

공유 schema/compiler/plan/consumer/공개 진단/계량이 함께 있어야 유효하므로 구현 단위를 지나치게 분리하지 않는다. 내부 편집 순서는 정할 수 있지만 schema-only 또는 query-only 중간 상태를 독립 제품 완료로 인수하지 않는다. source·test 파일 소유가 겹치면 순차 이관하고 제품 통합 runner와 시험 focused runner를 동시에 실행하지 않는다. 첫 작은 연결 검사 이후 필요한 기능·fault·회귀·성능 검증과 원인별 교정까지 Goal 범위에 포함한다.

## Finding triage와 남은 정보

F01(PE01–07,12,13): 현재 damage-only schema/query와 새 closure/typed 구조 사이의 차이. primary `implementation_finding`, disposition: 위 TD01–03 및 후속 Goal 구현. 기존 정책 acceptance 수정 없음.

F02(PE06–15): configuration transaction/restore에서 새 plan 검증, released owner와 source lifetime, allocation 확장 필요. primary `implementation_finding`, disposition: TD03–05와 runtime 검토 결합.

F03(PE08–11,13): 기존 aura/shared shield는 radius/self-target 구조이고 effect-local footprint는 현재 없다. primary `implementation_finding`, disposition: runtime 상세 검토의 필요한 owner/binding/consumer를 인계. U09 이전의 결함이라는 주장 없음.

F04(PE13,16,17): 기존 private setup 시험은 공개 authoring 소비 증거와 구분해야 한다. primary `implementation_finding`, disposition: Astra RQ04의 공개 seam 및 독립 oracle 사용. 옛 시험 삭제·작성자 변경 없음.

공식 S식/기간·침식 패널티·전체 조우는 범위 밖이며 검토 blocker가 아니다. 실제 새 기능·성능 비용은 미실행이므로 미검증이다. 내부 타입·cache layout·구체 lowering은 Sol 상세 설계에서 결정하되 위 공유 의미가 바뀌면 Root로 반환한다. 양 담당자의 필수 runtime/test 검토를 인수했으며 material policy finding0, 현재 코드 검토를 막는 필수 evidence gap0이다.

## Current Actual — RQ04와 검증 인계

시험 담당자의 `review_parts/TEST_SEAMS_REVIEW_001.json`을 전체 검토하고 30개 source hash 및 34개 acceptance의 exact 입력 revision/ref/text 일치를 확인했다. T01–T12는 12개 oracle 계열이며 12개 함수만 작성하면 된다는 뜻이 아니다. E01/E02는 회귀·완료 절차를 별도 소유한다. 보고서의 `minimum_new_evidence_groups`, `acceptance_mapping`, `regression_handoff`를 상세 검증 계획으로 인수한다. 새 기능은 아직 실행되지 않았다.

T01 closure/같은효과, T02 typed 게시·원자 거절, T03 typed 산술, T04 effect-only recipe, T05 대상 identity/거리, T06 shared2S, T07 태아 regular 값·속도, T08 Monk HpLost1회heal, T09 supported consumers/paused/released/source, T10 정상 외부 RON·획득·실행, T11 저장/재구성/수명, T12 budget/latefault/HPcost를 연결한다. 각 숫자 oracle는 독립 고정 입력으로 계산하며 제품 evaluator를 기대값 생성기로 호출하지 않는다.

기존 U08 public support의 admin fixture와 정상 Event/reward journey를 구분한다. `authoring/trial/tests/u08_relics.rs`의 정상 획득/저장/실행 경로와 `amount_completion.rs`의 공개 sustained/trace 경로를 재사용한다. R11/R12 boss_action/r09 helper는 내부 상태 주입을 포함하여 component seam으로만 계산한다. Root가 추가로 읽은 `src/game/relic/tests.rs:1`의 duplicate/variant/failed-transfer/permutation oracle도 inventory component이며 새로운 구조의 공개 소비 증거는 아니다.

FactSummary의 latest witness와 AmountQuery::Latest는 모든 동시 hit의 이력이 아니다. 다중 대상/동일 bucket은 독립 pre/post HP·shield·actor별 facts와 action aggregate를 결합하고 필요한 경우 보존 범위를 확인한 trace를 사용한다. query/provenance는 권한을 부여하지 않는다. exact owner 회수와 allocator 한계는 공개 root 비교와 별도 내부 수명 검증을 함께 요구한다.

선행 집합은 191고유 condition + U08의32수용문장 = 223이며 R06 31행은 별도다. U08 COVERAGE의207은 191+16정책군이므로 이 숫자를223대신 쓰지 않는다. 실제 ID 배열은 보고서에 보존되어 있고 후속 실제 delta에 맞춰 affected/non-impact를 판정한다. 기존 PC576 비시간값/U07M 비교, R05 peak9,683,689 대9,030,188(+653,501; 직전U07M보다+14,529) exit101 peak-only 경고와 shared-host/native historical binary SHA 한계를 보존한다.

시험 담당자의 종료 범위 질문은 현 PE08의 preserve 계약으로 해소한다. 선택 target의 철수·전투불능·사망은 그 연결을 terminal invalid 상태로 만들며 원 source activation의 비용·channel·만료를 임의 변경하지 않는다. 원래 source의 별도 종료 조건은 그대로 따른다. target invalid를 근거로 자동 retarget·재배치 identity 이전·shield refill을 하지 않는다. 새 정책 결정이나 acceptance 수정은 없다. public descriptor/diagnostic/occurrence/provenance의 구체 형식은 Sol 상세 설계가 제안하고 Root 공유 계약 검토 후 시험 작성에 전달한다.

## Current Actual — RQ02와 runtime 인수

`review_parts/RUNTIME_REVIEW_001.json`의 RT01–09 및 41개 actual source/range를 인수한다. runtime 사실·구조 제안·unknown은 그 보고서의 각각 다른 필드에 남아 있다. 6개 simultaneous integration group은 내부 연결 묶음이며 독립 완료 Phase라는 뜻이 아니다. 보고서의 source438/4c7 확인은 read-only 확인이고 빌드·시험·성능 실행0이다. visible process0은 해당 /proc 범위이며 shared-host quiescence 증거로 승격하지 않는다.

RT01/07: 현재 PendingSkillCast/ActiveSkillCast에는 개인 구조 plan owner가 없고 readiness, World cast selection, queued step, Flow wake, area tick은 global SkillDef를 다시 읽는다. 새 immutable owner를 initial resolution→accepted pending→active/Flow→released child에 결속하고 amount EvalCapture의 generation/site/fingerprint/Snapshot·Live guard와 함께 소비한다. Live context는 값을 live로 읽는 의미이며 구조나 generation을 현재 원형으로 재선택하는 면허가 아니다.

RT02/03: manual Unit target 입력은 이미 있으나 AllySingle의 살아있는 아군/air/tile-reach/usefulness와 sustained의 employee identity/trauma/combatant/world-distance radius 조건은 다르다. 원 radius와 자격을 selection/readiness/preview/start/membership에 공통 적용해야 한다. ChannelAura와 shared Shield의 caster lease를 recipient target으로 덮어쓰지 않는다. 기존 shared record 하나의 capacity/remaining을 사용하고 initial S는 source에서 한 번 계산한다. generic begin_sustained에 같은 producer shield의 recast 교체가 이미 있다고 가정하지 않고 exact provider/site/generation의 기존 activation 교체를 원자 추가한다.

RT04/05: 현재 BasicAttack snapshot은 Auto와 Triggered를 모두 포함하고 ExtraAttack도 Triggered로 진입한다. U09 regular predicate는 명시 occurrence 구분으로 새 typed coefficients와 연결하며 global Attack stat·active/proc/추가공격에 번지지 않게 한다. 원 aura 계산→수혜 공격자의 U08 numeric once→mitigation/protection 순서를 보존한다. 지원 caster의 copy/form/generation/relic contribution은 readonly 별도 출처로 담고 primary actor/kill credit/resource/reinput authority는 수혜 공격자에게 유지한다. observation-only source projection을 참고하되 이미 공개된 capability 제거 경계를 우회하지 않는다.

RT06: step 전체 target/delivery 수정은 sibling Heal의 대상·횟수도 바꾼다. effect-local 공간 실행을 우선 제안하며, 다른 lowering을 채택하려면 TD02의 동등성 조건을 충족해야 한다. Monk는 해당 Physical bite producer의 phase2 settled actual_hp_loss가0보다 큰지를 한 번 평가한 후 Self heal을 한 번 실행한다. resolved target count나 kill count로 대신하지 않는다. 현재 official fetus/monk fragment의 BasicAttackModifier placeholder를 전용 active 구조의 완료 증거로 사용하지 않는다. 대표 normal external RON으로 새 공통 실행 능력을 증명한다.

RT08/09: 기존 battle authority/settings/pause routing과 정확 activation reinput-stop을 재사용한다. global catalog는 원형 설명으로 유지할 수 있으나 actor-specific effective 선택/footprint/support/source 설명이 필요하다. 구조 DTO가 global catalog version3의 기존 의미를 묵시 변경하지 않게 한다. strong Arc plan을 보유한다는 사실만으로 pointee retained가 자동 계산되지 않으므로 native extent/CloneShape/entity/DTO 각각의 실제 visitor에 새 비용을 연결한다. mutable pool/terminal/member는 candidate value state에 남기며 rollback 밖의 shared mutable cache/pool로 이동하지 않는다.

## TD06 runtime 질문의 해소와 호환 경계

- RT02/07의 recipient `terminal end` 제안은 recipient 연결의 종료로 한정한다. source activation을 무조건 종료한다는 해석은 채택하지 않는다. PE08의 원 source channel/비용/만료 preserve가 그대로 권위다. 이 정정은 두 담당자에게 전달했고 원 보고서는 소급 수정하지 않았다.
- single2S는 membership 수가 하나여도 shared protection stage를 유지한다. 기존 personal→shared 순서를 보존하므로 radius=None을 이용해 personal 우선순위로 바꾸지 않는다. reach/recipient와 protection stage를 별도 의미로 표현한다. 새 정책이 아니라 PE09 보존 계약의 구체 인계다.
- typed regular occurrence와 support contribution을 damage snapshot 전용 타입으로 둘지 cold fact projection에 둘지는 Sol 재량이다. authority를 가진 actor/action/source를 바꾸지 않는다는 의미는 고정한다. 신규 구조 coefficient의 regular-only 적용을 위해 legacy aura 전체를 일괄 수정하지 않으며 legacy 비변형 동작과 실제 정책 적용 범위를 상세 설계에서 식별·회귀 검증한다.
- Source schema의 하위 호환 확장 방식, actor-specific DTO 명칭/version, cache 위치·fingerprint, amount/Flow catalog 구현 배치는 구현 세부 선택이다. 데이터/실행/조회/계량과 위 source/소유/수명 의미가 함께 준비됐는지 Root가 검토한다. 별도 사용자 승인 단계가 아니다.

## 검토 coverage와 한계

각17 요소는 Source Plan의 exact requirement membership, Code Review Artifact의 element mapping, Astra의34 acceptance mapping에 양방향 연결된다. RQ01/03/05/06의 Root 관찰과 RQ02/04의 담당자 관찰을 동일한 immutable reviewed source archive에 고정한다. 전체 파일을 archive했다고 모든 행을 읽었다는 뜻은 아니며 위 symbol/range와 보고서의 실제 inspected ranges가 검토 범위다. Released/Deployed actual은 이 workspace 구현 요청의 수용 대상으로 지정되지 않았으므로 별도 release/서비스 상태를 추정하지 않는다. source plan의6개 workspace requirement에는 모두 current observation을 만든다.

기존 API가 새 정책을 이미 충족한다는 판정은 없다. 구현·시험·성능 비용·최종 독립 완료는 GEX에서 수행할 미완료 작업이며 코드 검토 completeness와 구분한다. 후속 상세 설계는 공유 compiler/consumer/owner 계약을 유지하고 material 의미 변경이 필요하면 P2G-10으로 반환한다.
