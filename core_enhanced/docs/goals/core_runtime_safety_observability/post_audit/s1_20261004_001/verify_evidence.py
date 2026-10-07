import sys,json,hashlib,re,tarfile,tempfile,subprocess
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_safety_observability';T=H/'execution/s1/target/s1_001';A=Path(__file__).parent;E=H/'evidence'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
m=json.loads((T/'MANIFEST.json').read_text());c=json.loads((H/'execution/s1/COVERAGE.json').read_text());impact=json.loads((H/'execution/s1/PREDECESSOR_IMPACT.json').read_text())
passed={};summary={}
for name in ['all_default_002','all_feature_002','server_all_targets_001']:
 d=json.loads((E/(name+'.json')).read_text());log=E/(name+'.log');assert d['source_before']==d['source_after']==m['product_source'];assert d['exit_code']==0 and d['source_unchanged'];assert d['log_sha256']==sha(log)
 raw=log.read_text();passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',raw,re.M)); rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw);summary[name]={k:sum(int(r[i]) for r in rows) for i,k in enumerate(['passed','failed','ignored'])};assert summary[name]['passed']>0 and summary[name]['failed']==summary[name]['ignored']==0
core=passed['all_default_002']&passed['all_feature_002'];host=passed['server_all_targets_001']
for v in c['validations']:assert set(v['core_selectors'])<=core and set(v['host_selectors'])<=host
for cond in impact['inherited_conditions']:assert set(cond['selectors'])<=core
assert set(impact['f01_selectors'])<=core
bindings=[]
for b in c['receipt_bindings']:
 name=b['name'];p=E/(name+'.json');d=json.loads(p.read_text());assert sha(p)==b['receipt_sha256'];assert d['exit_code']==0 and d['source_unchanged'];assert d['log_sha256']==sha(E/(name+'.log'));now={r['path']:r for r in m['product_source']['files']};old={r['path']:r for r in d['source_after']['files']};assert all(now.get(p)==v for p,v in old.items());assert sorted(set(now)-set(old))==b['additional_test_only_inputs'];bindings.append(dict(name=name,reused=True,excluded_test_only_inputs=b['additional_test_only_inputs'],command=d['command']))
def actual(root):
 return [dict(path=str(p.relative_to(root)),bytes=p.stat().st_size,sha256=sha(p)) for p in sorted(root.rglob('*'), key=lambda p:str(p.relative_to(root))) if p.is_file()]
with tempfile.TemporaryDirectory(prefix='s1-audit-restore-') as tmp:
 b=Path(tmp)/'source';b.mkdir()
 with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as tf:tf.extractall(b,filter='data')
 assert actual(b)==m['before_files']
 for reverse,key in [(False,'source_files'),(True,'before_files')]:
  p=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=b,capture_output=True)
  (A/('restore_reverse.log' if reverse else 'restore_forward.log')).write_bytes(p.stdout+p.stderr);assert p.returncode==0;assert actual(b)==m[key]
 for group,key in [('SOURCE_AFTER','source_files'),('RECORDS',None),('REFERENCES',None)]:
  dst=Path(tmp)/group;dst.mkdir()
  with tarfile.open(T/(group+'.tar.gz')) as tf:tf.extractall(dst,filter='data')
  expected=m[key] if key else json.loads((T/(group+'.json')).read_text());assert actual(dst)==expected
f=json.loads((H/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]; p=json.loads((H/'P2G_40_PHASE_PLAN_001.json').read_text())['artifacts'][0];r=json.loads((H/'P2G_40_RECEIPT_001.json').read_text());ha=json.loads((H/'P2G_60_HANDOFF_001.json').read_text())['artifacts'][0]
assert p['payload']['Bound Final Policy Freeze Ref']==f['ref'];assert r['evaluated_phase_plan']['sha256']==sha(H/'P2G_40_PHASE_PLAN_001.json');assert r['bound_final_policy_freeze']['sha256']==sha(H/'P2G_30_FREEZE_001.json');assert ha['payload']['Result Composite Basis']['Final Policy Freeze Ref']==f['ref'];assert ha['payload']['Result Composite Basis']['Phase Plan Ref']==p['ref']
report=dict(full_suites=summary,all_v_selectors_present=True,inherited_conditions_with_current_nonzero_selectors=len(impact['inherited_conditions']),f01_selectors=len(impact['f01_selectors']),receipts=bindings,restore=dict(forward=True,reverse=True,fuzz=0,source_after_archive=True,records_archive=True,references_archive=True),upstream_refs=[f['ref'],p['ref'],ha['ref']])
(A/'EVIDENCE_REUSE.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['receipts','upstream_refs']}))
