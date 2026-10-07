import json,pathlib,hashlib,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_independent_objects';A=pathlib.Path(__file__).parent;S=H/'execution/single_goal';E=H/'evidence'
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
coverage=load(S/'COVERAGE_002.json');errors=[];inputs=0;rows=[]
for label,rel in coverage['public_summaries'].items():
 p=H/rel;d=load(p)
 for key in ['input_files','inputs','input_sha256','external_input_files','final_input_files']:
  values=d.get(key,{})
  if isinstance(values,dict):
   for path,digest in values.items():
    f=R/path
    if isinstance(digest,str) and len(digest)==64 and f.is_file():
     inputs+=1
     if sha(f)!=digest:errors.append([label,key,path])
 rows.append({'name':label,'path':rel,'sha256':sha(p),'recorded_passed':d.get('passed')})
# Independent raw public outcomes: do not use implementation's expected-object snapshots as sole oracle.
def steps(p):
 return {v['payload']['index']:v['payload'] for l in p.read_text().splitlines() if l.startswith('{') and (v:=json.loads(l)).get('payload',{}).get('kind')=='step'}
objects=[]
for case in ['actor','small','big','ice','fallback','segments']:
 p=E/'public_objects_004'/f'{case}_simulate.stdout';st=steps(p);end=st[31 if case=='small' else 27]['public_result']['payload']['battle_update']['checkpoint'];active=end['independent_objects']['active'];assert len(active)==1
 o=active[0];assert o['source']['fragment_copy_uuid'];assert o['epoch']==(2 if case=='small' else 1)
 if case=='segments':assert o['max_health']==50 and any(x.get('segmented',{}).get('remaining_health')==100 for x in end['sustained_effects']['active'])
 if case=='fallback':assert o['max_health']>0
 assert end['independent_objects']==load(E/'public_object_inputs_001'/case/'expected_objects.json')
 objects.append({'case':case,'definition':o['definition_id'],'epoch':o['epoch'],'max_health':o.get('max_health'),'raw_sha256':sha(p)})
combat=[]
for case in ['ice','small','actor']:
 p=E/'public_object_combat_004'/f'{case}.stdout';st=steps(p);up=lambda n:st[n]['public_result']['payload']['battle_update'];cp=lambda n:up(n)['checkpoint'];o=cp(27)['independent_objects']['active'][0];oid=o['object_id'];creator=o['source']['creator'];assert o['created_at_ms']==1101
 assert cp(28)['independent_objects']['active']==cp(27)['independent_objects']['active'];assert not any(u['unit_instance_id']==creator for u in cp(28)['units'])
 events=up(31)['events_delta']['events'];damage=[e for e in events if e['event']['type']=='HpChanged' and e['event'].get('source_instance_id')==oid and e['event']['delta']<0];removed=[e for e in events if e['event']['type']=='IndependentObjectRemoved' and e['event']['receipt']['object_id']==oid]
 assert all(e['time_ms']>1700 for e in damage+removed)
 if case=='ice':
  hits=[e['event']['hp_after'] for e in events if e['event']['type']=='HpChanged' and e['event'].get('target_instance_id')==oid];assert hits==[2,1,0];assert len(removed)==1 and removed[0]['event']['receipt']['reason']=='destroyed';assert not cp(31)['independent_objects']['active'];assert not any(e['event']['type']=='UnitDied' and oid in json.dumps(e) for e in events)
  enemy=removed[0]['event']['receipt']['source_actor'];assert any(u['unit_instance_id']==enemy and u['position']=={'x':4,'y':6} for u in cp(31)['units'])
 elif case=='small':assert len(damage)==len(removed)==1 and damage[0]['event']['raw_damage']==24 and damage[0]['event']['final_damage']==25 and damage[0]['event']['damage_source']=='Ability' and removed[0]['event']['receipt']['reason']=='attack_consumed' and damage[0]['seq']<removed[0]['seq']
 else:assert len(damage)==1 and damage[0]['event']['final_damage']==10 and damage[0]['event']['damage_source']=='BasicAttack' and cp(31)['independent_objects']['active'][0]['object_id']==oid
 combat.append({'case':case,'damage_count':len(damage),'removal_count':len(removed),'raw_sha256':sha(p)})
# Current strict public schema output must contain its exact expected negative diagnostic.
strict=[]
for label in ['object_schema','starter','schema']:
 p=H/coverage['public_summaries'][label];d=load(p)
 for r in d['rows']:
  case=r['case'];root=p.parent
  # Readers use either flat case logs or a case directory.
  for stream in ['stdout','stderr']:
   options=[root/(case+'.'+stream),root/case/stream,root/case/(stream+'.log')]
   found=next((f for f in options if f.exists()),None)
   if found and stream+'_sha256' in r:assert sha(found)==r[stream+'_sha256']
   if found and stream=='stderr' and r.get('expected_diagnostic'):assert r['expected_diagnostic'] in found.read_text()
  assert r['exit_code']==(1 if r.get('expected_diagnostic') else 0)
 strict.append({'family':label,'cases':len(d['rows']),'negative':sum(bool(r.get('expected_diagnostic')) for r in d['rows'])})
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'summary_records':rows,'input_hash_checks':inputs,'objects':objects,'combat':combat,'strict_schema':strict,'errors':errors,'scope':'Read-only independent raw outcome/identity/hash checks; no product execution.'}
(A/'PUBLIC_VERIFICATION.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k not in ['summary_records','objects','combat']}))
