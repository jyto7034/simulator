"""Record unchanged immediate-predecessor packs and exact new consumer harness deltas."""
import difflib,json
from run import ROOT,HERE,identity,sha
C=HERE/'execution/single_goal/compatibility/current_001'
intake=json.loads((C/'INTAKE.json').read_text());validation_path=C/'input_validation_001/SUMMARY.json';validation=json.loads(validation_path.read_text())
assert validation['passed'] and len(validation['rows'])==15
files=[];patches=[]
for row in intake['unchanged_input_files']:
 source,copy=ROOT/row['source'],ROOT/row['copy']
 assert sha(source)==sha(copy)==row['sha256'],row['copy']
 files.append({'source':row['source'],'source_sha256':sha(source),'copy':row['copy'],'copy_sha256':sha(copy),'semantic_change':'none; exact U05 final input bytes'})
for row in intake['scripts']:
 source,copy=ROOT/row['source'],ROOT/row['copy'];assert sha(source)==row['source_sha256']
 files.append({'source':row['source'],'source_sha256':sha(source),'copy':row['copy'],'copy_sha256':sha(copy),'semantic_change':'U06 owner path; see exact diff for any explicit current-run orchestration edits'})
 patches.extend(difflib.unified_diff(source.read_text().splitlines(keepends=True),copy.read_text().splitlines(keepends=True),fromfile=row['source'],tofile=row['copy']))
original={row['copy'] for row in files};added=[]
for p in sorted(C.glob('*.py')):
 name=str(p.relative_to(ROOT))
 if name in original:continue
 added.append({'path':name,'sha256':sha(p)})
 patches.extend(difflib.unified_diff([],p.read_text().splitlines(keepends=True),fromfile='/dev/null',tofile=name))
(C/'FINAL_INPUT_AND_HARNESS.diff').write_text(''.join(patches))
result={'source_sha256':identity()['sha256'],'status':'U06 current reader validation; independent review pending','predecessor_sources_unchanged':True,'files':files,'added_harness_files':added,'validation_summary_sha256':sha(validation_path),'final_diff_sha256':sha(C/'FINAL_INPUT_AND_HARNESS.diff'),'semantic_changes':['All419 immediate U05 public input files are byte-identical. Historical U05 migrations remain in PREDECESSOR_ADAPTATION_001 and the predecessor final record.','Current U06 official starters and strict negative variants are packaged and tested separately, not substituted into inherited numerical fixtures.','PC isolated fixture retains explicit historical nonemployee resonance0/100/1000. All raw new costs remain visible; original numeric caps are unchanged.','A layout observation precedes every measured PC region; the frozen576 workload rows are unchanged. The new comparator accounts40 bytes per runtime-unit bucket and80 bytes per initial spawn-vector element from the two inline action fields, then compares every remaining non-timing field exactly.','Output destinations and source-binding orchestration are U06-owned; failed attempts remain preserved.'],'original_oracle_limit':'No predecessor input or raw evidence rewritten; additive U06 assertions are separate.'}
with (C/'ADAPTATION_FINAL.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps({'files':len(files),'unchanged_inputs':419,'packs':15}))
