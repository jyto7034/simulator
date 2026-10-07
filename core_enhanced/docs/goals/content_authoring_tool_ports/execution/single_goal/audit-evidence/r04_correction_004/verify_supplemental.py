# Read-only adaptation of historical R03 audit reader, independently reviewed in this pass.
import pathlib,json,hashlib,datetime
R=pathlib.Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence';O=pathlib.Path(__file__).parent
checks=[]
def j(p):return json.loads(p.read_text())
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ck(n,v):checks.append({'check':n,'ok':bool(v)})
target=j(R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/AUDIT_TARGET_004.json')
def refs(v):
 if isinstance(v,dict):
  if 'path' in v and v.get('sha256') and 'bytes' in v and (R/v['path']).is_file():yield v
  for k,x in v.items():
   if k not in ['manifest','external_actual']:yield from refs(x)
 elif isinstance(v,list):
  for x in v:yield from refs(x)
for row in refs(target):ck('additional target role '+row['path'],h(R/row['path'])==row['sha256'] and (R/row['path']).stat().st_size==row['bytes'])
for gen in ['a','b']:
 d=j(E/'PREDECESSOR_SOURCE_003/api/off'/(gen+'_source.json'))
 for f in d['files']:
  p=E/'PREDECESSOR_SOURCE_003/fixture'/gen/f['path'];ck('exact caller source '+gen+'/'+f['path'],h(p)==f['sha256'] and p.stat().st_size==f['byte_count'])
p=E/'PREDECESSOR_DIAGNOSTIC_003';d=j(p/'receipt.json')
for row in d['commands']:
 for stream in ['stdout','stderr']:ck('diagnostic raw '+row['name']+stream,h(p/(row['name']+'.'+stream))==row[stream+'_sha256'])
for a,b in [('valid_b','restored_b'),('boss_valid','boss_restored')]:ck('diagnostic restored '+a,(p/(a+'.stdout')).read_bytes()==(p/(b+'.stdout')).read_bytes())
for name in ['invalid_b','boss_invalid_route','boss_invalid_zone']:
 ck('diagnostic repeat '+name,(p/(name+'_1.stderr')).read_bytes()==(p/(name+'_2.stderr')).read_bytes())
 for i in [1,2]:ck('no invalid publication '+name+str(i),(p/(name+'_'+str(i)+'.stdout')).stat().st_size==0)
attr=j(E/'ATTRIBUTION_002.json');evidence=j(O/'evidence_review.json');ck('all54 oldR03 changes attributable',len(evidence['old_R03_changed_members'])==54 and set(evidence['old_R03_changed_members'])<={x['path'] for x in attr['changed_files']})
ck('no official source R04 delta',not any(x['path'].startswith('game_resources/') for x in attr['changed_files']))
# Full corrected target includes the complete native search and original allocator identities.
manifest={m['path']:m for m in target['manifest']}
for inv in target['external_actual']:
 for m in inv['files']:manifest[inv['directory']+'/'+m['path']]=m
for name in ['ORDINARY_GUARDS_004/native_budget.json','ORDINARY_GUARDS_004/binaries/native_fact_budget','ORDINARY_GUARDS_004/kernel_alloc.json','ORDINARY_GUARDS_004/binaries/kernel_alloc']:
 p=E/name;rel=str(p.relative_to(R));ck('corrected capture '+rel,rel in manifest and h(p)==manifest[rel]['sha256'])
baseline=j(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')
current=j(E/'ORDINARY_GUARDS_004/kernel_alloc.json')
for kind in ['event_heavy','concurrent_sessions']:
 rows=baseline[kind]['raw_samples'];caps={k:max(x['heap_allocation'][k] for x in rows) for k in ['peak_delta_bytes','retained_delta_bytes']};ops=max(x['heap_allocation']['allocation_count']+x['heap_allocation']['reallocation_count'] for x in rows)
 for index,row in enumerate(current[kind]['allocation_raw_samples']):
  a=row['heap_allocation'];ck('raw original allocation limits '+kind+str(index),all(a[k]<=v for k,v in caps.items()) and a['allocation_count']+a['reallocation_count']<=ops)
 for index,row in enumerate(current[kind]['latency_raw_samples']):
  keys=['event_count','serialized_bytes']+(['event_log_sha256','ability_steps','projectile_launches','projectile_impacts','effect_events'] if kind=='event_heavy' else ['combined_event_log_sha256','session_count','lane_count','step_count_per_session'])
  ck('raw original gameplay parity '+kind+str(index),all(row[k]==rows[0][k] for k in keys))
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'passed':all(x['ok'] for x in checks),'check_count':len(checks),'checks':checks};(O/'supplemental_review.json').write_text(json.dumps(result,indent=2)+'\n');print('checks',len(checks),'failures',[x for x in checks if not x['ok']])
