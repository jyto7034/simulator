from pathlib import Path
import tempfile,shutil,subprocess,json,hashlib,time
R=Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal';O=B/'audit-evidence/fresh_001';E=B/'evidence'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
T=Path(tempfile.mkdtemp(prefix='catp-r03-fresh-audit-source-')); original=E/'COMBAT_EXTERNAL_001/fixture/a/official'
commands=[]
for case in ['A','B']:
 dest=T/case/'official';shutil.copytree(original,dest)
 path=dest/'data/skills/base.ron';path.write_bytes(b'// audit byte identity '+case.encode()+b'\n'+path.read_bytes())
 for op in ['validate','simulate']:
  cmd=[str(R/'target/release/author_content'),op,str(T/case),str(E/'COMBAT_EXTERNAL_001/fixture/selection.json')]
  if op=='simulate':cmd.append(str(E/'COMBAT_EXTERNAL_002/scenario.json'))
  before=time.monotonic();r=subprocess.run(cmd,cwd=R,capture_output=True,timeout=30); name=case+'_'+op
  (O/(name+'.stdout')).write_bytes(r.stdout);(O/(name+'.stderr')).write_bytes(r.stderr)
  commands.append({'name':name,'command':cmd,'exit_code':r.returncode,'seconds':time.monotonic()-before,'stdout_sha256':hashlib.sha256(r.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(r.stderr).hexdigest()})
  assert r.returncode==0,(name,r.stderr)
va=json.loads((O/'A_validate.stdout').read_text());vb=json.loads((O/'B_validate.stdout').read_text());sa=[json.loads(x) for x in (O/'A_simulate.stdout').read_text().splitlines()];sb=[json.loads(x) for x in (O/'B_simulate.stdout').read_text().splitlines()]
assert va['identity']==vb['identity'] and va['source_receipt']!=vb['source_receipt'];assert len(sa)==len(sb)==15;assert sa==sb
source_diffs=[{'a':a,'b':b} for a,b in zip(va['source_receipt']['files'],vb['source_receipt']['files']) if a!=b]
r={'scope':'Focused new audit-local default-release execution; only isolated copied fixtures gained equal-length inert comments; no frozen input or binary changed','temporary_directory':str(T),'binary_sha256':sha(R/'target/release/author_content'),'scenario_sha256':sha(E/'COMBAT_EXTERNAL_002/scenario.json'),'selection_sha256':sha(E/'COMBAT_EXTERNAL_001/fixture/selection.json'),'commands':commands,'same_semantic_identity':va['identity']==vb['identity'],'different_exact_source_receipt':va['source_receipt']!=vb['source_receipt'],'source_receipt_differences':source_diffs,'simulation_documents':len(sa),'simulation_byte_identical':(O/'A_simulate.stdout').read_bytes()==(O/'B_simulate.stdout').read_bytes(),'simulation_initial_keys':list(sa[0]['payload']),'simulation_receipt_kinds':[x['payload']['kind'] for x in sa],'actual_nonzero_sourced_hp':sa[-1]['payload']['observation']['last_terminal']['mandatory']['latest_nonzero_sourced_hp'],'fixture_inventory':{str(p.relative_to(T)):sha(p) for p in T.rglob('*') if p.is_file()}}
(O/'source_receipt_probe.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items() if k not in ['commands','fixture_inventory']},indent=2))
