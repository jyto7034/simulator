import json,hashlib,re,tarfile,difflib,math,itertools
from pathlib import Path
R=Path('/mnt/f/work/simulator'); B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal'; E=B/'evidence'; O=B/'audit-evidence/fresh_001'; checks=[]
def sha(b):return hashlib.sha256(b).hexdigest()
def ck(name,ok,detail=None):
 checks.append({'check':name,'pass':bool(ok),'detail':detail})
def read(p):return json.loads(p.read_text())
base=read(B/'baseline/BASELINE_001.json');before={}
with tarfile.open(B/'baseline/SOURCE_BEFORE_001.tar.gz') as t:
 for m in t.getmembers(): before[m.name]=t.extractfile(m).read()
for x in base['files']:ck('baseline:'+x['path'],sha(before[x['path']])==x['sha256'] and len(before[x['path']])==x['bytes'])
with tarfile.open(E/'SOURCE_AFTER_001.tar.gz') as t:
 for m in t.getmembers():ck('final_archive:'+m.name,t.extractfile(m).read()==(R/m.name).read_bytes())
inputs=read(E/'IMPLEMENTATION_INPUT_007.json')['files']
for p,h in inputs.items():ck('input007:'+p,sha((R/p).read_bytes())==h)
att=read(E/'ATTRIBUTION_001.json');patch=''
for x in att['changed_files']:
 p=x['path'];old=before.get(p,b'');new=(R/p).read_bytes()
 ck('attribution:'+p,sha(new)==x['after_sha256'] and len(new)==x['after_bytes'] and (not old or sha(old)==x['before_sha256']))
 patch+=''.join(difflib.unified_diff(old.decode().splitlines(True),new.decode().splitlines(True),fromfile=('before/'+p if p in before else '/dev/null'),tofile='after/'+p))
ck('patch_exact',patch.encode()==(E/'R04_DELTA_001.patch').read_bytes(),{'files':len(att['changed_files']),'sha256':sha(patch.encode())})
# Bind the complete actual upstream JSON/payload and Handoff source identities.
handoff=read(B.parent.parent/'P2G_60_HANDOFF_002.json')
for x in handoff['source_locations']+handoff['external_context_locations']:ck('handoff:'+x['path'],sha((R/x['path']).read_bytes())==x['sha256'])
selected=[]
for number in [164,165,167,168,170,171,172,173,174,175,176,177,178,179,180,181,182,183,184,185,186,187,188,189,190,191]:
 files=list(E.glob('r04_*'+str(number)+'.json'));ck('one receipt '+str(number),len(files)==1)
 d=read(files[0]);raw=E/d['raw_log'];ck('command:'+files[0].name,d['exit_code']==0 and sha(raw.read_bytes())==d['log_sha256'],d['command']);selected.append({'record':str(files[0].relative_to(R)),**d})
