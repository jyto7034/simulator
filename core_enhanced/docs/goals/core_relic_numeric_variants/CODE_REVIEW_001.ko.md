# U08 코드 기반 검토와 구현 설계

2026-10-06 · 계획 owner root · 입력 WR-CRNV-002(16 decided) · source plan RSP-CRNV-003.
[exact source121개](evidence/REVIEW_SOURCE_MANIFEST_003.json), [archive](evidence/REVIEW_SOURCE_003.tar.gz),
[선행 인수](PREDECESSOR_INTAKE_001.json)가 실제 바이트를 고정한다. 최초110개/후속118개 capture는 이력으로 보존한다.
현재 제품398파일 aggregate `f2144e9a4c5ddeb41245799294a75b41de9d843018cb2166f711b7f07821cda8`.

root의 실제 제한 검색·생산자/소비자 읽기와 기존 지정 Sol/xHigh의 읽기 전용 조사에 근거한다.
121개 파일의 모든 줄을 읽거나 새 테스트를 수행했다는 주장이 아니다. 감사자는 이 설계/조사에 참여하지 않았다.
이번 의미 검토는 P08 최신 유물·저장 규칙, H04의 R14 채택 범위와 ROADMAP U08/C02–04를 적용한다.
새 의미의 material finding0, 필요한 source gap0. 제품 구현·컴파일·실행 결과는 아직 없다.

## Current Actual

| 요구 | 실제 코드·처리 흐름 | 목표와 차이·적용 경계 |
| --- | --- | --- |
| RQ01 | `game/data/content_pack/raw.rs`192/200의 Equipment/SkillFragment raw 등록, catalog winner 조립과 `data/mod.rs`의 build/validation. `ability/authoring.rs`의 EffectSite/EffectSource/EffectPredicate는 같은 효과 주소와 태그를 표현한다. EffectOrigin::Added는 작성 귀속이며 persistent relic이 아니다. `value_expr/catalog.rs`는 final winning skill의 immutable compiled entry를 생성한다. | 유물 정의/variant/copy schema·catalog 자체가 없다. 기존 effect_support는 주로 literal/Instant이고 현재의 넓은 실제 amount executor와 동일 범위가 아니다. 숫자 유물의 typed capability를 실제 executor와 함께 선언·검증해야 한다. |
| RQ02 | `resources/inventory.rs`582 OwnedEquipment와 `skill_fragment.rs`26 OwnedFragment는 immutable definition과 copy/장착/form을 분리한다. `equipment_loadout.rs`65는 슬롯/역참조를 함께 계획·커밋한다. `world/grant_state.rs`22는 inventory/fragments/roster/UUID/enkephalin을 candidate로 묶는다. `maintenance.rs`386/433의 파편 분쇄는 검증→candidate 해제/제거→authored grant→commit이다. `event_node.rs`317/524는 현재 Event 결과의 grant를 먼저 계획한다. | 유물 전용3슬롯·역참조·variant·침식 촉매·무료변환이 없다. Any3 장비 슬롯을 유물 슬롯으로 재사용하면 의미가 깨진다. `GrantFragmentDust`는 존재하지만 snapshot762의 reserved legacy 자산이다. 최신 장비 가루로 자동 이행하지 않는다. SellItem은 현재 장비/소모품만 평가한다. |
| RQ03 | `combat_setup/player_spawns.rs`85/102와 `events/combat.rs`224/303→`battle/core/build.rs`114/275가 직원 profile·source를 실제 BattleCore로 전달한다. `value_expr_runtime.rs`2363은 catalog kind/Arc::ptr_eq와 generation/authored/action/provider 권한을 검사한다. EvalCapture726은 release plan/source/frame을 소유한다. `sim.rs`3487은 Expr실행,3560은 관측 켠 literal만 평가 기록을 만든다. literal 관측off는3086 이하 실행 값으로 바로 간다. queue/projectile/area는 기존 carrier와 source를 보유한다. 별도 `basic_attack.rs`463→`damage_runtime.rs`146/214→`basic_attack_projectile.rs`는 authored EffectSite/capture 없이 BasicAttack base snapshot을 만든다. | 단순 공유 catalog 변경 또는 임의 per-copy compiled entry 전달은 현재 권한 검사에 어긋난다. 개인 immutable overlay/binding과 별도 기본공격 typed numeric address가 새로 필요하다. Expr만 고치면 literal/관측off가 누락된다. 기존 stats418은 순차 modifier 적용으로 C04 합성 규칙을 대체하지 못한다. |
| RQ04 | `world/state.rs`180 GameCoreState,485 RunCheckpointPayload capture/restore,671 BattleEntrySnapshot,active_run/continuation의 fresh Core 복원과 content validation. `combat.rs`836 recover_assets_for_new_deaths는 새 사망의 장비·파편을 해제한다. `headquarters.rs` 등 새 런/종료와 node entry 흐름이 기존 owner를 넘긴다. | 새 roots의 checkpoint/entry/Continue/clone/사망/새런 reset·검증이 전부 필요하다. variant는 battle-local 정리로 지울 수 없다. 저장에서 derived plan/capability를 읽어 authoritative로 실행하면 안 된다. |
| RQ05 | `authoring/amount_query.rs`는 마지막 committed RecordedAmountEvaluation을 다시 계산하는 preview/explain이다. PreviewSkillTarget은 별도 targeting query다. `authoring/query.rs`,trial/scenario,CLI와 world snapshot/admin가 현재 공개 경로를 소유한다. | 기존 preview 명칭을 prospective 개인 수치 미리보기로 오인하지 않는다. 변환 전/후 설명과 effective plan query, 실제 기록 replay에 동일 숫자/기여 귀속이 필요하다. 새 catalog/정상 grant/시험 입력은 공개 접근까지 연결한다. |
| RQ06 | authoring control/CloneShape, native runtime_control의 누적 admission과 retained extent, store clone, world candidate rollback. U07-M은 strict기본8MiB·trusted개발profile·관측·allocator/RSS를 분리해 측정한다. U07 source wrapper가 PC31/5를 회복했다. | 새 Arc/vector/map의 owned extent,clone와 allocation 전 admission을 함께 설계한다. 모든 일반 source의 inline 크기 증가가 PC를 다시 거절로 바꾸는 위험이 크다. 최소 empty/none 경로를 유지하고 실제 비용을 초기 검증한다. |
| RQ07 | R07 copies/equipment의 전환·실패·분쇄·사망 tests, R06 amount replay/legacy, U04 checkpoint/Continue,U06/U07 정상 전투/생성물,기존 공개 RON과 PC/R05 harness. U07-M 독립 closure와 target은2개 published status delta 외 exact다. | 기존 시험은 새 유물 동작 증거가 아니다. 같은 public seams에 새 nonzero 사례를 추가하고191선행+31R06를 영향 분류한다. 전체 current source에서 필요한 회귀를 수행한다. |

