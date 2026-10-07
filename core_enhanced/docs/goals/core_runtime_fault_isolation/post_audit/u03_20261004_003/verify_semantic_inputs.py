"""Identify unchanged inherited assertion bodies in the actual predecessor source archive."""
from pathlib import Path
import json,tarfile,re,hashlib
R=Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_fault_isolation';A=Path(__file__).parent
C=D/'execution/single_goal/corrections/f02_origin_001';P=R/'core_enhanced/docs/goals/core_runtime_safety_observability/execution/master/followups/n02_001/target/master_002'
def j(p):return json.loads(p.read_text())
def h(b):return hashlib.sha256(b).hexdigest()
def write(n,x):(A/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
m=j(C/'target/u03_003/MANIFEST.json');old=j(P/'MANIFEST.json');pred=j(C/'PREDECESSOR_IMPACT.json')
oldfiles={x['path']:x for x in old['product_source']['files']};changes=[x['path'] for x in m['product_source']['files'] if oldfiles.get(x['path'])!=x]
with tarfile.open(P/'SOURCE_AFTER.tar.gz') as tf:
 oldrust={x.name:tf.extractfile(x).read().decode() for x in tf.getmembers() if x.isfile() and x.name.endswith('.rs')}
def fnbody(s,name):
 match=re.search(r'(?m)^\s*(?:async\s+)?fn\s+'+re.escape(name)+r'\s*\(',s)
 if not match:return None
 start=match.start();i=s.index('{',match.end());depth=0
 while i<len(s):
  if s.startswith('//',i):
   end=s.find('\n',i);i=len(s) if end<0 else end+1;continue
  if s.startswith('/*',i):
   nesting=1;i+=2
   while nesting:
    if s.startswith('/*',i):nesting+=1;i+=2
    elif s.startswith('*/',i):nesting-=1;i+=2
    else:i+=1
   continue
  raw=re.match(r'(?:br|r)(#{0,30})"',s[i:])
  if raw:
   end=s.index('"'+raw.group(1),i+raw.end());i=end+1+len(raw.group(1));continue
  if s[i]=='"':
   i+=1
   while s[i]!='"':i+=2 if s[i]=='\\' else 1
   i+=1;continue
  char=re.match(r"'(?:\\.|[^'\\])'",s[i:])
  if char:i+=char.end();continue
  if s[i]=='{':depth+=1
  elif s[i]=='}':
   depth-=1
   if depth==0:return s[start:i+1].lstrip()
  i+=1
 raise AssertionError(name)
selectors={s for r in pred['conditions'] for s in r.get('selectors',[])}
selectors.update(s for r in pred['u02_validation_selector_binding'] for k in ['core_selectors','host_selectors','observed_selectors'] for s in r.get(k,[]))
rows=[]
for selector in sorted(selectors):
 name=selector.split('::')[-1];found=[]
 for path,text in oldrust.items():
  if re.search(r'\bfn\s+'+re.escape(name)+r'\s*\(',text):found.append((path,fnbody(text,name)))
 assert len(found)==1,(selector,[x[0] for x in found])
 path,body=found[0];now=fnbody((R/path).read_text(),name)
 assert body==now,(selector,path)
 rows.append(dict(selector=selector,path=path,body_sha256=h(body.encode()),unchanged_from_u02=True))
write('INHERITED_SELECTOR_SOURCE.json',dict(predecessor_manifest_sha256=h((P/'MANIFEST.json').read_bytes()),current_manifest_sha256=h((C/'target/u03_003/MANIFEST.json').read_bytes()),selectors=rows,cumulative_product_delta=changes,assessment='Unchanged assertion bodies support reuse of original test meanings. Their current executions and changed production-owner applicability are independently assessed in PREDECESSOR_REVIEW.json; this file alone is not a semantic verdict.'))
print(len(rows),'original selector bodies unchanged; cumulative product delta',len(changes),flush=True)
