from pathlib import Path
import subprocess,json,hashlib
H=Path(__file__).resolve().parent;E=H/'evidence';P=E/'public_cli_001';R=Path('/mnt/f/work/simulator');B=R/'.cache/d07-runtime-amount-product-target/release'
(P/'explain_003.json').write_text(json.dumps({'query':'definition','key':{'kind':'Skill','id':'fragment_freischutz_black_round'}}))
rows=[]
def call(name,cmd):
 with (E/(name+'.log')).open('xb') as f:r=subprocess.run(list(map(str,cmd)),stdout=f,stderr=subprocess.STDOUT)
 rows.append({'name':name,'command':list(map(str,cmd)),'exit_code':r.returncode,'log_sha256':hashlib.sha256((E/(name+'.log')).read_bytes()).hexdigest()})
 (E/'CLI_RETRY_003_PARTIAL.json').write_text(json.dumps(rows,indent=2)+'\n')
 assert r.returncode==0,(name,(E/(name+'.log')).read_text()[-1500:])
call('cli_explain_003',[B/'author_content','explain',P/'content',P/'selection.json',P/'explain_003.json'])
for trace in [False,True]:
 call('cli_simulate_'+('on' if trace else 'off')+'_001',[B/'author_content','simulate',P/'content',P/'selection.json',P/'scenario.json','--amount-observation',*(['--trace-output',P/'trace.jsonl'] if trace else [])])
off=(E/'cli_simulate_off_001.log').read_text().splitlines();on=(E/'cli_simulate_on_001.log').read_text().splitlines()
assert off[-4:]==on[-4:]
assert '1267650600228229401496703205377' in '\n'.join(off[-4:])
call('cli_package_001',[B/'package_official_content',R/'game_resources',P/'packaged_official'])
call('cli_consume_001',[B/'consume_content_pack',P,'packaged_official'])
import shutil
for suffix,value,expected in [('min',-2147483648,0),('below_min',-2147483649,2),('above_max',2147483648,2)]:
 root=P/('signed_'+suffix);shutil.copytree(P/'content',root)
 skill=root/'official/data/skills/base.ron';raw=skill.read_text();assert 'Heal(amount: 45)' in raw
 skill.write_text(raw.replace('Heal(amount: 45)',f'Heal(amount: {value})',1))
 cmd=[str(B/'author_content'),'validate',str(root),str(P/'selection.json')]
 with (E/('cli_signed_'+suffix+'.log')).open('xb') as f:r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT)
 assert r.returncode==expected,(suffix,r.returncode)
 rows.append({'name':'signed_'+suffix,'command':cmd,'exit_code':r.returncode,'expected_exit':expected})
(E/'CLI_SUMMARY.json').write_text(json.dumps({'commands':rows,'amount_documents_raw_identical':True,'exact_i128_decimal':'1267650600228229401496703205377','trace_sha256':hashlib.sha256((P/'trace.jsonl').read_bytes()).hexdigest()},indent=2)+'\n')
print('CLI: explain/simulate off/on/package/consume passed; exact decimal and four raw documents equal')
