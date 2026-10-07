from pathlib import Path
"""Re-run the frozen independent-area counterexample against the current release."""
import json, subprocess, sys
from initialize import ROOT, HERE
from run import sha

label=sys.argv[1]
out=HERE/('evidence/public_independent_area_'+label)
out.mkdir(exist_ok=False)
fixture=Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_independent_objects')/'evidence/independent_attack_area_002'
binary=ROOT/'.cache/d07-runtime-amount-product-target/release/author_content'
command=[binary,'simulate',fixture/'content',fixture/'selection.json',fixture/'scenario.json']
result=subprocess.run(list(map(str,command)),cwd=ROOT,capture_output=True)
(out/'stdout.jsonl').write_bytes(result.stdout);(out/'stderr').write_bytes(result.stderr)
assert result.returncode==0,result.stderr[-2500:]
rows=[json.loads(line) for line in result.stdout.splitlines() if line.startswith(b'{')]
steps={row['payload']['index']:row['payload'] for row in rows if row.get('payload',{}).get('kind')=='step'}
def checkpoint(index):return steps[index]['public_result']['payload']['battle_update']['checkpoint']
before,after=checkpoint(26),checkpoint(27)
accepted=before['deployment']['deployed_units'][0]['skill_readiness']
actual=after['deployment']['deployed_units'][0]['skill_readiness']
assert accepted['resonance_use']['phase']=='accepted' and accepted['resonance_current']==0
assert after['battle_time_ms']-before['battle_time_ms']==101
assert actual['resonance_use'] is None and 'skill_use' not in actual['resonance_gain_block_reasons'],actual
summary={'passed':True,'command':list(map(str,command)),'binary_sha256':sha(binary),
         'accepted':accepted,'after_101_battle_ms':actual,'counterexample':'INDEPENDENT_AREA_COUNTEREXAMPLE.json',
         'stdout_sha256':sha(out/'stdout.jsonl'),'input_files':{str(p.relative_to(ROOT)):sha(p) for p in fixture.rglob('*') if p.is_file()}}
(out/'SUMMARY.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({'passed':True,'battle_time_ms':after['battle_time_ms'],'readiness':actual}))