log=(E/'r04_full164.log').read_text();names=re.findall(r'^test (.+) \.\.\. ok$',log,re.M);summaries=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',log)
ck('full164 individual nonzero tests',len(names)==1361,{'actual_test_rows':len(names),'summaries':summaries});ck('full164 all failures/omissions zero',all(all(int(v)==0 for v in row[1:]) for row in summaries));ck('full164 exact groups',[int(x[0]) for x in summaries]==[1309,4,4,25,15,4,0])
section=log.split('Running tests/skill_runtime_contract.rs')[1].split('test result:')[0];ck('skill_runtime_contract actual15',len(re.findall(r'^test .+ \.\.\. ok$',section,re.M))==15)
(O/'full164_test_rows.txt').write_text('\n'.join(names)+'\n')
# Recompute original performance guard arithmetic directly from raw samples.
p01=read(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json');kernel_source=R/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs'
for file in ['ORDINARY_GUARDS_003/kernel_plain.json','CLONE_CAP_CHARACTERIZATION_001/finalalloc168.json']:
 d=read(E/file);ck('kernel source '+file,d['tool_source_sha256']==sha(kernel_source.read_bytes()));ck('P01 baseline '+file,d['baseline']['receipt_sha256']==sha((R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json').read_bytes()))
 for mode in ['event_heavy','concurrent_sessions']:
  v=d[mode];baseline=p01[mode]['raw_samples'];samples=v['latency_raw_samples'];p95=sorted(x['elapsed_ns'] for x in samples)[math.ceil(len(samples)*.95)-1]
  ck(file+':'+mode+':p95',p95==v['nearest_rank_p95_ns'],p95)
  keys=['event_count','serialized_bytes']+(['event_log_sha256','ability_steps','projectile_launches','projectile_impacts','effect_events'] if mode=='event_heavy' else ['combined_event_log_sha256','session_count','lane_count','step_count_per_session'])
  ck(file+':'+mode+':baseline deterministic parity',all(all(s[k]==baseline[0][k] for k in keys) for s in samples+v['allocation_raw_samples']))
  if d['latency_gate_applicable']:
   base_ns=[x['elapsed_ns'] for x in baseline];new_ns=[x['elapsed_ns'] for x in samples];joined=base_ns+new_ns;observed=sum(new_ns)/len(new_ns)-sum(base_ns)/len(base_ns);extreme=0;total=0
   for subset in itertools.combinations(range(len(joined)),len(new_ns)):
    st=set(subset);diff=sum(joined[i] for i in st)/len(new_ns)-sum(joined[i] for i in range(len(joined)) if i not in st)/len(base_ns);total+=1;extreme+=diff>=observed
   prob=extreme/total;ck(file+':'+mode+':latency guards',p95<v['absolute_cap_ns'] and prob>=.05 and abs(prob-v['one_sided_exact_regression_p'])<1e-12,{'permutation_p':prob,'cap':v['absolute_cap_ns']})
  if d['allocation_gate_applicable']:
   obs=[x['heap_allocation'] for x in v['allocation_raw_samples']];old=[x['heap_allocation'] for x in baseline];fields={'operations':lambda x:x['allocation_count']+x['reallocation_count'],'peak':lambda x:x['peak_delta_bytes'],'retained':lambda x:x['retained_delta_bytes']};actual={k:max(f(x) for x in obs) for k,f in fields.items()};caps={k:max(f(x) for x in old) for k,f in fields.items()};ck(file+':'+mode+':original allocation guards',all(actual[k]<=caps[k] for k in fields),{'actual':actual,'original_caps':caps})
# Complete search and exact endpoint comparisons, not report passed fields.
d=read(E/'CLONE_CAP_CHARACTERIZATION_001/finalbudget167.json');baseline=d['baseline'];ck('budget kernel identity',d['kernel_source_sha256']==sha(kernel_source.read_bytes()));ck('budget search nonzero',len(d['search_cases'])==67);ck('budget baseline 5000ms',baseline['clock_ms']==5000 and baseline['passed'] and baseline['fault'] is None)
for x in d['search_cases']:
 if x['passed']:ck('budget search success',x['facts']==baseline['facts'] and x['log_sha256']==baseline['log_sha256'] and x['clock_ms']==5000)
 else:ck('budget search typed failure',x['fault']['code'] in ['clone_limit','work_limit','retained_byte_limit'])
for x in d['boundaries']:
 exact=x['exact'];below=x['one_below'];key={'clone_slots':'max_clone_slots','work':'max_work','retained_bytes':'max_retained_bytes'}[x['dimension']];code={'clone_slots':'clone_limit','work':'work_limit','retained_bytes':'retained_byte_limit'}[x['dimension']];ck('budget endpoint '+x['dimension'],exact['passed'] and exact['facts']==baseline['facts'] and exact['log_sha256']==baseline['log_sha256'] and exact['limits'][key]==x['minimum'] and below['limits'][key]==x['minimum']-1 and not below['passed'] and below['fault']['code']==code,{'minimum':x['minimum'],'one_less_fault':below['fault']})
ck('budget raw log complete boundaries',json.loads((E/'r04_finalbudget167.log').read_text())==d['boundaries'])
result={'pass':all(x['pass'] for x in checks),'checks':checks,'total_checks':len(checks),'failed':[x for x in checks if not x['pass']],'owner_commands_reused':selected,'new_gameplay_executions':0,'scope':'read-only reconstruction of captured source, command/results, original guards and native budget; not a new owner run'}
(O/'evidence_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['checks','owner_commands_reused']},indent=2))
