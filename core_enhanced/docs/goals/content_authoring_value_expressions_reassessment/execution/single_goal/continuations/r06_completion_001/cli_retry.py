from pathlib import Path
import subprocess,json,hashlib
H=Path(__file__).resolve().parent;E=H/'evidence';P=E/'public_cli_001';R=Path('/mnt/f/work/simulator');B=R/'.cache/d07-runtime-amount-product-target/release'
(P/'explain_002.json').write_text(json.dumps({'query':'compiled_graph','key':{'kind':'Skill','id':'fragment_freischutz_black_round'}}))
rows=[]
def call(name,cmd):
 with (E/(name+'.log')).open('xb') as f:r=subprocess.run(list(map(str,cmd)),stdout=f,stderr=subprocess.STDOUT)
 rows.append({'name':name,'command':list(map(str,cmd)),'exit_code':r.returncode,'log_sha256':hashlib.sha256((E/(name+'.log')).read_bytes()).hexdigest()})
 (E/'CLI_RETRY_PARTIAL.json').write_text(json.dumps(rows,indent=2)+'\n')
 assert r.returncode==0,(name,(E/(name+'.log')).read_text()[-1500:])
call('cli_explain_002',[B/'author_content','explain',P/'content',P/'selection.json',P/'explain_002.json'])
for trace in [False,True]:
 call('cli_simulate_'+('on' if trace else 'off')+'_001',[B/'author_content','simulate',P/'content',P/'selection.json',P/'scenario.json','--amount-observation',*(['--trace-output',P/'trace.jsonl'] if trace else [])])
off=(E/'cli_simulate_off_001.log').read_text().splitlines();on=(E/'cli_simulate_on_001.log').read_text().splitlines()
assert off[-4:]==on[-4:]
assert '1267650600228229401496703205377' in '\n'.join(off[-4:])
call('cli_package_001',[B/'package_official_content',R/'game_resources',P/'packaged_official'])
call('cli_consume_001',[B/'consume_content_pack',P,'packaged_official'])
(E/'CLI_SUMMARY.json').write_text(json.dumps({'commands':rows,'amount_documents_raw_identical':True,'exact_i128_decimal':'1267650600228229401496703205377','trace_sha256':hashlib.sha256((P/'trace.jsonl').read_bytes()).hexdigest()},indent=2)+'\n')
print('CLI: explain/simulate off/on/package/consume passed; exact decimal and four raw documents equal')
