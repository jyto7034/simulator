"""Correction successor, adapted from test_parts/drivers001/finalize.py.
Guarded restoration uses owned temporary trees only; predecessor stays immutable.
"""
from pathlib import Path
import json,tarfile,hashlib,io,subprocess,tempfile,difflib
R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/core_expedition_deterrence_rewards/execution/single_goal';C=E/'correction/u14_correction_001';P=E/'target/u14_current_001';O=E/'target/u14_current_002'
sha=lambda b:hashlib.sha256(b).hexdigest()
def write(p,v):
 with p.open('x') as f:json.dump(v,f,indent=2,ensure_ascii=False);f.write('\n')
def readtar(p):
 with tarfile.open(p) as t:return {m.name:t.extractfile(m).read() for m in t if m.isfile()}
def archive(p,files):
 with tarfile.open(p,'x:gz') as t:
  for name,raw in sorted(files.items()):
   m=tarfile.TarInfo(name);m.size=len(raw);m.mode=0o644;t.addfile(m,io.BytesIO(raw))
def inventory(files):return [{'path':p,'bytes':len(b),'sha256':sha(b)} for p,b in sorted(files.items())]
result=json.loads((C/'RESULT.json').read_text());assert result['correction_verified'] and not result['audit_performed']
O.mkdir(exist_ok=False)
before=readtar(P/'SOURCE.tar.gz');paths=set(before)
paths.update(str(p.relative_to(R)) for p in C.rglob('*') if p.is_file() and (p.suffix in ['.rs','.py'] or p.name in ['Cargo.toml','Cargo.lock','INTAKE.json']))
fixed={p:(R/p).read_bytes() for p in sorted(paths) if (R/p).is_file()}
archive(O/'SOURCE.tar.gz',fixed);assert readtar(O/'SOURCE.tar.gz')==fixed
# The predecessor archive is the exact original preimage, never reconstructed.
operations=[];diff=[]
for p in sorted(before.keys()|fixed.keys()):
 a=before.get(p);b=fixed.get(p)
 if a==b:continue
 operations.append({'path':p,'operation':'add' if a is None else 'delete' if b is None else 'replace','before_sha256':None if a is None else sha(a),'after_sha256':None if b is None else sha(b)})
 diff.extend(difflib.unified_diff((a or b'').decode().splitlines(True),(b or b'').decode().splitlines(True),fromfile='a/'+p if a is not None else '/dev/null',tofile='b/'+p if b is not None else '/dev/null'))
write(O/'DELTA.json',operations);(O/'SCOPED.diff').write_text(''.join(diff))
workspace=Path(tempfile.mkdtemp(prefix='u14-correction-restore-'))
def put(files):
 for p,b in files.items():
  dst=workspace/p;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(b)
def actual():return {str(p.relative_to(workspace)):p.read_bytes() for p in workspace.rglob('*') if p.is_file()}
put(before);assert actual()==before
for source,destination in [(before,fixed),(fixed,before)]:
 for op in operations:
  p=op['path'];dst=workspace/p;assert (dst.read_bytes() if dst.exists() else None)==source.get(p)
  if p in destination:dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(destination[p])
  else:dst.unlink()
 assert actual()==destination
write(O/'RESTORE.json',{'passed':True,'preimage':'../u14_current_001/SOURCE.tar.gz','preimage_sha256':sha((P/'SOURCE.tar.gz').read_bytes()),'isolated_root':str(workspace),'forward_members':len(fixed),'reverse_members':len(before),'operations':len(operations),'method':'guarded exact add/replace/delete; full archived byte and membership equality both directions','working_tree_mutations':0})
tracked=set(subprocess.check_output(['git','ls-files','-z'],cwd=R).decode().split('\0'))
source_rows=inventory(fixed)
for row in source_rows:row['kind']='tracked' if row['path'] in tracked else 'untracked'
write(O/'SOURCE_MANIFEST.json',{'files':source_rows,'predecessor':'u14_current_001','membership':'predecessor source plus correction drivers/probe/intake; no unrelated working tree or audit outputs'})
used={str(p.relative_to(R)):p.read_bytes() for p in C.rglob('*') if p.is_file() and str(p.relative_to(R)) not in fixed}
write(O/'EVIDENCE_MANIFEST.json',{'files':inventory(used),'predecessor_evidence_manifest':'../u14_current_001/EVIDENCE_MANIFEST.json','predecessor_evidence_manifest_sha256':sha((P/'EVIDENCE_MANIFEST.json').read_bytes()),'reuse_scope':'RESULT.json; predecessor results retain original source/binary identity, not approval of the correction executable','audit':'No audit performed; original independent001 findings/verdict preserved'})
write(O/'AUDIT_TARGET_RECORD.json',{'target_id':'u14_current_002','predecessor_target':'u14_current_001','goal_set':'GS-CEDR-001','goal':'GOAL-CEDR-01','phase':'PHASE-CEDR-01','HEAD':subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),'source_archive_sha256':sha((O/'SOURCE.tar.gz').read_bytes()),'source_manifest_sha256':sha((O/'SOURCE_MANIFEST.json').read_bytes()),'evidence_manifest_sha256':sha((O/'EVIDENCE_MANIFEST.json').read_bytes()),'restore_sha256':sha((O/'RESTORE.json').read_bytes()),'status':'correction verified; independent re-audit request pending','goal_complete':False,'audit_performed':False,'source_members':len(fixed),'evidence_members':len(used),'delta_members':len(operations),'repository':str(R),'final_policy':'FPF-CEDR-001','phase_plan':'PP-CEDR-001','prior_audit':'../../audit/u14_independent_001/AUDIT_CLOSURE.json','exclusions':['historical and future audit records; audit/u14_independent_001 retains original source/verdict','mutable HANDOFF/EXECUTION_HISTORY/ROADMAP_STATUS','unrelated files/cache/private issuer keys'],'currentness':'one final source snapshot after correction; archive and isolated forward/reverse equality verified'})
print(json.dumps({'target':str(O),'source_members':len(fixed),'evidence_members':len(used),'delta_members':len(operations),'restore_passed':True}))
