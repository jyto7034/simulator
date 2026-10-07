import pathlib,json,hashlib,subprocess,tarfile,sys,datetime,shutil
R=pathlib.Path('/mnt/f/work/simulator'); D=R/'core_enhanced/docs/goals/core_runtime_safety_observability'; A=pathlib.Path(__file__).resolve().parent;T=D/'execution/s2/corrections/n01_metric_docs_001/target/s2_n01_001'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(n,v):(A/n).write_text(json.dumps(v,indent=2)+'\n')
def rows(base):return sorted([dict(path=str(p.relative_to(base)),bytes=p.stat().st_size,sha256=sha(p)) for p in base.rglob('*') if p.is_file()],key=lambda x:x['path'])
def check(entries,base):
 bad=[]
 for x in entries:
  p=base/x['path']
  if not p.is_file() or p.stat().st_size!=x['bytes'] or sha(p)!=x['sha256']:bad.append(x['path'])
 return bad
m=json.loads((T/'MANIFEST.json').read_text());assert sha(T/'MANIFEST.json')=='784752ce20f3bba737c7c04a77ecede0686c237d339de62ea73f60cdba2cc47b'
mem=json.loads((T/'MEMBERSHIP.json').read_text());bad=check(mem,R);assert not bad,bad
assert not check(m['archives_and_patches'],T)
tracked=set(subprocess.check_output(['git','ls-files','-z'],cwd=R).decode().split('\0'));dirty=set(subprocess.check_output(['git','diff','--name-only','-z','HEAD'],cwd=R).decode().split('\0'))
assert all(x['kind']==('tracked/dirty' if x['path'] in dirty else 'tracked/clean' if x['path'] in tracked else 'untracked') for x in mem)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()==m['vcs_revision']
records=sorted(str(p.relative_to(R)) for p in D.rglob('*') if p.is_file() and not {'target','post_audit','__pycache__'}&set(p.relative_to(D).parts))
assert records==[x['path'] for x in json.loads((T/'RECORDS.json').read_text())]
sys.dont_write_bytecode=True;sys.path.insert(0,str(D/'tools'));import run
assert run.identity()==m['product_source']
# Fresh reference membership enumeration uses the frozen owner rule, without rerunning its writer.
C=R/'core_enhanced/docs/roadmaps/core_first';P=R/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime';F=P/'execution/single_goal/corrections/f01_action_flow_20261004_001'
refs={C/'ROADMAP.ko.md',C/'execution/CONTINUOUS_REQUEST_20261004.ko.md'}
refs.update(p for p in (C/'execution').rglob('*') if p.is_file())
refs.update(P/n for n in ['post_audit/20261004_f01_001/COMPLETION_AUDIT.md','reviews/r08_r12_20261002_001/CONDITION_REVIEW.json','reviews/r08_r12_20261002_001/R06_R07_IMPACT_REVIEW.json'])
refs.update(F/n for n in ['COVERAGE_CURRENT.json','target/f01_action_flow_001/MANIFEST.json']);refs.update(p for p in P.iterdir() if p.is_file())
refs.update(R/'core_enhanced/docs'/n for n in ['policy_to_goal_pipeline_revised_v2.md','goal_execution_pipeline(1).md','game_rulebook_next.md','game_rules_next/results-and-save.md','game_rules_next/open-decisions.md'])
refs.update(D/n for n in ['execution/s2/target/s2_001/MANIFEST.json','execution/s1/target/s1_001/MANIFEST.json','execution/s1/corrections/f01_active_battle_001/target/s1_f01_001/MANIFEST.json','execution/s1/corrections/f02_persistence_entry_001/target/s1_f02_001/MANIFEST.json'])
for audit in ['s1_20261004_001','s1_20261004_002','s1_20261004_003','s2_20261004_001']:refs.update(p for p in (D/'post_audit'/audit).rglob('*') if p.is_file() and '__pycache__' not in p.parts)
assert sorted(str(p.relative_to(R)) for p in refs)==[x['path'] for x in json.loads((T/'REFERENCES.json').read_text())]
extra=json.loads((A/'EXTRA_INPUTS.json').read_text()) if (A/'EXTRA_INPUTS.json').exists() else []
assert not check(extra,R)
write(('END' if '--end' in sys.argv else 'ENTRY')+'_CURRENTNESS.json',dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),verdict='current / unchanged',manifest_sha256=sha(T/'MANIFEST.json'),product_sha256=m['product_source']['sha256'],union_sha256=m['source_sha256'],membership_count=len(mem),records_membership_freshly_enumerated=True,member_bytes_and_kinds_match=True,archive_hashes_match=True,source_membership_freshly_enumerated=True,references_membership_freshly_enumerated=True,extra_inputs_checked=len(extra),extra_inputs_match=True))
if '--end' in sys.argv:print('END current / unchanged',len(mem));sys.exit()
P=A/'restore';P.mkdir(exist_ok=False)
results={}
for name,key in [('SOURCE_BEFORE','before_files'),('SOURCE_AFTER','source_files'),('RECORDS',None),('REFERENCES',None)]:
 dest=P/name;dest.mkdir()
 with tarfile.open(T/(name+'.tar.gz')) as tf:tf.extractall(dest,filter='data')
 expected=m[key] if key else json.loads((T/(name+'.json')).read_text())
 assert rows(dest)==expected,name
 results[name]={'members':len(expected),'exact_bytes_and_membership':True}
for reverse,expected in [(False,m['source_files']),(True,m['before_files'])]:
 out=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=P/'SOURCE_BEFORE',capture_output=True)
 (A/('restore_reverse.log' if reverse else 'restore_forward.log')).write_bytes(out.stdout+out.stderr)
 assert out.returncode==0
 assert rows(P/'SOURCE_BEFORE')==expected
 results['reverse' if reverse else 'forward']={'exact_bytes_and_membership':True,'fuzz':0,'raw_exit':out.returncode}
write('RESTORATION.json',results);print(json.dumps(results));shutil.rmtree(P)
