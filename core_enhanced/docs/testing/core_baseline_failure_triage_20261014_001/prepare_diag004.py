from pathlib import Path
import json,re,subprocess,difflib
r=Path('/mnt/f/work/simulator');o=r/'core_enhanced/docs/testing/core_baseline_failure_triage_20261014_001';d=Path('/tmp/core-failure-triage-20261014-001');p=d/'core_enhanced/src/game/world/tests/combat.rs';s=p.read_text();t=s
for arg in ['first','*id']:
 old='core.validate_employee_deployment_cell(match &preview { BehaviorResult::NodePreview{combat_preview:Some(p),..}=>p,_=>panic!() },'+arg+',deploy_position)'
 new='crate::game::combat_preview::DeploymentZoneKind::Ground.supports_affinity(crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(core.roster().unwrap(),core.inventory().unwrap(),&core.state.skill_fragments,core.game_data.as_ref(),'+arg+').unwrap().deployment_affinity)'
 t=t.replace(old,new)
t=t.replace('deployment_affinity).is_ok()','deployment_affinity)')
assert t!=s;p.write_text(t);(o/'DIAGNOSTIC_004.diff').write_text(''.join(difflib.unified_diff(s.splitlines(True),t.splitlines(True),fromfile='a/core_enhanced/src/game/world/tests/combat.rs',tofile='b/core_enhanced/src/game/world/tests/combat.rs')))
subprocess.run(['python3',str(o/'run.py'),'diagnostic5_004','/home/blast/.cargo/bin/cargo','test','-p','game_core','--lib','--offline','--locked','--target-dir',str(r/'.cache/u12-vault-cost-target'),'--','--exact','--nocapture',*json.loads((o/'diagnostic003_selection.json').read_text())],cwd=d)
