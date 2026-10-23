"""Read-only independent audit checks. Outputs only beside this file."""
import ast, hashlib, json, os, pathlib, re, tarfile, datetime
A=pathlib.Path(__file__).resolve().parent
ROOT=next(p for p in A.parents if (p/'Cargo.toml').exists() and (p/'game_server').exists())
G=ROOT/'core_enhanced/docs/goals/core_mod_validation_apply';E=G/'execution/single_goal';T=E/'target/u11_current_001'
def load(p): return json.loads(pathlib.Path(p).read_text())
def sha(p):
 h=hashlib.sha256()
 with pathlib.Path(p).open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
def ref(p):
 p=pathlib.Path(p);return {'path':str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
def write(n,d): (A/n).write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
def canon(x):return hashlib.sha256(json.dumps(x,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
m=load(T/'MANIFEST.json');source=load(ROOT/m['source_manifest']['path']);selection=load(E/'finalization001/generated001/EVIDENCE_SELECTION.json');coverage=load(E/'finalization001/generated001/COVERAGE.json');prior=load(E/'finalization001/generated001/PREDECESSOR_IMPACT.json')
# The entry run hashed every member in these maps; reuse those exact current observations.
entry=load(A/'INTEGRITY_ENTRY.json');assert all(not x['mismatches'] for x in entry['checks'])
verified={str(ROOT/r['path']):r for r in m['source_files']+load(T/'RECORDS.json')}
verified.update({str(T/r['path']):r for r in m['archives_and_patches']})
refs={};ptrs=[]
def walk(x):
 if isinstance(x,dict):
  if isinstance(x.get('path'),str) and 'bytes' in x and 'sha256' in x:
   p=pathlib.Path(x['path']);p=p if p.is_absolute() else ROOT/p
   refs[(str(p),x['bytes'],x['sha256'])]=x
   if 'json_pointer' in x and 'canonical_row_sha256' in x:ptrs.append((p,x['json_pointer'],x['canonical_row_sha256']))
  for v in x.values():walk(v)
 elif isinstance(x,list):
  for v in x:walk(v)
for obj in [selection,coverage,prior,m['predecessor_independent_closure']]:walk(obj)
errors=[];hash_reused=0;hashed=0
for (p,n,h),r in refs.items():
 v=verified.get(p)
 if v and (v['bytes'],v['sha256'])==(n,h):hash_reused+=1;continue
 try:
  actual=ref(pathlib.Path(p));hashed+=1
  if (actual['bytes'],actual['sha256'])!=(n,h):errors.append({'path':p,'expected_sha256':h,'actual':actual})
 except Exception as ex:errors.append({'path':p,'error':str(ex)})
print('references',len(refs),'errors',len(errors),flush=True)
jsons={};pointer_errors=[]
for p,pointer,h in set(ptrs):
 try:
  if p not in jsons:jsons[p]=load(p)
  val=jsons[p]
  for token in pointer.split('/')[1:]:
   token=token.replace('~1','/').replace('~0','~');val=val[int(token)] if isinstance(val,list) else val[token]
  if canon(val)!=h:pointer_errors.append({'path':str(p),'pointer':pointer,'expected':h,'actual':canon(val)})
 except Exception as ex:pointer_errors.append({'path':str(p),'pointer':pointer,'error':str(ex)})
# Membership: independently read declared scanner constants; do not execute its module.
scanner=E/'test_parts/runtime001/run.py';values={}
for node in ast.parse(scanner.read_text()).body:
 if isinstance(node,ast.Assign) and len(node.targets)==1 and isinstance(node.targets[0],ast.Name):
  try:values[node.targets[0].id]=ast.literal_eval(node.value)
  except (ValueError,TypeError):pass
paths=set()
for relative in list(values['SOURCE_DIRS'])+[str(scanner.parent.relative_to(ROOT))]:
 base=ROOT/relative;tools_only=relative=='tools' or relative.endswith('/tools') or base==scanner.parent
 if not base.exists():continue
 for directory,dirs,files in os.walk(base,followlinks=False):
  dirs[:]=[d for d in dirs if d not in values['SKIP_DIRS']]
  for name in files:
   p=pathlib.Path(directory)/name
   if name.endswith(('.pyc','.pyo')) or tools_only and p.suffix not in values['TOOL_SUFFIXES']:continue
   assert p.is_file() and not p.is_symlink();paths.add(str(p.relative_to(ROOT)))
paths.update(r for r in values['BUILD_FILES'] if (ROOT/r).is_file())
paths.update(r['path'] for r in load(scanner.parent/'INPUTS.json')['files'])
paths.add('core_enhanced/docs/goals/core_relic_structural_activation/execution/single_goal/test_parts/runtime_validation001/performance001/run.py')
paths.add(str(scanner.relative_to(ROOT)));paths.add(str((scanner.parent/'INPUTS.json').relative_to(ROOT)))
expected={r['path'] for r in source['files']};membership={'count':len(paths),'expected_count':len(expected),'added':sorted(paths-expected),'missing':sorted(expected-paths),'manifest_rows_digest':hashlib.sha256(json.dumps(source['files'],sort_keys=True).encode()).hexdigest(),'expected_digest':source['sha256']}
archive_rows=[]
for name,rows in [('SOURCE_BEFORE.tar.gz',m['before_files']),('SOURCE_AFTER.tar.gz',m['source_files']),('RECORDS.tar.gz',load(T/'RECORDS.json')),('REFERENCES.tar.gz',load(T/'REFERENCES.json'))]:
 expected={r['path']:r for r in rows};seen=set();bad=[]
 with tarfile.open(T/name) as archive:
  for member in archive:
   if not member.isfile() or member.name not in expected or member.name in seen:bad.append({'member':member.name,'reason':'type/membership/duplicate'});continue
   row=expected[member.name];seen.add(member.name);h=hashlib.sha256();f=archive.extractfile(member)
   for b in iter(lambda:f.read(1048576),b''):h.update(b)
   if member.size!=row['bytes'] or h.hexdigest()!=row['sha256']:bad.append({'member':member.name,'reason':'bytes/hash'})
 bad.extend({'member':n,'reason':'missing'} for n in expected.keys()-seen)
 archive_rows.append({'archive':name,'members':len(seen),'mismatches':bad});print('archive',name,len(seen),len(bad),flush=True)
raw_rows=[]
for ident,v in selection['evidence'].items():
 if not v.get('raw'):continue
 raw=(ROOT/v['raw']['path']).read_text();receipt=load(ROOT/v['receipt']['path'])
 row={'id':ident,'raw':v['raw'],'receipt':v['receipt'],'exit_code':receipt.get('exit_code'),'child_reaped':receipt.get('child_reaped'),'named_tests':[],'classification':v.get('claim_kind',v.get('status'))}
 for case in v.get('selected_tests',[]):
  pattern=r'^test '+re.escape(case['selector'])+r' \.\.\. ok$'
  row['named_tests'].append({'selector':case['selector'],'exact_ok_marker':bool(re.search(pattern,raw,re.M))})
 raw_rows.append(row)
write('MACHINE_AUDIT.json',{'references':{'unique':len(refs),'reused_entry_hashes':hash_reused,'new_hashes':hashed,'mismatches':errors},'row_pointers':{'unique':len(set(ptrs)),'mismatches':pointer_errors},'runtime_membership':membership,'archives':archive_rows,'selected_raw_rows':raw_rows,'counts':{'named_groups':sum(bool(x['named_tests']) for x in raw_rows),'named_pass_markers':sum(len(x['named_tests']) for x in raw_rows),'other_command_groups':sum(not x['named_tests'] for x in raw_rows)},'timestamp':datetime.datetime.now(datetime.timezone.utc).isoformat()})
print('done',len(pointer_errors),'pointer errors',membership['added'],membership['missing'],flush=True)
