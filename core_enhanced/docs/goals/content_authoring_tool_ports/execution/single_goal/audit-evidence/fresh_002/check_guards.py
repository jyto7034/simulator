from pathlib import Path
import json,hashlib,math
R=Path('/mnt/f/work/simulator');P=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal';E=P/'evidence';O=P/'audit-evidence/fresh_002'
def j(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
out={'scope':'Read-only reuse of original unchanged ordinary runtime/public guard seams; not a new latency run or changed Trial cost proof.','original_failed_kernel_attempts':[],'guard_files':{}}
a=j(E/'runtime_guards_001.json');assert a['passed'] is False
for c in a['commands']:
 if c['name'] in ['kernel_plain','kernel_alloc']:assert c['exit_code']==101;out['original_failed_kernel_attempts'].append({'name':c['name'],'exit':101})
for d,src in [('RUNTIME_GUARDS_001','public_guards.rs'),('KERNEL_GUARDS_002','final_performance.rs')]:
 for name in ['plain','alloc']:
  a=j(E/d/(name+'.json'));assert sha(R/'core_enhanced/tools/runtime_architecture_final_validation/src'/src)==a['tool_source_sha256'];checks={}
  def walk(x,p=''):
   if isinstance(x,dict):
    for k,v in x.items():
     path=p+k
     if k.endswith('_pass') or k in ['repeatable','exact_repeatable']:assert v is True,(path,v);checks[path]=v
     elif isinstance(v,dict):walk(v,path+'.')
  walk(a)
  if d=='KERNEL_GUARDS_002':
   assert len(a['fixture_conversion_elapsed_ns'])==5;assert 'one validated generation' in a['kernel_input'];assert a['baseline']['receipt_sha256']==sha(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')
   for phase in ['event_heavy','concurrent_sessions']:
    q=a[phase];assert q['nearest_rank_p95_ns']<=q['absolute_cap_ns'];checks[phase+'.p95_ns']=q['nearest_rank_p95_ns'];checks[phase+'.cap_ns']=q['absolute_cap_ns']
  else:
   for phase in ['run_snapshot','checkpoint_export']:
    q=a[phase];raw=sorted(q['raw_elapsed_ns']);assert len(raw)==q['sample_count'];assert raw[math.ceil(.95*len(raw))-1]==q['nearest_rank_p95_ns'];assert q['encoded_bytes']<=q['encoded_byte_cap']
  out['guard_files'][d+'/'+name+'.json']={'sha256':sha(E/d/(name+'.json')),'source_sha256':a['tool_source_sha256'],'checked':checks}
for src in ['public_guards.rs','final_performance.rs']:
 s=(R/'core_enhanced/tools/runtime_architecture_final_validation/src'/src).read_text();assert 'TrialSession' not in s and 'authoring::simulate' not in s
(O/'guards_check.json').write_text(json.dumps(out,indent=2)+'\n');print('Verified four retained actual guard files; original kernel101 failures preserved; current ordinary source exact.')
