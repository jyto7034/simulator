import json,pathlib,hashlib,subprocess,sys,datetime
ROOT=pathlib.Path('/mnt/f/work/simulator'); OUT=pathlib.Path(__file__).resolve().parent
TARGET=ROOT/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/AUDIT_TARGET_003.json'
EXPECT='a9e68225527c259f5c1eca054b12a92982139ed041928001f436889b6c7fb16c'
HASH_CACHE={}
def sha(p):
 stat=p.stat();key=(str(p),stat.st_size,stat.st_mtime_ns)
 if key in HASH_CACHE:return HASH_CACHE[key]
 h=hashlib.sha256()
 with p.open('rb') as f:
  for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
 value=h.hexdigest();HASH_CACHE[key]=value;return value
def check(m):
 p=ROOT/m['path']; expected=m.get('sha256')
 if expected is None:return {'path':m['path'],'ok':not p.exists(),'absence':True}
 return {'path':m['path'],'ok':p.is_file() and sha(p)==expected and p.stat().st_size==m['bytes'],'sha256':sha(p) if p.is_file() else None}
def verify(d):
 members=[check(m) for m in d['manifest']]; dirs=[]
 for e in d['external_actual']:
  base=ROOT/e['directory'];actual={str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()};expected={m['path'] for m in e['files']}
  rows=[check(dict(m,path=str(pathlib.Path(e['directory'])/m['path']))) for m in e['files']]
  dirs.append({'directory':e['directory'],'files':len(rows),'extra':sorted(actual-expected),'missing':sorted(expected-actual),'failures':[r for r in rows if not r['ok']]})
 binaries=[check(m) for m in d['default_release_binaries']]
 return {'manifest_members':len(members),'required_absences':sum(r.get('absence',False) for r in members),'manifest_failures':[r for r in members if not r['ok']], 'external_inventories':len(dirs),'external_files':sum(r['files'] for r in dirs),'inventory_results':dirs,'binaries':binaries}
d=json.loads(TARGET.read_text());r=verify(d);r['at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();r['target_sha256']=sha(TARGET);r['target_self_ok']=r['target_sha256']==EXPECT
r['environment']={k:subprocess.check_output(c,cwd=ROOT,text=True).strip() for k,c in {'head':['git','rev-parse','HEAD'],'branch':['git','branch','--show-current'],'rustc':['rustc','--version'],'cargo':['cargo','--version']}.items()};r['environment_ok']=all(r['environment'][k]==d[k] for k in ['head','branch']) and all(r['environment'][k]==v for k,v in d['toolchain'].items())
r['gex_revision_ok']='Document Revision: `'+d['gex_revision']+'`' in (ROOT/'core_enhanced/docs/goal_execution_pipeline(1).md').read_text();r['predecessors']=[]
for pred in d['required_predecessors']:
 refs=[check(pred[k]) for k in ['target','report','outputs']]; reg=json.loads((ROOT/pred['outputs']['path']).read_text());outputs=[check(x) for x in reg['files']];pd=json.loads((ROOT/pred['target']['path']).read_text())
 r['predecessors'].append({'target':pred['target']['path'],'registered_verdict':reg['verdict'],'references':refs,'registered_outputs':len(outputs),'output_failures':[x for x in outputs if not x['ok']],'currentness':verify(pd)})
def good(x):return not x['manifest_failures'] and all(not q['extra'] and not q['missing'] and not q['failures'] for q in x['inventory_results']) and all(q['ok'] for q in x['binaries'])
r['ok']=good(r) and r['target_self_ok'] and r['environment_ok'] and r['gex_revision_ok'] and all(all(q['ok'] for q in p['references']) and not p['output_failures'] and good(p['currentness']) for p in r['predecessors']);(OUT/(sys.argv[1]+'.json')).write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items() if k not in ['inventory_results','predecessors']},indent=2));print('PREDECESSORS',[(p['registered_outputs'],good(p['currentness']),p['output_failures']) for p in r['predecessors']]);sys.exit(0 if r['ok'] else 1)
