from pathlib import Path
import json,tarfile,tempfile,subprocess,shutil,hashlib,time
R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/core_dealer_minigames/execution/single_goal';A=E/'audit/independent001';work=Path(tempfile.mkdtemp(prefix='u163-independent-'))
# Select the highest coupling scenario: same-command IO retry, escrow identity,
# partial redemption, bundle and repeated purchase, duplicate leave, fresh Continue.
store=work/'store';store.mkdir()
with tarfile.open(E/'runs/host001/STORES.tar.gz') as t:
 for m in t:
  prefix='preimages/trade_complete/'
  if m.isfile() and m.name.startswith(prefix):
   dest=store/m.name[len(prefix):];dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(t.extractfile(m).read())
assert (store/'active').is_dir() and (store/'profile').is_dir()
request=json.loads((E/'test_parts/host_replays001/trade_complete.json').read_bytes());req=work/'request.json';req.write_text(json.dumps(request))
binary=E/'artifacts/normal002/u15_economy_host';config=E/'test_parts/normal_inputs001/admission.json';receipts=[]
for label,inp in [('trade',req),('continue',work/'continue.json')]:
 if label=='continue':inp.write_text(json.dumps({'commands':[{'type':'request_relic_state'}]}))
 out=A/label;cmd=[str(binary),str(config),str(store/'active'),str(store/'profile'),str(out),str(inp)];start=time.monotonic()
 with (A/(label+'.stdout')).open('xb') as stdout,(A/(label+'.stderr')).open('xb') as stderr:p=subprocess.run(cmd,cwd=R,stdout=stdout,stderr=stderr)
 receipts.append({'argv':cmd,'exit_code':p.returncode,'seconds':time.monotonic()-start});assert p.returncode==0
trade=json.loads((A/'trade/RESULT.json').read_bytes());cont=json.loads((A/'continue/RESULT.json').read_bytes());trace=json.loads((A/'trade/commands.json').read_bytes());old=json.loads((E/'runs/host001/trade_complete/commands.json').read_bytes());io=json.loads((A/'trade/IO_FAILURE.json').read_bytes())
assert io['memory_and_disk_unchanged'] and 'os error 21' in io['error'];assert trace==old
copies=trace[-1]['result']['payload']['copies'];original=json.loads((E/'runs/host001/initial_checkpoint/commands.json').read_bytes())[-1]['result']['payload']['copies'];pledged=request['commands'][8]['action']['copies']
assert len(copies)==6 and next(c for c in copies if c['copy_uuid']==pledged[0])==next(c for c in original if c['copy_uuid']==pledged[0]);assert not any(c['copy_uuid']==pledged[1] for c in copies)
assert trace[6]['result']['payload']['dealer']['points']==0
assert trace[9]['result']['payload']['dealer']['points']==20
assert trace[12]['result']['payload']['dealer']['points']==10
assert trace[15]['result']['payload']['dealer']['points']==1
assert trace[17]['rejected'];assert cont['run']==trade['run'] and cont['inventory']==trade['inventory']
result={'passed':True,'work_root':str(work),'fresh_processes':2,'commands':20,'actual_io_failure_retries':1,'raw_trace_equal_to_fixed_original':True,'independent_oracles':['30-10*3=0','two10principal=20','one10redemption=10','bundle5+2+2 leaves1','original Corrupt UUID/definition retained; other collateral absent;6 copies','fresh Continue exact run/inventory'],'receipts':receipts}
(A/'REPLAY_RESULT.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
