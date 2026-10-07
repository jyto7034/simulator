import json,hashlib,datetime,re
from pathlib import Path
H=Path('core_enhanced/docs/goals/core_relic_numeric_variants');A=H/'post_audit/u08_current_012_pass001';P=H/'execution/single_goal/compatibility/current_001/evidence/PC_IMMEDIATE_ACCOUNTED_U07M_007.json';p=json.loads(P.read_text())
def ref(path):
 b=Path(path).read_bytes();return {'path':str(path),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
refs={k:ref(p[k]['path']) for k in ['before','after','immediate_u07m']}
for k in refs:assert refs[k]['sha256']==p[k]['sha256'],k
rows={k:json.loads(Path(v['path']).read_text())['rows'] for k,v in refs.items()};assert all(len(r)==576 for r in rows.values())
def compare(before,after,delta):
 differences={};failures=[]
 def walk(x,y,path):
  if type(x)!=type(y):failures.append(['type',path]);return
  if isinstance(x,dict):
   if x.keys()!=y.keys():failures.append(['keys',path])
   for k in sorted(x.keys()&y.keys()):walk(x[k],y[k],path+'/'+k)
  elif isinstance(x,list):
   if len(x)!=len(y):failures.append(['length',path])
   for i,(u,v) in enumerate(zip(x,y)):walk(u,v,path+'/'+str(i))
  elif x!=y:
   generic=re.sub(r'/\d+','/*',path);differences[generic]=differences.get(generic,0)+1
   if path in ['/measurement/elapsed_ns','/setup/elapsed_ns']:return
   if delta==200:
    if path=='/retained_extent_at_end' and y-x==200:return
    if path.startswith('/native_usage/') and path.endswith('/2') and y-x==200:return
    if path=='/error':
     rx=re.search(r'requested: Some\((\d+)\)',x);ry=re.search(r'requested: Some\((\d+)\)',y)
     if rx and ry and int(ry[1])-int(rx[1])==200 and x[:rx.start()]+x[rx.end():]==y[:ry.start()]+y[ry.end():]:return
   failures.append(['non_timing_difference',path,x,y])
 for x,y in zip(before,after):walk(x,y,'')
 return {'rows':len(before),'allowed_non_timing_delta':delta,'differences_by_path':differences,'failures':failures,'passed':not failures}
results={'older_starter_tactics_to_u08':compare(rows['before'],rows['after'],200),'immediate_u07m_to_u08':compare(rows['immediate_u07m'],rows['after'],0)}
out={'schema':'U08_AUDIT_PC_BASELINE_CORRECTION_001','recorded_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_audit_preserved':ref(A/'COMPLETION_AUDIT.md'),'original_validator_preserved':ref(A/'verify_additional.py'),'cost_artifact':ref(P),'raw_refs':refs,'original_validator_actual_comparison':['before','after'],'original_label_incorrect':'The original validator comment and report called the before path immediate U07M; it is older core_battle_starter_tactics.','results':results,'findings_unchanged':['F01','F02'],'product_or_performance_runs':0};(A/'PC_BASELINE_CORRECTION_001.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
