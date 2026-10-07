from pathlib import Path
import json,hashlib,copy,tarfile,re
R=Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';S=D/'execution/s2';C=S/'compatibility/current_001';A=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest(); extras={}
def bind(p,expected=None):
 p=p.resolve();s=sha(p);assert expected is None or s==expected,(str(p),s,expected);extras[str(p)]={'path':str(p),'bytes':p.stat().st_size,'sha256':s};return s
def inputs(v):
 for p,h in (v.items() if isinstance(v,dict) else ((x['path'],x['sha256']) for x in v)):bind(R/p,h)
def write(n,v):(A/n).write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
probe=load(C/'public_probe_001/RESULT.json');assert probe['exit_code']==0;inputs(probe['input_files'])
for x in probe['output_files']:bind(C/'public_probe_001'/x['path'],x['sha256'])
log=(C/'public_probe_001/run.log').read_text()
for n in range(2):
 assert f'PASS failed attempt {n}: exact root and on-disk bytes retained' in log
 assert f'PASS failed attempt {n}: discarded observation retains work/peak without commit or invented fault' in log
assert 'persists peak 204 -> 213' in log
boundary=load(C/'boundary_001/RESULT.json');inputs(boundary['libraries']);assert len(boundary['cases'])==8
for row in boundary['cases']:
 bind(C/'boundary_001'/f'{row["name"]}.rs',row['source_sha256']);bind(C/'boundary_001'/f'{row["name"]}.log',row['log_sha256'])
 assert row['exit_code']==(0 if row['expected_error'] is None else 1)
 if row['expected_error']:assert row['expected_error'] in (C/'boundary_001'/f'{row["name"]}.log').read_text()
public={}
for name in ['public_boundary_001','public_profile_001','public_api','public_after','public_after_trace']:
 folder=C/'evidence'/name;p=folder/('RESULT.json' if name=='public_api' else 'SUMMARY.json');value=load(p)
 if 'input_files' in value:inputs(value['input_files'])
 for x in value.get('files',[]):bind(folder/x['path'],x['sha256'])
 public[name]={k:len(value[k]) if isinstance(value[k],(list,dict)) else value[k] for k in ['passed','failed','commands','cases','hosts','rows','negative_compile_consumers'] if k in value}
 if name=='public_profile_001':
  assert len(value['commands'])==24 and len(value['hosts'])==6
  for host in value['hosts']:
   h=folder/f'host_{host["name"]}'; candidates=[p for p in h.rglob('*') if p.is_file()];assert any(sha(p)==host['config_sha256'] for p in candidates);assert any(sha(p)==host['process_log_sha256'] for p in candidates)
   assert host['message_count']==(0 if host['name']=='invalid' else 5)
  assert next(x for x in value['hosts'] if x['name']=='invalid_observation')['expected_profile']==value['default_hash']
# Exact immediate PC parity includes all fields except the two documented timings.
a=D/'execution/s1/corrections/f02_persistence_entry_001/evidence/pc_raw_current_002.json';b=C/'evidence/pc_raw_current_002.json';old=load(a);new=load(b)
for d in [old,new]:
 assert len(d['rows'])==576
 for row in d['rows']:
  for key in ('measurement','setup'):
   if key in row:row[key].pop('elapsed_ns')
assert old==new
pc=load(C/'evidence/pc_isolate_001.json');product=load(S/'target/s2_001/MANIFEST.json')['product_source']['files'];actual=[[x['path'],x['bytes'],x['sha256']] for x in product]
assert pc['current_source']['files']==actual
assert pc['current_source']==pc['copied_source']
base={x[0]:x[1:] for x in pc['current_source']['files']};built={x[0]:x[1:] for x in pc['built_source']['files']}
assert [p for p in base if base[p]!=built[p]]==pc['instrumented_paths']
archive=C/'evidence/pc_isolate_hooks_001.tar.gz';bind(archive,pc['hooks_archive']['sha256'])
with tarfile.open(archive) as tf:
 assert sorted(tf.getnames())==sorted(pc['instrumented_paths']+list(pc['supplemental']))
 for name in tf.getnames():
  data=tf.extractfile(name).read();h=hashlib.sha256(data).hexdigest()
  assert h==(built[name][1] if name in built else pc['supplemental'][name])
  if name.endswith('runtime_control.rs'):assert data.startswith((R/name).read_bytes())
  if name.endswith('value_expr_runtime.rs'):assert data.startswith((R/name).read_text().rstrip()[:-1].encode())
def events(path):
 found={}
 def walk(v):
  if isinstance(v,dict):
   if {'seq','time_ms','event'}<=v.keys():
    if v['seq'] in found:assert found[v['seq']]==v
    found[v['seq']]=v
   for x in v.values():walk(x)
  elif isinstance(v,list):
   for x in v:walk(x)
 for line in path.read_text().splitlines():walk(json.loads(line))
 return found
oracle=[]
for row in load(C/'evidence/public_after/SUMMARY.json')['rows']:
 name=row['case'];raw=C/f'evidence/public_after/{name}.jsonl';bind(raw,row['log_sha256']);current=events(raw)
 previous=D/f'execution/s1/corrections/f02_persistence_entry_001/evidence/public_after/{name}.jsonl';bind(previous);assert current==events(previous)
 trace=C/f'evidence/public_after_trace/{name}.jsonl';assert current==events(trace)
 steps={v['event']['step_id']:v['time_ms'] for v in current.values() if v['event']['type']=='AbilityStepTriggered' and v['event'].get('skill_id') in ('r08_strike','r12_boss_marker')}
 assert steps==row['steps'];assert ('area' in steps)==(name=='detach');assert 'marker' in steps
 hp=[v for v in current.values() if v['event']['type']=='HpChanged' and v['event'].get('raw_damage')==1];assert hp==row['hp']
 assert (not hp and 'projectile' not in steps) if name=='early_cancel' else any(v['time_ms']==5885 and v['event']['delta']==-1 for v in hp)
 oracle.append({'case':name,'events':len(current),'steps':steps,'immediate_s1_events_identical':True,'trace_events_identical':True})
write('PUBLIC_COST_VERIFICATION.json',{'public':public,'public_oracles':oracle,'prepared_host_probe':{'raw_exit':0,'two_failures_preserve_exact_disk_root':True,'each_failure_one_discarded_observation':True,'no_commit_or_invented_fault':True,'retry_persistent_peak':[204,213]},'prepared_compile_cases':[{k:r[k] for k in ['name','exit_code','expected_error']} for r in boundary['cases']],'pc_immediate':{'before_sha256':bind(a),'after_sha256':bind(b),'rows':576,'all_non_timing_fields_equal':True,'unmodified_isolate_exact_product_membership':True},'R05':'R05_RECLASSIFIED.json; raw101 warning_concurrent_peak_only +343390 over9030188, no other hard failures'})
# Keep exact additional transitive input/library evidence for the end check.
write('EXTRA_INPUTS.json',list(extras.values()));print('Verified public inputs',len(extras),'and exact PC576 non-timing parity.')
