from record_sources import add,ROOT
G='core_enhanced/src/game/'
for p,r,a in [
('resources/equipment_loadout.rs',[(1,345)],['pub(crate) struct EquipmentLoadoutTransition','*self.roster = next_roster;']),
('resources/inventory.rs',[(1,265),(385,635)],['pub struct OwnedEquipment','pub meta: Arc<EquipmentItem>']),
('world/grant_state.rs',[(1,91)],['pub(super) struct EconomyGrantState','pub(super) fn commit_into']),
('reward.rs',[(1,390)],['pub enum RewardEffect','pub fn grant_effects_with_state']),
('world/application.rs',[(1,268)],['struct PreparedApplicationCommit<T>','self.state = state;','self.run_seed = run_seed;']),
('world/state.rs',[(325,615)],['macro_rules! define_run_checkpoint_payload','pub struct BattleEntrySnapshot','pub struct BattleEmployeeAttemptState']),
('world/active_run.rs',[(1,312)],['const ACTIVE_RUN_CORE_PAYLOAD_VERSION: u32 = 5;','const ACTIVE_RUN_V4_READER_REMOVAL_CERTIFIED: bool = false;']),
('world/node_flow.rs',[(1220,1545)],['fn validate_run_checkpoint_economy_graph','pub(super) fn save_run_checkpoint']),
('world/helpers.rs',[(145,260)],['pub(super) fn validate_skill_fragment_equip_limit']),
('data/skill_fragment_data.rs',[(1,315)],['pub enum SkillFragmentAcquisitionSource','pub enum SkillFragmentEffectDef']),
('stats.rs',[(1,420)],['pub struct UnitStats','pub fn apply_modifier']),
('world/combat.rs',[(1,172),(636,785),(1300,1650)],['fn reconcile_employee_attempt','fn commit_staged_combat_result_state','battle.employee_conditions = self'])]:
 print(add(G+p,r,a))
for p,n in [('run_relic_system_design_notes_2026-08-21.ko.md',467),('ego_corrosion_system_design_notes_2026-08-19.ko.md',177)]:
 print(add('core_enhanced/docs/tmp/'+p,[(1,n)],[],kind='policy_record'))
with (ROOT/'evidence/RESEARCH_LOG.ko.md').open('a') as f:f.write('''
## 조사 4 — 소유·게시 transaction과 복원 종류
- 장비에는 독립 OwnedEquipment.instance_uuid와 roster slot / inventory reverse binding을 함께 변경하는 EquipmentLoadoutTransition이 이미 있다. 이 기반은 재사용하며 파편의 definition-keyed progress만 같은 것으로 취급하지 않는다.
- GrantExecutor는 효과별 mutable 인수를 순서대로 바꾼다. Core의 원자성은 EconomyGrantState의 5 roots capture/apply/commit 및 execute_application에 있다. 유물 상태를 추가할 경우 관련 root와 preview·실패 복원에 같이 포함해야 한다.
- world/application.rs의 PreparedApplicationCommit와 commit_application은 state·run_seed만 커밋한다. 새 런 선택을 draft.game_data 변경으로만 구현하면 그 교체는 커밋되지 않는다. 외부 팩 generation 바인딩은 명시된 소유·커밋 경계를 함께 바꾸거나 별도 새 Core 생성 경계를 채택해야 하며, metadata-only Goal에는 이 변경이 필요하지 않다.
- RunCheckpointPayload 전체 field 매크로에는 content generation fingerprint가 없다. V5 SHA는 checkpoint 내용의 canonical hash이며 외부 데이터 동일성 보증이 아니다. 현재 V4 reader 제거는 별도 인증 대상으로 남아 있다.
- BattleEntrySnapshot은 전투 진입 상태를 되돌리고 active battle을 제거한다. durable checkpoint, 같은 process의 live session, attempt-local 직원 상태를 서로 같은 저장 수명으로 취급하지 않는다.
- combat 결과 경로는 완료 직후 만들어진 결과의 employee_conditions를 reconcile_employee_attempt 후 다시 복사한다. 침식 대가는 이 attempt-local 경계에 반영해야 마지막 적 처치와 동시에 전투가 끝나도 결과 commit에 빠지지 않는다.
- ServerTick·Elapsed·NextEventBucket 진입점이 구별된다. 침식 정지는 server tick만 막는 것으로 완료 주장할 수 없다.
- 정책 원문 2개 전체를 읽음: 유물 정책 467줄, 침식 정책 177줄. 유물 3공용 슬롯/독립 사본/비활성 유지/사망 회수/재배치당 일시 상태와 전투당 한도 분리, 획득 정책 보류를 유지. 침식 actual cast·trauma100 attempt-local·필수 wave bookkeeping·1배속 연출정지·최종보스 제외를 유지. 임계치/수치/old save 전환은 임의 확정하지 않는다.
- stats.rs는 Attack만 있고 SpellPower 필드가 없다. modifier는 현재값에 순차 적용·clamp하므로 가역 효과는 단순 역연산으로 해결할 수 없다는 설계 근거다. 기존 산술 변경은 최소 metadata Goal과 분리한다.
''')
