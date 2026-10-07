"""Verify handoff links, archived delta and pause publication without product tests."""
import hashlib,json,pathlib,re,subprocess,sys,tarfile,tempfile,urllib.parse
ROOT=pathlib.Path.cwd();D=pathlib.Path(__file__).resolve().parent
H=ROOT/'core_enhanced/docs/goals/core_battle_starter_tactics'
sys.path.insert(0,str(H/'tools'))
from run import identity
sha=lambda b:hashlib.sha256(b).hexdigest()
state=json.loads((D/'STATE.json').read_text());product=json.loads((D/'PRODUCT_CURRENT.json').read_text())
assert identity()==product,'product changed after user stop'
rows=state['changed_product_files'];old=json.loads((H/'execution/single_goal/baseline/SOURCE_BEFORE.json').read_text());old={r['path']:r for r in old['files']}
with tarfile.open(D/'PRODUCT_DELTA.tar.gz','r:gz') as delta:
 for row in rows:assert sha(delta.extractfile(row['path']).read())==row['sha256']
with tempfile.TemporaryDirectory(prefix='u06-handoff-patch-') as tmp:
 tmp=pathlib.Path(tmp)
 with tarfile.open(H/'execution/single_goal/baseline/SOURCE_BEFORE.tar.gz','r:gz') as before:
  for row in rows:
   if row['path'] in old:
    p=tmp/row['path'];p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(before.extractfile(row['path']).read())
 for reverse in [False,True]:
  command=['patch','--batch','-p1','-d',str(tmp),'-i',str(D/'PRODUCT_DELTA.patch')]+(['-R'] if reverse else [])
  result=subprocess.run(command,capture_output=True,text=True)
  assert result.returncode==0,result.stdout+result.stderr
  for row in rows:
   p=tmp/row['path']
   if reverse and row['path'] not in old:assert not p.exists()
   else:assert sha(p.read_bytes())==(old[row['path']]['sha256'] if reverse else row['sha256'])
manual=ROOT/'core_enhanced/docs/roadmaps/core_first/execution/HANDOFF_20261005.ko.md'
links=[]
for link in re.findall(r'\]\(([^)]+)\)',manual.read_text()):
 if '://' in link or link.startswith('#'):continue
 p=(manual.parent/urllib.parse.unquote(link.split('#')[0])).resolve()
 assert p.exists(),link
 links.append(link)
docs=[]
with tarfile.open(D/'DOCUMENTS_BEFORE.tar.gz','r:gz') as before:
 for path in state['document_paths_before']:
  docs.append({'path':path,'sha256_before':sha(before.extractfile(path).read()),'sha256_after':sha((ROOT/path).read_bytes())})
record={'status':'paused_by_user_handoff_published','product_unchanged_since_pause_capture':identity()==product,'product_sha256':product['sha256'],'product_files':len(product['files']),'u06_changed_product_files':len(rows),'delta_archive_verified':True,'delta_patch_forward_and_reverse_verified':True,'handoff_local_links_verified':len(links),'handoff':{'path':str(manual.relative_to(ROOT)),'sha256':sha(manual.read_bytes())},'document_updates':docs,'artifact_sha256':{name:sha((D/name).read_bytes()) for name in ['STATE.json','PRODUCT_CURRENT.json','PRODUCT_DELTA.patch','PRODUCT_DELTA.tar.gz','DOCUMENTS_BEFORE.tar.gz','capture.py','verify.py']},'product_tests_after_user_stop':False,'new_independent_audit':False,'implementation_complete':False}
with (D/'PUBLICATION_RECORD.json').open('x') as f:json.dump(record,f,indent=2,ensure_ascii=False);f.write('\n')
print(json.dumps({k:v for k,v in record.items() if k not in ['document_updates','artifact_sha256','handoff']},ensure_ascii=False))
