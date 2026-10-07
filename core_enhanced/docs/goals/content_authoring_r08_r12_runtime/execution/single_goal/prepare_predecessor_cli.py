"""Preserve predecessor packs; add only the newly required explicit R09 policy."""
import hashlib,json,shutil
from pathlib import Path
H=Path(__file__).resolve().parent;E=H/'evidence';R=Path('/mnt/f/work/simulator');O=R/'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence'
N=E/'cli_predecessor_current_001';N.mkdir()
current=(R/'game_resources/data/run/policy.ron').read_bytes()
line=next(v for v in current.splitlines(keepends=True) if v.startswith(b'    corrosion:'))
sha=lambda b:hashlib.sha256(b).hexdigest();rows=[]
for tag,p in [('r06',O/'public_cli_003'),('r07_snapshot',O/'r07_public_cli_003/Snapshot'),('r07_live',O/'r07_public_cli_003/Live')]:
 dest=N/tag;shutil.copytree(p,dest);policy=dest/'content/official/data/run/policy.ron';old=policy.read_bytes()
 assert current.replace(line,b'',1)==old,tag
 policy.write_bytes(current)
 oldfiles=sorted(f for f in p.rglob('*') if f.is_file());newfiles=sorted(f for f in dest.rglob('*') if f.is_file())
 assert [f.relative_to(p) for f in oldfiles]==[f.relative_to(dest) for f in newfiles]
 changes=[];files=[]
 for a,b in zip(oldfiles,newfiles):
  x,y=a.read_bytes(),b.read_bytes()
  if x!=y:changes.append(str(a.relative_to(p)))
  files.append({'path':str(a.relative_to(p)),'before_sha256':sha(x),'after_sha256':sha(y),'before_bytes':len(x),'after_bytes':len(y)})
 assert changes==['content/official/data/run/policy.ron']
 rows.append({'tag':tag,'original':str(p.relative_to(R)),'current_copy':str(dest.relative_to(H)),'changes':changes,'files':files})
(E/'CLI_PREDECESSOR_MIGRATION_001.json').write_text(json.dumps({'reason':'R09 requires an explicit corrosion policy; old packages reject missing field rather than silently taking defaults.','inserted_exact_line':line.decode().rstrip(),'scenario_skill_copy_recipe_and_other_policy_bytes_unchanged':True,'originals_unmodified':True,'packs':rows},indent=2)+'\n')
print('Three predecessor pack copies migrated; only one explicit policy line changed.')
