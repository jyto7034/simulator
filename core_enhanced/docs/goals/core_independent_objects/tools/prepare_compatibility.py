"""Copy exact U06 input bytes; change only current harness ownership paths."""
import difflib,json,shutil
from initialize import ROOT,HERE
from run import sha
OLD=ROOT/'core_enhanced/docs/goals/core_battle_starter_tactics'
prior=OLD/'execution/single_goal/compatibility/current_001'
current=HERE/'execution/single_goal/compatibility/current_001'
rows=[];scripts=[];patches=[]
def copy_inputs(src,dst):
 assert not dst.exists(),dst
 shutil.copytree(src,dst)
 for p in sorted(src.rglob('*')):
  if not p.is_file():continue
  q=dst/p.relative_to(src);assert sha(p)==sha(q)
  rows.append({'source':str(p.relative_to(ROOT)),'copy':str(q.relative_to(ROOT)),'sha256':sha(p),'bytes':p.stat().st_size})
copy_inputs(prior/'inputs',current/'inputs')
copy_inputs(OLD/'evidence/independent_attack_area_002',HERE/'evidence/independent_attack_area_002')
for name in ['probe.py','boundary.py','public_probe.py','public_trace_probe.py','public_cli.py','public_profile_probe.py','public_boundary_probe.py','default_parity.py','host_peak_failure_probe.rs']:
 p=prior/name;q=current/name;s=p.read_text();out=s.replace(str(OLD),str(HERE));q.write_text(out)
 scripts.append({'source':str(p.relative_to(ROOT)),'copy':str(q.relative_to(ROOT)),'source_sha256':sha(p),'copy_sha256':sha(q)})
 patches.extend(difflib.unified_diff(s.splitlines(keepends=True),out.splitlines(keepends=True),fromfile=str(p.relative_to(ROOT)),tofile=str(q.relative_to(ROOT))))
for name in ['public_continuation_probe.py','public_rewind_probe.py','public_schema_probe.py','public_starter_schema.py','public_independent_area_probe.py']:
 p=OLD/'tools'/name;q=HERE/'tools'/name;s=p.read_text();out=s.replace(str(OLD),str(HERE));q.write_text(out)
 scripts.append({'source':str(p.relative_to(ROOT)),'copy':str(q.relative_to(ROOT)),'source_sha256':sha(p),'copy_sha256':sha(q)})
 patches.extend(difflib.unified_diff(s.splitlines(keepends=True),out.splitlines(keepends=True),fromfile=str(p.relative_to(ROOT)),tofile=str(q.relative_to(ROOT))))
# Existing parser helper and constants are imported, but its main migration is
# never executed. This preserves the documented old conversions without adding any.
s=(OLD/'tools/prepare_public_inputs.py').read_text();(HERE/'tools/prepare_public_inputs.py').write_text(s)
# This helper already resolves HERE from the local initialize module.
(current/'HARNESS_OWNER.diff').write_text(''.join(patches))
(current/'INTAKE_U07.json').write_text(json.dumps({'unchanged_input_files':rows,'scripts':scripts,'scope':'Exact immediate U06 packs and its independent-area counterexample; no gameplay input migration or numeric normalization. New U07 cases are separate.','historical_migrations':str((prior/'ADAPTATION_FINAL.json').relative_to(ROOT))},indent=2)+'\n')
print(json.dumps({'unchanged_files':len(rows),'adapted_harnesses':len(scripts)}))
