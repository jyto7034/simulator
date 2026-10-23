from pathlib import Path
import re,json,difflib,subprocess
r=Path('/mnt/f/work/simulator');o=r/'core_enhanced/docs/testing/core_baseline_failure_triage_20261014_001';d=Path('/tmp/core-failure-triage-20261014-001'); edits={}
def edit(path,name,fn):
 p=d/path;s=edits.get(path,(p.read_text(),p.read_text()))[1];a=s.index('fn '+name+'(');n=re.search(r'\nfn \w+\(',s[a+3:]);b=a+3+n.start() if n else len(s);t=s[:a]+fn(s[a:b])+s[b:];assert t!=s;edits[path]=(edits.get(path,(s,s))[0],t)
p='core_enhanced/src/game/world/tests/combat.rs'
names=['paused_live_battle_commits_deploy_and_withdraw_without_advancing_time','track_and_rejoin_raw_live_bundle_reaches_typed_world_setup_identity']
for n in names:
 edit(p,n,lambda s:s.replace('let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];','''let first = core.roster().unwrap().combat_eligible_employee_ids()[0];
    eprintln!("TRIAGE first={:?} ground_validation={:?}",core.roster().unwrap().get(&first).unwrap().employee_definition_id, core.validate_employee_deployment_cell(match &preview { BehaviorResult::NodePreview{combat_preview:Some(p),..}=>p,_=>panic!() },first,deploy_position));
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids().into_iter().find(|id| core.validate_employee_deployment_cell(match &preview { BehaviorResult::NodePreview{combat_preview:Some(p),..}=>p,_=>panic!() },*id,deploy_position).is_ok()).unwrap();'''))
edit(p,'live_ron_defense_route_playable_path_runs_to_combat_result',lambda s:s.replace('let event_log: BattleEventLog = serde_json::from_value(trace["event_log"].clone()).unwrap();','let event_log: BattleEventLog = serde_json::from_value(trace["event_log"].clone()).unwrap();\n    eprintln!("TRIAGE retained_floor={} entries={} first={:?}",event_log.floor_seq(),event_log.entries.len(),event_log.entries.first().map(|e|e.time_ms));'))
edit('core_enhanced/src/game/world/tests/r09_combat.inc.rs','r12_world_two_employees_shared_personal_parry_cost_result_and_fresh_restore',lambda s:s.replace('for _ in 0..200 {','for _ in 0..2000 {'))
edit('core_enhanced/src/application/fault_tests.rs','native_fault_preserves_last_root_first_cause_and_every_public_mutation_gate',lambda s:s.replace('waiting(1000)','waiting(3000)'))
patch=[]
for path,(a,b) in edits.items():
 (d/path).write_text(b);patch+=difflib.unified_diff(a.splitlines(True),b.splitlines(True),fromfile='a/'+path,tofile='b/'+path)
(o/'DIAGNOSTIC_003.diff').write_text(''.join(patch))
shorts=names+['live_ron_defense_route_playable_path_runs_to_combat_result','r12_world_two_employees_shared_personal_parry_cost_result_and_fresh_restore','native_fault_preserves_last_root_first_cause_and_every_public_mutation_gate'];selected=[n for n in json.loads((o/'selection.json').read_text()) if n.split('::')[-1] in shorts]
(o/'diagnostic003_selection.json').write_text(json.dumps(selected,indent=2))
subprocess.run(['python3',str(o/'run.py'),'diagnostic5_003','/home/blast/.cargo/bin/cargo','test','-p','game_core','--lib','--offline','--locked','--target-dir',str(r/'.cache/u12-vault-cost-target'),'--','--exact','--nocapture',*selected],cwd=d)
