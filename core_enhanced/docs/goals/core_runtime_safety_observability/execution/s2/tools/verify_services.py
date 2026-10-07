from pathlib import Path
import json,hashlib,math
D=Path(__file__).resolve().parents[3];S=D/'execution/s2';out=S/'evidence/services';live=out/'live006';render=out/'render007';frames=[]
for folder in [live,render]:
 j=json.loads((folder/'browser.json').read_text());assert not j['errors']
 for response in j['responses']:
  assert response['status']==200
  for result in response['body'].get('results',{}).values():
   assert not result.get('error')
   # Grafana attaches the expression only to the first frame of each refId.
   series=result.get('frames',[])
   expr=next((f['schema'].get('meta',{}).get('executedQueryString') for f in series if f['schema'].get('meta',{}).get('executedQueryString')),'')
   for frame in series:
    values=frame.get('data',{}).get('values',[])
    finite=[v for column in values[1:] for v in column if isinstance(v,(int,float)) and math.isfinite(v)]
    frames.append({'source':str(folder.relative_to(S)),'expression':expr,'finite_samples':len(finite),'positive_samples':sum(v>0 for v in finite)})
for token in ['core_runtime_operation_seconds_bucket','core_runtime_selected_usage{slot="0",quantity="simulation_time_ms"','core_runtime_selected_usage{slot="0",quantity="accounted_extent_bytes"']:
 assert any(token in f['expression'] and f['positive_samples']>0 for f in frames),token
for name in ['duration-distribution.txt','simulation-time.txt']:assert 'No data' not in (render/name).read_text()
ended=json.loads((live/'ended.json').read_text());assert ended['selected_session'] is None and not any(x['current']['battle'] for x in ended['sessions']);assert ended['ring'][-1]['boundary']=='ended'
files=[]
for folder in [live,render]:
 for p in sorted(folder.iterdir()):
  if p.is_file():files.append({'path':str(p.relative_to(S)),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
(out/'SERVICE_VERIFICATION.json').write_text(json.dumps({'passed':True,'final_runtime_receipt':'../../../../evidence/s2_services_live006.json','render_receipt':'../../../../evidence/s2_services_render007.json','frames':frames,'files':files,'note':'live006 duration-distribution filename actually captures native work panel12; render007 explicitly captures wall-duration panel11 over the exact live006 interval. Both are real browser responses/screenshots. Owned helper003 services stopped cleanly afterward.'},indent=2))
print('actual service/browser frames checked',len(frames))
