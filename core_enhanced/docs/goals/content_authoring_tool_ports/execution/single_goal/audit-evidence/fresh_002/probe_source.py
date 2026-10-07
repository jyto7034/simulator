from pathlib import Path
import subprocess,json,hashlib,shutil,tempfile,time
R=Path('/mnt/f/work/simulator');P=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal';O=P/'audit-evidence/fresh_002';T=Path(tempfile.mkdtemp(prefix='catp-r03-audit002-'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def inv(p):return {str(x.relative_to(p)):sha(x) for x in p.rglob('*') if x.is_file()}
fixture=T/'fixture';shutil.copytree(P/'audit-evidence/fresh_001/reproduction_inputs',fixture)
# Original auditor's actual A/B fixture and finite scenario are retained, not regenerated.
before=inv(fixture); binary=P/'evidence/SOURCE_RECEIPT_EXTERNAL_001/source_receipt_binary'; assert sha(binary)=='0339a9fec0c0e573bb54a23b534b8049d80a8a3f8966e6c90837f6762763a274'
commands=[]
def run(name,cmd):
 start=time.monotonic();p=subprocess.run([str(x) for x in cmd],cwd=R,capture_output=True);(O/(name+'.stdout')).write_bytes(p.stdout);(O/(name+'.stderr')).write_bytes(p.stderr);commands.append({'name':name,'command':[str(x) for x in cmd],'exit':p.returncode,'seconds':time.monotonic()-start,'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()});assert p.returncode==0,(name,p.stderr);return p.stdout
run('source_api',[binary,fixture,fixture/'scenario.json',T/'api']);assert inv(fixture)==before
shutil.copytree(T/'api',O/'api')
cli=R/'target/release/author_content'; vals={};docs={}
for gen in ['A','B']:
 vals[gen]=json.loads(run(gen+'_validate',[cli,'validate',fixture/gen,fixture/'selection.json']))
 for mode in ['off','on']:
  cmd=[cli,'simulate',fixture/gen,fixture/'selection.json',fixture/'scenario.json'];
  if mode=='on':cmd+=['--trace-output',O/(gen+'_trace.ndjson')]
  raw=run(gen+'_'+mode,cmd);assert raw==(O/'api'/mode/(gen.lower()+'.ndjson')).read_bytes(); docs[gen]=[json.loads(x) for x in raw.splitlines()];assert len(docs[gen])==15
 source=json.loads((O/'api/off'/(gen.lower()+'_source.json')).read_text())
 for d in docs[gen]:assert d['payload']['source_receipt']==source
assert docs['A']!=docs['B']
stripped={}
for gen in docs:
 stripped[gen]=json.loads(json.dumps(docs[gen]))
 for d in stripped[gen]:del d['payload']['source_receipt']
assert stripped['A']==stripped['B'];assert (O/'A_trace.ndjson').read_bytes()==(O/'B_trace.ndjson').read_bytes()
shutil.copytree(fixture,O/'fixture'); result={'passed':True,'temp':str(T),'binary_sha256':sha(binary),'cli_sha256':sha(cli),'fixture_before':before,'fixture_after':inv(fixture),'commands':commands,'source_documents_per_generation':15,'all_except_source_receipt_equal':True,'cli_api_full_bytes_equal':True,'a_b_trace_bytes_equal':True,'api_result':json.loads((O/'api/receipt.json').read_text())};(O/'probe_source_result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['fixture_before','fixture_after','api_result','commands']}));print('commands',len(commands));print((O/'api/receipt.json').read_text()[:500])