RQ08 Released Actual은 별도 외부 released-version target이 없으므로 N/A이고 로컬 release CLI는 필수다.
RQ09 Deployed Actual은 현 단계 deployed/Unity 범위가 아니므로 N/A다. 실제 기존 host 저장/명령 소비는 RQ04/05의 required다.

## TD01 — 정의·사본·슬롯의 소유와 전환

**목적:** 원형을 공유하는 직원들이 독립 유물 상태와 수치를 갖고3슬롯/휴면/전환 의미를 보존한다.
권장 모듈은 `game/data/relic_data.rs`(schema/validation/immutable catalog), `game/relic.rs` 또는 `game/relic/`(run state/전환),
`world/relics.rs`(normal command/query/cost)다. 이름과 내부 분리는 구현자 재량이다.

- `RelicDefinition`은 stable definition/effect ID, rarity, duplicate/incompatibility metadata, normal numeric entries,
  optional authored corruption variant(effects+penalties), authored dismantle dust yield를 가진다. finalized catalog에만 게시한다.
  strict unknown-field·ID/참조/지원 capability validation을 기존 content_pack winner/identity 경로에 연결한다.
- `OwnedRelic`은 copy UUID,definition ID,normal/corrupt variant,장착 대상/slot을 가진 run mutable record다.
  사본 UUID는 기존 UuidManager의 별도 namespace를 사용하고 성공 commit 때만 의미가 드러난다.
  원형 Arc/derived plan을 save에 authoritative 실행값으로 저장하지 않는다.
