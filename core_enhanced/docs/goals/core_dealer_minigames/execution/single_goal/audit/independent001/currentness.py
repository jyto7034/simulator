from pathlib import Path
import hashlib,json,sys,tarfile,subprocess
R=Path('/mnt/f/work/simulator'); E=R/'core_enhanced/docs/goals/core_dealer_minigames/execution/single_goal'; T=E/'target/u16_3_current_001'
sha=lambda b:hashlib.sha256(b).hexdigest()
record=json.loads((T/'AUDIT_TARGET_RECORD.json').read_bytes()); result={'stage':sys.argv[1], 'mismatches':[], 'counts':{}, 'duplicates':[]}
allpaths=set()
for name in ['SOURCE_MANIFEST','EVIDENCE_MANIFEST']:
 rows=json.loads((T/(name+'.json')).read_bytes())['files']; result['counts'][name]=len(rows)
 for row in rows:
  p=row['path']
  if p in allpaths:result['duplicates'].append(p)
  allpaths.add(p); f=R/p
  if not f.is_file() or f.stat().st_size!=row['bytes'] or sha(f.read_bytes())!=row['sha256']:result['mismatches'].append(p)
for field,name in [('source_manifest_sha256','SOURCE_MANIFEST.json'),('evidence_manifest_sha256','EVIDENCE_MANIFEST.json'),('restore_sha256','RESTORE.json'),('source_archive_sha256','SOURCE.tar.gz')]:
 if sha((T/name).read_bytes())!=record[field]:result['mismatches'].append(name)
source={r['path']:r for r in json.loads((T/'SOURCE_MANIFEST.json').read_bytes())['files']}
with tarfile.open(T/'SOURCE.tar.gz') as t:
 archive={m.name:sha(t.extractfile(m).read()) for m in t if m.isfile()}
result['archive_exact']=archive=={p:r['sha256'] for p,r in source.items()}
expected=set()
for root in ['core_enhanced/src','core_enhanced/tests','game_server/src','game_resources/data']:
 expected.update(str(p.relative_to(R)) for p in (R/root).rglob('*') if p.is_file())
for p in E.parent.parent.iterdir():
 if p.is_file() and p.suffix in ['.md','.json']:expected.add(str(p.relative_to(R)))
for p in E.iterdir():
 if p.is_file() and p.suffix in ['.md','.json'] and not p.name.startswith(('BASELINE','GEX_')) and p.name!='COMPLETION_AUDIT.md':expected.add(str(p.relative_to(R)))
for p in (E/'test_parts').rglob('*'):
 if p.is_file() and not {'target','__pycache__'} & set(p.relative_to(E).parts):expected.add(str(p.relative_to(R)))
result['membership_missing']=sorted(expected-set(source)); result['membership_additional_bound_dependencies']=sorted(set(source)-expected)
evidence_expected=set()
for p in E.rglob('*'):
 if p.is_file() and not {'target','audit','issuer_private','node_modules','__pycache__'} & set(p.relative_to(E).parts) and p.name not in ['COMPLETION_AUDIT.md','GEX_10_RESULT_001.json']:
  evidence_expected.add(str(p.relative_to(R)))
result['evidence_membership_missing']=sorted(evidence_expected-allpaths)
result['HEAD']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()
result['passed']=not result['mismatches'] and not result['duplicates'] and not result['membership_missing'] and not result['evidence_membership_missing'] and result['archive_exact'] and result['HEAD']==record['HEAD']
(E/'audit/independent001'/('CURRENTNESS_'+sys.argv[1].upper()+'.json')).write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items() if k!='membership_additional_bound_dependencies'},indent=2))
