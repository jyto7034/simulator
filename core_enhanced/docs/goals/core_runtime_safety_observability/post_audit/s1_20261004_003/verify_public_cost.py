import json,hashlib,tarfile,tempfile
from pathlib import Path
R=Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_safety_observability';C=H/'execution/s1/corrections/f02_persistence_entry_001';A=Path(__file__).parent;E=C/'evidence';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
s=json.loads((E/'public_after/SUMMARY.json').read_text());ts=json.loads((E/'public_after_trace/SUMMARY.json').read_text());assert ts['input_summary_sha256']==sha(E/'public_after/SUMMARY.json')
for path,digest in s['input_files'].items():assert sha(R/path)==digest
assert sha(Path(s['rows'][0]['command'][0]))==s['binary_sha256']
def events(path):
 out={}
 def walk(x):
  if isinstance(x,dict):
   if {'seq','time_ms','event'}<=x.keys():out[x['seq']]=x
   for v in x.values():walk(v)
  elif isinstance(x,list):
   for v in x:walk(v)
 for line in path.read_text().splitlines():walk(json.loads(line))
 return out
public=[]
for row,trow in zip(s['rows'],ts['rows']):
 name=row['case'];assert name==trow['case'];off=E/'public_after'/(name+'.jsonl');on=E/'public_after_trace'/(name+'.jsonl');assert sha(off)==row['log_sha256'] and sha(on)==trow['output_sha256'];assert sha(E/'public_after_trace'/(name+'.trace.jsonl'))==trow['trace_sha256'];a=events(off);b=events(on);assert a==b and a
 steps={e['event']['step_id']:e['time_ms'] for e in a.values() if e['event'].get('skill_id') in ('r08_strike','r12_boss_marker') and e['event']['type']=='AbilityStepTriggered'}
 hp=[e for e in a.values() if e['event']['type']=='HpChanged' and e['event'].get('raw_damage')==1]
 assert steps['marker']=={'early_cancel':5051,'inflight_cancel':5501}.get(name,6501)
 assert len(hp)==(0 if name=='early_cancel' else 3 if name=='detach' else 1)
 if name=='early_cancel':assert 'projectile' not in steps
 else:
  assert steps['projectile']==5300 and hp[0]['time_ms']==5885
  assert all(e['event']['delta']==-1 and e['event']['hp_before']-e['event']['hp_after']==1 and e['event']['source_instance_id']=='a1fbb727-5065-44b3-b248-2cd9e88a484a' for e in hp)
 assert ('area' in steps)==(name=='detach')
 if name=='detach':assert steps['area']==6885 and all(e['time_ms']==6885 for e in hp[1:]) and len({e['event']['target_instance_id'] for e in hp[1:]})==2
 public.append(dict(case=name,events=len(a),steps=steps,hp_events=len(hp),trace_equal=True))
cli=json.loads((E/'cli_current/CLI_FINAL_SUMMARY_002.json').read_text());assert len(cli['commands'])==45
for row in cli['commands']:
 assert row['exit_code']==0;assert sha(E/'cli_current'/(row['name']+'.log'))==row['log_sha256']
# Isolated PC36 source copy is the current runtime membership; two measurement-only hooks are explicit.
p=json.loads((E/'pc_isolate_001.json').read_text());m=json.loads((C/'target/s1_f02_001/MANIFEST.json').read_text());assert p['current_source']==p['copied_source'];base={n:(size,digest) for n,size,digest in p['current_source']['files']};assert base=={x['path']:(x['bytes'],x['sha256']) for x in m['product_source']['files']};built={n:(size,digest) for n,size,digest in p['built_source']['files']};assert sorted(n for n in base if base[n]!=built[n])==sorted(p['instrumented_paths']);assert sha(E/'pc_isolate_hooks_001.tar.gz')==p['hooks_archive']['sha256']
with tempfile.TemporaryDirectory(prefix='s1f02-audit-hooks-') as tmp:
 with tarfile.open(E/'pc_isolate_hooks_001.tar.gz') as tf:tf.extractall(tmp,filter='data')
 for n in p['instrumented_paths']:assert sha(Path(tmp)/n)==built[n][1]
 for n,digest in p['supplemental'].items():assert sha(Path(tmp)/n)==digest
# Re-execute the frozen R05 classifier with only its output destination changed.
code=(C/'summarize_r05.py').read_text().replace("(E / 'R05_SUMMARY_CURRENT_002.json').open('x')", "(Path("+repr(str(A/'R05_RECHECK.json'))+")).open('x')")
(A/'recheck_r05.py').write_text(code)
exec(compile(code,str(A/'recheck_r05.py'),'exec'),{})
assert json.loads((A/'R05_RECHECK.json').read_text())==json.loads((E/'R05_SUMMARY_CURRENT_002.json').read_text())
(A/'PUBLIC_COST_VERIFICATION.json').write_text(json.dumps(dict(public_oracles=public,trace_cases=5,cli_commands=45,pc_source_copy_current=True,pc_measurement_hook_paths=p['instrumented_paths'],pc_hook_archive_sha256=sha(E/'pc_isolate_hooks_001.tar.gz'),r05_classifier_reexecuted=True),indent=2)+'\n')
print('PASS public oracles/trace/45CLI hashes/current isolated PC36 source/R05 classifier')