- 권장 `RelicInventory`는 copy map과 직원별 `[Option<CopyId>;3]`을 하나의 aggregate로 소유한다.
  직원 roster existence/eligibility와 함께 검증하며 기존 equipment ItemSlot/Any3와 분리한다.
  사본 역참조를 추가하면 슬롯과 정확한 양방향 bijection을 같은 owner가 plan/commit한다.
  단일 canonical slot map으로 역조회만 만드는 동등 대안도 가능하며 두 공개 경로가 독립 수정되게 두지 않는다.
- equip/move/swap/remove는 원·대상 직원 전체 예상 슬롯을 먼저 검사한다. duplicate/incompatibility는 variant/휴면 여부와 무관한
  직원별 집합 검사다. 검사 중 mutable source를 지우지 않는다. 무기 미장착은 유물 보유·장착 허용의 실패가 아니다.
- lifecycle gate는 Safezone 준비/지도·Maintenance의 현재 허용 포트와 직원 상태를 재사용한다. battle 중에는 거절한다.
  death recovery의 새 사망 집합에서 유물 slot을 해제하고 copy/variant를 공용 보관에 남긴다. 같은 사망을 반복 처리해도 중복 회수하지 않는다.

**대안:** 원형에 corrupt bool/수치 쓰기는 사본 독립성 실패, 장비 슬롯 공용은 정책 위반이다.
GameCoreState 새 relic root와 stage roots/checkpoint를 함께 확장한다. 기존 사본 클래스를 복사해 별도 ID/권한 시스템을 만들지 않는다.

## TD02 — 정상 거래·침식·무료 Event와 가루

`EconomyGrantState::capture/commit_into`와 GrantExecutor의 새 companion API/내부 context를 확장하여 relics와 필요한 런 재료를
기존 inventory/roster/UUID/enkephalin과 **한 candidate**로 처리한다. 기존 public typed API를 깨는 경우 compatibility wrapper를 유지한다.
정상 RewardEffect/admin/trial/노드 grant는 이 owner를 공유하며 private fixture 삽입만으로 완료하지 않는다.

권장 명령/조회 의미는 `EquipRelic`, `UnequipRelic`, `PreviewRelicCorruption`, `UseRelicCatalyst`, `DismantleRelic`,
현재 Event session의 `ChooseRelicCorruption(copy or decline)`다. exact naming/DTO 배치는 기존 convention을 따른다.
각 명령은 현재 권한·copy·정의·variant·slot 상태·비용을 commit 직전에 재검증한다. preview receipt는 실행 권한이 아니다.

- catalyst 경로: 별도 원정용 침식 촉매 잔량1 검증→candidate 감소→variant 변경→new effective plan검증→commit.
  부활 촉매와 field/ID/DTO를 공유해 혼용하지 않는다.
- free Event: authored Event scene/선택 action이 실제 pending target-selection authority를 발행한다.
  선택 거절은 정상적으로 완료되며 무소비다. 임의 command의 `free=true`는 금지한다. current node/session/scene 및 한번소비 증표와
  사본 variant commit을 원자 처리한다. 기존 employee aptitude roll을 무료 침식에 억지로 추가하지 않는다.
  EventSuccessAction/SceneNext를 필요한 최소 선택 variant로 확장하고 기존 단일roll/중복실행 계약을 보존한다.
- 같은 `validate_corruption_target`를 조회/두 확정 경로에 사용한다. 없는 variant/이미침식/다른사본/누락def/미지원 효과·패널티는 거절한다.
  variant definition은 publish시 완전 검증하고 실제 변환에도 현재 generation/binding 확인이 필요하다.
- sale는 SellItem의 유물 분기에서 명시 거절하고 snapshot에서도 not-sellable을 보여준다. 전체 상점 상품 로직은 U16 소유다.
- dismantle은 Maintenance gate→확인→copy 해제/제거와 authored equipment dust 지급→commit이다.
  장착본은 explicit confirmation 필수. 이번 유물 포트는 모든 대상의 preview/confirm/cancel을 지원하여 취소 시 무변경을 명확히 한다.
  overflow/late grant/admission failure가 발생하면 copy와 dust를 모두 보존한다. 투자환급이나50%규칙을 유물에 만들지 않는다.

