"""Validate every explicitly migrated pack through the current release reader."""
import json, subprocess, sys
from prepare_public_inputs import ROOT, HERE, OUT, INPUTS, PACKS, sha

destination=OUT/('input_validation_'+(sys.argv[1] if len(sys.argv)>1 else '001'))
destination.mkdir(exist_ok=False)
binary=ROOT/'.cache/d07-runtime-amount-product-target/release/author_content'
rows=[]
for index,relative in enumerate(PACKS):
    fixture=INPUTS/relative
    command=[binary,'validate',fixture/'content',fixture/'selection.json']
    result=subprocess.run(list(map(str,command)),cwd=ROOT,capture_output=True)
    stdout,stderr=destination/f'{index:02}.stdout',destination/f'{index:02}.stderr'
    stdout.write_bytes(result.stdout);stderr.write_bytes(result.stderr)
    rows.append({'fixture':str(relative),'command':list(map(str,command)),'exit_code':result.returncode,
                 'stdout_sha256':sha(stdout),'stderr_sha256':sha(stderr)})
    print(json.dumps({'fixture':str(relative),'exit_code':result.returncode,
                      'error':result.stderr.decode(errors='replace')[-1500:] if result.returncode else None}),flush=True)
originals=json.loads((OUT/'PREDECESSOR_ADAPTATION_001.json').read_text())['files']
assert all(sha(ROOT/row['source'])==row['source_sha256'] for row in originals)
summary={'passed':all(row['exit_code']==0 for row in rows),'rows':rows,
         'binary_sha256':sha(binary),'predecessor_inputs_unchanged':True,
         'final_input_files':{str(p.relative_to(ROOT)):sha(p) for p in INPUTS.rglob('*') if p.is_file()}}
(destination/'SUMMARY.json').write_text(json.dumps(summary,indent=2)+'\n')
assert summary['passed'],'Migrated inputs refused; inspect all individual diagnostics'
