import json,hashlib,re,datetime,math
from pathlib import Path
R=Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal';O=B/'audit-evidence/fresh_001'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
results=[];errors=[]
for name in ['full_core_002','all_targets_001','release_check_001','release_build_005','release_ab_003','api_batch_001','public_guards_002','public_guards_allocation_001']:
 d=json.loads((B/'evidence'/f'{name}.json').read_text());checks={}
 for k in ['input','log','receipt']:
  if k in d:checks[k]=sha(B/d[k])==d[k+'_sha256']
 if not all(checks.values()):errors.append(name)
 results.append({'name':name,'command':d['command'],'exit_code':d['terminal_exit_code'],'identity_checks':checks})
log=(B/'evidence/full_core_002.log').read_text();counts=[tuple(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)];assert sum(m[0] for m in counts)==1214 and all(m[1:]==(0,0) for m in counts)
h=json.loads((B.parent.parent/'P2G_60_HANDOFF_001.json').read_text());upstream=[{'path':m['path'],'matches':sha(R/m['path'])==m['sha256']} for m in h['source_locations']+h['external_context_locations']];assert all(m['matches'] for m in upstream)
api=json.loads((B/'evidence/api_batch_001/receipt.json').read_text());api_rows=[]
for c in api['commands']:
 p=B/'evidence/api_batch_001'/Path(c['log']).name;text=p.read_text();checks={'log_identity':sha(p)==c['log_sha256'],'terminal_result':(c['exit_code']==0)==(c['kind']=='valid'),'expected_diagnostics':all(x in text for x in c['required_diagnostics'])};assert all(checks.values());api_rows.append({'name':c['name'],'kind':c['kind'],'exit_code':c['exit_code'],'checks':checks})
world=(R/'core_enhanced/src/game/world.rs').read_text();old=world.replace('    #[cfg(test)]\n    pub fn execute_with_transport_context(', '    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]\n    pub fn execute_with_transport_context(');expected='af895613ecd394a409239e02ee7ff5a8c0ed1444192c69ca49b888f98c30e442';assert hashlib.sha256(old.encode()).hexdigest()==expected
ab=json.loads((B/'evidence/release_ab_003/commands.json').read_text());abrows=[]
for c in ab:
 checks={k:sha(B/'evidence/release_ab_003'/Path(c[k]).name)==c[k+'_sha256'] for k in ['stdout','stderr']};assert all(checks.values());abrows.append({'name':c['name'],'exit_code':c['exit_code'],'checks':checks})
values=[json.loads((B/'evidence/release_ab_003'/f'{n}.stdout').read_text()) for n in ['consume_a1','consume_b','consume_a2']];currency=[v['run_snapshot']['resources']['enkephalin'] for v in values];assert currency==[500,733,500] and values[0]==values[2] and values[0]['content_identity']!=values[1]['content_identity']
guards=[]
for name in ['public_guards_002_receipt','public_guards_allocation_001_receipt']:
 g=json.loads((B/'evidence'/f'{name}.json').read_text());checks={}
 for key in ['run_snapshot','checkpoint_export']:
  v=g[key];p95=sorted(v['raw_elapsed_ns'])[math.ceil(len(v['raw_elapsed_ns'])*.95)-1];checks[key]={'count':len(v['raw_elapsed_ns']),'p95_matches':p95==v['nearest_rank_p95_ns'],'cap_pass_matches':(p95<=v['absolute_cap_ns'])==v['absolute_cap_pass']}
 guards.append({'name':name,'allocation_observer_enabled':g['allocation_observer_enabled'],'checks':checks,'other_observation_keys':[k for k in g if k not in ['run_snapshot','checkpoint_export']]})
res={'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'results':results,'errors':errors,'whole_core_counts':counts,'whole_core_total':1214,'handoff_inputs':upstream,'api_rows':api_rows,'API_world_prior_cfg_exact_reconstruction':True,'API_nonimpact':'three test modules plus private wrapper excluded from both default-release cfg forms; source actual read, VI022→023 only rustdoc','release_rows':abrows,'actual_currencies':currency,'repeat_A_complete_value_equal':True,'guard_checks':guards,'fresh_game_commands':'4 terminal subprocesses in diagnostic2 probe (plus initial probe 2 terminal subprocesses before audit-script assertion defect); no broad suite rerun'};(O/'validation_review.json').write_text(json.dumps(res,indent=2)+'\n');print(json.dumps({'result_records':len(results),'hash_errors':errors,'whole_core':1214,'handoff_inputs':len(upstream),'api':len(api_rows),'release':len(abrows),'currency':currency,'guards':len(guards)}));assert not errors
