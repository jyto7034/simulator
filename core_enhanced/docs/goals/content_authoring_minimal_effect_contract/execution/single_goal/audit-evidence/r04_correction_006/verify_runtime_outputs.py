import pathlib,json,hashlib,math,datetime
R=pathlib.Path('/mnt/f/work/simulator'); B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence'; O=R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/audit-evidence/r04_correction_006'
def j(p):return json.loads(p.read_text())
def rows(p):return [json.loads(x) for x in p.read_text().splitlines()]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
checks=[]
def require(condition,label,actual=None):
 checks.append({'check':label,'passed':bool(condition),'actual':actual})
# Independently bind retained command streams; no owner probe is executed.
command_counts={}; stream_count=0
for d in B.glob('PREDECESSOR_*_003'):
 p=d/'commands.json'
 if not p.exists():continue
 commands=j(p); commands=commands if isinstance(commands,list) else commands.get('commands',[]);command_counts[d.name]=len(commands)
 for c in commands:
  for key in ['stdout','stderr']:
   if key+'_sha256' not in c:continue
   p=pathlib.Path(c[key]);p=p if p.is_absolute() else d/p
   require(sha(p)==c[key+'_sha256'],d.name+'/'+c.get('name','unnamed')+'/'+key);stream_count+=1
# Real common executor and source, including exact trace-mode/API/CLI receipts.
combat=B/'PREDECESSOR_COMBAT_003';outcomes={}
for g,expected in [('a',61),('b',43)]:
 summary=j(combat/(g+'_off')/'RESULT.json');receipt=(combat/(g+'_off')/'receipts.ndjson').read_bytes();w=summary['first_sourced'];outcomes[g]=w
 require(summary['failure'] is None and summary['is_complete'] and summary['accepted_steps']==13,'combat '+g+' completion')
 require(w['time_ms']==7950 and w['change']['before']==79 and w['change']['after']==expected,'combat '+g+' HP/source',w)
 require(w['source']['proc_origin'] is not None and w['source']['authored']['site']['effect_id']=='effect_01','combat '+g+' real proc/authored origin')
 for mode in ['on','gap']:
  require(j(combat/(g+'_'+mode)/'RESULT.json')==summary,'combat '+g+' '+mode+' complete summary parity')
  require((combat/(g+'_'+mode)/'receipts.ndjson').read_bytes()==receipt,'combat '+g+' '+mode+' receipt parity')
 for mode in ['off','on']:require((combat/('cli_'+g+'_'+mode+'.stdout')).read_bytes()==receipt,'combat '+g+' CLI '+mode+' parity')
 explain=j(combat/('explain_'+g+'.stdout'));require(explain['source']==w['source']['authored'],'combat '+g+' source explain parity')
require(all(outcomes['a'][k]==outcomes['b'][k] for k in ['ordinal','time_ms','source_unit','target_unit','source']),'combat same occurrence with amount-only delta')
# Resource/buff outputs exercise unchanged real command path after runtime instrumentation.
for folder in ['PREDECESSOR_RESOURCE_003','PREDECESSOR_BUFF_003']:
 d=B/folder
 for g in ['a','b']:
  receipt=(d/(g+'_off')/'receipts.ndjson').read_bytes()
  for mode in ['on','gap']:require((d/(g+'_'+mode)/'receipts.ndjson').read_bytes()==receipt,folder+' '+g+' '+mode+' receipt parity')
  require((d/('cli_'+g+'.stdout')).read_bytes()==receipt,folder+' '+g+' CLI parity')
  rs=rows(d/(g+'_on')/'terminal_trace.ndjson')
  resource=[x for x in rs if x['change']['channel']=='resonance' and x['source']]
  require(bool(resource) and resource[0]['change']['after']-resource[0]['change']['before']==(17 if g=='a' else 31),folder+' '+g+' actual sourced resource',resource[:1])
  if folder.endswith('BUFF_003'):
   buffs=[x for x in rs if x['change']['channel']=='buff' and x['source']]
   require(len(buffs)==5 and all(x['change']['kind']=='Silence' and x['source']['authored']['site']['effect_id']=='effect_02' and x['change']['after']=={'stacks':1,'expires_at_ms':x['time_ms']+700,'next_tick_ms':None} for x in buffs),folder+' '+g+' actual buffs',buffs[:1])
# Delayed carriers retained after control lifetimes; source is compared with explain.
d=B/'PREDECESSOR_DELIVERY_003';receipt=(d/'off/receipts.ndjson').read_bytes();source_rows=rows(d/'on/terminal_trace.ndjson');counts={}
for mode in ['on','gap','repeat']:require((d/mode/'receipts.ndjson').read_bytes()==receipt,'delivery '+mode+' full receipt parity')
for mode in ['off','on']:require((d/('cli_'+mode+'.stdout')).read_bytes()==receipt,'delivery CLI '+mode+' parity')
for action,effect in [('delayed_repeat','repeat_hit'),('delayed_projectile','projectile_hit'),('periodic_area','area_hit')]:
 hits=[x for x in source_rows if x['source'] and x['source']['authored']['site']['action']['action_id']==action];counts[action]=len(hits)
 require(bool(hits) and all(x['change']['channel']=='hp' and x['change']['before']>x['change']['after'] and x['source']['authored']['site']['effect_id']==effect for x in hits),'delivery '+action+' actual HP/source')
 require(j(d/('explain_'+action+'.stdout'))['source']==hits[0]['source']['authored'],'delivery '+action+' explain source')
