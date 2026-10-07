from pathlib import Path
import hashlib,json,tarfile,difflib,re
HERE=Path(__file__).resolve().parent
CFG=json.loads((HERE/'config.json').read_text())
ROOT,SEG=Path(CFG['root']),Path(CFG['segment'])
def sha(b):return hashlib.sha256(b).hexdigest()
def check(rec):
 b=(ROOT/rec['path']).read_bytes();assert len(b)==rec['bytes'] and sha(b)==rec['sha256'],rec['path']
mp=SEG/'target/d04_f01_correction_001/MANIFEST.json'
assert sha(mp.read_bytes())==CFG['target_sha256']
m=json.loads(mp.read_text());expected={f['path']:f['after'] for f in m['source']['files']}
assert len(expected)==len(m['source']['files'])==333
for p,rec in expected.items():check(dict(path=p,**rec))
for rec in [m['baseline'],m['predecessor'],m['after_archive']]+m['artifacts']+m['records']:check(rec)
with tarfile.open(ROOT/m['after_archive']['path']) as t:
 members=t.getmembers();assert len(members)==333 and {x.name for x in members}==set(expected)
 for x in members:
  assert x.isfile();b=t.extractfile(x).read();assert dict(bytes=len(b),sha256=sha(b))==expected[x.name]
rows=sorted([[p,x['bytes'],x['sha256']] for p,x in expected.items()]);hashes={p:x['sha256'] for p,x in expected.items()}
reused=json.loads((HERE/'reused_evidence.json').read_text())
for tag,summary in reused.items():
 d=json.loads((SEG/'evidence'/f'{tag}.json').read_text())
 assert d['source_before']==d['source_after'] and sorted(d['source_before']['files'])==rows
 assert d['exit_code']==summary['exit_code']
 assert sha((SEG/'evidence'/f'{tag}.log').read_bytes())==d['log_sha256']
pc=json.loads((SEG/'evidence/pc_isolate_002.json').read_text());pr=json.loads((SEG/'evidence/pc_36_final_002.json').read_text())
assert pc['current_source']==pc['copied_source'] and sorted(pc['copied_source']['files'])==rows
assert pc['built_source']==pr['source_before']==pr['source_after'] and pr['exit_code']==0
assert pr['source_dir']==pr['cwd']==pc['isolate']
assert sha((SEG/'evidence/pc_36_final_002.log').read_bytes())==pr['log_sha256']
control=json.loads((SEG/'evidence/r05_predecessor_control_002.json').read_text())
before_rows=sorted([[f['path'],f['before']['bytes'],f['before']['sha256']] for f in m['source']['files']])
assert control['source_before']==control['source_after'] and sorted(control['source_before']['files'])==before_rows
assert control['exit_code']==101 and sha((SEG/'evidence/r05_predecessor_control_002.log').read_bytes())==control['log_sha256']
ci=json.loads((SEG/'evidence/r05_control_isolate_001.json').read_text())
assert control['cwd']==str(Path(ci['isolate'])/'core_enhanced') and control['source_dir']==ci['isolate']
for rec in ci['tool_files']:check(rec)
for tag,exit_code in [('focused_correction_001',101),('focused_correction_002',0)]:
 d=json.loads((HERE/f'{tag}.json').read_text());assert d['exit_code']==exit_code
 assert d['source_before']==d['source_after']==d['workspace_before']==d['workspace_after']==hashes
assert '9 passed; 0 failed' in (HERE/'focused_correction_002.log').read_text()
p='core_enhanced/src/game/battle/core/mod.rs';iso=Path(CFG['isolate']);original=(ROOT/p).read_text();patched=(iso/p).read_text()
hook='\n#[path = "'+str(HERE/'projectile_probes.rs')+'"]\nmod independent_d04_review;\n'
i=original.rfind('}');assert patched==original[:i]+hook+original[i:]
(HERE/'test_only_hook.patch').write_text(''.join(difflib.unified_diff(original.splitlines(True),patched.splitlines(True),fromfile='a/'+p,tofile='b/'+p)))
d=json.loads((HERE/'original_reproductions_001.json').read_text());assert d['exit_code']==0 and d['source_before']==d['source_after']
assert [k for k,h in d['source_before'].items() if h!=hashes[k]]==[p] and d['source_before'][p]==sha(patched.encode())
assert d['workspace_before']==d['workspace_after']==hashes
assert '4 passed; 0 failed' in (HERE/'original_reproductions_001.log').read_text()
prior=HERE.parent/'wp02_d04_resolved_target_001_review_001/projectile_probes.rs';assert prior.read_bytes()==(HERE/'projectile_probes.rs').read_bytes()
for rec in CFG['supplemental']:assert sha((iso/rec['path']).read_bytes())==rec['sha256']
summary=dict(target_sha256=CFG['target_sha256'],current_source_files=333,records_verified=len(m['records']),artifacts_verified=len(m['artifacts']),archive_exact=True,pc_source_binding=True,r05_control_source_binding=True,direct_product_tests_passed=9,original_reproduction_tests_passed=4,original_probe_sha256=sha(prior.read_bytes()),workspace_source_unchanged=True)
(HERE/'verification.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