**가루의 현재 차이:** 기존 `fragment_dust`는 reserved legacy다. 최신 정책의 equipment dust와 합치거나 과거값을 변환하지 않는다.
권장 새 `RunCraftingResources {equipment_dust, relic_corruption_catalysts}` 또는 동등한 런 자산 field를 기존 resource/경제 소유자에 둔다.
이번에는 정상 authored grant·유물 분쇄·조회·저장/복원·run reset만 닫고 U15 무기 강화의 소비를 선구현하지 않는다.
정의한 새 field가 향후 U15의 단일 가루 owner가 되도록 별도 유물 전용 dust 통장을 만들지 않는다.
현재 형식이 이미 generic material로 정확히 장비 가루를 제공한다는 추가 코드 근거가 있으면 동등 재사용 가능하나,
legacy 잔액 의미 변경은 root에 반환한다. fixture yield는 명시 시험값이고 최종 공급/가격/분쇄량 확정이 아니다.

## TD03 — 개인 수치 계획과 권한

가장 먼저 작은 두 직원/같은 SkillId 사례로 이 경계를 확인한다. 기본 추천은 원 compiled entry를 유지하는
**immutable numeric overlay**이며 base plan graph를 사본마다 복제할 필요가 없다.

`RelicNumericPlan`은 content generation/digest,직원/장착 구성 revision,정확한 source provider/form binding,
EffectSite별 numeric contribution 목록과 dormancy 이유를 가진다. contribution의 키는 copy UUID+relic effect ID이고
정의 ID/variant도 포함한다. deterministic한 map/order로 동일 장착 집합의 permutation이 같은 gameplay numeric plan이 된다.
copy UUID는 귀속 key로 사용할 수 있으나 충돌 승자를 고르는 정책이 아니다. derived plan은 전투/예비계획 수명이며 run state에서 재구성한다.

**selector:** 실제 같은 AuthoredSkillEffect의 all-tags AND + typed damage amount capability를 검사한다.
전용 fragment/equipment/form은 accepted source provider의 정확 identity와 AND다. 서로 다른 효과에서 태그를 모으지 않는다.
U08은 기존 원형 시작 효과에 대한 numeric modification만 제공한다. unsupported created-effect/closure/Replace/NOT-cycle를
unconditional활성으로 낮추지 않고 구체적으로 거절한다. 지원 안 된 조합 거절은 공개 capability에 표시한다.

**기본공격 별도 주소:** 기존 기본공격은 AuthoredSkillEffect/CompiledAmountEntry가 없으므로 가짜 EffectSite를 만들어 catalog 권한을 통과시키지 않는다.
권장 `RelicNumericTarget`은 `SkillDamage(EffectSite)`와 `BasicAttackDamage`를 구분한다. 기본공격의 typed damage capability와
실제 무기 definition/copy·profile·action(kind Damage)·attack epoch/generation의 검증된 provider binding을 별도 owner가 제공한다.
태그 조건은 해당 effect가 실제 가진 태그만 검사하며 현재 기본공격에 없는 태그를 다른 스킬에서 가져오지 않는다.
태그 없는 범용 numeric 대상/정확 무기 조건 및 실제 damage_type 등의 지원 typed selector로 기본공격을 선택할 수 있다.
기본 Heal은 damage 대상이 아니다. prospective query도 이 별도 주소/원 provider를 정확히 보여준다.
기본공격은 현재 `damage_source_snapshot_template_for_unit`의 aura 반영 후 base를 유물 산술의 입력으로 삼고 once만 합성한다.
기존 aura와 이후 critical/resistance/bonus·minimum damage 의미는 바꾸지 않는다. 스킬 damage를 같은 helper에서 다시 합성하지 않는다.
결과 숫자와 contribution/binding을 DamageSourceSnapshot에 고정해 windup/투사체·철수 이후까지 보존한다.
source_attack 같은 원 stats fact는 원 의미를 유지한다. 예시 aura10% 기본100이면기존base110→flat20/percent50으로195이며,
유물 없는 값은 그대로110이다. basic heal·BuffTick·Environment·반사·보너스·R09 피해는 일반 helper 위치 때문에 적용되지 않도록
명시 positive capability/producer로 선택한다. 이 반례와 기본 근접/원거리·자동/triggered·epoch 취소를 실제 검증한다.

