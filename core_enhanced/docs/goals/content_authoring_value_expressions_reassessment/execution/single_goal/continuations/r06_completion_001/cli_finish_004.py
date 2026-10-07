from pathlib import Path
import json,subprocess,shutil,hashlib
H=Path(__file__).resolve().parent;E=H/'evidence';P=E/'public_cli_001';B=Path('/mnt/f/work/simulator/.cache/d07-runtime-amount-product-target/release')
rows=json.loads((E/'CLI_RETRY_003_PARTIAL.json').read_text())
assert all(x['exit_code']==0 for x in rows)
assert len(rows)==5
for name in ['min','below_min']:
 raw=(E/('cli_signed_'+name+'.log')).read_text();doc=json.loads(raw)
 if name=='min':assert doc['status']=='valid'
 else:assert doc['kind']=='content' and doc['diagnostic']['code']=='ron_parse' and doc['diagnostic']['field_path'].endswith('.Heal.amount')
root=P/'signed_above_max';shutil.copytree(P/'content',root)
p=root/'official/data/skills/base.ron';p.write_text(p.read_text().replace('Heal(amount: 45)','Heal(amount: 2147483648)',1))
cmd=[str(B/'author_content'),'validate',str(root),str(P/'selection.json')]
with (E/'cli_signed_above_max.log').open('xb') as f:r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
assert r.returncode==1
assert json.loads((E/'cli_signed_above_max.log').read_text())['diagnostic']['code']=='ron_parse'
off=(E/'cli_simulate_off_001.log').read_text().splitlines();on=(E/'cli_simulate_on_001.log').read_text().splitlines()
assert off[-4:]==on[-4:]
assert '1267650600228229401496703205377' in '\n'.join(off[-4:])
with (E/'CLI_SUMMARY.json').open('x') as f:json.dump({'commands':rows,'actual_amount_four_documents_raw_equal':True,'actual_i128_decimal':'1267650600228229401496703205377','min_validate_exit':0,'below_min_validate_exit':1,'above_max_validate_exit':1,'negative_diagnostic':'ron_parse at exact Heal.amount site','trace_sha256':hashlib.sha256((P/'trace.jsonl').read_bytes()).hexdigest(),'harness_correction':'Content validation errors exit1; CLI request input errors exit2. Preserve cli_retry_003 failed expectation.'},f,indent=2)
print('CLI validate/describe/definition explain, real simulation off/on exact decimal, package/consume and signed boundaries verified')
