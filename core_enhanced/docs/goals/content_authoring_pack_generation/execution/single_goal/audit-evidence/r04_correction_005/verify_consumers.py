import json,hashlib,math,datetime
from pathlib import Path
R=Path('/mnt/f/work/simulator'); B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence'; O=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r04_correction_005'
def j(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
checks=[]
def check(name,ok,detail=None):checks.append({'name':name,'pass':bool(ok),'detail':detail})
command_counts={};stream_count=0
for d in sorted(B.glob('PREDECESSOR_*_003')):
 p=d/'commands.json'
 if not p.exists():p=d/'receipt.json'
 if not p.exists():continue
 x=j(p);cmds=x if isinstance(x,list) else x.get('commands',[])
 if not isinstance(cmds,list):continue
 command_counts[d.name]=len(cmds)
 for c in cmds:
  if 'expected_exit' in c:check(d.name+':exit:'+c['name'],c['exit_code']==c['expected_exit'])
  elif c.get('kind') in ['valid','invalid']:check(d.name+':exit:'+c['name'],c['exit_code']==(0 if c['kind']=='valid' else 101))
  for key in ['stdout','stderr','log']:
   if key+'_sha256' not in c:continue
   f=Path(c.get(key,c['name']+'.'+key));f=f if f.is_absolute() else d/f;check(d.name+':'+c.get('name','')+':'+key,sha(f)==c[key+'_sha256']);stream_count+=1
  for diag in c.get('required_diagnostics',[]):check(d.name+':forbidden diagnostic:'+diag,diag in Path(c['log']).read_text())
d=B/'PREDECESSOR_PACK_003'; receipt=j(d/'receipt.json');values=[j(d/(n+'.stdout')) for n in ['consume_a1','consume_b','consume_a2']]
check('actual full A/B/A output',values[0]==values[2] and values[0]!=values[1]);check('actual A/B identities',values[0]['content_identity']==receipt['a_identity'] and values[1]['content_identity']==receipt['b_identity'] and receipt['a_identity']!=receipt['b_identity'])
check('same release consumers',all(sha(Path(p))==h for p,h in receipt['binary_sha256'].items()))
check('real briefing/starter output',all(v['briefing_scripts']==['hq_briefing_standard_intro'] and v['run_snapshot']['inventory']['equipments'] for v in values))
for key in ['source_before_after','custom_before_after','official_before_after']:
 x=receipt[key]; root=d/({'source_before_after':'source','custom_before_after':'live_data/b','official_before_after':'live_data/official'}[key]); actual={str(p.relative_to(root)):sha(p) for p in root.rglob('*') if p.is_file()};check('pack preservation '+key,actual==x,{'files':len(x)})
# Direct historical R02 F01 rejection and valid output restoration on current binaries.
d=B/'PREDECESSOR_DIAGNOSTIC_003'
for a,b in [('valid_b','restored_b'),('boss_valid','boss_restored')]:check('full restored output '+a,(d/(a+'.stdout')).read_bytes()==(d/(b+'.stdout')).read_bytes())
for name in ['invalid_b','boss_invalid_route','boss_invalid_zone']:
 err=(d/(name+'_1.stderr')).read_bytes();check('source rejection repeat '+name, bool(err) and err==(d/(name+'_2.stderr')).read_bytes() and not (d/(name+'_1.stdout')).read_bytes() and not (d/(name+'_2.stdout')).read_bytes())
check('policy winning original source',b'b/policy.ron' in (d/'invalid_b_1.stderr').read_bytes() and b'spawn_zone_ids[0]' in (d/'invalid_b_1.stderr').read_bytes())
# Every retained current invalid case independently examines typed diagnostic and no stdout.
d=B/'PREDECESSOR_INVALID_003';cases=j(d/'cases.json');api=j(d/'api/RESULT.json')
check('actual API invalid/recovery cases',len(api['cases'])==13 and all(c['a_preserved'] and c['valid_replay_recovered'] and c['rejected_attempts']==2 for c in api['cases']))
for c in cases:
 name=c['name'];x=j(d/(name+'_0.stderr'));diag=x['diagnostic'];check('negative repeat:'+name,x==j(d/(name+'_1.stderr')) and not (d/(name+'_0.stdout')).read_bytes() and not (d/(name+'_1.stdout')).read_bytes());check('negative typed:'+name,diag['code']==c['code'] and diag['phase']==c['phase'])
 for field,key in [('source_file','source'),('field_path','field')]:
  if c.get(key):check('negative '+field+':'+name,diag.get(field)==c[key])
# Changed runtime carriers must retain actual A/B effect behavior, source and exact mode parity.
combat={}
for g,hp in [('a',61),('b',43)]:
 d=B/'PREDECESSOR_COMBAT_003';x=j(d/(g+'_off/RESULT.json'));w=x['first_sourced'];combat[g]=w
 check('combat '+g+' actual result',x['is_complete'] and x['failure'] is None and x['accepted_steps']==13 and w['time_ms']==7950 and w['change']['before']==79 and w['change']['after']==hp,w)
 data=(d/(g+'_off/receipts.ndjson')).read_bytes()
 for mode in ['on','gap']:check('combat '+g+' '+mode+' parity',j(d/(g+'_'+mode+'/RESULT.json'))==x and (d/(g+'_'+mode+'/receipts.ndjson')).read_bytes()==data)
 for mode in ['on','off']:check('combat CLI '+g+' '+mode,(d/('cli_'+g+'_'+mode+'.stdout')).read_bytes()==data)
 check('combat explained source '+g,j(d/('explain_'+g+'.stdout'))['source']==w['source']['authored'])
check('A/B same occurrence source',all(combat['a'][k]==combat['b'][k] for k in ['ordinal','time_ms','source_unit','target_unit','source']))
# Existing ordinary guards: exact input/harness and raw percentiles, deterministic values and allocator caps.
guards={};baseline=j(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')
for name in ['ORDINARY_GUARDS_004/kernel_plain.json','ORDINARY_GUARDS_004/kernel_alloc.json']:
 x=j(B/name);check('guard baseline '+name,x['baseline']['receipt_sha256']==sha(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json'));check('guard source '+name,x['tool_source_sha256']==sha(R/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs'));res={}
 for key in ['event_heavy','concurrent_sessions']:
  part=x[key];samples=part['latency_raw_samples'];elapsed=sorted(s['elapsed_ns'] for s in samples);p95=elapsed[math.ceil(len(elapsed)*.95)-1];check('p95 '+name+key,p95==part['nearest_rank_p95_ns'] and (not x['latency_gate_applicable'] or p95<=part['absolute_cap_ns']))
  original=baseline[key]['raw_samples'];fields=['event_count','serialized_bytes','event_log_sha256' if key=='event_heavy' else 'combined_event_log_sha256'];check('original parity '+name+key,all(all(s[f]==original[0][f] for f in fields) for s in samples));alloc=part['allocation_raw_samples'];metrics={'operations':lambda h:h['allocation_count']+h['reallocation_count'],'peak':lambda h:h['peak_delta_bytes'],'retained':lambda h:h['retained_delta_bytes']};caps={k:max(f(s['heap_allocation']) for s in original) for k,f in metrics.items()}
  check('original allocator bounds '+name+key,all(all(f(s['heap_allocation'])<=caps[k] for k,f in metrics.items()) for s in alloc),caps)
  res[key]={'p95_ns':p95,'absolute_cap_ns':part['absolute_cap_ns'],'allocator_peaks':{k:max((f(s['heap_allocation']) for s in alloc),default=0) for k,f in metrics.items()}}
 guards[name]=res
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'command':'python3 '+str(Path(__file__).relative_to(R)),'fresh_gameplay_executions':0,'command_counts':command_counts,'stream_hashes':stream_count,'cases':len(cases),'guards':guards,'checks':checks,'pass':all(c['pass'] for c in checks)}
(O/'consumer_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='checks'},indent=2));print('checks',len(checks),'failures',[c for c in checks if not c['pass']])
