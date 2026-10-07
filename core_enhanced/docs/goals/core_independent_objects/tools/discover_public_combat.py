"""Disposable public-input discovery, never selected as final release evidence."""
import json,shutil,subprocess,sys
from initialize import ROOT,HERE
label=sys.argv[1];out=HERE/('evidence/public_combat_discovery_'+label);out.mkdir()
for case,target in [('ice',{'x':1,'y':2}),('small',{'x':1,'y':2}),('actor',{'x':0,'y':2})]:
 src=HERE/'evidence/public_object_inputs_001'/case;dst=out/case;shutil.copytree(src,dst)
 if case=='ice':
  p=dst/'content/official/data/summoned_units/base.ron';s=p.read_text();start=s.index('id:"common_ice_wall"');end=s.index('id:"melting_love_meat_wall"',start);part=s[start:end];assert 'max_health:120' in part;part=part.replace('max_health:120','max_health:3').replace('defense:10','defense:0');s=s[:start]+part+s[end:];p.write_text(s)
 scenario=json.loads((dst/'scenario.json').read_text());steps=scenario['steps'];steps[:]=steps[:19]
 steps[15]['clock']['delta_ms']=1000
 steps[17]['command']['target']['position']=target
 withdraw={'kind':'command','command':{'type':'withdraw_unit','employee_uuid':None},'context':{'expected_battle_authority':{'battle_uuid':None,'battle_state_revision':None}},'bindings':[
 {'from':{'step':27,'pointer':'/payload/battle_uuid'},'to':'/context/expected_battle_authority/battle_uuid','value_type':'string'},
 {'from':{'step':27,'pointer':'/payload/battle_update/battle_state_revision'},'to':'/context/expected_battle_authority/battle_state_revision','value_type':'number'},
 {'from':{'step':9,'pointer':'/payload/employee_uuid'},'to':'/command/employee_uuid','value_type':'string'}]}
 steps.append(withdraw)
 for delta in [5000,5000,5000]:steps.append({'kind':'clock','clock':{'mode':'elapsed','delta_ms':delta,'after_seq':0}})
 (dst/'scenario.json').write_text(json.dumps(scenario,indent=2)+'\n')
 r=subprocess.run([str(ROOT/'.cache/d07-runtime-amount-product-target/debug/author_content'),'simulate',str(dst/'content'),str(dst/'selection.json'),str(dst/'scenario.json')],capture_output=True)
 (dst/'result.stdout').write_bytes(r.stdout);(dst/'result.stderr').write_bytes(r.stderr);print(case,r.returncode,r.stderr[-800:].decode(),flush=True)
 for l in r.stdout.splitlines():
  try:p=json.loads(l).get('payload',{})
  except:continue
  if p.get('kind')=='step' and p['index']>=27:
   b=p.get('public_result',{}).get('payload',{}).get('battle_update',{});c=b.get('checkpoint',{});print(p['index'],b.get('server_battle_time_ms'),c.get('independent_objects'),flush=True)
