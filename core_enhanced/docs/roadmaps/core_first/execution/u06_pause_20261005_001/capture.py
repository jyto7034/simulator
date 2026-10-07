"""Capture the user-requested pause; never edits product or immutable evidence."""
import datetime,difflib,hashlib,json,pathlib,subprocess,sys,tarfile
ROOT=pathlib.Path.cwd()
D=ROOT/'core_enhanced/docs/roadmaps/core_first/execution/u06_pause_20261005_001'
H=ROOT/'core_enhanced/docs/goals/core_battle_starter_tactics'
sys.path.insert(0,str(H/'tools'))
from run import identity
current=identity()
baseline=json.loads((H/'execution/single_goal/baseline/SOURCE_BEFORE.json').read_text())
old={r['path']:r for r in baseline['files']}
changed=[r for r in current['files'] if old.get(r['path'])!=r]
# Baseline includes 17 documentation dependencies absent from the product runner.
# Absence from runner membership is NOT a deletion from the workspace.
missing=[r['path'] for r in baseline['files'] if not (ROOT/r['path']).is_file()]
assert not missing,missing
with (D/'PRODUCT_CURRENT.json').open('x') as f:json.dump(current,f,indent=2);f.write('\n')
with tarfile.open(H/'execution/single_goal/baseline/SOURCE_BEFORE.tar.gz','r:gz') as before:
 names={m.name:m for m in before.getmembers() if m.isfile()}
 patch=[]
 for row in changed:
  path=row['path']
  prev=before.extractfile(names[path]).read().decode() if path in old else ''
  now=(ROOT/path).read_text()
  patch.extend(difflib.unified_diff(prev.splitlines(True),now.splitlines(True),fromfile=('a/'+path if path in old else '/dev/null'),tofile='b/'+path))
 with (D/'PRODUCT_DELTA.patch').open('x') as f:f.write(''.join(patch))
with tarfile.open(D/'PRODUCT_DELTA.tar.gz','x:gz') as out:
 for row in changed:out.add(ROOT/row['path'],arcname=row['path'])
mutable=['core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md','core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md','core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md']+[str((H/'execution/single_goal'/name).relative_to(ROOT)) for name in ['PLAN.md','EXPERIMENTS.md','EXPERIMENT_NOTES.md']]
with tarfile.open(D/'DOCUMENTS_BEFORE.tar.gz','x:gz') as out:
 for path in mutable:out.add(ROOT/path,arcname=path)
receipts=[]
for path in sorted((H/'evidence').glob('*.json')):
 r=json.loads(path.read_text())
 if 'exit_code' in r:receipts.append({'path':str(path.relative_to(ROOT)),'command':r['command'],'exit_code':r['exit_code'],'source_unchanged':r['source_unchanged'],'exact_current_product':r['source_after']==current})
state={'time_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'time_kst':'2026-10-05','reason':'User explicitly requested implementation pause and handoff documentation.','status':'paused_by_user','selected_goal':'GS-CBST-001/GOAL-CBST-01','p2g':'00–60 complete','gex':'00 complete; 10 partial implementation; no completion target or independent U06 audit','product_sha256':current['sha256'],'product_files':len(current['files']),'baseline_manifest':str((H/'execution/single_goal/baseline/SOURCE_BEFORE.json').relative_to(ROOT)),'baseline_contains_additional_docs':17,'changed_product_files':changed,'missing_baseline_files':missing,'execution_receipts':receipts,'document_paths_before':mutable,'no_product_mutation_after_user_stop':True,'running_implementation_or_test_processes':[],'resume':'U06 GEX-10: BASIC residual review/extent correction, then STARTER CombatModifiers/grant/RON; TACTICS/BLOCK/TIME/VERIFY remain.'}
with (D/'STATE.json').open('x') as f:json.dump(state,f,indent=2,ensure_ascii=False);f.write('\n')
(D/'GIT_STATUS.txt').write_bytes(subprocess.check_output(['git','status','--short'],cwd=ROOT))
print(json.dumps({'product_files':len(current['files']),'changed_product_files':len(changed),'new_product_files':[r['path'] for r in changed if r['path'] not in old],'sha256':current['sha256'],'exact_current_receipts':[r['path'] for r in receipts if r['exact_current_product']]},ensure_ascii=False))
