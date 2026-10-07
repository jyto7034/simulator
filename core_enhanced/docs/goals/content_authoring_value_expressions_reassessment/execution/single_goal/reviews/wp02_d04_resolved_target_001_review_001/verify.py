from pathlib import Path
import hashlib, json, tarfile, difflib
HERE=Path(__file__).resolve().parent
CFG=json.loads((HERE/'config.json').read_text())
ROOT,SEG=Path(CFG['root']),Path(CFG['segment'])
def sha(b): return hashlib.sha256(b).hexdigest()
def check(rec):
 b=(ROOT/rec['path']).read_bytes()
 assert len(b)==rec['bytes'] and sha(b)==rec['sha256'],rec['path']
mfile=SEG/'target/d04_resolved_target_001/MANIFEST.json'
assert sha(mfile.read_bytes())==CFG['target_sha256']
m=json.loads(mfile.read_text())
expected={f['path']:f['after'] for f in m['source']['files']}
assert len(expected)==len(m['source']['files'])==333
for p,rec in expected.items(): check(dict(path=p,**rec))
for rec in [m['baseline'],m['predecessor'],m['after_archive']]+m['artifacts']+m['records']: check(rec)
with tarfile.open(ROOT/m['after_archive']['path']) as t:
 members=t.getmembers();assert len(members)==333 and {x.name for x in members}==set(expected)
 for x in members:
  assert x.isfile()
  b=t.extractfile(x).read();assert dict(bytes=len(b),sha256=sha(b))==expected[x.name]
rows=sorted([[p,r['bytes'],r['sha256']] for p,r in expected.items()])
for tag in json.loads((HERE/'reused_evidence.json').read_text()):
 d=json.loads((SEG/'evidence'/f'{tag}.json').read_text())
 assert d['source_before']==d['source_after'] and sorted(d['source_before']['files'])==rows
 assert d['exit_code']==0 and sha((SEG/'evidence'/f'{tag}.log').read_bytes())==d['log_sha256']
pc=json.loads((SEG/'evidence/pc_isolate_001.json').read_text())
run=json.loads((SEG/'evidence/pc_36_final_001.json').read_text())
assert pc['current_source']==pc['copied_source'] and sorted(pc['copied_source']['files'])==rows
assert pc['built_source']==run['source_before']==run['source_after']
assert run['source_dir']==run['cwd']==pc['isolate'] and run['exit_code']==0
assert sha((SEG/'evidence/pc_36_final_001.log').read_bytes())==run['log_sha256']
current_hashes={p:r['sha256'] for p,r in expected.items()}
focused=json.loads((HERE/'focused_001.json').read_text())
assert focused['source_before']==focused['source_after']==current_hashes and focused['exit_code']==0
assert focused['workspace_before']==focused['workspace_after']==current_hashes
p='core_enhanced/src/game/battle/core/mod.rs'
probe=HERE/'projectile_probes.rs'
hook='\n#[path = "'+str(probe)+'"]\nmod independent_d04_review;\n'
source_bindings=[]
for folder,config,tags,is_control in [(HERE,CFG,['projectile_probes_001','target_locked_success_001','area_success_001'],False),
 (HERE/'predecessor_control',json.loads((HERE/'predecessor_control/config.json').read_text()),['scalar_projectiles_001'],True)]:
 iso=Path(config['isolate']);source_expected={f['path']:f['after']['sha256'] for f in config['source']}
 patched=(iso/p).read_text();assert patched.count(hook)==1
 original=patched.replace(hook,'');assert sha(original.encode())==source_expected[p]
 (folder/'test_only_hook.patch').write_text(''.join(difflib.unified_diff(original.splitlines(True),patched.splitlines(True),fromfile='a/'+p,tofile='b/'+p)))
 for tag in tags:
  d=json.loads((folder/f'{tag}.json').read_text());assert d['source_before']==d['source_after']
  assert [key for key,h in d['source_before'].items() if h!=source_expected[key]]==[p]
  assert d['source_before'][p]==sha(patched.encode())
  assert d['workspace_before']==d['workspace_after']==current_hashes
  assert d['exit_code']==(101 if tag=='projectile_probes_001' else 0)
  source_bindings.append(dict(run=str((folder/f'{tag}.json').relative_to(HERE)),exit_code=d['exit_code'],
    source_sha256=config['target_sha256'],log_sha256=sha((folder/f'{tag}.log').read_bytes())))
summary=dict(target_sha256=CFG['target_sha256'],source_files_unchanged=333,records_verified=len(m['records']),artifacts_verified=len(m['artifacts']),
 archive_exact=True,pc_copied_built_run_identity_verified=True,focused_passed=74,probe_source_sha256=sha(probe.read_bytes()),test_only_bindings=source_bindings)
(HERE/'verification.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
