import hashlib,json,subprocess,tarfile,tempfile,sys,datetime
from pathlib import Path
R=Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_battle_starter_tactics';T=H/'execution/single_goal/target/u06_002';O=Path(__file__).parent
sha=lambda b:hashlib.sha256(b).hexdigest()
read=lambda p:json.loads(p.read_text())
m=read(T/'MANIFEST.json');errors=[]
def verify(rows):
 for r in rows:
  p=R/r['path']
  if not p.is_file() or p.stat().st_size!=r['bytes'] or sha(p.read_bytes())!=r['sha256']:errors.append(['current',r['path']])
def archive(path,rows):
 expected={r['path']:r for r in rows};seen=set()
 with tarfile.open(path) as a:
  for x in a:
   if not x.isfile() or x.name in seen or x.name not in expected:errors.append(['member',str(path),x.name]);continue
   seen.add(x.name);r=expected[x.name]
   if x.size!=r['bytes'] or sha(a.extractfile(x).read())!=r['sha256']:errors.append(['archive_bytes',str(path),x.name])
 if seen!=set(expected):errors.append(['archive_membership',str(path)])
 return len(seen)
mem=read(T/'MEMBERSHIP.json');verify(mem)
for r in m['archives_and_patches']:
 p=T/r['path']
 if len(p.read_bytes())!=r['bytes'] or sha(p.read_bytes())!=r['sha256']:errors.append(['target_file',str(p)])
records=read(T/'RECORDS.json');refs=read(T/'REFERENCES.json');external=read(T/'EXTERNAL_ARCHIVES.json')
expected_groups={}
for group,rows in [('source',m['source_files']),('records',records),('references',refs),('external_reference',external)]:
 for r in rows: expected_groups.setdefault(r['path'],[]).append(group)
if expected_groups!={r['path']:r['groups'] for r in mem}:errors.append(['membership_union'])
actual_records={str(p.relative_to(R)) for p in H.rglob('*') if p.is_file() and not {'target','post_audit','__pycache__'}&set(p.relative_to(H).parts)}
if actual_records!={r['path'] for r in records}:errors.append(['record_enumeration',sorted(actual_records-{r['path'] for r in records}),sorted({r['path'] for r in records}-actual_records)])
product=set()
for f in ['core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data']:
 product.update(str(p.relative_to(R)) for p in (R/f).rglob('*') if p.is_file())
product.update(['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml','core_enhanced/docs/audit/live_skill_catalog_manifest.ron','core_enhanced/docs/skills/live_content_index.md','core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'])
if product!={r['path'] for r in m['product_source']['files']}:errors.append(['product_enumeration'])
tracked=set(subprocess.check_output(['git','ls-files','-z'],cwd=R).decode().split('\0'));dirty=set(subprocess.check_output(['git','diff','--name-only','-z','HEAD'],cwd=R).decode().split('\0'))
for r in mem:
 kind='tracked/dirty' if r['path'] in dirty else 'tracked/clean' if r['path'] in tracked else 'untracked'
 if kind!=r['kind']:errors.append(['attribution',r['path'],kind,r['kind']])
counts={}
if sys.argv[1]=='begin':
 for n,rows in [('SOURCE_BEFORE',m['before_files']),('SOURCE_AFTER',m['source_files']),('RECORDS',records),('REFERENCES',refs)]:counts[n]=archive(T/(n+'.tar.gz'),rows)
 for e in external:
  p=R/e['path'];pm=read(p.parent/'MANIFEST.json');name=p.name
  rows=pm['before_files'] if name=='SOURCE_BEFORE.tar.gz' else pm['source_files'] if name=='SOURCE_AFTER.tar.gz' else read(p.with_name(name.replace('.tar.gz','.json')))
  counts[e['path']]=archive(p,rows)
 with tempfile.TemporaryDirectory(prefix='u06-audit-restore-') as d:
  b=Path(d)
  with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as a:a.extractall(b,filter='data')
  for rev,expected in [(False,m['source_files']),(True,m['before_files'])]:
   q=subprocess.run(['patch',*(['-R'] if rev else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=b,capture_output=True)
   (O/('reverse.log' if rev else 'forward.log')).write_bytes(q.stdout+q.stderr)
   got=sorted([{'path':str(p.relative_to(b)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in b.rglob('*') if p.is_file()],key=lambda r:r['path'])
   if q.returncode or got!=expected:errors.append(['restore',rev,q.returncode])
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'manifest_sha256':sha((T/'MANIFEST.json').read_bytes()),'product_sha256':m['product_source']['sha256'],'membership_count':len(mem),'source_count':len(m['source_files']),'record_count':len(records),'reference_count':len(refs),'archive_counts':counts,'attribution':{k:sum(r['kind']==k for r in mem) for k in ['tracked/clean','tracked/dirty','untracked']},'errors':errors}
(O/('CURRENTNESS_'+sys.argv[1].upper()+'.json')).write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
