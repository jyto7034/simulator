from pathlib import Path
import json,hashlib,subprocess,datetime,sys
R=Path('/mnt/f/work/simulator');A=Path(__file__).resolve().parent;T=A.parent.parent/'target/u13_current_002'
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
record=json.loads((T/'AUDIT_TARGET_RECORD.json').read_text())
result={'time':datetime.datetime.now(datetime.timezone.utc).isoformat(),'target':record['target_id'],'HEAD':subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),'checks':{},'mismatches':[]}
for name,key in [('SOURCE.tar.gz','source_archive_sha256'),('SOURCE_MANIFEST.json','source_manifest_sha256'),('EVIDENCE_MANIFEST.json','evidence_manifest_sha256'),('RESTORE.json','restore_sha256')]:
 if sha(T/name)!=record[key]:result['mismatches'].append(name)
source=set()
for name in ['SOURCE_MANIFEST.json','EVIDENCE_MANIFEST.json']:
 rows=json.loads((T/name).read_text())['files'];result['checks'][name]=len(rows)
 for row in rows:
  p=R/row['path']
  if name=='SOURCE_MANIFEST.json':source.add(row['path'])
  if not p.is_file() or p.stat().st_size!=row['bytes'] or sha(p)!=row['sha256']:result['mismatches'].append(row['path'])
for root in ['core_enhanced/src','game_server/src','game_resources/data']:
 actual={str(p.relative_to(R)) for p in (R/root).rglob('*') if p.is_file()}
 expected={p for p in source if p.startswith(root+'/')}
 if actual!=expected:result['mismatches'].append({'root':root,'unexpected':sorted(actual-expected),'missing':sorted(expected-actual)})
result['passed']=not result['mismatches'] and result['HEAD']==record['HEAD']
result['currentness']='current / unchanged' if result['passed'] else 'stale'
result['scope_note']='Audit output and explicitly excluded mutable HANDOFF/ROADMAP_STATUS/EXECUTION_HISTORY locators are separate; product membership and frozen source/evidence checked.'

# Successor reuses predecessor evidence, which remains bound to its original inputs.
rows=json.loads((T.parent/'u13_current_001/EVIDENCE_MANIFEST.json').read_text())['files']
result['checks']['predecessor_evidence']=len(rows)
for row in rows:
 p=R/row['path']
 if not p.is_file() or p.stat().st_size!=row['bytes'] or sha(p)!=row['sha256']:result['mismatches'].append(row['path'])
result['passed']=not result['mismatches'] and result['HEAD']==record['HEAD']
result['currentness']='current / unchanged' if result['passed'] else 'stale'
output=A/(sys.argv[1] if len(sys.argv)>1 else 'CURRENTNESS_EXIT.json')
with output.open('x') as f:json.dump(result,f,indent=2,ensure_ascii=False);f.write('\n')
print(json.dumps(result,ensure_ascii=False));assert result['passed']
