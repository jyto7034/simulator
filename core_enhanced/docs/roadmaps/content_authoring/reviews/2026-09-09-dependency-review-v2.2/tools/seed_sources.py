from record_sources import add
D='core_enhanced/docs/'
R=D+'roadmaps/content_authoring/'
T=D+'goals/content_authoring_minimal_effect_contract/'
for p,r,a in [
(R+'ROADMAP_STATUS.md',[(1,137)],['NEXT-0009']),
(R+'OPERATING_RULES.md',[(1,160)],[]),
(R+'roadmap/ROADMAP.ko.md',[(1,220)],['CA23']),
(R+'roadmap/DEPENDENCIES.ko.md',[(1,97)],['계획상 선행 후보']),
(R+'records/CA00_NEXT-0001_INTAKE.ko.md',[(1,113)],['B01']),
(R+'records/CA00_NEXT-0002_REQUIREMENTS.ko.md',[(1,162)],['354']),
(T+'WORKING_POLICY.ko.md',[(1,89)],['HOF-CAMEC-P2G60-001']),
(T+'CODE_REVIEW_001.ko.md',[(1,110)],['SourceMap']),
(T+'P2G_50_GOAL_SET_001.ko.md',[(1,147)],['GOAL-CAMEC-01']),
(T+'P2G_40_PHASE_PLAN_002.json',[(1,301)],['AG-CAMEC-01']),
(T+'P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_002.json',[(1,214)],['p2g40-receipt-validator/1.2.0'])]:add(p,r,a,'existing_document')
B='core_enhanced/src/'
for p,r,a in [
('game/data/live_ron.rs',[(1,280)],['pub const LIVE_RON_SCHEMA_VERSION: u32 = 1;',"pub source_file: &'static str,"]),
('game/data/mod.rs',[(1,360),(790,955)],['pub struct ValidatedGameData(Arc<GameDataBase>);','    pub(crate) fn load_live_sources(']),
('application.rs',[(770,945)],['pub struct CoreApplication {']),
('game/world.rs',[(1,245)],['    game_data: Arc<GameDataBase>,']),
('game/ability.rs',[(490,970)],['pub struct SkillStepDef {','    pub effects: Vec<SkillEffectDef>,']),
('game/data/skill_data.rs',[(1,700)],['    pub(crate) fn try_into_unvalidated_database(self) -> Result<SkillDatabase, String> {']),
('game/battle/core/sim.rs',[(1975,2335)],['    pub(super) fn build_skill_step_commands(']),
('game/battle/core/skill_runtime/cast.rs',[(1,300)],['    pub(in crate::game::battle::core) fn previous_step_damage_gate(']),
('game/battle/core/skill_runtime/projectile.rs',[(811,942)],['        let Some(skill) = self.game_data.skill_data.get_by_id(&skill_id).cloned() else {']),
('game/battle/core/skill_runtime/area.rs',[(576,776)],['            .get_by_id(&runtime.skill_id)']),
('game/battle/core/build.rs',[(350,555)],['    fn cancel_active_skill_runtime_for_withdrawn_unit(']),
('game/battle/core/damage_runtime.rs',[(1,340)],['    pub(in crate::game::battle::core) fn execute_committed_damage(']),
('game/battle/core/commands.rs',[(440,760)],['    pub(in crate::game::battle::core) fn process_commands_with_modifier_source(']),
('game/battle/core/death_runtime.rs',[(1,215)],['        self.note_wave_actor_death(target_instance_id);']),
('game/battle/core/triggers.rs',[(1,360)],['    fn collect_item_trigger_payload(']),
('game/battle/core/types.rs',[(180,480)],['pub(super) struct ActiveSkillCast {','pub(super) struct AbilityProcKey {']),
('game/battle/damage.rs',[(1,205)],['pub struct DamageSourceSnapshot {']),
('game/battle/buffs.rs',[(1,269)],['pub enum BuffKind {']),
('game/battle/stat_pipeline.rs',[(1,360)],['pub(crate) fn employee_combat_profile_for_certified_power(']),
('game/skill_fragment.rs',[(1,380),(555,775)],['    progress: BTreeMap<SkillFragmentId, SkillFragmentProgress>,']),
('game/world/maintenance.rs',[(150,458)],['    pub(super) fn handle_dismantle_skill_fragment(']),
]:add(B+p,r,a)
