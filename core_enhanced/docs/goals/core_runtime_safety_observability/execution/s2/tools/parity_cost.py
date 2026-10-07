"""Paired actual public gameplay, with raw JSONL/timings; no new cost gate."""
from pathlib import Path
import hashlib,json,os,statistics,subprocess,sys,time,urllib.request,socket
R=Path.cwd();D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';S=D/'execution/s2'
OUT=Path(os.environ['S2_PARITY_OUT']);OUT.mkdir(parents=True,exist_ok=False)
binaries=json.loads((S/'evidence/driver_binaries002.json').read_text());bins={b['mode']:b['path'] for b in binaries}
for b in binaries:assert hashlib.sha256(Path(b['path']).read_bytes()).hexdigest()==b['sha256']
# Scrapes stopped: all owned service processes have been stopped by the caller.
for port in [9090,3000]:
    with socket.socket() as probe:
        probe.settimeout(.5)
        assert probe.connect_ex(('127.0.0.1',port))!=0,port
normal=S/'evidence/service_inputs/observability.ron';failed=OUT/'bind_failure.ron';failed.write_text(normal.read_text().replace('127.0.0.1:9109','127.0.0.1:19109'))
rows=[];reference=None
for sample in range(5):
    modes=['off','none','detached','queue_full','on','stalled','exporter_failed']
    if sample%2:modes.reverse()
    for mode in modes:
        reserved=None
        if mode=='exporter_failed':
            reserved=socket.socket();reserved.bind(('127.0.0.1',19109));reserved.listen(1)
        cmd=[bins['off' if mode=='off' else 'on'],str(D/'evidence/public_profile_003/content'),str(D/'evidence/public_profile_003/selection.json'),str(failed if reserved else normal),'on' if reserved else mode,'1100','0']
        raw=OUT/f'{sample}_{mode}.jsonl';started=time.monotonic()
        with raw.open('xb') as log:p=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT)
        elapsed=time.monotonic()-started
        if reserved:reserved.close()
        assert p.returncode==0,(mode,raw)
        payloads=[];timings={};observation=None
        for line in raw.read_text().splitlines():
            v=json.loads(line)
            if 'observation' in v:observation=v['observation'];continue
            payloads.append({'step':v['step'],'result':v['result']})
            category='normal_tick' if v['step'].startswith('tick_') else v['step']
            timings.setdefault(category,[]).append(v['operation_wall_ns'])
        identity=hashlib.sha256(json.dumps(payloads,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        if reference is None:reference=identity
        assert identity==reference,(mode,identity,reference)
        assert len(timings['normal_tick'])==1100
        if mode=='queue_full':assert observation['queue_dropped']>0,observation
        if mode=='exporter_failed':assert observation['status']=='failed',observation
        rows.append({'sample':sample,'mode':mode,'command':cmd,'exit_code':p.returncode,'elapsed_seconds':elapsed,'gameplay_sha256':identity,'raw_sha256':hashlib.sha256(raw.read_bytes()).hexdigest(),'timings_ns':timings,'observation':observation})
        print(json.dumps({'sample':sample,'mode':mode,'gameplay_sha256':identity,'elapsed':elapsed}),flush=True)
summary={}
for mode in ['off','none','detached','queue_full','on','stalled','exporter_failed']:
    summary[mode]={}
    for category in ['entry','normal_tick','concentrated_tick','pause_battle','resume_battle','authority_rejection','native_rejection','end_battle']:
        values=[v for row in rows if row['mode']==mode for v in row['timings_ns'][category]]
        summary[mode][category]={'n':len(values),'median_ns':statistics.median(values),'p95_ns':sorted(values)[int((len(values)-1)*.95)],'max_ns':max(values)}
(OUT/'RESULT.json').write_text(json.dumps({'binaries':binaries,'gameplay_sha256':reference,'rows':rows,'summary':summary,'cost_gate':None,'scrapes':'stopped, ports9090/3000 closed'},indent=2))
print(json.dumps(summary))
