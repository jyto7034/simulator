import json, pathlib, subprocess, time, urllib.request, urllib.parse, os, hashlib
ROOT=pathlib.Path.cwd(); D=ROOT/'core_enhanced/docs/goals/core_runtime_safety_observability'; S=D/'execution/s2'
OUT=pathlib.Path(os.environ['S2_SERVICE_OUT']);OUT.mkdir(parents=True,exist_ok=False)
def get(url, payload=None):
    request=urllib.request.Request(url,data=None if payload is None else json.dumps(payload).encode(),headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(request,timeout=5) as response:return response.read()
def save(name, url, payload=None):
    raw=get(url,payload);(OUT/name).write_bytes(raw);return json.loads(raw)
def query(name, expression):
    return save(name,'http://127.0.0.1:9090/api/v1/query?query='+urllib.parse.quote(expression))
driver=next(row for row in json.loads((S/'evidence/driver_binaries002.json').read_text()) if row['mode']=='on')
assert hashlib.sha256(pathlib.Path(driver['path']).read_bytes()).hexdigest()==driver['sha256']
command=[driver['path'],str(D/'evidence/public_profile_003/content'),str(D/'evidence/public_profile_003/selection.json'),str(S/'evidence/service_inputs/observability.ron'),'on','160','200']
deadline=time.monotonic()+25
while json.loads(get('http://127.0.0.1:3000/api/dashboards/uid/core-runtime'))['dashboard']['panels'][-1].get('pluginVersion')!='13.2.3':
    assert time.monotonic()<deadline
    time.sleep(.5)
log=(OUT/'driver.jsonl').open('xb'); p=subprocess.Popen(command,stdout=log,stderr=subprocess.STDOUT)
try:
    deadline=time.monotonic()+20
    while True:
        if p.poll() is not None:raise RuntimeError('driver exited early')
        try:
            state=json.loads(get('http://127.0.0.1:9110/state'))
            if len(state['sessions'])==2:break
        except OSError:pass
        assert time.monotonic()<deadline
        time.sleep(.2)
    time.sleep(2)
    save('health.json','http://127.0.0.1:3000/api/health')
    save('datasource.json','http://127.0.0.1:3000/api/datasources/uid/core-runtime')
    save('dashboard.json','http://127.0.0.1:3000/api/dashboards/uid/core-runtime')
    state=save('active.json','http://127.0.0.1:9110/state')
    assert len(state['sessions'])==2 and state['selected_session']==1
    metrics=get('http://127.0.0.1:9109/metrics')
    (OUT/'metrics.prom').write_bytes(metrics)
    lines=[line for line in metrics.decode().splitlines() if line and not line.startswith('#')]
    assert len(lines)==4197,len(lines)
    assert 'core_runtime_active_battles 2' in lines
    for reason in ['battle_runtime_work_limit','missing_battle_state_authority']:
        assert 'core_runtime_rejections_total{reason="'+reason+'"} 1' in lines
    assert 'core_runtime_usage{quantity="units",statistic="sum"} 2' in lines
    assert 'core_runtime_usage{quantity="units",statistic="max"} 1' in lines
    for name,q in {'usage':'core_runtime_usage','rejections':'core_runtime_rejections_total > 0','selected':'core_runtime_selected_usage','work':'core_runtime_work_sum > 0','active-count':'core_runtime_active_battles'}.items():
        result=query(name+'.json',q);assert result['status']=='success' and result['data']['result']
    result=subprocess.run(['node',str(S/'tools/browser.cjs'),str(OUT)],env={**os.environ,'PLAYWRIGHT_BROWSERS_PATH':'/tmp/crso-s2-browser/browsers'},capture_output=True,text=True)
    (OUT/'browser.log').write_text(result.stdout+result.stderr);assert result.returncode==0
    assert 'No data' not in (OUT/'simulation-time.txt').read_text()
    assert 'No data' not in (OUT/'duration-distribution.txt').read_text()
    assert not json.loads((OUT/'browser.json').read_text())['errors']
    save('selection2.json','http://127.0.0.1:9110/selection',{'session':2})
    time.sleep(1.5)
    switched=save('switched.json','http://127.0.0.1:9110/state');assert switched['selected_session']==2 and switched['generation']>state['generation']
    save('selection1.json','http://127.0.0.1:9110/selection',{'session':1})
    deadline=time.monotonic()+35
    while time.monotonic()<deadline:
        ended=save('latest.json','http://127.0.0.1:9110/state')
        if ended['selected_session'] is None and not any(s['current']['battle'] for s in ended['sessions']):break
        time.sleep(.2)
    else:raise RuntimeError('end cleanup timed out')
    (OUT/'ended.json').write_text(json.dumps(ended,indent=2))
    time.sleep(1.3)
    r=query('ended-active.json','core_runtime_active_battles');assert r['data']['result'][0]['value'][1]=='0'
    r=query('ended-selected.json','core_runtime_selected_active');assert r['data']['result'][0]['value'][1]=='0'
    assert p.wait(timeout=10)==0
    files=[{'path':str(f.relative_to(OUT)),'sha256':hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted(OUT.iterdir()) if f.is_file()]
    (OUT/'MANIFEST.json').write_text(json.dumps({'command':command,'exit_code':0,'files':files},indent=2))
    print(json.dumps({'output':str(OUT),'status':'pass'}))
finally:
    if p.poll() is None:
        p.terminate()
        try:p.wait(timeout=5)
        except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
    log.close()
