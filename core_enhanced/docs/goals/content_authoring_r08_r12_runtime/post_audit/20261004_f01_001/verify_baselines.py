from pathlib import Path
import json,hashlib,subprocess,tarfile,copy
R=Path('/mnt/f/work/simulator');G=R/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime';A=Path(__file__).resolve().parent
E=G/'execution/single_goal/corrections/f01_action_flow_20261004_001/evidence'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();load=lambda p:json.loads(Path(p).read_text())
old=G/'execution/single_goal/target/r08_r12_integrated_001';m=load(old/'MANIFEST.json')
for row in m['records']:
 p=R/row['path'];assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'],str(p)
for row in m['archives_and_patches']:
 p=old/row['path'];assert p.stat().st_size==row['bytes'] and sha(p)==row['sha256'],str(p)
cli=load(E/'cli_current/CLI_FINAL_SUMMARY_002.json');assert len(cli['commands'])==45 and len(cli['comparisons'])==9
for row in cli['commands']:
 assert row['exit_code']==0 and sha(E/'cli_current'/(row['name']+'.log'))==row['log_sha256']
# The named comparator is unchanged and inspected; run it with a new audit-local output.
compare=G/'execution/single_goal/compare_pc_v32.py'
p=subprocess.run(['python3','-B',str(compare),str(E/'pc_raw_current_001.json'),str(A/'PC_RECOMPARE.json')],capture_output=True)
(A/'pc_recompare.log').write_bytes(p.stdout+p.stderr);assert p.returncode==0,p.stderr
pc=load(A/'PC_RECOMPARE.json');assert pc==load(E/'PC_SUMMARY_CURRENT_001.json')
iso=load(E/'pc_isolate_current_001.json');assert iso['current_source']==iso['copied_source']
assert iso['current_source']['digest']=='5b05f2454b83b9b1dc4a4db50b7d4fc066efeca84d319c6095212c5c505ac876'
original={x[0]:x for x in iso['copied_source']['files']};built={x[0]:x for x in iso['built_source']['files']}
assert sorted(n for n in original if original[n]!=built[n])==sorted(iso['instrumented_paths'])
assert sha(E/'pc_isolate_hooks_current_001.tar.gz')==iso['hooks_archive']['sha256']
with tarfile.open(E/'pc_isolate_hooks_current_001.tar.gz') as ar:
 for n in iso['instrumented_paths']:
  data=ar.extractfile(n).read();assert hashlib.sha256(data).hexdigest()==built[n][2]
  source=(R/n).read_text();instrumented=data.decode()
  if n.endswith('runtime_control.rs'):assert instrumented.startswith(source) and '#[cfg(test)]' in instrumented[len(source):]
  else:assert instrumented.startswith(source.rstrip()[:-1])
 for n,h in iso['supplemental'].items():assert hashlib.sha256(ar.extractfile(n).read()).hexdigest()==h
 probe=ar.extractfile('pc_probe.rs').read().decode()
 assert 'strip_prefix(b"{\\"version\\":32,")' in probe and 'hash.update(suffix)' in probe
raw=load(E/'r05_raw_current_002.json');basepath=G/'execution/single_goal/evidence/P01_LOG_V32_BASELINE_001.json';base=load(basepath)
originalbase=load(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json');b1=copy.deepcopy(originalbase);b2=copy.deepcopy(base)
for family,field in [('event_heavy','event_log_sha256'),('concurrent_sessions','combined_event_log_sha256')]:
 for left,right in zip(b1[family]['raw_samples'],b2[family]['raw_samples']):left.pop(field);right.pop(field)
assert b1==b2
assert raw['baseline']['receipt_sha256']==sha(basepath)
assert raw['tool_source_sha256']==sha(R/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs')
assert raw['workload_input_sha256']==load(R/'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence/r05_raw_integrated_003.json')['workload_input_sha256']
assert raw['allocation_gate_applicable'] and raw['allocation_observer_enabled'] and not raw['latency_gate_applicable']
summary={}
for family in ['event_heavy','concurrent_sessions']:
 work=raw[family];samples=work['allocation_raw_samples'];oldrows=base[family]['raw_samples'];assert len(samples)==len(oldrows)==5 and len(work['latency_raw_samples'])==21
 def amounts(rows):
  h=[r['heap_allocation'] for r in rows]
  return {'operations':max(r['allocation_count']+r['reallocation_count'] for r in h),'retained':max(r['retained_delta_bytes'] for r in h),'peak':max(r['peak_delta_bytes'] for r in h)}
 caps=amounts(oldrows);now=amounts(samples);assert now['operations']<=caps['operations'] and now['retained']<=caps['retained']
 assert work['repeatable'] and work['deterministic_baseline_parity']
 if family=='event_heavy':assert now['peak']<=caps['peak']
 else:assert caps['peak']==9030188 and now['peak']==9373775
 summary[family]={'caps':caps,'observed_maxima':now,'parity':True,'repeatability':True}
receipt=load(E/'r05_current_002.json');assert receipt['exit_code']==101
log=(E/'r05_current_002.log').read_text();assert log.count("thread 'main'")==1 and 'assertion failed: receipt.concurrent_sessions.allocation_retained_peak_guard_pass' in log
out={'old_records_preserved':len(m['records']),'old_archives_and_patches_preserved':len(m['archives_and_patches']),'cli_commands_validated':45,'cli_comparisons':cli['comparisons'],'pc_comparator_sha256':sha(compare),'pc_receipt_identical_to_independent_recompare':True,'pc_instrumentation_only_verified':True,'r05_raw_sha256':sha(E/'r05_raw_current_002.json'),'r05_baseline_sha256':sha(basepath),'r05_raw_exit_code':101,'r05_classification':'authorized concurrent peak warning only','r05_concurrent_peak_excess':343587,'r05_other_hard_failures':[], 'r05':summary}
with (A/'BASELINE_REVIEW.json').open('x') as f:json.dump(out,f,indent=2);f.write('\n')
print(json.dumps({k:v for k,v in out.items() if k!='cli_comparisons'},indent=2))
