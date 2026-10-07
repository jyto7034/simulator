import sys,json,hashlib,re,tarfile,tempfile,subprocess,os
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_safety_observability';C=H/'execution/s1/corrections/f01_active_battle_001';T=C/'target/s1_f01_001';A=Path(__file__).parent;E=H/'evidence'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
m=json.loads((T/'MANIFEST.json').read_text());c=json.loads((C/'COVERAGE.json').read_text());impact=json.loads((C/'PREDECESSOR_IMPACT.json').read_text());old=json.loads((H/'execution/s1/target/s1_001/MANIFEST.json').read_text())
assert m['before_files']==old['source_files'];assert m['changed']==['core_enhanced/src/application.rs','game_server/src/game/player_game_actor/handlers.rs'];assert not m['added'] and not m['deleted']
passed={};summary={};newreceipts=[]
for binding in c['receipt_bindings']:
 name=binding['name'];p=E/(name+'.json');d=json.loads(p.read_text());assert sha(p)==binding['sha256'];log=E/(name+'.log');assert d['source_before']==d['source_after']==m['product_source'];assert d['exit_code']==0 and d['source_unchanged'];assert d['log_sha256']==sha(log);newreceipts.append(dict(name=name,command=d['command'],sha256=sha(p)))
 if name in c['suites']:
  raw=log.read_text();passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',raw,re.M));rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',raw);summary[name]=[sum(int(r[i]) for r in rows) for i in range(3)];assert summary[name][0]>0 and summary[name][1:]==[0,0]
core=passed['s1f01_all_default_001']&passed['s1f01_all_feature_001'];host=passed['s1f01_server_001']
for v in c['validations']:assert set(v['core_selectors'])<=core and set(v['host_selectors'])<=host
for cond in impact['inherited_conditions']:assert set(cond['selectors'])<=core
assert set(impact['f01_selectors'])<=core
reuse=[]
for b in c['preserved_prior_evidence']:
 name=b['name'];p=E/(name+'.json');d=json.loads(p.read_text());assert sha(p)==b['receipt_sha256'];assert d['exit_code']==0 and d['source_unchanged'];assert d['log_sha256']==sha(E/(name+'.log'));now={r['path']:r for r in m['product_source']['files']};previous={r['path']:r for r in d['source_after']['files']};assert all(now.get(p)==v for p,v in previous.items() if p not in m['changed']);reuse.append(dict(name=name,sha256=sha(p),basis='Only production delta is active-battle rollback refusal; test-only host expansion. Full suites/check/release replaced by current receipts.'))
def actual(root):
 return [dict(path=str(p.relative_to(root)),bytes=p.stat().st_size,sha256=sha(p)) for p in sorted(root.rglob('*'),key=lambda p:str(p.relative_to(root))) if p.is_file()]
with tempfile.TemporaryDirectory(prefix='s1f01-audit-restore-') as tmp:
 b=Path(tmp)/'source';b.mkdir()
 with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as tf:tf.extractall(b,filter='data')
 assert actual(b)==m['before_files']
 prior_app=(b/'core_enhanced/src/application.rs').read_text();prior_host=(b/'game_server/src/game/player_game_actor/handlers.rs').read_text()
 guard='''        if self.has_active_battle() {
            return Err(ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidAction,
                "active_battle_persistence_rollback",
                "persistence rollback is unavailable during an active battle",
            ));
        }
'''
 app=(R/'core_enhanced/src/application.rs').read_text();assert app.count(guard)==1 and app.replace(guard,'')==prior_app
 def outside(s):
  start=s.index('    #[test]\n    fn first_public_battle_entry_uses_host_work_limit_and_active_battle_has_no_rollback_token()');end=s.index('    fn start_new_game_with_default_starters(',start);return s[:start]+s[end:]
 assert outside(prior_host)==outside((R/'game_server/src/game/player_game_actor/handlers.rs').read_text())
 for reverse,key in [(False,'source_files'),(True,'before_files')]:
  p=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=b,capture_output=True);(A/('restore_reverse.log' if reverse else 'restore_forward.log')).write_bytes(p.stdout+p.stderr);assert p.returncode==0;assert actual(b)==m[key]
 for group,key in [('SOURCE_AFTER','source_files'),('RECORDS',None),('REFERENCES',None)]:
  dst=Path(tmp)/group;dst.mkdir()
  with tarfile.open(T/(group+'.tar.gz')) as tf:tf.extractall(dst,filter='data')
  expected=m[key] if key else json.loads((T/(group+'.json')).read_text());assert actual(dst)==expected
