from record_sources import *
p='core_enhanced/src/game/'
for path,ranges,anchors in [
('world.rs',[(680,840)],['handle_start_new_game','handle_select_starter_employees']),
('battle/core/sim.rs',[(383,890),(2870,3045)],['resolve_manual_skill_base_request','caster.resonance_current = 0;']),
('world/combat.rs',[(2650,2850)],['handle_activate_skill','snapshot.restore(&mut self.state);']),
('world/state.rs',[(725,865)],['simulation_delta_for_tick']),
('world/node_flow.rs',[(1010,1220)],['advance_employee_psychology_node']),
('employee.rs',[(180,340),(515,690)],['pub struct EmployeeCondition','pub fn apply_incapacitation']),
('employee_psychology.rs',[(1,103)],['pub fn advance_node(&mut self) {']),
('data/run_policy_data.rs',[(1,315)],['pub struct RunPolicyData','pub struct CertifiedCombatPowerWeights']),
('combat_power.rs',[(1,320)],['fn certified_power_for_profile','profile.skill_id.is_some()']),
('range_preview.rs',[(1,245)],['fn active_skill_preview','game_data.skill_data.get_by_id']),
('combat_setup/player_spawns.rs',[(1,287)],['battle_unit_draft_for_employee_from_item_slot','certified_combat_profile_for_employee']),
('battle/core/build.rs',[(100,300)],['Self::make_item_instance_id','base_uuid: *equipment_uuid,'])]:
 add(p+path,ranges,anchors)
with (ROOT/'evidence/RESEARCH_LOG.ko.md').open('a') as f:
 f.write('''\n## 조사 묶음 6 — 원본 사본·시전 시점·직원 대가·표시/평가\n\n확인: player_spawns는 장비 base UUID와 enhancement만 draft에 전달하고 build는 owner/배치/index로 전투 item ID를 만든다. 영구 사본 UUID는 이 경로에서 그대로 전달되지 않는다. 향후 SourceBinding은 이를 명시적으로 연결해야 한다. 현재 metadata Goal의 정의 주소와는 다른 변경이다.\n\n확인: Player Manual route는 진영·모드·공명·대상을 검증한다. 보스가 이 포트를 그대로 호출할 수 없으므로 공통 내부 Action admission과 별도 authority adapter가 필요하다. Auto/ManualCastEnd에서 pending start_seq 확인 후 실행과 공명 소비가 발생한다. 침식의 실제 시전 의미를 단순히 UI 접수 또는 효과 release로 치환하면 안 된다.\n\n확인: Breakdown 판정은 trauma 기준이나 직원 condition에는 collapse countdown이 없고, node_flow의 psychology.advance_node는 기억 수명만 갱신한다. 침식은 직원별 countdown/회복/사망/자산 회수/저장도 도입해야 한다. apply_incapacitation은 신체 손상도 늘리므로 침식 대가 구현에 그대로 사용할 수 없다.\n\n확인: preview/readiness는 전역 SkillId 조회, Certified는 스킬 존재 정액 평가다. effective plan 도입 시 실행·preview·평가 입력을 같은 단위가 소유해야 한다. 미정 평가 가중치를 창작하지 않는다.\n\n판단: 새 런 generation pin은 첫 전투가 아니라 briefing/starter 선택부터 유효해야 한다. state-only commit 경계 또는 새 factory의 profile 보존을 명시한다.\n\n한계: 본 묶음은 정적 검토다. 게임 테스트·회귀·성능은 미실행. 전체 dirty 스냅샷 해시는 확보하지 못했다.\n''')
