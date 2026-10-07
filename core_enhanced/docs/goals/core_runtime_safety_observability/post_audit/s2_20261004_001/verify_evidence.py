import pathlib,json,hashlib,re,math,statistics
R=pathlib.Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';S=D/'execution/s2';A=pathlib.Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def write(n,v):(A/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
c=load(S/'COVERAGE.json');m=load(S/'target/s2_001/MANIFEST.json');cur={x['path']:x for x in m['product_source']['files']}; receipts={};logs={};results={}
for name,binding in c['receipt_bindings'].items():
 p=D/'evidence'/f'{name}.json';r=load(p);log=D/'evidence'/f'{name}.log';receipts[name]=r;logs[name]=log.read_text()
 assert sha(p)==binding['receipt_sha256'];assert sha(log)==binding['log_sha256']==r['log_sha256'];assert r['exit_code']==binding['raw_exit_code'];assert r['source_before']==r['source_after'];assert r['source_unchanged'];assert r['source_before']['sha256']==binding['source_sha256']
 old={x['path']:x for x in r['source_after']['files']};delta=[p for p in sorted(set(old)|set(cur)) if old.get(p)!=cur.get(p)]
 assert delta==binding['reuse_test_only_delta'],(name,delta)
 for x in r['command_inputs']:assert sha(D/x['archive'])==x['sha256']
 results[name]={'raw_exit':r['exit_code'],'source_sha256':r['source_after']['sha256'],'current_delta':delta,'command':r['command'],'log_sha256':sha(log),'input_archives_checked':len(r['command_inputs'])}
suites={}
for name,s in c['suites'].items():
 totals=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',logs[s['receipt']])]
 actual=[sum(x[i] for x in totals) for i in range(3)]
 assert actual==[s['passed'],s['failed'],s['ignored']],(name,actual,s)
 suites[name]={'passed':actual[0],'failed':actual[1],'ignored':actual[2],'log_totals':totals}
selector_rows=[]
for row in c['validations']:
 checked=[]
 for typ in ['core','host']:
  for selector in row.get(typ+'_selectors',[]):
   valid=[k for k,v in logs.items() if k.startswith('s2_final_'+typ) and f'test {selector} ... ok' in v]
   assert valid,(row['id'],selector)
   if 'runtime_observation' not in selector and not selector.startswith('observability::'): assert any('default' in k for k in valid),(selector,valid)
   checked.append({'selector':selector,'receipts':valid})
 selector_rows.append({'id':row['id'],'verified_selectors':checked})
p=load(S/'PREDECESSOR_IMPACT.json');prior=load(D/'post_audit/s1_20261004_003/PREDECESSOR_REVIEW.json');priorrows={(x['group'],x['id']):x for x in prior['conditions']}; inherited=[]
for row in p['inherited_conditions']:
 group=row.get('group','U01');old=priorrows[(group,row['id'])];checks=[]
 assert set(row['selectors'])==set(old['selectors'])
 for selector in row['selectors']:
  valid=[k for k in row['receipts'] if f'test {selector} ... ok' in logs[k]];assert len(valid)==len(row['receipts']),(row['id'],selector,valid)
  checks.append({'selector':selector,'receipts':valid})
 inherited.append({'group':group,'id':row['id'],'requirement':old['requirement'],'verified_current_selectors':checks})
assert len(inherited)==88
for selector in p['f01_selectors']:
 for suite in ['s2_final_core_default001','s2_final_core_fixtures001','s2_final_core_observation001']:assert f'test {selector} ... ok' in logs[suite]
heap=re.search(r'S2_HEAP (\{.*\})',logs['s2_final_heap001']);assert heap;heap=loadstr=json.loads(heap.group(1));assert heap['held_start']==heap['held_end'];assert heap['samples']==1024512;assert heap['idle_live']<=heap['baseline_live']+heap['deferred_bound']
write('EVIDENCE_VERIFICATION.json',{'receipt_count':len(results),'receipts':results,'suites':suites,'validations':selector_rows,'inherited_conditions':inherited,'f01_selectors':p['f01_selectors'],'heap':heap})
# Recompute actual release gameplay payload identity and costs from each raw line.
cost=load(S/'evidence/parity_cost002/RESULT.json'); rows=[]
for row in cost['rows']:
 raw=S/f'evidence/parity_cost002/{row["sample"]}_{row["mode"]}.jsonl';assert sha(raw)==row['raw_sha256'];values=[json.loads(line) for line in raw.read_text().splitlines()];payload=[{'step':x['step'],'result':x['result']} for x in values if 'step' in x]
 identity=hashlib.sha256(json.dumps(payload,sort_keys=True,separators=(',',':')).encode()).hexdigest();assert identity==row['gameplay_sha256']==cost['gameplay_sha256'];times={}
 for x in values:
  if 'step' in x:times.setdefault('normal_tick' if x['step'].startswith('tick_') else x['step'],[]).append(x['operation_wall_ns'])
 assert times==row['timings_ns'];assert len(times['normal_tick'])==1100;assert row['exit_code']==0
 observation=next((x['observation'] for x in values if 'observation' in x),None);assert observation==row['observation']
 if row['mode']=='queue_full':assert observation['queue_dropped']==84
 if row['mode']=='exporter_failed':assert observation['status']=='failed' and 'Address already in use' in observation['error']
 rows.append({'sample':row['sample'],'mode':row['mode'],'gameplay_sha256':identity,'normal_ticks':len(times['normal_tick']),'raw_sha256':sha(raw)})
