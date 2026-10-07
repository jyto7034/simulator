import pathlib,json,tarfile,re,hashlib,difflib
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_result_persistence';A=pathlib.Path(__file__).parent;T=H/'execution/single_goal/target/u04_001';M=json.loads((T/'MANIFEST.json').read_text());P=json.loads((H/'execution/single_goal/PREDECESSOR_IMPACT.json').read_text())
with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as t:before={m.name:t.extractfile(m).read().decode() for m in t if m.isfile() and m.name.endswith('.rs')}
after={r['path']:(R/r['path']).read_text() for r in M['source_files'] if r['path'].endswith('.rs')}
def bodies(sources,name):
 result=[]
 for p,s in sources.items():
  m=re.search(r'(?m)^(\s*)(?:async )?fn '+re.escape(name)+r'\(',s)
  if m:
   # A function closing brace at the declaration indentation includes nested helpers.
   start=m.start();indent=m[1].split('\n')[-1];end=re.search(r'(?m)^'+re.escape(indent)+r'}\s*$',s[m.end():])
   assert end,name
   body=s[start:m.end()+end.end()];result.append((p,body))
 return result
rows=[];diffs=[]
selectors=sorted(set(s for r in P['conditions'] for s in r.get('selectors',[])))
for s in selectors:
 old=bodies(before,s.split('::')[-1]);new=bodies(after,s.split('::')[-1]);same=len(old)==len(new)==1 and old[0][1]==new[0][1]
 row={'selector':s,'old_matches':len(old),'current_matches':len(new),'body_unchanged':same,'current_files':[p for p,b in new]}
 if len(old)==len(new)==1:
  row.update(old_sha256=hashlib.sha256(old[0][1].encode()).hexdigest(),current_sha256=hashlib.sha256(new[0][1].encode()).hexdigest())
  if not same:diffs.extend(difflib.unified_diff(old[0][1].splitlines(True),new[0][1].splitlines(True),fromfile='before:'+s,tofile='after:'+s))
 rows.append(row)
(A/'INHERITED_SELECTOR_SOURCE.json').write_text(json.dumps(rows,indent=2)+'\n');(A/'CHANGED_INHERITED_TESTS.diff').write_text(''.join(diffs));print('total',len(rows),'unchanged',sum(r['body_unchanged'] for r in rows));print(*[r for r in rows if not r['body_unchanged']],sep='\n')