**authority:** `capture_value_expr_entry_with_prefix`의 원 catalog ptr_eq/kind/generation 검사는 그대로 통과해야 한다.
그 뒤 core가 검증한 개인 overlay를 exact site/provider/action binding으로 선택해 release carrier가 보유한다.
이미 검증된 base entry와 별도의 overlay owner를 함께 사용하면 기존 catalog 권한을 약화할 이유가 없다.
다른 구현(개인 compiled plan registry)은 소유·원형연결·generation·accepted provider와 immutable fingerprint를 동등 이상 검증해야 하며
단순 ptr_eq 제거 또는 plan/hash만 비교하는 우회는 허용하지 않는다.

**산술:** damage typed base에 flat 합을 더하고 `(100 + 동일종 percent 합)/100`을 한번 적용한다.
권장 공개 percentage 단위는 기존 정수 percent이며 checked i128 합/곱→기존 Rounding::TowardZero→narrow_damage를 사용한다.
100/20/20/30의180,3에50%의4 등 fraction rounding을 작성/조회에서 명시하고 경계 overflow는 typed failure로 낸다.
이 선택은 새 유물 damage 합성의 표현 세부이며 기존 모든 damage/stat 산술을 바꾸지 않는다.
원 base expression의 단위/음수/range/overflow 오류는 기존대로 먼저 검증한다. 유물 flat으로 원래 실패하던 base overflow를 구제하지 않는다. base typed값을 보존한 채 checked 합성→최종 narrow를 따로 수행한다. 음수 최종 damage나 유효하지 않은 coefficient는 기존 nonnegative damage 진단을 유지한다. 공통 증폭 gameplay cap은 없다.
독립된 amplification 종류를 구현자가 새로 만들지 않는다. 처음에는 명시 단일 additive relic damage class로도 충분하며
다른 종류 schema를 게시하려면 실제 산술 의미/consumer를 정의해 root 검토에 반환한다.

**empty cost:** 유물 없는 actor/source에는 기존 계산/구조의 빠른 경로를 유지한다.
모든 RuntimeFactSource에 큰 inline map/vector를 추가하지 않는다. optional 공유 owner/기존 extended storage에 필요한 경우만
결속하고 size·retained·clone·allocator 비용을 실제 측정한다. U07 PC 실패를 되살리면 설계/구현을 줄여 복구한다.

## TD04 — 실제 소비·출처·preview·패널티

- `combat_setup/player_spawns`의 normal/effective profile과 실제 battle builder에 같은 개인 plan 연결을 전달한다.
  certified power는 기존 고정 기준 의미를 보존한다. 유물을 새 PW 성장 항목으로 조용히 포함하거나 certified profile에
  battle-only penalty를 영구 쓰지 않는다. battle/preview가 사용하는 현재 loadout projection과 certification의 목적을 구별한다.
- `sim`의 literal Damage/Expr,기본공격/자동/명령·queued cast·flow·projectile·area 실제 amount consumer를 전방/역방향 추적한다.
  literal은 관측off에서도 합성을 실행해야 한다. 기존 effect.base/기본공격 request의 의미 있는 피해 base에 적용하고
  기존 bonus/critical/resistance/HP-loss 순서를 유지한다. amount observation toggle로 gameplay 계산 경로가 달라지면 실패다.
- queued acceptance/release에서 확정한 source/provider/effective numeric binding을 실제 carrier까지 보존한다.
  Expr의 Snapshot/Live는 base 값의 평가 시점을 계속 소유하고 immutable relic 구성은 같은 방출 owner를 따른다.
  source 철수/죽음/형태 변화 뒤 이미 합법 방출된 payload는 원칙대로 실행하고 새로운 권한은 얻지 않는다.
  원래 U05/U07 취소·Detach·미방출/방출 구분을 그대로 유지한다.
- RuntimeFactSource의 원 authored/cast/equipment/fragment/action/flow/reinput/independent origin을 보존하면서
  relic contribution attribution을 추가한다. Added/Relic 문자열 하나로 원 출처를 치환하지 않는다.
  원 source의 equipment 값을 유물 metadata로 바꾸거나 출처 종료 뒤 live query로 새 구성에 연결하지 않는다.
