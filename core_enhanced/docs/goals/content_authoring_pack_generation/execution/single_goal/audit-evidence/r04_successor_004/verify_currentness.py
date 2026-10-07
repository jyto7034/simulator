import json,hashlib,subprocess,sys,datetime
from pathlib import Path
ROOT=Path('/mnt/f/work/simulator'); BASE=ROOT/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal'; OUT=BASE/'audit-evidence/r04_successor_004'
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
def ident(e,p=None):
 p=p or ROOT/e['path']; exists=p.is_file(); absent=e.get('sha256') is None
 actual={'exists':exists,'sha256':sha(p) if exists else None,'bytes':p.stat().st_size if exists else 0}
 return {'path':str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p),'expected':e,'actual':actual,'pass':(not p.exists()) if absent else exists and actual['sha256']==e['sha256'] and actual['bytes']==e.get('bytes',actual['bytes'])}
def verify(path):
 t=json.loads(path.read_text()); checks=[ident(x) for x in t['manifest']]; ext=[]
 for d in t['external_actual']:
  p=ROOT/d['directory']; actual=sorted(str(q.relative_to(p)) for q in p.rglob('*') if q.is_file()) if p.exists() else []
  expected=sorted(x['path'] for x in d['files']); ext.append({'directory':d['directory'],'inventory_pass':actual==expected,'missing':sorted(set(expected)-set(actual)),'added':sorted(set(actual)-set(expected)),'files':[ident(x,p/x['path']) for x in d['files']]})
 extra=[]
 def walk(x):
  if isinstance(x,dict):
   if 'path' in x and 'sha256' in x:extra.append(ident(x))
   else:
    for v in x.values():walk(v)
  elif isinstance(x,list):
   for v in x:walk(v)
 for k,v in t.items():
  if k not in ['manifest','external_actual']:walk(v)
 env={k:subprocess.check_output(c,cwd=ROOT,text=True).strip() for k,c in {'head':['git','rev-parse','HEAD'],'branch':['git','branch','--show-current'],'rustc':['rustc','--version'],'cargo':['cargo','--version']}.items()}
 envok=env['head']==t['head'] and env['branch']==t['branch'] and all(env[k]==v for k,v in t['toolchain'].items())
 allchecks=checks+extra+[c for d in ext for c in d['files']]
 return {'target':str(path.relative_to(ROOT)),'target_sha256':sha(path),'record_id':t['record_id'],'manifest_members':len(checks),'required_absences':sum(x['expected']['sha256'] is None for x in checks),'external_directories':len(ext),'external_files':sum(len(d['files']) for d in ext),'default_binaries':len(t['default_release_binaries']),'manifest_checks':checks,'external_checks':ext,'extra_checks':extra,'environment':env,'gex_revision':t['gex_revision'],'environment_pass':envok,'pass':all(c['pass'] for c in allchecks) and all(d['inventory_pass'] for d in ext) and envok and t['gex_revision']=='2026-09-09-gex-r1'}
target=BASE/'AUDIT_TARGET_005.json';t=json.loads(target.read_text());result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'command':'python3 '+str(Path(__file__).relative_to(ROOT))+' '+sys.argv[1],'r02':verify(target)}
r=t['required_predecessors'][0];result['r01']=verify(ROOT/r['target']['path']);registry=json.loads((ROOT/r['outputs']['path']).read_text());result['r01_outputs']=[ident(x) for x in registry['files']];result['r01_registered_count']=len(registry['files']);result['r01_verdict']=registry['verdict'];result['target_self_pass']=sha(target)=='a090b1856086f1c98c5d1630bb4464deb28b244eaed950514ce1132300fda57b';result['pass']=result['r02']['pass'] and result['r01']['pass'] and all(x['pass'] for x in result['r01_outputs']) and result['target_self_pass']
(OUT/(sys.argv[1]+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['r02','r01','r01_outputs']}));print([(k,{x:result[k][x] for x in ['manifest_members','required_absences','external_directories','external_files','pass']}) for k in ['r02','r01']]);print('failures',[(k,x['path']) for k in ['r02','r01'] for x in result[k]['manifest_checks']+result[k]['extra_checks'] if not x['pass']]);sys.exit(0 if result['pass'] else 1)
