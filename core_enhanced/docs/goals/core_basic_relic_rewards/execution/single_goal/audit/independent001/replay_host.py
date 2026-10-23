"""Replay archived supported preimages on fixed host; write only audit output and owned /tmp stores."""
from pathlib import Path
import json,tarfile,tempfile,subprocess,time,hashlib
A=Path(__file__).resolve().parent; E=A.parent.parent; R=next(p for p in E.parents if (p/'game_server/Cargo.toml').is_file())
load=lambda p:json.loads(p.read_bytes())
root=Path(tempfile.mkdtemp(prefix='u162-independent-host-')); host=E/'artifacts/normal002/u15_economy_host'; config=E/'test_parts/normal_inputs007/admission.json'
rows=[]
(A/'host002').mkdir()
with tarfile.open(E/'runs/host001/STORES.tar.gz') as tar:
 for name in ['choice_complete','dust_complete002','fusion_complete','choice_checkpoint','dust_skip']:
  store=root/name; prefix='preimages/'+name+'/'
  for m in tar:
   if m.isfile() and m.name.startswith(prefix):
    rel=Path(m.name[len(prefix):]); assert not rel.is_absolute() and '..' not in rel.parts
    p=store/rel;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(tar.extractfile(m).read())
  tar.members # iteration can be restarted on seekable tar
  assert (store/'active/active_run.json').is_file(),name
  request=E/'test_parts/host_replays001'/f'{name}.json'; out=A/'host002'/name
  argv=[str(x) for x in [host,config,store/'active',store/'profile',out,request]]
  start=time.monotonic()
  with (A/f'host002-{name}.stdout.log').open('xb') as stdout,(A/f'host002-{name}.stderr.log').open('xb') as stderr:
   p=subprocess.run(argv,cwd=R,stdout=stdout,stderr=stderr,timeout=60)
  row={'name':name,'argv':argv,'exit_code':p.returncode,'seconds':time.monotonic()-start};rows.append(row)
  (A/'HOST_PROGRESS.json').write_text(json.dumps(rows,indent=2))
  assert p.returncode==0,row
  result=load(out/'RESULT.json'); trace=load(out/'commands.json'); req=load(request)
  assert result['passed'] and result['fresh_process'] and len(trace)==len(req['commands'])
  assert [x['command'] for x in trace]==req['commands']
  assert {x['index'] for x in trace if x.get('rejected')}==set(req.get('reject_indices',[]))
  failure=load(out/'IO_FAILURE.json');assert failure['memory_and_disk_unchanged'] and 'os error 21' in failure['error']
  assert failure['index']==req['fail_index'] and trace[failure['index']]['command']==failure['command'] and 'result' in trace[failure['index']]
  # Compare deterministic domain observables, not timing or temporary store paths.
  original=load(E/'runs/host001'/name/'RESULT.json')
  for key in ['inventory','run']:assert result[key]==original[key],(name,key)
  row.update({'commands':len(trace),'actual_io_unchanged_and_retry':True,'exact_original_domain_result':True})
result={'passed':True,'host_sha256':hashlib.sha256(host.read_bytes()).hexdigest(),'store_root':str(root),'fresh_processes':len(rows),'commands':sum(r['commands'] for r in rows),'actual_io_failures_and_retries':len(rows),'rows':rows}
(A/'HOST_REPLAY.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='rows'}),flush=True)
