from pathlib import Path
import shutil,json,hashlib,difflib
import run
old=run.OLD/'evidence/r12_public_cli_003/effectssettled';new=run.HERE/'evidence/timeout_probe'
new.mkdir()
rows=[]
for policy in ['CancelUnreleased','Detach']:
 dest=new/policy;shutil.copytree(old,dest)
 p=dest/'content/official/data/boss_runtime/base.ron';original=p.read_text()
 # Timeout occurs after focus release but before the second, delayed flow action.
 changed=original.replace('timeout_ms:20000,on_timeout:CancelUnreleased','timeout_ms:1500,on_timeout:'+policy,1)
 assert changed!=original;p.write_text(changed)
 p=dest/'content/official/data/skills/base.ron';original=p.read_text();changed=original.replace('Wait(id: "pause", duration_ms: 100)','Wait(id: "pause", duration_ms: 1000)')
 assert changed!=original;p.write_text(changed)
 for p in dest.rglob('*'):
  if p.is_file():rows.append(dict(path=str(p.relative_to(run.ROOT)),sha256=run.sha(p)))
with (new/'INPUT_MANIFEST.json').open('x') as f:json.dump(rows,f,indent=2)
print(new)
