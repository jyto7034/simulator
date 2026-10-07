from pathlib import Path
import json,hashlib,re,tarfile,datetime
R=Path('/mnt/f/work/simulator'); G=R/'core_enhanced/docs/goals/core_resonance_lifecycle'; B=G/'execution/single_goal'; A=G/'post_audit/u05_20261004_001'; T=B/'target/u05_001'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
errors=[]; out={'time':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def ck(ok,what):
 if not ok:errors.append(what)
m=read(T/'MANIFEST.json'); c=read(B/'COVERAGE.json'); p=read(B/'PREDECESSOR_IMPACT.json'); sel=read(B/'EVIDENCE_SELECTION.json')
# Verify complete product membership, including newly appearing files in declared roots.
paths=set()
for folder in ('core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data'):
 paths.update(str(x.relative_to(R)) for x in (R/folder).rglob('*') if x.is_file())
paths.update(('Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml','core_enhanced/docs/audit/live_skill_catalog_manifest.ron','core_enhanced/docs/skills/live_content_index.md','core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'))
ck(paths=={x['path'] for x in m['product_source']['files']},'product membership')
rows=[{'path':s,'sha256':sha(R/s),'bytes':(R/s).stat().st_size} for s in sorted(paths,key=Path)]
product=hashlib.sha256(json.dumps(rows,sort_keys=True).encode()).hexdigest();ck(product==m['product_source']['sha256'],'product digest');out['product_sha256']=product
# Archive members are checked individually, not only archive digests.
archives=[]
for archive, members in [('SOURCE_BEFORE.tar.gz',m['before_files']),('SOURCE_AFTER.tar.gz',m['source_files']),('RECORDS.tar.gz',read(T/'RECORDS.json')),('REFERENCES.tar.gz',read(T/'REFERENCES.json'))]:
 with tarfile.open(T/archive) as t:
  ts=[x for x in t if x.isfile()];ck({x.name for x in ts}=={x['path'] for x in members},archive+' membership')
  expected={x['path']:x for x in members}
  for x in ts:
   data=t.extractfile(x).read();e=expected.get(x.name,{});ck(len(data)==e.get('bytes') and hashlib.sha256(data).hexdigest()==e.get('sha256'),archive+':'+x.name)
 archives.append({'archive':archive,'members':len(members),'sha256':sha(T/archive)})
for x in read(T/'EXTERNAL_ARCHIVES.json'):
 archive=R/x['path'];ck(sha(archive)==x['sha256'],'external archive '+x['path']); pm=read(archive.parent/'MANIFEST.json');ck(sha(archive.parent/'MANIFEST.json')==x['basis_manifest_sha256'],'external manifest')
 key={'SOURCE_BEFORE.tar.gz':'before_files','SOURCE_AFTER.tar.gz':'source_files'}.get(archive.name)
 members=pm[key] if key else read(archive.parent/(archive.name.replace('.tar.gz','.json')))
 with tarfile.open(archive) as t:
  ts=[z for z in t if z.isfile()];expected={z['path']:z for z in members};ck({z.name for z in ts}==set(expected),'external membership '+str(archive))
  for z in ts:
   data=t.extractfile(z).read();e=expected.get(z.name,{});ck(len(data)==e.get('bytes') and hashlib.sha256(data).hexdigest()==e.get('sha256'),'external member '+z.name)
 archives.append({'archive':str(archive.relative_to(R)),'members':len(members),'sha256':sha(archive)})
out['archives']=archives
# Check selected receipts and all actual selector successes against logs.
logs={}; receipts=[]
for name,binding in c['receipt_bindings'].items():
 f=G/'evidence'/(name+'.json');log=G/'evidence'/(name+'.log');d=read(f)
 ck(sha(f)==binding['receipt_sha256'],name+' receipt hash');ck(sha(log)==binding['log_sha256']==d['log_sha256'],name+' log hash')
 ck(d['source_before']==d['source_after']==m['product_source'],name+' full source binding')
 ck(d['source_unchanged'] and d['exit_code']==binding['raw_exit_code'],name+' outcome')
 for x in d.get('command_inputs',[]):
  ck(sha(Path(x['path']))==x['sha256'],name+' current input '+x['path']);ck(sha(G/x['archive'])==x['sha256'],name+' archived input')
 s=log.read_text();logs[name]=set(re.findall(r'^test (.+) \.\.\. ok$',s,re.M));results=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',s)
 receipts.append({'name':name,'exit_code':d['exit_code'],'command':d['command'],'tests':{'passed':sum(int(r[0]) for r in results),'failed':sum(int(r[1]) for r in results),'ignored':sum(int(r[2]) for r in results)},'receipt_sha256':sha(f),'log_sha256':sha(log)})
out['receipts']=receipts
conditions=[]
for group,items in [('U05',c['validations']),('inherited',p['conditions'])]:
 for x in items:
  selectors=x.get('current_selectors',x.get('core_selectors',[])+x.get('host_selectors',[]));bindings=[]
  for s in selectors:
   matches=[n for n in sel['suites'] if s in logs[n]]
   ck(bool(matches),x['id']+' missing selector '+s);bindings.append({'selector':s,'success_in':matches})
  ck(bool(selectors) or bool(x.get('public_receipts')),x['id']+' no execution binding');conditions.append({'group':group,'id':x['id'],'requirement':x.get('requirement'),'selectors':bindings})
out['condition_execution_bindings']=conditions
# Preserve original requirement and R06 acceptance bindings.
for x in p['original_requirement_sources']:ck(sha(R/x['path'])==x['sha256'],'original requirement '+x['path'])
ids={x['id'] for x in p['conditions']}
for x in p['r06_acceptances']:
 ck(bool(x['iva']) and set(x['iva'])<=ids,'R06 mapping '+x['acceptance'])
out['r06_acceptances']=p['r06_acceptances'];out['dimension_mapping']=c['required_dimensions']
ad=read(B/'compatibility/current_001/ADAPTATION_FINAL.json')
for x in ad['files']:
 ck(sha(R/x['source'])==x['source_sha256'],'adapter original '+x['source']);ck(sha(R/x['copy'])==x['copy_sha256'],'adapter migrated '+x['copy'])
out['adapter_original_and_migrated_files']=len(ad['files']);out['errors']=errors;out['passed']=not errors
(A/'EVIDENCE_VERIFICATION.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:v for k,v in out.items() if k not in ['receipts','condition_execution_bindings','r06_acceptances','dimension_mapping']},ensure_ascii=False,indent=2))