- Pending/RecordedAmountEvaluation과 authoring amount query는 base typed amount와 effective relic amount/기여 및
  immutable replay inputs를 함께 보여준다. 기록재생은 runtime 권한을 재발행하지 않고 동일 numerical kernel을 재사용한다.
  현재 value_expr_runtime836 kernel은 evaluate 직후 base narrow를 수행하고 amount_query166은 base-only replay와 record.amount/evaluated를 exact 비교한다. 따라서 runtime과 query가 같은 optional-overlay numerical kernel을 사용하고 base typed/narrowed값·effective typed/narrowed값·immutable contribution을 분리해 기록한다. 기존 base 필드를 effective값으로 몰래 바꾸지 않는다. base guard와 최종 guard/trace/source 종료 후 retained replay·literal관측off·overflow 반례를 함께 검증한다.
  기존 JSON 순서·생략·의미는 영향 검토 대상이며 유물 없는 경로는 가능한 exact 보존한다.
  prospective loadout/effective-plan query와 변형 설명은 별도 명확한 이름/상태를 가진다. PreviewSkillTarget의 의미를 바꾸지 않는다.

**대표 침식 패널티:** 공식 이름을 가진 유물의 미정 penalty를 결정하지 않는다. 신규 `u08_*_probe` 외부 RON으로
일반 damage 증폭과 더 강한 corrupt 증폭+작성된 전투 능력치 penalty를 검증한다.
최소 지원 예로 signed flat Defense/MagicResist 감소를 기존 stat pipeline에 실제 반영할 수 있다.
이는 생존 위험을 갖는 시험용 authored numeric penalty이며 가격/밸런스/공식 유물 선택이 아니다.
같은 penalty의 flat 합산과 canonical stat 순서를 명시해 slot permutation을 보존하고 기존 장비 modifier의 계산법은 변경하지 않는다.
실제 effective_stats_for_draft(stat_pipeline78)의 persistent stats→apply_draft_stat_scale→최종 battle stats 흐름에서 scale/exact_override 뒤에 battle-only penalty를 적용한다. persistent_stats_for_draft113은 certified power(player_spawns225)에도 쓰므로 변경하지 않는다. preview도 같은 final battle projection을 쓰고 normal/scaled/exact_override 각각에서 penalty가 유지되며 certified 값은 불변인지 검증한다. HP modifier를 이 경계에 몰래 추가하지 않는다.
지원 penalty는 variant schema의 typed capability로 엄격히 열고 조회만 가능한 임의 penalty 문자열을 성공시키지 않는다.
HP 감소/비용·주기형/구조 penalty를 추가하면 기존 U07 HP/사망/원자성·U09/U10 scope를 먼저 대조한다.
필요 이상으로 미래 연쇄 엔진을 구현하지 않는다. 핵심 침식 상태/비용/실제 penalty 실행은 이번에 닫는다.

## TD05 — 저장·복원·수명·원자성과 계량

GameCoreState/EconomyGrantState/RunCheckpointPayload/BattleEntrySnapshot/active_run writer-reader/default/new-run
및 result/retreat/death candidate의 새 roots를 한 inventory 목록으로 추적한다. 일부 capture만 고치면 완료가 아니다.

- 저장은 copy/definition/variant/slot과 런 재료를 담고 각 restore는 definition 존재·variant 자격·unique copy/slot·직원
  ownership·duplicate/incompatibility·정수 overflow·content identity를 검증한다. derived numeric plan은 현재 generation으로 재구성한다.
- 실제 public durable export/Continue는 active_run215/261→continuation.rs의 VERSION11을 사용한다. 파일 안의 별도 V12 상수만 바꿔 전환 완료로 보지 않는다. continuation490은 deserialize 후 canonical 재직렬화 checksum을 비교하므로 #[serde(default)]만의 새 field 추가는 이전 checksum을 깨뜨릴 수 있다. 생략 canonical을 유지하거나 원 bytes의 checksum을 검증한 뒤 명시 이행한다. 현재 save schema revision과 content identity gate를 확인해 명시 successor codec 또는 안전한 optional-empty member를 택한다.
  field가 없는 이전 상태와 존재하지만 malformed인 상태를 구별한다. legacy asset을 삭제/환산하지 않는다.
  U04의 구버전 이행·원본 보호를 인수하며 실제 migration 미지원이면 원본 보존 typed 진단이다. 이전 field 없음/정상빈상태 성공,field 존재하되 malformed 실패,old checksum 위조 실패,content generation mismatch 실패를 별개 검증한다. 새 relic 이전 빈 상태의 정상 입력을
  부당하게 잃지 않도록 지원 가능 범위를 구현자가 설명·검증한다.
