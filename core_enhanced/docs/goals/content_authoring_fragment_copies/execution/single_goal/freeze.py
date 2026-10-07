"""Freeze exact dynamic source membership and demonstrate isolated forward/reverse restoration."""
from pathlib import Path
import hashlib,json,tarfile,subprocess,tempfile,shutil,sys
import run
ROOT=run.ROOT;H=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def rows(base,names):return [{'path':n,'bytes':(base/n).stat().st_size,'sha256':sha(base/n)} for n in names]
def digest(r):return hashlib.sha256(json.dumps([(x['path'],x['bytes'],x['sha256']) for x in r],separators=(',',':')).encode()).hexdigest()
def extract(archive,destination):
 with tarfile.open(archive) as t:t.extractall(destination,filter='data')
def writej(p,x):
 with p.open('x') as f:json.dump(x,f,ensure_ascii=False,indent=2);f.write('\n')
name=sys.argv[1];T=H/'target'/name;T.mkdir(parents=True,exist_ok=False)
old=json.loads((H/'baseline/MANIFEST.json').read_text())['files'];new=rows(ROOT,run.source_names());oldnames=[r['path'] for r in old];newnames=[r['path'] for r in new]
shutil.copyfile(H/'baseline/SOURCE_BEFORE.tar.gz',T/'SOURCE_BEFORE.tar.gz')
with tarfile.open(T/'SOURCE_AFTER.tar.gz','w:gz') as t:
 for n in newnames:t.add(ROOT/n,arcname=n,recursive=False)
with tempfile.TemporaryDirectory(prefix='r06-restore-') as td:
 B=Path(td)/'before';A=Path(td)/'after';B.mkdir();A.mkdir();extract(T/'SOURCE_BEFORE.tar.gz',B);extract(T/'SOURCE_AFTER.tar.gz',A)
 assert rows(B,oldnames)==old
 assert rows(A,newnames)==new
 patches=[]
 for n in sorted(set(oldnames)|set(newnames)):
  b=B/n if n in oldnames else Path('/dev/null');a=A/n if n in newnames else Path('/dev/null')
  p=subprocess.run(['diff','-u','--label','a/'+n if n in oldnames else '/dev/null','--label','b/'+n if n in newnames else '/dev/null',str(b),str(a)],capture_output=True)
  assert p.returncode in (0,1)
  patches.append(p.stdout)
 (T/'FORWARD.patch').write_bytes(b''.join(patches))
 p=subprocess.run(['patch','-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=B,capture_output=True)
 (T/'RESTORE_FORWARD.log').write_bytes(p.stdout+p.stderr);assert p.returncode==0
 actual=sorted(str(p.relative_to(B)) for p in B.rglob('*') if p.is_file());assert actual==newnames;assert rows(B,newnames)==new
 p=subprocess.run(['patch','-R','-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=B,capture_output=True)
 (T/'RESTORE_REVERSE.log').write_bytes(p.stdout+p.stderr);assert p.returncode==0
 actual=sorted(str(p.relative_to(B)) for p in B.rglob('*') if p.is_file());assert actual==oldnames;assert rows(B,oldnames)==old
 assert rows(ROOT,run.source_names())==new
 writej(T/'RESTORE_VERIFICATION.json',{'current_membership_and_bytes_match':True,'forward_membership_and_bytes_match':True,'reverse_membership_and_bytes_match':True,'patch_fuzz':0,'before_digest':digest(old),'after_digest':digest(new),'added':sorted(set(newnames)-set(oldnames)),'deleted':sorted(set(oldnames)-set(newnames))})
# Freeze live records/fixtures/scripts; exclude archives of this freeze and mutable Python cache.
records=sorted(p for p in H.rglob('*') if p.is_file() and not any(x in p.relative_to(H).parts for x in ('target','__pycache__')))
with tarfile.open(T/'RECORDS.tar.gz','w:gz') as t:
 for p in records:t.add(p,arcname=str(p.relative_to(ROOT)),recursive=False)
references=sorted(set([ROOT/'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md', *[p for p in H.parents[1].rglob('*') if p.is_file() and 'execution' not in p.relative_to(H.parents[1]).parts and '__pycache__' not in p.parts], *[p for p in (ROOT/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs').glob('R06*') if p.is_file()], ROOT/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001/target/r06_boundary_001/MANIFEST.json']))
reference_basis = ROOT/'core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review'
references = sorted(set(references + [reference_basis/n for n in ['goals/R07.ko.md','DEPENDENCIES.ko.md','POLICY_GATES.ko.md','ACCEPTANCE.ko.md']] + [ROOT/'core_enhanced/docs'/n for n in ['game_rules_next/equipment-and-fragments.md','game_rules_next/open-decisions.md','policy_to_goal_pipeline_revised_v2.md']]))
assert all(p.is_file() for p in references)
with tarfile.open(T/'REFERENCES.tar.gz','w:gz') as t:
 for p in references:t.add(p,arcname=str(p.relative_to(ROOT)),recursive=False)
writej(T/'REFERENCES.json',[{'path':str(p.relative_to(ROOT)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in references])
writej(T/'MANIFEST.json',{'schema':1,'owner':'R06 + R07 integrated implementation / independent audit not performed','baseline_manifest_sha256':sha(H/'baseline/MANIFEST.json'),'before_digest':digest(old),'source_digest':digest(new),'source_files':new,'before_files':old,'records':[{'path':str(p.relative_to(ROOT)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in records],'archives_and_patches':[{'path':p.name,'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(T.iterdir()) if p.is_file()]})
print(json.dumps({'manifest':str(T/'MANIFEST.json'),'sha256':sha(T/'MANIFEST.json'),'source_digest':digest(new),'files':len(new)},indent=2))