require(counts=={'delayed_repeat':14,'delayed_projectile':7,'periodic_area':48},'delivery actual hit counts',counts)
# Real negative inputs: preserved invalid stdout, repeatable typed phase/code/location.
d=B/'PREDECESSOR_INVALID_003';cases=j(d/'cases.json');negative=[]
for c in cases:
 name=c['name'];first=j(d/(name+'_0.stderr'));second=j(d/(name+'_1.stderr'));diagnostic=first.get('diagnostic',{})
 require(first==second and (d/(name+'_0.stdout')).read_bytes()==b'' and (d/(name+'_1.stdout')).read_bytes()==b'','invalid '+name+' repeat rejection no token/output')
 require(diagnostic.get('code')==c['code'] and diagnostic.get('phase')==c['phase'],'invalid '+name+' exact code/phase',diagnostic)
 if c.get('source'):require(diagnostic.get('source_file')==c['source'],'invalid '+name+' exact source')
 if c.get('field'):require(diagnostic.get('field_path')==c['field'],'invalid '+name+' exact field')
 negative.append(name)
# Packaging and source-diagnostic restoration use actual output, not receipt booleans.
d=B/'PREDECESSOR_PACK_003';vals=[j(d/(n+'.stdout')) for n in ['consume_a1','consume_b','consume_a2']]
require(vals[0]==vals[2] and vals[0]!=vals[1],'pack A/B/A complete output restoration')
d=B/'PREDECESSOR_DIAGNOSTIC_003'
for a,b in [('valid_b','restored_b'),('boss_valid','boss_restored')]:require((d/(a+'.stdout')).read_bytes()==(d/(b+'.stdout')).read_bytes(),'diagnostic '+a+' restoration')
for a in ['invalid_b','boss_invalid_route','boss_invalid_zone']:
 require((d/(a+'_1.stdout')).read_bytes()==b'' and (d/(a+'_2.stdout')).read_bytes()==b'' and (d/(a+'_1.stderr')).read_bytes()==(d/(a+'_2.stderr')).read_bytes(),'diagnostic '+a+' rejection repeat')
# Original guard data, original harness identity, recompute p95/parity and maxima.
guards={}
for name in ['ORDINARY_GUARDS_004/kernel_plain.json','ORDINARY_GUARDS_004/kernel_alloc.json']:
 d=j(B/name);parts={}; baseline=j(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')
 require(d['baseline']['receipt_sha256']==sha(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json'),'guard '+name+' original baseline hash')
 require(d['tool_source_sha256']==sha(R/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs'),'guard '+name+' current unchanged tool hash')
 for key in ['event_heavy','concurrent_sessions']:
  part=d[key];samples=part['latency_raw_samples'];times=sorted(x['elapsed_ns'] for x in samples);p95=times[math.ceil(len(times)*.95)-1]
  require(p95==part['nearest_rank_p95_ns'],'guard '+name+' '+key+' recomputed p95',p95)
  if d['latency_gate_applicable']:require(p95<part['absolute_cap_ns'],'guard '+name+' '+key+' absolute latency')
  parity_fields=['event_count','serialized_bytes','event_log_sha256'] if key=='event_heavy' else ['event_count','serialized_bytes','combined_event_log_sha256']
  require(all(all(x[f]==samples[0][f] for f in parity_fields) for x in samples),'guard '+name+' '+key+' raw repeat parity')
  original=baseline[key]['raw_samples']
  require(all(all(x[f]==original[0][f] for f in parity_fields) for x in samples),'guard '+name+' '+key+' original baseline parity')
  alloc=part['allocation_raw_samples']
  if alloc:
   fields={'operations':lambda h:h['allocation_count']+h['reallocation_count'],'peak':lambda h:h['peak_delta_bytes'],'retained':lambda h:h['retained_delta_bytes']}
   limits={k:max(f(x['heap_allocation']) for x in original) for k,f in fields.items()}
   require(all(all(f(x['heap_allocation'])<=limits[k] for k,f in fields.items()) for x in alloc),'guard '+name+' '+key+' original allocation/peak/retained caps',limits)
  peak=max((x['heap_allocation']['peak_delta_bytes'] for x in alloc),default=0);operations=max((x['heap_allocation']['allocation_count']+x['heap_allocation']['reallocation_count'] for x in alloc),default=0)
  parts[key]={'p95_ns':p95,'cap_ns':part['absolute_cap_ns'],'max_peak_delta':peak,'max_alloc_realloc':operations,'samples':len(samples),'allocation_samples':len(alloc)}
 guards[name]=parts
result={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'fresh_gameplay_executions':0,'retained_stream_hash_checks':stream_count,'command_counts':command_counts,'invalid_cases':negative,'delivery_counts':counts,'guard_recomputations':guards,'checks':checks,'passed':all(x['passed'] for x in checks)}
(O/'runtime_outputs.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='checks'},indent=2));print('failed',[x for x in checks if not x['passed']]);print('checks',len(checks))

raise SystemExit(0 if result["passed"] else 1)
