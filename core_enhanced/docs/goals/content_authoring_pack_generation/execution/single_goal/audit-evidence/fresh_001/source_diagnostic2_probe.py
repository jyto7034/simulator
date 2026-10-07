from pathlib import Path
import hashlib,json,shutil,subprocess,datetime
R=Path('/mnt/f/work/simulator'); E=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/fresh_001';T=Path('/tmp/capg-r02-fresh-source-probe-002');T.mkdir(exist_ok=False)
shutil.copytree('/tmp/capg-r02-release-ab-003/live_data',T/'live_data')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def inventory():return {str(p.relative_to(T/'live_data')):sha(p) for p in (T/'live_data').rglob('*') if p.is_file()}
before=inventory();policy=T/'live_data/b/policy.ron';old=policy.read_text(); start=old.index('id: "default_facility"');at=old.index('extra_waves: []',start)
wave='extra_waves: [(id: "audit_missing_spawn_zone", activation: (trigger_id: "audit_missing_spawn_zone", priority: 0, primary: AtTimeMs(time_ms: 5000)), spawn_zone_ids: ["audit_missing_spawn_zone"], route_id: Some("defense_main"), required_for_victory: true, source: Manual([CorrodedEmployee(profile_id: "corroded_guard", tier: I, count: 1)]))]'
bad=old[:at]+old[at:].replace('extra_waves: []',wave,1);(E/'diagnostic2_policy_delta.txt').write_text(wave+'\n');rows=[]
def run(name,args):
 cmd=[str(R/'target/release/consume_content_pack'),str(T/'live_data')]+args;r=subprocess.run(cmd,capture_output=True);(E/(name+'.stdout')).write_bytes(r.stdout);(E/(name+'.stderr')).write_bytes(r.stderr);rows.append({'name':name,'command':cmd,'exit_code':r.returncode,'stdout_sha256':hashlib.sha256(r.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(r.stderr).hexdigest()});return r
assert run('diagnostic2_valid_b',['official','b']).returncode==0
policy.write_text(bad);invalid_before=inventory()
for i in [1,2]:
 r=run('diagnostic2_invalid_b_'+str(i),['official','b']);assert r.returncode!=0;assert b'generated_combat_preview' in r.stderr;assert b'spawn wave references missing spawn zone' in r.stderr;print(r.stderr.decode());assert inventory()==invalid_before
policy.write_text(old);assert run('diagnostic2_restored_b',['official','b']).returncode==0;assert inventory()==before
result={'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary_sha256':sha(R/'target/release/consume_content_pack'),'fixture_original':before,'only_mutated_input':'b/policy.ron','mutated_sha256':hashlib.sha256(bad.encode()).hexdigest(),'mutation_locator':'combat_difficulty.profiles[1].ordinary_bands[0].tuning.extra_waves[0].spawn_zone_ids[0]','commands':rows,'original_inputs_restored':inventory()==before,'source_fidelity_observation':'Both negative release executions identify official/data/pve/encounters.ron encounters[0]; actual invalid winning definition resides in b/policy.ron. Diagnostics omit b/policy.ron and the contributing source list.','contract_oracle_pass':False}
(E/'source_diagnostic2_probe.json').write_text(json.dumps(result,indent=2)+'\n');shutil.copyfile(__file__,E/'source_diagnostic2_probe.py');print(json.dumps({'commands':len(rows),'contract_oracle_pass':False,'inputs_restored':True}))
