import json,pathlib,hashlib,re,sys,datetime,tarfile,copy
sys.dont_write_bytecode=True
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_measurement_profiles';S=H/'execution/single_goal';A=pathlib.Path(__file__).parent;E=H/'evidence';C=S/'compatibility/current_001';T=S/'target/u07m_001'
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();err=[];m=load(T/'MANIFEST.json');c=load(S/'COVERAGE.json');p=load(S/'PREDECESSOR_IMPACT.json');sel=load(S/'EVIDENCE_SELECTION.json');logs={};summaries={}
def ref(b):
 assert sha(R/b['path'])==b['sha256'],b['path']
 if 'bytes' in b:assert (R/b['path']).stat().st_size==b['bytes']
def mapped(d):return {x['path']:x for x in d['files']}
proofs=[]
for name,changed in [('docs_delta_001',['game_server/config/observability/README.md']),('host_exit_correction_002',['game_server/src/main.rs'])]:
 d=load(E/name/'PROOF.json');b=mapped(d['source_before']);a=mapped(d['source_after']);assert b.keys()==a.keys();assert [k for k in b if b[k]!=a[k]]==changed;proofs.append(d)
 if name=='docs_delta_001':
  assert d['source_after']==m['product_source']
  for row in d['documents']:
   assert sha(R/row['preimage'])==row['before_sha256'];assert sha(R/row['postimage'])==sha(R/row['path'])==row['after_sha256'];assert (R/row['postimage']).read_bytes().startswith((R/row['preimage']).read_bytes())
assert proofs[1]['source_after']==proofs[0]['source_before']
for b in c['receipt_bindings']:
 name=b['name'];ref(b);r=load(R/b['path']);log=E/(name+'.log');logs[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',log.read_text(),re.M));counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',log.read_text());total={k:sum(int(x[i]) for x in counts) for i,k in enumerate(['passed','failed','ignored'])}
 assert sha(log)==r['log_sha256'];assert r['source_before']==r['source_after'] and r['source_unchanged'];assert r['source_before']['sha256']==b['actual_source_sha256'];assert r['exit_code']==b['exit_code']==(101 if name==sel['r05_receipt'] else 0)
 assert r['source_before'] in [m['product_source'],proofs[0]['source_before'],proofs[1]['source_before']]
 assert sha(E/'runner_versions'/(r['runner_sha256']+'.py'))==r['runner_sha256']
 for x in r['command_inputs']:assert sha(H/x['archive'])==x['sha256']
 if name in c['suite_counts']:assert total['passed']==c['suite_counts'][name] and total['failed']==0
 summaries[name]={'source':b['actual_source_sha256'],'currentness':b['currentness'],'command':r['command'],'toolchain':r['toolchain'],'environment':r['environment'],'exit':r['exit_code'],'counts':total,'actual_pass_selectors':len(logs[name])}
rows=[]
for group,rs in [('U07-M',c['validations']),('predecessor',p['conditions'])]:
 for x in rs:
  for test,names in x['selector_receipts'].items():assert names and all(test in logs.get(name,set()) for name in names),(group,x['id'],test)
  for name in x.get('direct_receipts',[]):assert name in summaries
  for b in x.get('normal_public_evidence',{}).values():ref(b)
  rows.append({'group':group,'id':x['id'],'selector_count':len(x['selector_receipts']),'direct_receipts':x.get('direct_receipts',[])})
freeze=load(H/'P2G_30_FREEZE_001.json')['artifacts'][0]
assert [x['Observable Policy Acceptance'] for x in freeze['payload']['Observable Policy Acceptance Set']]==[x['requirement'] for x in c['validations']]
for row in p['original_requirement_sources']:ref(row)
for b in sel['public_summaries'].values():ref(b)
ad=load(C/'ADAPTATION_FINAL.json');adapt=[]
for row in ad['immediate_unchanged_inputs']:assert sha(R/row['source'])==sha(R/row['copy'])==row['sha256'];adapt.append(row)
original=load(E/'gex00_001/HARNESS_INPUT_BEFORE.json')
with tarfile.open(E/'gex00_001/HARNESS_INPUT_BEFORE.tar.gz') as arc:
 for row in original:assert sha(R/row['path'])==hashlib.sha256(arc.extractfile(row['path']).read()).hexdigest()==row['sha256']
for b in ad['prepared_harness_proofs']:
 ref(b);d=load(R/b['path'])
 for row in d['rows']:
  assert sha(R/row.get('path',row.get('current')))==row.get('sha256',row.get('current_sha256'))
  if row.get('original'):assert sha(R/row['original'])==row['original_sha256']
for row in ad['executed_harness_bindings']:ref(row['executed_archive']);assert sha(pathlib.Path(row['script']))==row['current_sha256']
sys.path.insert(0,str(C));sys.path.insert(0,str(H/'tools'))
from compare_pc_cost import compare,controls,BEFORE
layout=json.loads(re.findall(r'U07_LAYOUT (\{[^\n]+\})',(E/'compat_pc_002.log').read_text())[0]);before=load(BEFORE);after=load(C/'evidence/pc_raw_current_002.json');pc=compare(before,after,layout);pc_controls=controls(before,after,layout,load(R/'core_enhanced/docs/goals/core_independent_objects/execution/single_goal/compatibility/current_001/evidence/pc_raw_current_001.json'))
def nontiming(v):
 v=copy.deepcopy(v)
 for row in v['rows']:
  for stage in ['setup','measurement']:
   if stage in row:row[stage].pop('elapsed_ns')
 return v
assert nontiming(after)==nontiming(load(R/'core_enhanced/docs/goals/core_independent_objects/execution/single_goal/compatibility/current_001/evidence/pc_raw_current_005.json'))
from r05_harness import verify
proof_path,proof=verify('002',m['product_source']['sha256'])
assert sha(R/'.cache/u07m-freeze-001/EXECUTION.json')=='1fa9c128a53b575bc43bf956ab468ba96cd00a0cab6bcc5e1f5bec300797b332'
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'receipts':summaries,'condition_bindings':rows,'immediate_inputs_verified':len(adapt),'historical_harness_files_verified':len(original),'pc':{k:v for k,v in pc.items() if k!='per_row_cost_changes'},'immediate_u07_pc_nontiming_exact':True,'pc_controls':pc_controls,'r05_proof_verified':str(proof_path.relative_to(R)),'errors':err,'scope':'Independent receipt/log/input/hash/actual-selector/currentness and strict raw PC comparison. Product tests not rerun; semantic source judgment separately recorded.'}
(A/'EVIDENCE_VERIFICATION.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'receipts':len(summaries),'conditions':len(rows),'adaptation':len(adapt),'historical':len(original),'pc':out['pc'],'negative_controls':len(pc_controls)-1,'errors':err}))
