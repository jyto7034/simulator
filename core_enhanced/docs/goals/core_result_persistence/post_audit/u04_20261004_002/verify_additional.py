import pathlib,json,hashlib,re,math
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_result_persistence';S=H/'execution/single_goal';A=pathlib.Path(__file__).parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
p=json.loads((S/'PREDECESSOR_IMPACT.json').read_text());c=json.loads((S/'COVERAGE_003.json').read_text());e=json.loads((A/'EVIDENCE_BINDINGS.json').read_text())
selectors={r['selector'] for r in e['selectors']}
def walk(x,k=''):
 if isinstance(x,dict):
  for key,value in x.items():walk(value,key)
 elif isinstance(x,list):
  for v in x:walk(v,k)
 elif isinstance(x,str) and 'selector' in k and '::' in x:selectors.add(x)
walk(p)
renamed=p['u04_renamed_selectors'];retired={r['predecessor_selector'] for r in p['u04_changed_host_selector_dispositions']}
logs={n:(H/'evidence'/f'{n}.log').read_text(errors='replace') for n in c['receipt_bindings']}
rows=[]
for original in sorted(selectors):
 name=renamed.get(original,original);hits=[n for n,l in logs.items() if re.search(r'^test '+re.escape(name)+r' \.\.\. ok$',l,re.M)]
 disposition='renamed same validation for V10' if name!=original else 'current exact pass'
 if original in retired:disposition='superseded independent-write semantics by approved U04 paired-intent semantics'
 assert hits or original in retired,original
 rows.append({'selector':original,'current_selector':name,'passing_receipts':hits,'disposition':disposition})
service=json.loads((S/'SERVICE_REUSE.json').read_text());checks=[]
for row in service['unchanged_service_inputs']:
 assert sha(R/row['path'])==row['sha256'];checks.append(row)
for k in ['reused_audit','reused_service_evidence','reused_master_source_review']:
 assert sha(R/service[k])==service[k+'_sha256']
alloc=json.loads((H/'evidence/public_guards_allocator002_raw.json').read_text())['transaction_staging'];low=alloc['low_allocation_raw_samples'];high=alloc['high_allocation_raw_samples'];assert len(low)==len(high)==5 and low==high and all(x==low[0] for x in low)
mode_values={}
for mode in ['none','detached','queue_full','on','off']:
 js=[json.loads(l) for l in logs['final_fault_004_'+mode].splitlines() if l.startswith('{')]
 result=next(x for x in js if 'result' in x);mode_values[mode]=result['result']
assert all(x==mode_values['none'] for x in mode_values.values())
raw=[]
for folder,count in [('public_continuation_004',78),('public_rewind_004',114)]:
 messages=json.loads((H/'evidence'/folder/'messages.json').read_text());assert len(messages)==count
 states=[m['state'] for m in messages if m.get('type')=='state_snapshot'];errors=[m for m in messages if m.get('type')=='error'];assert states[-1]['active_run_continue']['available']==False
 raw.append({'folder':folder,'messages':len(messages),'state_snapshots':len(states),'errors':errors,'final_continue':states[-1]['active_run_continue']})
(A/'ADDITIONAL_BINDINGS.json').write_text(json.dumps({'selectors':rows,'selector_count':len(rows),'service_inputs_exact':checks,'service_reuse_artifacts_exact':True,'allocator_low_high_equal':True,'allocation_sample':low[0],'fault_modes_equal':list(mode_values),'public_raw_checks':raw},indent=2)+'\n')
print('additional selectors',len(rows),'service',len(checks),'allocator/fault/raw checks exact')
