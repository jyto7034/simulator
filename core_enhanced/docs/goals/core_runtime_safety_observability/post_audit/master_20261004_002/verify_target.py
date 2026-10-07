"""Independent Master currentness/restoration; no product writes or imported freeze writer."""
import pathlib,json,hashlib,subprocess,tarfile,tempfile,sys,datetime
R=pathlib.Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';A=pathlib.Path(__file__).resolve().parent;T=D/'execution/master/followups/n02_001/target/master_002'
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
def load(p):return json.loads(p.read_text())
def write(n,v):(A/n).write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
def rows(base,paths):return [dict(path=n,bytes=(base/n).stat().st_size,sha256=sha(base/n)) for n in sorted(paths)]
def names(p):return sorted(str(x.relative_to(p)) for x in p.rglob('*') if x.is_file())
def check(rs,base):
 bad=[]
 for x in rs:
  p=base/x['path']
  if not p.is_file() or p.stat().st_size!=x['bytes'] or sha(p)!=x['sha256']:bad.append(x['path'])
 assert not bad,bad
m=load(T/'MANIFEST.json');assert sha(T/'MANIFEST.json')=='d6a2b4d18e377862323f934a23fe9ec50871de8af824743c1188fa99ec17c5b3'
mem=load(T/'MEMBERSHIP.json');check(mem,R);check(m['archives_and_patches'],T)
assert m['before_files']==m['source_files'] and not m['changed'] and not m['added'] and not m['deleted']
assert hashlib.sha256(json.dumps(m['source_files'],sort_keys=True).encode()).hexdigest()==m['source_sha256']
tracked=set(subprocess.check_output(['git','ls-files','-z'],cwd=R).decode().split('\0'));dirty=set(subprocess.check_output(['git','diff','--name-only','-z','HEAD'],cwd=R).decode().split('\0'))
assert all(x['kind']==('tracked/dirty' if x['path'] in dirty else 'tracked/clean' if x['path'] in tracked else 'untracked') for x in mem)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()==m['vcs_revision']
records=sorted(str(p.relative_to(R)) for p in D.rglob('*') if p.is_file() and not {'target','post_audit','__pycache__'}&set(p.relative_to(D).parts))
assert records==[x['path'] for x in load(T/'RECORDS.json')]
product=set()
for folder in ['core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data']:
 product.update(str(p.relative_to(R)) for p in (R/folder).rglob('*') if p.is_file())
product.update(['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml','core_enhanced/docs/audit/live_skill_catalog_manifest.ron','core_enhanced/docs/skills/live_content_index.md','core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'])
assert rows(R,product)==sorted(m['product_source']['files'],key=lambda x:x['path'])
assert hashlib.sha256(json.dumps(m['product_source']['files'],sort_keys=True).encode()).hexdigest()==m['product_source']['sha256']
C=R/'core_enhanced/docs/roadmaps/core_first';P=R/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime';F=P/'execution/single_goal/corrections/f01_action_flow_20261004_001'
refs={C/'ROADMAP.ko.md',C/'execution/CONTINUOUS_REQUEST_20261004.ko.md'}
refs.update(p for p in (C/'execution').rglob('*') if p.is_file())
refs.update(P/n for n in ['post_audit/20261004_f01_001/COMPLETION_AUDIT.md','reviews/r08_r12_20261002_001/CONDITION_REVIEW.json','reviews/r08_r12_20261002_001/R06_R07_IMPACT_REVIEW.json'])
refs.update(F/n for n in ['COVERAGE_CURRENT.json','target/f01_action_flow_001/MANIFEST.json']);refs.update(p for p in P.iterdir() if p.is_file())
refs.update(R/'core_enhanced/docs'/n for n in ['policy_to_goal_pipeline_revised_v2.md','goal_execution_pipeline(1).md','game_rulebook_next.md','game_rules_next/results-and-save.md','game_rules_next/open-decisions.md'])
refs.update(D/n for n in ['execution/s1/target/s1_001/MANIFEST.json','execution/s1/corrections/f01_active_battle_001/target/s1_f01_001/MANIFEST.json','execution/s1/corrections/f02_persistence_entry_001/target/s1_f02_001/MANIFEST.json','execution/s2/target/s2_001/MANIFEST.json','execution/s2/corrections/n01_metric_docs_001/target/s2_n01_001/MANIFEST.json','execution/master/target/master_001/MANIFEST.json','execution/s2/corrections/n02_contract_link_001/target/s2_n02_001/MANIFEST.json'])
for audit in ['s1_20261004_001','s1_20261004_002','s1_20261004_003','s2_20261004_001','s2_20261004_002','master_20261004_001','s2_20261004_003']:refs.update(p for p in (D/'post_audit'/audit).rglob('*') if p.is_file() and '__pycache__' not in p.parts)
assert sorted(str(p.relative_to(R)) for p in refs)==[x['path'] for x in load(T/'REFERENCES.json')]
union={}
for group,rr in [('source',m['source_files']),('records',load(T/'RECORDS.json')),('references',load(T/'REFERENCES.json'))]:
 for x in rr:
  if x['path'] in union:assert union[x['path']]['sha256']==x['sha256']
  union[x['path']]=x
assert set(union)=={x['path'] for x in mem}
extra=load(D/'post_audit/s2_20261004_003/EXTRA_INPUTS.json');check(extra,R)
prior=load(D/m['predecessor_target']);assert prior['product_source']==m['product_source'] and prior['source_files']==m['source_files']
write(('END' if '--end' in sys.argv else 'ENTRY')+'_CURRENTNESS.json',dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),verdict='current / unchanged',manifest_sha256=sha(T/'MANIFEST.json'),product_sha256=m['product_source']['sha256'],union_sha256=m['source_sha256'],membership_count=len(mem),source_count=len(m['source_files']),records_count=len(records),references_count=len(refs),records_membership_freshly_enumerated=True,member_bytes_and_kinds_match=True,archive_hashes_match=True,source_membership_freshly_enumerated=True,references_membership_freshly_enumerated=True,extra_inputs_checked=len(extra),extra_inputs_match=True,S2_N02_source_exact=True))
print('current / unchanged',len(mem),'frozen members',len(extra),'extra inputs',flush=True)
if '--end' in sys.argv:sys.exit()
write('EXTRA_INPUTS.json',extra)
results={}
with tempfile.TemporaryDirectory(prefix='crso-master-independent-restore-') as tmp:
 p=pathlib.Path(tmp)
 for name,key in [('SOURCE_BEFORE','before_files'),('SOURCE_AFTER','source_files'),('RECORDS',None),('REFERENCES',None)]:
  dest=p/name;dest.mkdir()
  with tarfile.open(T/(name+'.tar.gz')) as tf:
   members=tf.getmembers();assert all(x.isfile() for x in members)
   assert len({x.name for x in members})==len(members)
   tf.extractall(dest,filter='data')
  expected=m[key] if key else load(T/(name+'.json'))
  assert rows(dest,names(dest))==expected,name
  results[name]=dict(members=len(expected),exact_bytes_and_membership=True)
 for reverse,expected in [(False,m['source_files']),(True,m['before_files'])]:
  out=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=p/'SOURCE_BEFORE',capture_output=True)
  (A/('restore_reverse.log' if reverse else 'restore_forward.log')).write_bytes(out.stdout+out.stderr)
  assert out.returncode==0;assert rows(p/'SOURCE_BEFORE',names(p/'SOURCE_BEFORE'))==expected
  results['reverse' if reverse else 'forward']=dict(exact_bytes_and_membership=True,fuzz=0,raw_exit=out.returncode)
write('RESTORATION.json',results);print(json.dumps(results))
