import json,pathlib,hashlib,tarfile,copy,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_independent_objects';C=H/'execution/single_goal/compatibility/current_001';A=pathlib.Path(__file__).parent
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
p=C/'evidence/r05_harness_004/PROOF.json';proof=load(p);raw=load(C/'evidence/r05_raw_current_004.json');baseline=load(R/proof['baseline']['path']);original=load(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')
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
assert results['concurrent_sessions']['maximum_operations_retained_peak'][2]-9030188==630076
log=(H/'evidence/compat_r05_004.log').read_text();assert log.count("thread 'main'")==1 and 'assertion failed: receipt.concurrent_sessions.allocation_retained_peak_guard_pass' in log
cache=H/'evidence/r05_cache_relocation_001';after=load(cache/'AFTER.json');assert sha(cache/'FRESH_TARGET_DEPINFO_004.d')==after['fresh_target_depinfo_sha256'];dep=(cache/'FRESH_TARGET_DEPINFO_004.d').read_text();assert after['fresh_target_include_path'] in dep and after['executed_main_path'] in dep
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'historical_archive_members_match':True,'baseline_only_log_hash_migration':True,'workload_input_exact_prior':True,'raw_exit_code':load(H/'evidence/compat_r05_004.json')['exit_code'],'raw_sha256':sha(C/'evidence/r05_raw_current_004.json'),'fresh_depinfo_verified':True,'measurements':results,'classification':'Only inherited concurrent peak warning; original hard guard remains failed in raw execution. No new allowance.', 'errors':[]}
(A/'PERFORMANCE_VERIFICATION.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
