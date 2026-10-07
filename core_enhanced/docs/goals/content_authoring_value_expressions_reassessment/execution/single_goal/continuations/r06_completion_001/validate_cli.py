import subprocess,sys,json,time,hashlib
from pathlib import Path
H=Path(__file__).resolve().parent; E=H/'evidence'; P=E/'public_cli_001'
ROOT=Path('/mnt/f/work/simulator'); BIN=ROOT/'.cache/d07-runtime-amount-product-target/release'
while not (E/'final_release_001.json').exists():time.sleep(10)
assert json.loads((E/'final_release_001.json').read_text())['exit_code']==0
(P/'explain.json').write_text(json.dumps({'query':'compiled_graph','key':{'kind':'skill','id':'fragment_freischutz_black_round'}}))
def run(name,args):
 r=subprocess.run([sys.executable,str(H/'run.py'),name,'--',*map(str,args)])
 assert r.returncode==0,name
for op in ['validate','describe','explain']:
 run('cli_'+op+'_001',[BIN/'author_content',op,P/'content',P/'selection.json',*([P/'explain.json'] if op=='explain' else [])])
for trace in [False,True]:
 run('cli_simulate_'+('on' if trace else 'off')+'_001',[BIN/'author_content','simulate',P/'content',P/'selection.json',P/'scenario.json','--amount-observation',*(['--trace-output',P/'trace.jsonl'] if trace else [])])
off=(E/'cli_simulate_off_001.log').read_text().splitlines();on=(E/'cli_simulate_on_001.log').read_text().splitlines()
# Four numeric documents are the final records written by the CLI; compare raw bytes, not a JSON float projection.
assert off[-4:]==on[-4:]
assert '1267650600228229401496703205377' in '\n'.join(off[-4:])
assert any('"status":"evaluated"' in s or '"status":"available"' in s or '1267650600228229401496703205377' in s for s in off[-4:])
run('cli_package_001',[BIN/'package_official_content',ROOT/'game_resources',P/'packaged_official'])
run('cli_consume_001',[BIN/'consume_content_pack',P,'packaged_official'])
summary={'amount_documents_raw_identical':True,'exact_i128_decimal':'1267650600228229401496703205377','source':'cli_simulate_off_001.json','trace_sha256':hashlib.sha256((P/'trace.jsonl').read_bytes()).hexdigest()}
(E/'CLI_SUMMARY.json').write_text(json.dumps(summary,indent=2)+'\n')
