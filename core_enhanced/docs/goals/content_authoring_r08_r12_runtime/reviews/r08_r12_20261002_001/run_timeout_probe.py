"""Public release CLI probe; only new external RON inputs and evidence are written."""
import json,shutil,subprocess,sys,hashlib
from pathlib import Path
import run
root=run.HERE/'evidence/timeout_marker_probe';root.mkdir()
base=run.HERE/'evidence/timeout_probe'
summary=[]
for name,policy,timeout in [('early_cancel','CancelUnreleased',50),('cancel','CancelUnreleased',1500),('detach','Detach',1500)]:
 dest=root/name;shutil.copytree(base/policy,dest)
 p=dest/'content/official/data/boss_runtime/base.ron';s=p.read_text()
 old='outcome_id:"timedout",target_node_id:"timedout"';assert s.count(old)==1
 s=s.replace(old,'outcome_id:"timedout",target_node_id:"completed"')
 if timeout!=1500:s=s.replace('timeout_ms:1500','timeout_ms:'+str(timeout),1)
 p.write_text(s)
 commands=[('validate',[str(dest/'content'),str(dest/'selection.json')]),('simulate',[str(dest/'content'),str(dest/'selection.json'),str(dest/'scenario.json'),'--amount-observation'])]
 for op,args in commands:
  output_name=f'timeout_marker_{name}_{op}_001'
  command=[str(run.ROOT/'.cache/d07-runtime-amount-product-target/release/author_content'),op,*args]
  receipt=subprocess.run([sys.executable,str(run.HERE/'run.py'),output_name,'--',*command],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
  (root/f'{output_name}.console.log').write_bytes(receipt.stdout);assert receipt.returncode==0,receipt.stdout[-1000:]
 events={}
 def walk(v):
  if isinstance(v,dict):
   if 'time_ms' in v and 'event' in v:events[v['seq']]=v
   for x in v.values():walk(x)
  elif isinstance(v,list):
   for x in v:walk(x)
 for s in (run.HERE/f'evidence/timeout_marker_{name}_simulate_001.log').read_text().splitlines():walk(json.loads(s))
 boss_steps=[e for e in events.values() if e['event'].get('skill_id') in ('r08_strike','r12_boss_marker') and e['event']['type']=='AbilityStepTriggered']
 hp=[e for e in events.values() if e['event']['type']=='HpChanged' and e['event'].get('raw_damage')==1]
 row=dict(name=name,policy=policy,timeout_ms=timeout,steps=boss_steps,hp=hp)
 summary.append(row)
with (root/'OBSERVED.json').open('x') as f:json.dump(summary,f,indent=2)
rows=[dict(path=str(p.relative_to(run.ROOT)),bytes=p.stat().st_size,sha256=run.sha(p)) for p in root.rglob('*') if p.is_file()]
with (root/'MANIFEST.json').open('x') as f:json.dump(rows,f,indent=2)
print(json.dumps(summary,indent=2))
