import sys,json,hashlib,re,tarfile,tempfile,subprocess
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_safety_observability';C=H/'execution/s1/corrections/f02_persistence_entry_001';T=C/'target/s1_f02_001';A=Path(__file__).parent;E=H/'evidence'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
m=json.loads((T/'MANIFEST.json').read_text());c=json.loads((C/'COVERAGE.json').read_text());impact=json.loads((C/'PREDECESSOR_IMPACT.json').read_text());old=json.loads((H/'execution/s1/corrections/f01_active_battle_001/target/s1_f01_001/MANIFEST.json').read_text())
assert m['before_files']==old['source_files'];assert m['changed']==c['correction']['changed'];assert not m['added'] and not m['deleted']
passed={};summary={};receipts=[]
for binding in c['receipt_bindings']:
 name=binding['name'];p=E/(name+'.json');d=json.loads(p.read_text());assert sha(p)==binding['receipt_sha256'];log=E/(name+'.log');assert d['source_before']==d['source_after']==m['product_source'],name;assert d['exit_code']==binding['exit_code'] and d['source_unchanged'];assert d['log_sha256']==sha(log);receipts.append(dict(name=name,command=d['command'],sha256=sha(p),exit_code=d['exit_code']))
 if name in c['suites']:
  raw=log.read_text();passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',raw,re.M));rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw);summary[name]=[sum(int(r[i]) for r in rows) for i in range(3)];assert summary[name]==c['suites'][name]
core=passed['s1f02_all_default_001']&passed['s1f02_all_feature_001'];host=passed['s1f02_server_002']
for v in c['validations']:assert set(v['core_selectors'])<=core and set(v['host_selectors'])<=host
for cond in impact['inherited_conditions']:
 assert set(cond['selectors'])<=core
 assert cond['selectors'] or cond['id'] in ['X09','X10']
assert set(impact['f01_selectors'])<=core
assert len(c['acceptance'])==7 and len(c['validations'])==9 and len(c['required_dimensions'])==10

def actual(root):
 return [dict(path=str(p.relative_to(root)),bytes=p.stat().st_size,sha256=sha(p)) for p in sorted(root.rglob('*'),key=lambda p:str(p.relative_to(root))) if p.is_file()]
with tempfile.TemporaryDirectory(prefix='s1f02-audit-restore-') as tmp:
 b=Path(tmp)/'source';b.mkdir()
 with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as tf:tf.extractall(b,filter='data')
 assert actual(b)==m['before_files']
 def restore_function(s):
  start=s.index('    pub fn restore_persistence_rollback(');end=s.index('\n    pub fn ',start+1);return s[start:end]
 assert restore_function((b/'core_enhanced/src/application.rs').read_text())==restore_function((R/'core_enhanced/src/application.rs').read_text())
 for reverse,key in [(False,'source_files'),(True,'before_files')]:
  p=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=b,capture_output=True);(A/('restore_reverse.log' if reverse else 'restore_forward.log')).write_bytes(p.stdout+p.stderr);assert p.returncode==0;assert actual(b)==m[key]
 for group,key in [('SOURCE_AFTER','source_files'),('RECORDS',None),('REFERENCES',None)]:
  dst=Path(tmp)/group;dst.mkdir()
  with tarfile.open(T/(group+'.tar.gz')) as tf:tf.extractall(dst,filter='data')
  expected=m[key] if key else json.loads((T/(group+'.json')).read_text());assert actual(dst)==expected
# Input hashes include exact compiler-selected release libraries and external pack.
pdir=C/'public_probe_001';pr=json.loads((pdir/'RESULT.json').read_text())
for row in pr['input_files']:assert sha(R/row['path'])==row['sha256'],row['path']
for row in pr['output_files']:assert sha(pdir/row['path'])==row['sha256']
libs={}
for line in (pdir/'build.jsonl').read_text().splitlines():
 item=json.loads(line)
 if item.get('reason')=='compiler-artifact' and item['target']['name'] in ['game_core','game_server','actix','serde_json','uuid']:
  for filename in item['filenames']:
   if filename.endswith('.rlib'):libs[item['target']['name']]=Path(filename)
assert set(libs)=={'game_core','game_server','actix','serde_json','uuid'}
assert all(str(p) in ' '.join(pr['command']) for p in libs.values())
(A/'CURRENT_LIBRARIES.json').write_text(json.dumps({name:dict(path=str(p),sha256=sha(p)) for name,p in libs.items()},indent=2)+'\n')
# Recompute immediate cost parity with only time excluded.
pc=json.loads((C/'evidence/PC_IMMEDIATE_PARITY.json').read_text());before=H/'execution/s1/evidence/pc_raw_current_002.json';after=C/'evidence/pc_raw_current_002.json'
if not before.exists():
 before=next(p for p in H.rglob('pc_raw_current_002.json') if p!=after and sha(p)==pc['before_sha256'])
assert sha(before)==pc['before_sha256'] and sha(after)==pc['after_sha256']
a=json.loads(before.read_text());b=json.loads(after.read_text())
def strip_time(x):
 if isinstance(x,list):return [strip_time(i) for i in x]
 if isinstance(x,dict):return {k:strip_time(v) for k,v in x.items() if k!='elapsed_ns'}
 return x
assert strip_time(a)==strip_time(b)
report=dict(target_manifest_sha256=sha(T/'MANIFEST.json'),full_suites=summary,all_v_selectors_present=True,inherited_conditions=len(impact['inherited_conditions']),inherited_conditions_with_current_nonzero_selectors=sum(bool(x['selectors']) for x in impact['inherited_conditions']),non_selector_rows=['X09: full checks/public/cost evidence','X10: exact archive restoration and currentness'],f01_selectors=len(impact['f01_selectors']),receipts=receipts,restore=dict(forward=True,reverse=True,fuzz=0,source_after_archive=True,records_archive=True,references_archive=True),exact_delta=dict(paths=m['changed'],f01_restore_byte_identical=True),immediate_cost_parity_excluding_elapsed_ns=True)
(A/'EVIDENCE_VERIFICATION.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='receipts'}))