- 변형전 checkpoint→cost+variant 되돌림,변형후→둘다유지,동일process reconnect→current,Continue→기존 pre-entry,
  retreat→battle entry,새원정→원정 전용 촉매/가루 정리. player vault나 전투중 disk codec를 새로 만들지 않는다.
- death/redeploy/withdraw는 slots/battle refs만 기존 정책대로 정리한다. 런 variant를 정상형으로 초기화하지 않는다.
  임시 plan·carrier·관측레코드의 Arc 수명은 실제 전투/방출/관측 윈도우 종료에 따라 끝나며 콘텐츠 전체를 불필요하게 붙들지 않는다.
- 새 candidate allocation/clone/serialized input은 control/CloneShape와 runtime admitted owners에 연결한다.
  admission 전에 source extent를 계산하고 saturating 숫자로 overflow를 감추지 않는다. 중첩 operation은 allowance를 재시작/환급하지 않는다.
  U07-M의 owner extent/accounted cumulative/allocator live·peak/RSS를 각각 기록한다. outside-native world rollback capture는
  기존 측정범위를 따른다는 사실을 명시하며 native로 잘못 보고하지 않는다.
- 대조 input은 실패 직전/후 inventory,slots,catalysts,dust,UUID,RNG,event session,queue,facts,저장 원본 해시다.
  비용만 소비되거나 variant만 바뀌는 late failure가 없어야 한다. success retry/duplicated Event확정은 한번 의미만 갖는다.

## TD06 — 순서·검증·인계 재량

권장 구현 순서는 DEFINE→OWNERS→PLAN→LIFECYCLE→PUBLIC→VERIFY다. 하나의 Single Goal 내부 작업이며
각 내부 단계가 독립 최종 배포본/완료 감사 대상이라는 뜻은 아니다.

1. GEX00 실제 dirty bytes/membership/index/worktree 보존. source-policy-build input capture와191선행+31R06 exact 인수.
2. DEFINE/OWNERS의 최소 정상 grant+3슬롯+copy/variant candidate를 세운 뒤 작은 rollback/query tests로 소유권 확인.
3. 두 직원 같은 SkillId의 literal 관측off/Expr/queued/projectile을 먼저 연결해 권한·원형불변·180 oracle를 검증한다.
   empty-path size/retained 비용과 PC 예비비교를 비싼 최종 통합보다 먼저 수행한다.
4. catalyst/free Event/dismantle+save/entry/death/Continue·공개 RON 등록을 연결한다. trial과 실제 normal commands를 모두 사용한다.
5. 각 acceptance·10차원·191선행+31R06 impact/adaptation/evidence를 source에 결속한다. final core 기본/alltargets/test-fixtures,
   event_log_checks/releasebins/clippy/fmt와 영향받은 host/public/fault/measurement·PC/R05를 실행한다.
   functional 검증과 성능 측정을 겹치지 않는다.20분stress는 제외,기존 R05 concurrent peak 초과만의 경고를 확대하지 않는다.
6. evidence selection→predecessor impact→adaptation current→coverage→GEX10완료→PLAN/NOTES→exact target/정역복원 순서.
   비참여 기존 Astra/High가 전체 U08를 GEX20 감사하며 finding은 지정 Sol/xHigh가 교정한다.
   Complete/High/open0와 현재성을 root가 인수한 뒤 U09. Single Goal이므로 Master/GEX30은 만들지 않는다.

구현자는 함수 분리·자료구조·내부 이름·시험 배치·일반 오류 교정을 자율 결정한다. root가 제품 구현/테스트를 대신하지 않는다.
정책/공개 계약/상태소유/완료기준 변경,selector 지원 범위를 의미 없이 줄임,base 권한 약화,
새 generation/save 의미 변경,old asset 환산,default budget 완화가 필요하면 코드 근거·영향·대안을 root로 반환한다.
반환은 자동 사용자 승인 대기가 아니다. 구현의 상세 함수/입력은 GEX10 초기에 실제 상태와 다시 맞추되 유효한 P2G를 다시 쓰지 않는다.

