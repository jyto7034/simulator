import json,pathlib,hashlib,tarfile,copy,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_independent_objects';C=H/'execution/single_goal/compatibility/current_001';A=pathlib.Path(__file__).parent
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
p=C/'evidence/r05_harness_005/PROOF.json';proof=load(p);raw=load(C/'evidence/r05_raw_current_005.json');baseline=load(R/proof['baseline']['path']);original=load(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')
with tarfile.open(R/proof['archive']['path']) as archive:
 for family in ['preservation','final']:
  assert archive.extractfile(proof[family+'_workspace']).read()==(p.parent/(family+'_original.rs')).read_bytes()
old=copy.deepcopy(original);new=copy.deepcopy(baseline)
for family,field in [('event_heavy','event_log_sha256'),('concurrent_sessions','combined_event_log_sha256')]:
 for a,b in zip(old[family]['raw_samples'],new[family]['raw_samples']):a.pop(field);b.pop(field)
assert old==new
assert raw['allocation_observer_enabled'] and raw['allocation_gate_applicable'] and not raw['latency_gate_applicable']
assert raw['tool_source_sha256']==sha(p.parent/'final_current.rs')
prior=load(R/'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence/r05_raw_integrated_003.json');assert raw['workload_input_sha256']==prior['workload_input_sha256']
results={}
for family in ['event_heavy','concurrent_sessions']:
 d=raw[family];a=d['allocation_raw_samples'];b=baseline[family]['raw_samples'];assert len(a)==len(b)==5 and len(d['latency_raw_samples'])==21
 def measures(s):
  x=s['heap_allocation'];return [x['allocation_count']+x['reallocation_count'],x['retained_delta_bytes'],x['peak_delta_bytes']]
 actual=[max(measures(s)[i] for s in a) for i in range(3)];caps=[max(measures(s)[i] for s in b) for i in range(3)]
 assert actual[0]<=caps[0] and actual[1]<=caps[1] and (family=='concurrent_sessions' or actual[2]<=caps[2]);assert d['repeatable'] and d['deterministic_baseline_parity']
 results[family]={'maximum_operations_retained_peak':actual,'original_caps':caps,'repeatable':True,'baseline_parity':True}
assert results['concurrent_sessions']['original_caps'][2]==9030188
assert results['concurrent_sessions']['maximum_operations_retained_peak'][2]-9030188==639853
log=(H/'evidence/compat_r05_005.log').read_text();assert log.count("thread 'main'")==1 and 'assertion failed: receipt.concurrent_sessions.allocation_retained_peak_guard_pass' in log
deps=load(p.parent/'EXECUTION_DEPENDENCIES.json');dep_path=p.parent/'FRESH_TARGET_DEPINFO.d';assert sha(dep_path)==deps['depinfo_sha256'];tokens=set()
for line in dep_path.read_text().splitlines():
 if ': ' in line:tokens.update(line.split(': ',1)[1].split())
assert tokens=={'src/final_performance.rs','src/../../runtime_architecture_preservation/src/main.rs'}
resolved={token:str((pathlib.Path(deps['depinfo_resolution_base'])/token).resolve()) for token in sorted(tokens)}
assert resolved==deps['depinfo_sources'] and set(resolved.values())=={str(pathlib.Path(deps['executed_main']).resolve()),str(pathlib.Path(deps['actual_include']).resolve())}
for key,path in [('executed_main',pathlib.Path(deps['executed_main'])),('actual_include',pathlib.Path(deps['actual_include'])),('receipt',R/deps['receipt']),('environment',R/deps['environment']),('raw',C/'evidence/r05_raw_current_005.json'),('summary',C/'evidence/R05_SUMMARY_CURRENT_005.json'),('binary',pathlib.Path(deps['target'])/'release/final_performance')]:assert sha(path)==deps[key+'_sha256'],key
assert sha(p)==deps['harness_proof_sha256']
assert deps['executed_main_sha256']=='bbbae7691cdc824bf2828f59e18b9cd2205baacca6fdca48cf98bfdd64b1e0f4' and deps['actual_include_sha256']=='64debfd6546afb617636caf520fcb9e8ee49acc1ee2377306cf8a19afbd8c631'
environment=load(R/deps['environment']);assert environment['r05_target_existed_before_measurement'] is False and environment['fresh_r05_target']==deps['target']
assert f"Compiling runtime_architecture_final_validation v0.1.0 ({deps['depinfo_resolution_base']})" in log
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'historical_archive_members_match':True,'baseline_only_log_hash_migration':True,'workload_input_exact_prior':True,'raw_exit_code':load(H/'evidence/compat_r05_005.json')['exit_code'],'raw_sha256':sha(C/'evidence/r05_raw_current_005.json'),'fresh_depinfo_verified':True,'measurements':results,'classification':'Only inherited concurrent peak warning; original hard guard remains failed in raw execution. No new allowance.', 'errors':[]}
(A/'PERFORMANCE_VERIFICATION.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
