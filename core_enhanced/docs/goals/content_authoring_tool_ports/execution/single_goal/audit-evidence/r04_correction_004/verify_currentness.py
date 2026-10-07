import pathlib,json,hashlib,subprocess,sys,datetime
ROOT=pathlib.Path('/mnt/f/work/simulator')
OUT=pathlib.Path(__file__).parent
TARGET=ROOT/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/AUDIT_TARGET_004.json'
EXPECTED='b59521416b76bec8487a9d9353645fc7a5e4bceb0b05818a179e78f2c1197cf1'
errors=[]
def identity(p):
 b=p.read_bytes();return {'path':str(p.relative_to(ROOT)),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def check(m,base=ROOT):
 p=base/m['path']
 if m.get('sha256') is None:
  if p.exists():errors.append(['required_absence',str(p)])
 elif not p.is_file():errors.append(['missing',str(p)])
 else:
  b=p.read_bytes()
  if hashlib.sha256(b).hexdigest()!=m['sha256'] or len(b)!=m['bytes']:errors.append(['identity',str(p)])
def target(p,sha):
 got=identity(p)
 if got['sha256']!=sha:errors.append(['target_self',str(p)])
 d=json.loads(p.read_text())
 for m in d['manifest']:check(m)
 for e in d['external_actual']:
  base=ROOT/e['directory']; actual={str(x.relative_to(base)) for x in base.rglob('*') if x.is_file()};expected={m['path'] for m in e['files']}
  if actual!=expected:errors.append(['inventory',str(base),sorted(actual-expected),sorted(expected-actual)])
  for m in e['files']:check(m,base)
 for m in d['default_release_binaries']:check(m)
 return dict(identity=got,members=len(d['manifest']),absences=sum(m.get('sha256') is None for m in d['manifest']),external_inventories=len(d['external_actual']),external_files=sum(len(e['files']) for e in d['external_actual']),binaries=len(d['default_release_binaries']))
d=json.loads(TARGET.read_text());result={'captured_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'target':target(TARGET,EXPECTED),'predecessors':[]}
for pre in d['required_predecessors']:
 for k in ('target','report','outputs'):check(pre[k])
 reg=json.loads((ROOT/pre['outputs']['path']).read_text())
 for m in reg['outputs']:check(m)
 result['predecessors'].append(dict(target=target(ROOT/pre['target']['path'],pre['target']['sha256']),report=pre['report'],registry=pre['outputs'],registered_outputs=len(reg['outputs']),verdict=reg.get('verdict')))
result['head']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
result['branch']=subprocess.check_output(['git','branch','--show-current'],cwd=ROOT,text=True).strip()
result['toolchain']={k:subprocess.check_output([k,'--version'],cwd=ROOT,text=True).strip() for k in ('rustc','cargo')}
if result['head']!=d['head'] or result['branch']!=d['branch'] or result['toolchain']!=d['toolchain']:errors.append(['environment'])
result['errors']=errors;result['currentness']='current / unchanged' if not errors else 'stale'
(OUT/(sys.argv[1]+'.json')).write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
sys.exit(bool(errors))
