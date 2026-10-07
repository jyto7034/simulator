import hashlib,json,pathlib,tarfile,tempfile,subprocess,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_result_persistence';A=pathlib.Path(__file__).parent;T=H/'execution/single_goal/target/u04_003';M=json.loads((T/'MANIFEST.json').read_text())
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  while b:=f.read(1048576):h.update(b)
 return h.hexdigest()
def check_archive(path,rows):
 expected={x['path']:x for x in rows};seen=set()
 with tarfile.open(path) as t:
  for m in t:
   assert m.isfile() and m.name in expected and m.name not in seen,(path,m.name)
   e=expected[m.name];h=hashlib.sha256()
   with t.extractfile(m) as f:
    while b:=f.read(1048576):h.update(b)
   assert m.size==e['bytes'] and h.hexdigest()==e['sha256'],(path,m.name)
   seen.add(m.name)
 assert seen==set(expected)
 return {'path':str(path.relative_to(R)),'sha256':sha(path),'members':len(seen)}
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'archives':[]}
for name,rows in [('SOURCE_BEFORE',M['before_files']),('SOURCE_AFTER',M['source_files']),('RECORDS',json.loads((T/'RECORDS.json').read_text())),('REFERENCES',json.loads((T/'REFERENCES.json').read_text()))]:result['archives'].append(check_archive(T/(name+'.tar.gz'),rows))
for e in json.loads((T/'EXTERNAL_ARCHIVES.json').read_text()):
 p=R/e['path'];mt=json.loads((p.parent/'MANIFEST.json').read_text());assert sha(p)==e['sha256'];assert sha(p.parent/'MANIFEST.json')==e['basis_manifest_sha256']
 if p.name.startswith('SOURCE_'):rows=mt['before_files' if 'BEFORE' in p.name else 'source_files']
 else:rows=json.loads(p.with_name(p.name.replace('.tar.gz','.json')).read_text())
 result['archives'].append(check_archive(p,rows))
with tempfile.TemporaryDirectory(prefix='u04-audit-restore-') as tmp:
 b=pathlib.Path(tmp)
 with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as t:t.extractall(b,filter='data')
 for reverse,rows in [(False,M['source_files']),(True,M['before_files'])]:
  p=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=b,capture_output=True)
  (A/('RESTORE_REVERSE.log' if reverse else 'RESTORE_FORWARD.log')).write_bytes(p.stdout+p.stderr);assert p.returncode==0
  assert sorted(str(p.relative_to(b)) for p in b.rglob('*') if p.is_file())==sorted(r['path'] for r in rows)
  for r in rows:assert sha(b/r['path'])==r['sha256'] and (b/r['path']).stat().st_size==r['bytes']
 result['fuzz0_forward_and_reverse_exact']=True
before={r['path']:r for r in M['before_files']};after={r['path']:r for r in M['source_files']}
assert sorted(r for r in after if after[r]!=before.get(r))==sorted(M['changed'])
assert sorted(set(after)-set(before))==M['added']
assert sorted(set(before)-set(after))==M['deleted']
for key,rows in [('before_sha256',M['before_files']),('source_sha256',M['source_files'])]:assert hashlib.sha256(json.dumps(rows,sort_keys=True).encode()).hexdigest()==M[key]
result['delta_membership_exact']=True
(A/'INTEGRITY.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
