import json, hashlib, pathlib, subprocess, datetime, sys
ROOT=pathlib.Path('/mnt/f/work/simulator'); BASE=ROOT/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal'; TARGET=BASE/'AUDIT_TARGET_005.json'; OUT=BASE/'audit-evidence/r04_successor_005'
expected_self='631c3aa01a8ef7b3303bc9b979e572c81fdb0e11e2d41915112baff98e548af2'
def digest(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
 return h.hexdigest()
d=json.loads(TARGET.read_text()); findings=[]; checked=[]
def check(m,base=ROOT,role='manifest'):
 p=base/m['path']; actual={'path':str(p.relative_to(ROOT)),'role':role}
 if m.get('sha256') is None:
  actual['absent']=not p.exists()
  if not actual['absent']: findings.append(actual)
 else:
  actual.update(sha256=digest(p) if p.is_file() else None,bytes=p.stat().st_size if p.is_file() else None)
  if actual['sha256']!=m['sha256'] or actual['bytes']!=m['bytes']:findings.append({'expected':m,'actual':actual})
 checked.append(actual)
for m in d['manifest']:check(m)
inventories=[]
for ext in d['external_actual']:
 base=ROOT/ext['directory']; expected=sorted(m['path'] for m in ext['files']); actual=sorted(str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()); row={'directory':ext['directory'],'expected_count':len(expected),'actual_count':len(actual),'missing':sorted(set(expected)-set(actual)),'extra':sorted(set(actual)-set(expected))}; inventories.append(row)
 if row['missing'] or row['extra']:findings.append(row)
 for m in ext['files']:check(m,base,'external_actual')
for m in d['default_release_binaries']:check(m,role='default_release_binary')
for k in ['validation_inputs','source_archive','predecessor_target','previous_report']:check(d[k],role=k)
for k,m in d['successor_implementation'].items():
 if isinstance(m,dict) and 'path' in m:check(m,role='successor_'+k)
commands={};
for k,cmd in [('head',['git','rev-parse','HEAD']),('branch',['git','branch','--show-current']),('rustc',['rustc','--version']),('cargo',['cargo','--version'])]:
 r=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True);commands[k]={'command':cmd,'exit':r.returncode,'stdout':r.stdout.strip(),'stderr':r.stderr.strip()}; expected=d[k] if k in ['head','branch'] else d['toolchain'][k]
 if r.returncode or r.stdout.strip()!=expected: findings.append({'environment':k,'expected':expected,'actual':commands[k]})
actual_self=digest(TARGET)
if actual_self!=expected_self:findings.append({'self_hash':actual_self,'expected':expected_self})
result={'stage':sys.argv[1],'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'target':str(TARGET.relative_to(ROOT)),'target_sha256':actual_self,'expected_target_sha256':expected_self,'manifest_count':len(d['manifest']),'required_absences':sum(m.get('sha256') is None for m in d['manifest']),'external_inventory_count':len(inventories),'external_file_count':sum(len(x['files']) for x in d['external_actual']),'named_binary_count':len(d['default_release_binaries']),'checked_records_count':len(checked),'commands':commands,'inventories':inventories,'checked':checked,'findings':findings,'passed':not findings}
(OUT/(sys.argv[1]+'_currentness.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['checked','inventories','commands']},indent=2));sys.exit(bool(findings))
