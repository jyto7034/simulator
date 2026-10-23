from pathlib import Path
import json,re,hashlib,collections
R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/core_facility_map_nodes/execution/single_goal';A=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text())
result={}
src=(R/'game_resources/data/map/facility_templates.ron').read_text()
assignment=load(R/'core_enhanced/docs/ui-layout/node/data/facility-room-assignment-v1.json')
# Independent cross-check against adopted authored inputs, not generator output.
blocks=re.findall(r'\(id: "([^"]+)", depth: (\d+), (.*?)(?=^\(id: |\], connections:)',src,re.M|re.S)
rooms={id:(int(depth),body) for id,depth,body in blocks}
errors=[]
for expected in assignment['rooms']:
 id=expected['id'];depth,body=rooms[id]
 actual_template=re.search(r'template_id: Some\("([^"]+)"\)',body).group(1)
 wing=re.search(r'wing: Some\("([^"]+)"\)',body).group(1)
 name=re.search(r'name: "([^"]+)"',body).group(1)
 reservations=re.search(r'reservations: \[(.*?)\], omen_slot:',body,re.S).group(1)
 slots=re.findall(r'slot_id: "([^"]+)"',reservations)
 elite=re.search(r'elite_slots: \[(.*?)\]',body,re.S).group(1)
 safe=re.search(r'safe_access_slots: \[(.*?)\]',body,re.S).group(1)
 omen=re.search(r'omen_slot: (None|Some\("([^"]+)"\))',body).group(2)
 observed=(depth,wing,name,actual_template,sorted(slots),sorted(re.findall(r'"([^"]+)"',elite)),sorted(re.findall(r'"([^"]+)"',safe)),omen)
 desired=(expected['depth'],expected['wing'],expected['name'],'facility_'+expected['template_id'],sorted(str(r['node']) for r in expected['reservations']),sorted(expected['elite_candidates']),sorted(expected['safe_access_nodes']),expected['omen_node'])
 if observed!=desired:errors.append({'room':id,'observed':observed,'expected':desired})
edges=re.findall(r'\(to: "([^"]+)", direction: (\w+), condition: (?:None|Some\("[^"]*"\)), from: "([^"]+)"\)',src)
actual={(f,t,d) for t,d,f in edges}
expected={(e['a'],e['b'],'Bidirectional' if rooms[e['a']][0]==rooms[e['b']][0] else 'ForwardOnly') for e in assignment['connections']}
result['authored_graph']={'room_count':len(rooms),'activity_rooms_compared':len(assignment['rooms']),'activity_slots':sum(r['size'] for r in assignment['rooms']),'room_field_differences':errors,'connections':len(actual),'missing_connections':sorted(expected-actual),'extra_connections':sorted(actual-expected)}
result['test_reuse']={f:re.findall(r'test result:.*',(E/f).read_text()) for f in ['test013.stdout.log','test012.stdout.log','test014.stdout.log','server_test001.stdout.log']}
saved=load(E/'external004/route/saved.json');resumed=load(E/'external004/route/resumed.json')
assert saved['roots']==resumed['roots']
result['continue']={'exact_public_roots_and_checkpoint_equal':True,'writer':json.loads(bytes(saved['checkpoint']['bytes']))['version']}
steps=sorted((E/'normal_route001').glob('*.execution.json'));assert len(steps)==30
assert all(load(p)['exit_code']==0 for p in steps)
raw=sorted((E/'normal_route001').glob('[0-9]*_*/RESULT.json'));assert len(raw)==30 and all(load(p)['passed'] for p in raw)
admissions=[]
for p in sorted((E/'normal_route001').glob('[0-9]*_*/admission.json')):
 x=load(p);states=[m['state'] for m in x['individuals']];assert states and all(s=='passed' for s in states);admissions.append(str(p.relative_to(E)))
result['normal_host']={'actual_execution_exit0':len(steps),'actual_result_passed':len(raw),'individual_passed_admissions':len(admissions)}
start=[json.loads(l) for l in (E/'external004/battle_start.jsonl').open()]
preview=next(x['result']['combat_preview'] for x in reversed(start) if 'combat_preview' in x['result'])
trace=[json.loads(l) for l in (E/'external004/battle/trace.jsonl').open()]
entry=trace[0]['payload']; setup=entry['battle_setup_snapshot']
comparisons={key:preview[key]==entry[key] for key in ['encounter_id','mission_variant','node_type']}
for key in ['width','height','tiles','valid_tiles']:
 comparisons[key]=preview[key]==setup['battlefield'][key]
for key in ['routes','deployment_zones']:comparisons[key]=preview[key]==setup[key]
comparisons['static_obstacles']=preview['obstacles']==setup['battlefield']['static_obstacles']
assert all(comparisons.values())
last_tick=[t for t in trace if t['operation']=='host_tick'][-1]
end=trace[-1];assert end['success'] and end['error'] is None and last_tick['result']['finished']
assert end['profile']['encounter_compendium']['registered_ids']==['corroded_guard','corroded_marksman']
result['battle']={'briefing_actual_equal':comparisons,'host_ticks':sum(t['operation']=='host_tick' for t in trace),'real_deploy_commands':sum(t['operation']=='command' and t['input']['type']=='deploy_unit' for t in trace),'finished':True,'registered_profiles':end['profile']['encounter_compendium']['registered_ids'],'scope':'Raw command/setup/final registration cross-check; source and selected negative compendium tests establish excluded wave/summon semantics.'}
# Existing restore evidence is checked from archive bytes rather than running a mutating finalizer.
import tarfile
T=E/'target/u13_current_001'
def archived(p):
 with tarfile.open(p) as t:return {m.name:t.extractfile(m).read() for m in t if m.isfile()}
before=archived(T/'BEFORE.tar.gz');after=archived(T/'SOURCE.tar.gz');delta=load(T/'DELTA.json')
manifest={x['path']:x for x in load(T/'SOURCE_MANIFEST.json')['files']}
assert set(after)==set(manifest)
assert all(hashlib.sha256(v).hexdigest()==manifest[k]['sha256'] for k,v in after.items())
work=before.copy()
for reverse in [False,True]:
 original,destination=(after,before) if reverse else (before,after)
 for row in delta:
  p=row['path'];digest=row['after_sha256' if reverse else 'before_sha256'];assert (hashlib.sha256(work[p]).hexdigest() if p in work else None)==digest
  if p in destination:work[p]=destination[p]
  else:del work[p]
 assert work==destination
result['archive_restore_review']={'source_members':len(after),'baseline_members':len(before),'operations':len(delta),'archived_source_manifest_bytes_equal':True,'guarded_forward_and_reverse_equal':True,'mode':'in-memory independent re-evaluation; GEX10 isolated filesystem restore is reused'}
(A/'RAW_REVIEW.json').write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
print(json.dumps(result,indent=2,ensure_ascii=False))
