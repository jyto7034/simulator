import json,hashlib,subprocess,sys,datetime
from pathlib import Path
ROOT=Path('/mnt/f/work/simulator'); BASE=ROOT/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal'; OUT=BASE/'audit-evidence/fresh_001'
def identity(p):
 p=Path(p);p=p if p.is_absolute() else ROOT/p
 return {'path':str(p.relative_to(ROOT)),'sha256':hashlib.file_digest(p.open('rb'),'sha256').hexdigest(),'bytes':p.stat().st_size}
def check(x,p=None):
 p=Path(p or x['path']); p=p if p.is_absolute() else ROOT/p
 if x.get('kind')=='required absence':return None if not p.exists() else {'path':str(p),'error':'required absence exists'}
 if not p.is_file():return {'path':str(p),'error':'missing'}
 a=identity(p)
 return None if a['sha256']==x['sha256'] and a['bytes']==x['bytes'] else {'expected':x,'actual':a}
def target(p):
 d=json.load(open(p));errs=[]
 for x in d['manifest']:
  e=check(x)
  if e:errs.append(e)
 extcount=0
 for inv in d['external_actual']:
  b=ROOT/inv['directory']; exp={x['path'] for x in inv['files']};act={str(x.relative_to(b)) for x in b.rglob('*') if x.is_file()};extcount+=len(act)
  if exp!=act:errs.append({'directory':inv['directory'],'added':sorted(act-exp),'missing':sorted(exp-act)})
  for x in inv['files']:
   e=check(x,b/x['path'])
   if e:errs.append(e)
 for x in d.get('default_release_binaries',[]):
  e=check(x)
  if e:errs.append(e)
 return {'target':identity(p),'manifest_members':len(d['manifest']),'required_absences':sum(x.get('kind')=='required absence' for x in d['manifest']),'external_inventories':len(d['external_actual']),'external_files':extcount,'default_binaries':len(d.get('default_release_binaries',[])),'errors':errs}
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'branch':subprocess.check_output(['git','branch','--show-current'],cwd=ROOT,text=True).strip(),'rustc':subprocess.check_output(['rustc','--version'],text=True).strip(),'cargo':subprocess.check_output(['cargo','--version'],text=True).strip(),'R04':target(BASE/'AUDIT_TARGET_002.json'),'supplemental':[]}
assert result['R04']['target']['sha256']=='12dae8985bcb07f80f003e654d4076d4528683c3fe98631300300f39fd693d91'
for topic,t,a in [('minimal_effect_contract','005','005'),('pack_generation','005','004'),('tool_ports','003','003')]:
 b=ROOT/f'core_enhanced/docs/goals/content_authoring_{topic}/execution/single_goal';r=b/f'audit-evidence/r04_successor_{a}/AUDIT_OUTPUTS.json';j=json.load(open(r));s=target(b/f'AUDIT_TARGET_{t}.json');s['report']=identity(b/f'COMPLETION_AUDIT_{a}.md');s['registry']=identity(r);s['registered_outputs']=len(j['files']);s['output_errors']=[]
 for x in j['files']:
  e=check(x)
  if e:s['output_errors'].append(e)
 result['supplemental'].append(s)
result['pass']=all(not t['errors'] and not t.get('output_errors') for t in [result['R04']]+result['supplemental']) and result['head']=='f318414fb0156a67e04c5479000c2a2683fc0c09' and result['branch']=='cleanup/simulator-dirty-worktree-20260803' and result['rustc']=='rustc 1.91.1 (ed61e7d7e 2025-11-07)' and result['cargo']=='cargo 1.91.1 (ea2d97820 2025-10-10)'
(OUT/sys.argv[1]).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
