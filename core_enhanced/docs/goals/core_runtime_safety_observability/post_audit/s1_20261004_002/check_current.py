import sys,json,hashlib,subprocess,datetime
from pathlib import Path
R=Path('/mnt/f/work/simulator'); H=R/'core_enhanced/docs/goals/core_runtime_safety_observability'; T=H/'execution/s1/corrections/f01_active_battle_001/target/s1_f01_001'; A=Path(__file__).parent
sys.dont_write_bytecode=True
sys.path.insert(0,str(H/'tools'))
import run
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
m=json.loads((T/'MANIFEST.json').read_text()); mismatches=[]
assert sha(T/'MANIFEST.json')=='4ed7c32c15891a67ec4580a8ad7a11b409f92c707f1e73ffd4da84dee1b2fece'
member=json.loads((T/'MEMBERSHIP.json').read_text())
for e in member:
 p=R/e['path']
 if not p.is_file() or p.stat().st_size!=e['bytes'] or sha(p)!=e['sha256']: mismatches.append(e['path'])
for e in m['archives_and_patches']:
 p=T/e['path']
 if not p.is_file() or sha(p)!=e['sha256']:mismatches.append(str(p.relative_to(R)))
product=run.identity();assert product==m['product_source']
records=sorted(str(p.relative_to(R)) for p in H.rglob('*') if p.is_file() and not {'target','post_audit','__pycache__'} & set(p.relative_to(H).parts))
assert records==[e['path'] for e in json.loads((T/'RECORDS.json').read_text())]
# Re-evaluate captured reference directory membership, not only existing bytes.
C=R/'core_enhanced/docs/roadmaps/core_first'; P=R/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime'; F=P/'execution/single_goal/corrections/f01_action_flow_20261004_001'
refs={C/'ROADMAP.ko.md',C/'execution/CONTINUOUS_REQUEST_20261004.ko.md'}
refs.update(p for p in (C/'execution').rglob('*') if p.is_file())
refs.update(P/n for n in ['post_audit/20261004_f01_001/COMPLETION_AUDIT.md','reviews/r08_r12_20261002_001/CONDITION_REVIEW.json','reviews/r08_r12_20261002_001/R06_R07_IMPACT_REVIEW.json'])
refs.update(F/n for n in ['COVERAGE_CURRENT.json','target/f01_action_flow_001/MANIFEST.json']); refs.update(p for p in P.iterdir() if p.is_file())
refs.update(R/'core_enhanced/docs'/n for n in ['policy_to_goal_pipeline_revised_v2.md','goal_execution_pipeline(1).md','game_rulebook_next.md','game_rules_next/results-and-save.md','game_rules_next/open-decisions.md'])
refs.add(H/'execution/s1/target/s1_001/MANIFEST.json')
refs.update(p for p in (H/'post_audit/s1_20261004_001').rglob('*') if p.is_file() and '__pycache__' not in p.parts)
assert sorted(str(p.relative_to(R)) for p in refs)==[e['path'] for e in json.loads((T/'REFERENCES.json').read_text())]
tracked=set(subprocess.check_output(['git','ls-files','-z'],cwd=R).decode().split('\0')); dirty=set(subprocess.check_output(['git','diff','--name-only','-z','HEAD'],cwd=R).decode().split('\0'))
for e in member:
 kind='tracked/dirty' if e['path'] in dirty else 'tracked/clean' if e['path'] in tracked else 'untracked'
 if kind!=e['kind']:mismatches.append(e['path']+':vcs-kind')
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()==m['vcs_revision']
result=dict(at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),manifest_sha256=sha(T/'MANIFEST.json'),source_sha256=product['sha256'],members=len(member),mismatches=mismatches,new_membership_checked=True,status='current / unchanged' if not mismatches else 'stale')
(A/(sys.argv[1]+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));assert not mismatches
