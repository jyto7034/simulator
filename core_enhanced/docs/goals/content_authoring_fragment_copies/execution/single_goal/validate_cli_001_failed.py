from pathlib import Path
import subprocess,sys,json,hashlib,shutil
H=Path(__file__).resolve().parent;E=H/'evidence';R=Path('/mnt/f/work/simulator');B=R/'.cache/d07-runtime-amount-product-target/release';P=E/'public_cli_001';rows=[]
def run(name,args,expected=0):
 code=subprocess.run([sys.executable,str(H/'run.py'),name,'--',*map(str,args)]).returncode
 assert code==expected,(name,code)
 rows.append({'name':name,'exit_code':code})
run('integrated_authoring_002',['cargo','test','-p','game_core','--lib','authoring','--offline','--locked'])
run('integrated_value_expr_002',['cargo','test','-p','game_core','--lib','value_expr','--offline','--locked'])
run('integrated_list_002',['cargo','test','-p','game_core','--lib','--offline','--locked','--','--list'])
run('cli_export_r06_001',['env','R06_PUBLIC_CLI_EXPORT='+str(P),'cargo','test','-p','game_core','--lib','r06_public_dynamic_query_limits_identity_lifecycle_and_trace_independence','--offline','--locked'])
run('cli_export_r07_001',['env','R07_PUBLIC_CLI_EXPORT='+str(E/'r07_public_cli_001'),'cargo','test','-p','game_core','--lib','r07_f07_f08_f09_public_ron_recipe_forms_sources_resources_and_fresh_process','--offline','--locked'])
for tag,root,skill in [('r06',P,'fragment_freischutz_black_round'),('r07_snapshot',E/'r07_public_cli_001/Snapshot','r07_awakened'),('r07_live',E/'r07_public_cli_001/Live','r07_awakened')]:
 (root/'explain.json').write_text(json.dumps({'query':'compiled_graph','key':{'kind':'Skill','id':skill}}))
 for op in ['validate','describe','explain']:
  run('cli_'+tag+'_'+op+'_001',[B/'author_content',op,root/'content',root/'selection.json',*([root/'explain.json'] if op=='explain' else [])])
 for trace in [False,True]:
  run('cli_'+tag+('_on' if trace else '_off')+'_001',[B/'author_content','simulate',root/'content',root/'selection.json',root/'scenario.json','--amount-observation',*(['--trace-output',root/'trace.jsonl'] if trace else [])])
 off=(E/('cli_'+tag+'_off_001.log')).read_text().splitlines()[-4:];on=(E/('cli_'+tag+'_on_001.log')).read_text().splitlines()[-4:]
 assert off==on
 assert ('1267650600228229401496703205377' if tag=='r06' else '"form_id":"awakened"') in '\n'.join(off)
run('cli_package_001',[B/'package_official_content',R/'game_resources',P/'packaged_official'])
run('cli_consume_001',[B/'consume_content_pack',P,'packaged_official'])
for name,value in [('min',-2147483648),('below_min',-2147483649),('above_max',2147483648)]:
 root=P/('signed_'+name);shutil.copytree(P/'content',root);path=root/'official/data/skills/base.ron';text=path.read_text();assert 'Heal(amount: 45)' in text;path.write_text(text.replace('Heal(amount: 45)',f'Heal(amount: {value})',1))
 run('cli_signed_'+name+'_001',[B/'author_content','validate',root,P/'selection.json'],0 if name=='min' else 1)
 doc=json.loads((E/('cli_signed_'+name+'_001.log')).read_text())
 if name=='min':assert doc['status']=='validated'
 else:assert doc['diagnostic']['code']=='ron_parse' and doc['diagnostic']['field_path'].endswith('.Heal.amount')
(E/'CLI_SUMMARY.json').write_text(json.dumps({'commands':rows,'r06_i128_exact_decimal':'1267650600228229401496703205377','r07_snapshot_live_fragment_source':True,'trace_off_on_four_amount_documents_identical':True,'signed_boundaries':True},indent=2)+'\n')

run("cli_aggregate_001",[sys.executable,H/"cli_aggregate.py"])
