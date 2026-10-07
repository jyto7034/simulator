import json,pathlib,hashlib,re,sys,importlib.util,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_independent_objects';S=H/'execution/single_goal';A=pathlib.Path(__file__).parent;E=H/'evidence';C=S/'compatibility/current_001';T=S/'target/u07_001'
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();err=[];m=load(T/'MANIFEST.json');c=load(S/'COVERAGE.json');p=load(S/'PREDECESSOR_IMPACT.json');sel=load(S/'EVIDENCE_SELECTION.json');logs={};summaries={}
for name,b in c['receipt_bindings'].items():
 r=load(E/(name+'.json'));log=E/(name+'.log');logs[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',log.read_text(),re.M));counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',log.read_text());total={k:sum(int(x[i]) for x in counts) for i,k in enumerate(['passed','failed','ignored'])}
 checks={'receipt_hash':sha(E/(name+'.json'))==b['receipt_sha256'],'log_hash':sha(log)==b['log_sha256']==r['log_sha256'],'source':r['source_before']==r['source_after']==m['product_source'],'unchanged':r['source_unchanged'],'exit':r['exit_code']==b['raw_exit_code'] and r['exit_code']==(101 if name==sel['r05_receipt'] else 0),'runner_archive':sha(E/'runner_versions'/(r['runner_sha256']+'.py'))==r['runner_sha256']}
 for x in r['command_inputs']:
  checks['command_archive:'+x['path']]=sha(H/x['archive'])==x['sha256']
 if not all(checks.values()):err.append([name,checks])
 if name in c['suite_counts'] and total!=c['suite_counts'][name]:err.append([name,'counts',total])
 summaries[name]={'checks':checks,'command':r['command'],'toolchain':r['toolchain'],'environment':r['environment'],'exit':r['exit_code'],'counts':total,'actual_pass_selectors':len(logs[name])}
rows=[]
for group,rs in [('U07',c['validations']),('predecessor',p['conditions'])]:
 for x in rs:
  missing=[]
  for test,names in x['selector_receipts'].items():
   if not names or any(test not in logs.get(name,set()) for name in names):missing.append(test)
  for name in x['direct_receipts']:
   if name not in summaries:missing.append('direct:'+name)
  if missing:err.append([group,x['id'],missing])
  rows.append({'group':group,'id':x['id'],'selector_count':len(x['selector_receipts']),'direct_receipts':x['direct_receipts'],'missing':missing})
freeze=load(H/'P2G_30_FREEZE_001.json')['artifacts'][0]
if [x['Observable Policy Acceptance'] for x in freeze['payload']['Observable Policy Acceptance Set']] != [x['requirement'] for x in c['validations']]:err.append('freeze meanings')
for row in p['original_requirement_sources']:
 if sha(R/row['path'])!=row['sha256']:err.append('original requirement:'+row['path'])
# All adaptation current copies and original bytes remain exact.
ad=load(C/'ADAPTATION_FINAL_002.json');adapt=[]
for row in ad['files']:
 good=sha(R/row['copy'])==row.get('current_copy_sha256',row['copy_sha256'])
 if not good:err.append(['adaptation',row])
 adapt.append(good)
sys.path.insert(0,str(C));sys.path.insert(0,str(H/'tools'))
from compare_pc_cost_u07 import compare,controls,BEFORE
layout=json.loads(re.findall(r'U07_LAYOUT (\{[^\n]+\})',(E/'compat_pc_004.log').read_text())[0]);before=load(BEFORE);after=load(C/'evidence/pc_raw_current_004.json');pc=compare(before,after,layout);pc_controls=controls(before,after,layout,load(C/'evidence/pc_raw_current_001.json'))
from r05_harness import verify
proof_path,proof=verify('004',m['product_source']['sha256'])
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'receipts':summaries,'condition_bindings':rows,'adaptation_files_verified':len(adapt),'pc':{k:v for k,v in pc.items() if k!='per_row_cost_changes'},'pc_controls':pc_controls,'r05_proof_verified':str(proof_path.relative_to(R)),'errors':err,'scope':'Independent receipt/log/input/hash/actual-selector/currentness and strict raw PC comparison. Product tests not rerun; semantic source judgment separately recorded.'}
(A/'EVIDENCE_VERIFICATION.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'receipts':len(summaries),'conditions':len(rows),'adaptation':len(adapt),'pc':out['pc'],'negative_controls':len(pc_controls)-1,'errors':err}))
