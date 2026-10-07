import json,hashlib,difflib,re,datetime
from pathlib import Path
R=Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal';O=B/'audit-evidence/fresh_001'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
base=json.loads((B/'BASELINE_001.json').read_text()); saved=Path(base['backup_path']); declared=json.loads((B/'evidence/PRESERVATION_002.json').read_text()); badcopies=[];changes=[];removed=[]
for p,h in base['scope_hashes'].items():
 if sha(saved/p)!=h:badcopies.append(p)
 if sha(R/p)!=h:changes.append(p)
 if h is not None and not (R/p).is_file():removed.append(p)
patch=(B/'evidence/R02_SCOPED_DIFF_002.patch').read_text();paths=re.findall(r'^\+\+\+ b/(.*)$',patch,re.M);generated=''
for p in paths:
 before=(saved/p).read_text().splitlines(keepends=True) if (saved/p).is_file() else []
 after=(R/p).read_text().splitlines(keepends=True)
 generated+=''.join(difflib.unified_diff(before,after,fromfile='a/'+p,tofile='b/'+p))
# Exact current data/source/test/config enumeration against VI023.
vi=json.loads((B/'VALIDATION_INPUT_023.json').read_text());members={x['path'] for x in vi['inputs']};enum=[]
for pre in ['core_enhanced/src','core_enhanced/tests','game_resources/data']:
 enum += [str(p.relative_to(R)) for p in (R/pre).rglob('*') if p.is_file()]
missing=[p for p in enum if p not in members]
# Prove observer body differs only by required source-free event field.
p='core_enhanced/tools/runtime_architecture_final_validation/src/public_guards.rs'; current=(R/p).read_text();lines=current.splitlines(keepends=True);hits=[i for i,l in enumerate(lines) if 'skill_effect_source: None,' in l];before=''.join(l for i,l in enumerate(lines) if i not in hits)
expected=next(m['before_sha256'] for m in json.loads((B/'evidence/TOOL_SUPPLEMENTAL_002.json').read_text())['files'] if m['path']==p)
observer_ok=hashlib.sha256(before.encode()).hexdigest()==expected
res={'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'baseline_count':len(base['scope_hashes']),'baseline_copy_mismatches':badcopies,'changed_existing':changes,'changes_match_attribution':set(changes)=={m['path'] for m in declared['changed_existing']},'removed':removed,'patch_paths':len(paths),'patch_reconstruct_exact':generated==patch,'patch_sha256':sha(B/'evidence/R02_SCOPED_DIFF_002.patch'),'source_test_data_enumerated':len(enum),'VI023_omissions':missing,'observer_removed_line_count':len(hits),'observer_exact_before_reconstructed':observer_ok,'tool_mains_without_prior_hash':8,'limits':'No claim of byte comparison for every unrelated dirty repository file or the eight API mains missing pre-edit hashes.'};(O/'preservation_review.json').write_text(json.dumps(res,indent=2)+'\n');print(json.dumps(res,indent=2));assert not badcopies and not removed and res['changes_match_attribution'] and generated==patch and not missing and observer_ok