assert len(rows)==35
for mode,categories in cost['summary'].items():
 for category,value in categories.items():
  v=[v for row in cost['rows'] if row['mode']==mode for v in row['timings_ns'][category]]
  assert value=={'n':len(v),'median_ns':statistics.median(v),'p95_ns':sorted(v)[int((len(v)-1)*.95)],'max_ns':max(v)}
for b in cost['binaries']:assert sha(pathlib.Path(b['path']))==b['sha256']
write('PARITY_VERIFICATION.json',{'rows':rows,'summary':cost['summary'],'binary_hashes_current':True,'all_35_gameplay_payloads_exact':True,'limitations':'No new latency gate. These are paired local release executions. Later product delta is solely cfg(test) amount_order.rs.'})
# Actual service raw fields and finite samples, independent of report pass booleans.
services=S/'evidence/services';frames=[]
for folder in ['live006','render007']:
 b=load(services/folder/'browser.json');assert not b['errors']
 for resp in b['responses']:
  assert resp['status']==200
  for result in resp['body'].get('results',{}).values():
   assert not result.get('error');series=result.get('frames',[]);expr=next((f['schema'].get('meta',{}).get('executedQueryString') for f in series if f['schema'].get('meta',{}).get('executedQueryString')),'')
   for frame in series:
    v=[v for col in frame.get('data',{}).get('values',[])[1:] for v in col if isinstance(v,(int,float)) and math.isfinite(v)];frames.append({'source':'evidence/services/'+folder,'expression':expr,'finite_samples':len(v),'positive_samples':sum(x>0 for x in v)})
assert frames==load(services/'SERVICE_VERIFICATION.json')['frames'];assert len(frames)==251
for token in ['core_runtime_operation_seconds_bucket','core_runtime_selected_usage{slot="0",quantity="simulation_time_ms"','core_runtime_selected_usage{slot="0",quantity="accounted_extent_bytes"']:assert any(token in f['expression'] and f['positive_samples'] for f in frames)
live=services/'live006';met=(live/'metrics.prom').read_text().splitlines();samples=[x for x in met if x and not x.startswith('#')];assert len(samples)==4197
for x in load(live/'MANIFEST.json')['files']:assert sha(live/x['path'])==x['sha256']
def vector(n):return load(live/n)['data']['result']
assert float(vector('active-count.json')[0]['value'][1])==2
assert float(vector('ended-active.json')[0]['value'][1])==0
assert float(vector('ended-selected.json')[0]['value'][1])==0
reasons={x['metric']['reason']:float(x['value'][1]) for x in vector('rejections.json')};assert reasons=={'battle_runtime_work_limit':1,'missing_battle_state_authority':1}
end=load(live/'ended.json');assert end['selected_session'] is None and not any(x['current']['battle'] for x in end['sessions']);assert end['ring'][-1]['boundary']=='ended'
selection=[load(live/n)['selected_session'] for n in ['active.json','switched.json','selection1.json']];assert selection==[1,2,1]
write('SERVICE_VERIFICATION.json',{'frames':frames,'metric_sample_lines':4197,'reasons':reasons,'selection':selection,'termination_clears_active_selected':True,'screenshots_reviewed':['render007/duration-distribution.png','render007/simulation-time.png'],'scope':'Captured actual standard local services, not live current endpoint. Services stopped. live006 misleading duration filename is native panel12; actual wall-duration evidence is render007 panel11.'})
print('Verified 34 receipt bindings, five suites, 88 inherited rows, 35 release parities, 251 browser frames, 4197 samples.')
