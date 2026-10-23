"""Check complete exported scenarios and the CLI's paired amount documents."""
import json
from pathlib import Path
E=Path(__file__).resolve().parent
results=[]
for prefix in ['cli_020','cli_022']:
 for path in sorted(E.glob(prefix+'_*.result.json')):
  result=json.loads(path.read_text())
  if result['exit_code']!=0:
   results.append({'run':path.stem,'exit_code':result['exit_code'],'accepted_steps':0,'diagnostic':json.loads((E/result['stderr']).read_text())})
   continue
  cmd=result['command'];scenario=json.loads(Path(cmd[4]).read_text())
  docs=[json.loads(line) for line in (E/result['stdout']).read_text().splitlines()]
  steps=[v['payload'] for v in docs if v.get('payload',{}).get('kind')=='step']
  expected=len(scenario.get('preparation_steps',[]))+len(scenario['steps'])
  assert [s['index'] for s in steps]==list(range(expected)),path
  amount=[v for v in docs if v.get('kind') in ['amount_preview','amount_explanation']]
  assert len(amount)==4,path
  for preview,explain in zip(amount[::2],amount[1::2]):
   assert preview['status']==explain['status'],path
   for key in ['selector','identity','typed_base_amount','source','site','trace']:
    assert preview.get(key)==explain.get(key),(path,key)
  trace=Path(cmd[cmd.index('--trace-output')+1]);assert trace.exists() and trace.stat().st_size>0,path
  results.append({'run':path.stem,'exit_code':0,'scenario':str(Path(cmd[4]).relative_to(E)), 'runtime_safety':cmd[cmd.index('--runtime-safety')+1] if '--runtime-safety' in cmd else 'embedded default','expected_steps':expected,'accepted_steps':len(steps),'amount_statuses':[v['status'] for v in amount], 'trace_bytes':trace.stat().st_size})
(E/'CLI_VERIFICATION.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
print('successful complete scenarios',sum(r['exit_code']==0 for r in results),'preserved default-limit refusal',sum(r['exit_code']!=0 for r in results))