# The probe must use the compiler-selected CURRENT libraries, never a guessed cached hash.
pdir=C/'public_probe_002';pr=json.loads((pdir/'RESULT.json').read_text())
for row in pr['input_files']:assert sha(R/row['path'])==row['sha256'],row['path']
for row in pr['output_files']:assert sha(pdir/row['path'])==row['sha256']
libs={}
for line in (pdir/'build.jsonl').read_text().splitlines():
 item=json.loads(line)
 if item.get('reason')=='compiler-artifact' and item['target']['name'] in ['game_core','serde_json','uuid']:
  for filename in item['filenames']:
   if filename.endswith('.rlib'):libs[item['target']['name']]=Path(filename)
assert set(libs)=={'game_core','serde_json','uuid'}
assert all(str(p) in ' '.join(pr['command']) for p in libs.values())
probe=(C/'rollback_battle_probe.rs').read_text().replace(' let mut token=None;',' let mut token=None;\n let mut duplicate=None;').replace('   token=app.capture_persistence_rollback().unwrap();assert!(token.is_some());','   token=app.capture_persistence_rollback().unwrap();assert!(token.is_some());\n   duplicate=app.capture_persistence_rollback().unwrap();').replace(' assert!(app.has_active_battle());\n assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(),active_run);',' assert_eq!(app.restore_persistence_rollback(duplicate.unwrap()).unwrap_err().code(),"active_battle_persistence_rollback");\n assert!(app.has_active_battle());\n assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(),active_run);')
(A/'rollback_battle_probe.rs').write_text(probe)
with tempfile.TemporaryDirectory(prefix='s1f01-audit-public-') as tmp:
 binary=Path(tmp)/'probe';command=['/home/blast/.cargo/bin/rustc','--edition=2021',str(A/'rollback_battle_probe.rs')]
 for name,p in libs.items():command+=['--extern',name+'='+str(p)]
 command+=['-L','dependency='+str(libs['game_core'].parent),'-o',str(binary)]
 compiled=subprocess.run(command,capture_output=True,cwd=R);(A/'probe_compile.log').write_bytes(compiled.stdout+compiled.stderr);assert compiled.returncode==0
 result=subprocess.run([str(binary)],capture_output=True,cwd=R);(A/'probe_run.log').write_bytes(result.stdout+result.stderr);assert result.returncode==0
 probe_record=dict(command=command,compile_exit=compiled.returncode,run_exit=result.returncode,probe_sha256=sha(A/'rollback_battle_probe.rs'),binary_sha256=sha(binary),log_sha256=sha(A/'probe_run.log'),libraries={name:dict(path=str(p.relative_to(R)),sha256=sha(p)) for name,p in libs.items()},expectation='Both same-revision pre-entry tokens return active_battle_persistence_rollback; unchanged run/reconnect; battle retained; durable checkpoint unchanged across entry.')
(A/'ROLLBACK_PROBE.json').write_text(json.dumps(probe_record,indent=2)+'\n')
report=dict(target_manifest_sha256=sha(T/'MANIFEST.json'),full_suites=summary,all_v_selectors_present=True,inherited_conditions_with_current_nonzero_selectors=len(impact['inherited_conditions']),f01_selectors=len(impact['f01_selectors']),new_receipts=newreceipts,reused_receipts=reuse,restore=dict(forward=True,reverse=True,fuzz=0,source_after_archive=True,records_archive=True,references_archive=True),exact_delta=dict(paths=m['changed'],application_guard_only=True,host_test_only=True),independent_public_probe_exit=result.returncode)
(A/'EVIDENCE_REUSE.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['new_receipts','reused_receipts']}))
