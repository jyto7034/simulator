"""Expected contract oracle, deliberately exits 1 while F01 reproduces."""
import json,sys
import run
E=run.HERE/'evidence'
rows=[]
for name in ['timeout_marker_probe_002','timeout_marker_probe_003']:
 for case in json.loads((E/name/'OBSERVED.json').read_text()):
  steps=case['steps'];by_step={v['event']['step_id']:v['time_ms'] for v in steps}
  marker=by_step.get('marker');area=by_step.get('area');projectile=by_step.get('projectile')
  assert marker is not None
  keep=case['name']=='detach'
  expected_area=keep
  valid=(area is not None)==expected_area
  if case['name']=='early_cancel':valid=valid and projectile is None and not case['hp']
  else:valid=valid and projectile is not None and any(h['time_ms']==5885 and h['event']['delta']==-1 for h in case['hp'])
  rows.append(dict(case=case['name'],policy=case['policy'],timeout_ms=case['timeout_ms'],marker_ms=marker,projectile_ms=projectile,area_ms=area,expected_area=expected_area,hp_events=[dict(time_ms=h['time_ms'],delta=h['event']['delta'],before=h['event']['hp_before'],after=h['event']['hp_after']) for h in case['hp']],passed=valid))
def events(path):
 out={}
 def walk(v):
  if isinstance(v,dict):
   if 'time_ms' in v and 'event' in v:out[v['seq']]=v
   for x in v.values():walk(x)
  elif isinstance(v,list):
   for x in v:walk(x)
 for line in path.read_text().splitlines():walk(json.loads(line))
 return out
trace_same=events(E/'timeout_marker_cancel_simulate_002.log')==events(E/'timeout_marker_cancel_trace_002.log')
assert trace_same
result=dict(finding='F01',oracle_cases=len(rows),passed=sum(r['passed'] for r in rows),failed=sum(not r['passed'] for r in rows),trace_on_off_all_public_battle_events_equal=trace_same,rows=rows,verdict='Not Complete')
with (E/'CANCELLATION_ORACLE.json').open('x') as f:json.dump(result,f,indent=2)
print(json.dumps(result,indent=2));sys.exit(0 if all(r['passed'] for r in rows) else 1)
