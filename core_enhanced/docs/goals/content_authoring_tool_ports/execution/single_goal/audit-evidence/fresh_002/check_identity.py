from pathlib import Path
import json,hashlib,subprocess,datetime,sys
ROOT=Path('/mnt/f/work/simulator'); P=ROOT/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal'; OUT=P/'audit-evidence/fresh_002'
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
def check(rows,base=ROOT):
 bad=[]
 for r in rows:
  p=base/r['path']; got=digest(p)
  if got!=r['sha256'] or (r['sha256'] is None and p.exists()) or ('bytes' in r and (not p.exists() or p.stat().st_size!=r['bytes'])): bad.append({'path':str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p),'expected':r['sha256'],'actual':got})
 return {'count':len(rows),'absences':sum(r['sha256'] is None for r in rows),'differences':bad}
def inventories(a):
 out=[]
 for d in a['external_actual']:
  b=ROOT/d['directory']; expected={r['path'] for r in d['files']}; actual={str(p.relative_to(b)) for p in b.rglob('*') if p.is_file()};out.append({'directory':d['directory'],**check(d['files'],b),'extra':sorted(actual-expected),'missing':sorted(expected-actual)})
 return out
a=json.loads((P/'AUDIT_TARGET_002.json').read_text()); out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'stage':sys.argv[1],'target_sha256':digest(P/'AUDIT_TARGET_002.json'),'expected_target_sha256':'5d4a4771cac412e41c6bdaf29d8b0e10240735bab68cca7a19dc2a4936715029','identities':{},'manifest':check(a['manifest']),'external_actual':inventories(a),'default_release_binaries':check(a['default_release_binaries']),'validation_inputs':check(json.loads((ROOT/a['validation_inputs']).read_text())['files']),'predecessors':[]}
for key,cmd,expected in [('head',['git','rev-parse','HEAD'],a['head']),('branch',['git','branch','--show-current'],a['branch']),('rustc',['rustc','--version'],a['toolchain']['rustc']),('cargo',['cargo','--version'],a['toolchain']['cargo']),('environment',['uname','-a'],a['toolchain']['environment'])]:
 actual=subprocess.check_output(cmd,cwd=ROOT,text=True).strip();out['identities'][key]={'actual':actual,'expected':expected,'equal':actual==expected}
for pr,reg in zip(a['predecessors'],['r03_successor_004','r03_successor_003']):
 old=json.loads((ROOT/pr['target']).read_text()); registry=ROOT/pr['report'];registry=registry.parent/'audit-evidence'/reg/'AUDIT_OUTPUTS.json'; rr=json.loads(registry.read_text()); out['predecessors'].append({'goal_set_ref':pr['goal_set_ref'],'target_sha256':digest(ROOT/pr['target']),'target_expected':pr['target_sha256'],'report_sha256':digest(ROOT/pr['report']),'report_expected':pr['report_sha256'],'manifest':check(old['manifest']),'external_actual':inventories(old),'binaries':check(old['default_release_binaries']),'registry_sha256':digest(registry),'actual_outputs':check(rr['files'])})
(OUT/(sys.argv[1]+'_identity.json')).write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'target_equal':out['target_sha256']==out['expected_target_sha256'],'identities':out['identities'],'manifest':out['manifest'],'external_directories':len(out['external_actual']),'external_files':sum(x['count'] for x in out['external_actual']),'external_errors':[x for x in out['external_actual'] if x['differences'] or x['extra'] or x['missing']],'binaries':out['default_release_binaries'],'inputs':out['validation_inputs'],'predecessors':[{'goal':x['goal_set_ref'],'manifest':x['manifest'],'binaries':x['binaries'],'actual_outputs':x['actual_outputs'],'external_errors':[d for d in x['external_actual'] if d['differences'] or d['extra'] or d['missing']]} for x in out['predecessors']]},indent=2))
