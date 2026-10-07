import pathlib,json,hashlib,subprocess,datetime,sys
ROOT=pathlib.Path('/mnt/f/work/simulator'); OUT=pathlib.Path(__file__).resolve().parent
base='core_enhanced/docs/goals/'
specs=[('R04','content_authoring_runtime_facts','003','a599773bc1f084a46c55b01ddf3a37e2ab47f10323080d5c67df3bceeff6b4c8',None,None,None),('R01','content_authoring_minimal_effect_contract','006','a1f87aded22960fce13999d609c3d00f10ab4aa0c86423e63108313bc15b03af','006','r04_correction_006','2b75f9ffc58fc07b72a0c0a49c0ffb23ed9c77d06b37424231794db441d73386'),('R02','content_authoring_pack_generation','006','3e57d1f541f7f1ac94ee9638c8cf2f6ff6821e2c81668350da698c2921f56ee1','005','r04_correction_005','b9842ae6c7f94cd0b995fd9829a66260f23514f69c84b8eca40ee3ca4eaa6896'),('R03','content_authoring_tool_ports','004','b59521416b76bec8487a9d9353645fc7a5e4bceb0b05818a179e78f2c1197cf1','004','r04_correction_004','9a76dfd1792907de15788906932f6eff13a00b8c3e6a73cf26dad9498f96528d')]
cache={};errors=[]
def identity(p):
 p=pathlib.Path(p);p=p if p.is_absolute() else ROOT/p
 if p not in cache:
  if not p.is_file():cache[p]=None
  else:
   h=hashlib.sha256();n=0
   with p.open('rb') as f:
    while b:=f.read(1024*1024): h.update(b);n+=len(b)
   cache[p]={'path':str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p),'bytes':n,'sha256':h.hexdigest()}
 return cache[p]
def check(e,p=None):
 p=p or e['path'];a=identity(p)
 if e.get('sha256') is None:
  if pathlib.Path(ROOT/p).exists():errors.append({'unexpected_exists':str(p)})
 elif a is None or a['sha256']!=e['sha256'] or ('bytes' in e and a['bytes']!=e['bytes']):errors.append({'expected':e,'actual':a})
 return a
results=[]
for label,d,t,h,r,ad,rh in specs:
 q=pathlib.Path(base+d+'/execution/single_goal'); tp=q/f'AUDIT_TARGET_{t}.json'; check({'path':str(tp),'sha256':h});a=json.loads((ROOT/tp).read_text());
 for e in a['manifest']:check(e)
 ex=[]
 for inv in a['external_actual']:
  directory=ROOT/inv['directory'];expected={e['path'] for e in inv['files']};actual={str(p.relative_to(directory)) for p in directory.rglob('*') if p.is_file()}
  if expected!=actual:errors.append({'directory':str(directory),'missing':sorted(expected-actual),'extra':sorted(actual-expected)})
  for e in inv['files']:check(e,str(pathlib.Path(inv['directory'])/e['path']))
  ex.append({'directory':inv['directory'],'files':len(expected),'exact':expected==actual})
 for e in a['default_release_binaries']:check(e)
 z={'label':label,'target':identity(tp),'record':a['record_id'],'members':len(a['manifest']),'absences':sum(x['sha256'] is None for x in a['manifest']),'external_inventories':ex,'external_files':sum(v['files'] for v in ex),'binaries':[identity(e['path']) for e in a['default_release_binaries']]}
 if r:
  reg=q/'audit-evidence'/ad/'AUDIT_OUTPUTS.json';check({'path':str(reg),'sha256':rh});b=json.loads((ROOT/reg).read_text())
  for e in b['outputs']:check(e)
  z.update(registry=identity(reg),report=identity(q/f'COMPLETION_AUDIT_{r}.md'),outputs=len(b['outputs']),verdict=b['verdict'])
 results.append(z)
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip();branch=subprocess.check_output(['git','branch','--show-current'],cwd=ROOT,text=True).strip()
toolchain={v:subprocess.check_output([v,'--version'],text=True).strip() for v in ['rustc','cargo']}
if head!='f318414fb0156a67e04c5479000c2a2683fc0c09':errors.append({'head':head})
if branch!='cleanup/simulator-dirty-worktree-20260803':errors.append({'branch':branch})
for v in ['rustc','cargo']:
 if not toolchain[v].startswith(v+' 1.91.1 '):errors.append({'toolchain':toolchain})
result={'stage':sys.argv[1],'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reviewer':'/root/r04_correction_fresh_completion_audit','head':head,'branch':branch,'toolchain':toolchain,'targets':results,'distinct_files_hashed':len(cache),'errors':errors,'currentness':'current / unchanged' if not errors else 'stale'}
(OUT/f'currentness_{sys.argv[1]}.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'stage':sys.argv[1],'counts':[{k:z[k] for k in ['label','members','absences','external_files']} for z in results],'errors':errors,'distinct_files_hashed':len(cache)},indent=2))