## Element Coverage

아래 selector는 앞으로 구현할 실제 검증 설계이며 지금 존재/통과한다는 표시가 아니다.

| PE/수용군 | 입력과 기대 결과 | 코드·통합 검증 seam |
| --- | --- | --- |
| 01 | 같은 ID2copy·1copy만침식·두 직원동일SkillId;원형해시불변 | raw catalog→normal grant→copy query→battle source |
| 02 |3slot/4th·samecopy2owner·duplicate/incompatible permutation와휴면 | relic transition→roster/slot validation→save validator |
| 03 |safe/maintenance무료move·weapon해제·battle거절·death1회회수 | action capability→normal handlers→recover_assets |
| 04 |normal grant·corruptdirect금지·sale거부·분쇄확인/취소/overflow | reward/EconomyGrant→maintenance/shop→new dust query |
| 05 |same-effect AND·정확 ID/form·태그상실/회복·미지원closure진단 | metadata selector→effective plan→public explain |
| 06 |100+20+20%+30%=180·fraction·permutation·overflow·원기존값 | checked numeric kernel→literal/Expr→actual HP loss |
| 07 |두직원·basic/auto/command·queued/projectile/area·Snapshot/Live | profile/build→capture/carrier→sim→amount replay |
| 08 |원 source+copy효과·철수뒤방출·stale/위조·R09suppression | runtime source/capture/authority→fact/query→U05/U07회귀 |
| 09 |없음/잘못된variant·pre-disclosure와actualpenalty·새외부RON | schema/validator→preview→combat stats/numeric |
| 10 |1회·재요청·2copy독립·cancel/late/admissionfailure무변경 | staged target/cost/plan→commit/fault/retry |
| 11 |catalyst1/0부족·Event0cost·decline·outsideEvent거절 | normal item/Event authority→same validation→run reset |
| 12 |변형전후 checkpoint·fresh Core·reconnect·Continue/retreat | all save roots/codecs→restore validate→원본해시 |
| 13 |해제/이전/사망/철수/재배치에도variant유지·반복회수 | owner/plan/carrier lifecycle→clone/retention |
| 14 |새ID RON·제3자 조합·정상CLI/trial/명령·실제결과/save | raw/parser/compiler/register→public consumers |
| 15 |empty-source비용·lowbudgetlatefailure·observeron/off·PC31/5 | native/CloneShape/measurement→frozenbaseline comparison |
| 16 |16군/10차원/191선행+31R06·정역복원·전체독립Complete | actual receipts→impact/adaptation→coverage→freeze/audit |

## Findings and limits

IF01 구현: relic 정의/run owner/variant/catalyst가 없으므로 모든 owner 경계를 함께 전환한다(TD01/02/05).
IF02 구현: catalog exact-plan authority와 literal 관측off 경로, 별도 기본공격의 non-authored typed numeric address를 모두 보존한 개인 계획 소비가 필요하다(TD03/04). Sol 준비검토로 빠진3파일과 기본공격 주소 전제를 발견했고 root가 실제 경로를 읽어 source003/TD03을 보완했다. 정책범위 축소 없음.
IF03 구현: legacy dust와 목표 equipment dust는 다른 의미이며 최소 새 run owner를 연결한다(TD02).
IF04 구현: capture/restore/death/Event authority/source attribution 중 일부 누락이 부분 상태를 만들 수 있다(TD02/04/05). 실제 VERSION11 canonical checksum 경계와 base/effective replay·scale/exactoverride penalty 경계를 Sol 준비검토→root 직접읽기로 확인해 동결 전에 보완했다.
TD01–06은 기존 의미를 실현할 구조 선택이며 정책 재답변으로 기록하지 않는다.
ED01 실행 증거 한계: 현재 조사는 static이고 exact 새 Rust API·배치별 allocation 비용·최종 receipt는 구현 뒤에만 확인된다.
이는 미래 실행을 PASS로 기록하는 근거가 아니다. required source 미확인 gap은 없고 최종 실행·독립 감사는 GEX owner가 수행한다.
