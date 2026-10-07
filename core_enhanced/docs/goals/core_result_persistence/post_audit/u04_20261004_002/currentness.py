import pathlib,json,hashlib,subprocess,datetime,sys
R=pathlib.Path('/mnt/f/work/simulator');A=pathlib.Path(__file__).parent;T=R/'core_enhanced/docs/goals/core_result_persistence/execution/single_goal/target/u04_003'
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  while b:=f.read(1048576): h.update(b)
 return h.hexdigest()
M=json.loads((T/'MANIFEST.json').read_text());rows=json.loads((T/'MEMBERSHIP.json').read_text());bad=[]
for r in rows:
 p=R/r['path']
 if not p.is_file() or p.stat().st_size!=r['bytes'] or sha(p)!=r['sha256']:bad.append(r['path'])
artifacts=[{'path':str(p.relative_to(R)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(T.iterdir()) if p.is_file()]
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'manifest_sha256':sha(T/'MANIFEST.json'),'source_count':len(M['source_files']),'membership_count':len(rows),'mismatches':bad,'target_artifacts':artifacts,'git_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()}
phase=sys.argv[1];(A/(phase+'_CURRENTNESS.json')).write_text(json.dumps(result,indent=2)+'\n');(A/('GIT_STATUS_'+phase+'.txt')).write_bytes(subprocess.check_output(['git','status','--porcelain=v1','--untracked-files=all'],cwd=R))
assert not bad,bad
assert result['manifest_sha256']=='f53fed3ecc1f6abc65b480f8f6a735e1fe3a4a1d385948244bdd980972bbe42c'
print(phase,len(rows),'members exact',result['manifest_sha256'])
