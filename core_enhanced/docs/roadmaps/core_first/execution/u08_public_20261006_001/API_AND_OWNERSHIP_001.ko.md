# 공개 인터페이스·파일 인계 부록 — 2026-10-06/001

root/제품Sol/시험Astra read-only 제안을 대조해 승인한 계약이다. 실행은 아직 HOLD. 제품/시험은 다른 파일에서 작성하며 첫 작은 A경계 source-ready 이후 전 writer 고정→Sol fmt/check/집중검사한다.

## 확정 인터페이스

A: `RequestRelicNumericPlan { employee_uuid }` → `RelicNumericPlan { result: RelicNumericPlanDto }`.
DTO: employee_uuid, generation:ContentIdentity, scope:ProspectiveLoadout, base_battle_stats:UnitStats, effective_battle_stats:UnitStats, penalties, targets.
base_battle_stats는 유물 penalty 직전 finalbattle projection이며 certified/persistent 통계와 구별한다.
Target: SkillDamage {site:EffectSite,provider:{equipment:Option<EquipmentSourceSnapshot>,fragment:Option<FragmentSourceSnapshot>},evaluation,effects} 또는 BasicAttackDamage {weapon:EquipmentSourceSnapshot,evaluation,effects}.
Evaluation: Available{base_amount,effective_amount}, Unresolved{RuntimeContextRequired|ProviderUnavailable}, Rejected{typed base/final fault}.
Effects: copy_uuid/definition_id/variant/effect_id/flat/percent 및 Active 또는 Dormant의 typed mismatch(TargetKind/SkillId/DamageType/Tags/Equipment/Fragment/Form), canonical copy/effect 정렬.
정확 보조 타입·enum 필드 이름은 Sol이 신규 game/relic/plan.rs에 먼저 공개해 Astra가 그 코드와 결속한다. 의미 변경은 root에 반환한다.
기본공격 현지원 타입은 Damage와 Heal이며 새 Buff variant를 발명하지 않는다. Heal은 Damage target으로 표현/증폭하지 않는다.

B: EventSceneNext::RelicCorruption{next_scene_id:EventSceneId}.
PlayerBehavior::ChooseRelicCorruption{node_id,event_id,current_scene_id,copy_uuid:Option<Uuid>}.
ActionKind와 ApplicationCommandKind는 해당 enum의 기존 no-payload 표현 관례를 따른다.
요청 current_scene_id는 요청자가 선택한 원 source scene이며 역사 재시도 때도 같은 값이다.
EventSessionState.relic_corruption_selections:Option<Vec<CommittedRelicCorruptionSelection>>, entry={source_scene_id,copy_uuid}.
EventSceneSnapshotDto optional history도 relic_corruption_selections 이름을 사용한다. None default+skipNone으로 old canonical 유지.
EventInteractionDto::RelicCorruption{candidates:Vec<RelicCorruptionCandidateDto>,can_decline:bool,catalyst_cost:u32}, 출력 true/0 불변.
Candidate={copy:OwnedRelic,eligibility:PayloadEligibilityDto,effects:Vec<RelicDamageEffect>,penalties:Vec<RelicPenalty>}; corrupt 후 효과/penalty 사전공개.
후보 열거 방식은 구현자가 시험자에게 명시한다. 선택 불가능한 copy를 eligible로 표시할 수 없다. 응답은 기존 BehaviorResult::EventState.

C: 기존 SellItem/qty1에서 ActionDisabledReasonCode::RelicNotSellable, code relic_not_sellable.
Unknown UUID OfferUnavailable과 기존 문맥/quantity 거절 우선순위를 보존한다.
RelicStateDto.not_sellable_copy_uuids:Vec<Uuid>는 derived, sorted, empty skip이다. OwnedRelic layout/권한에는 새 필드를 넣지 않는다.

## 파일 소유

Sol: 신규 src/game/relic/plan.rs; game/relic.rs,battle/core/relic_numeric.rs,world/relic_runtime.rs,world/snapshot.rs,combat_setup/player_spawns.rs,game/behavior.rs,src/application.rs,game/world.rs,world/application.rs,world/action_capability.rs,data/event_data.rs,data/validation.rs,data/live_ron.rs,resources/selection.rs,resources/mod.rs,world/event_node.rs,world/state.rs,world/active_run/continuation.rs,world/node_flow.rs,world/shop.rs 및 필요한 제품 owner helper. 기존 inline cfg(test) 내용은 수정하지 않으며 필요한 시험 compile 교정도 Astra에게 반환한다.
Astra: 신규 core_enhanced/tests/u08_relic_public.rs, tests/u08_relic_public/**, tests/fixtures/u08/public001/**, src/game/world/tests/u08_event_owner.rs, u08_lifecycle.rs, world/tests/mod.rs. 기존 game/relic/numeric.rs wholefile는 시험 담당 소유로 별도 인계 없이 Sol 편집 금지.
공유제품 파일 안의 시험 등록/수정이 필요하면 wholefile lock을 순차 인계한다. 공개 시험은 Cargo integration autodiscovery를 사용하므로 제품 테스트 등록이 우선 필요하지 않다.
A/지원 fixture부터 시작하고 B/C/D/E를 이어간다. Sol의 EXPERIMENTS/STATE_INVENTORY 실제 구현 문서 보완도 허용했다. root의 PLAN/NOTES publication 편집은 종료했다.
포맷도 source mutation이므로 전체 writer HOLD 이후에만 수행한다. 기능과 성능을 겹치지 않는다. 아직 새 public 기능의 compile/test/PASS는 없다.
