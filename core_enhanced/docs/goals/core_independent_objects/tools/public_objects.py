"""Run preserved normal trial inputs through the current release CLI."""
import json,subprocess,sys
from initialize import ROOT,HERE
from run import sha
label=sys.argv[1]
inputs=HERE/'evidence/public_object_inputs_001'
out=HERE/('evidence/public_objects_'+label);out.mkdir()
binary=ROOT/'.cache/d07-runtime-amount-product-target/release/author_content'
rows=[]
for case in ['actor','small','big','ice','fallback','segments']:
 fixture=inputs/case
 request=out/(case+'_explain.json')
 request.write_text(json.dumps({'query':'definition','key':{'kind':'Skill','id':'fragment_freischutz_black_round'}}))
 for op in ['validate','describe','explain','simulate']:
  command=[binary,op,fixture/'content',fixture/'selection.json']
  if op=='simulate':command.append(fixture/'scenario.json')
  if op=='explain':command.append(request)
  r=subprocess.run(list(map(str,command)),cwd=ROOT,capture_output=True)
  (out/(case+'_'+op+'.stdout')).write_bytes(r.stdout);(out/(case+'_'+op+'.stderr')).write_bytes(r.stderr)
  assert r.returncode==0,(case,op,r.stderr[-2000:])
  if op=='simulate':
   values=[json.loads(line) for line in r.stdout.splitlines() if line.startswith(b'{')]
   steps=[v['payload'] for v in values if v.get('payload',{}).get('kind')=='step']
   final_index=31 if case=='small' else 27
   step=next(s for s in steps if s['index']==final_index)
   checkpoint=step['public_result']['payload']['battle_update']['checkpoint']
   assert checkpoint['independent_objects']==json.loads((fixture/'expected_objects.json').read_text()),case
   if case=='segments':
    assert checkpoint['independent_objects']['active'][0]['max_health']==50
    assert any(p.get('segmented',{}).get('remaining_health')==100 for p in checkpoint['sustained_effects']['active'])
  rows.append({'case':case,'operation':op,'exit_code':r.returncode,'stdout_sha256':sha(out/(case+'_'+op+'.stdout'))})
summary={'passed':True,'binary_sha256':sha(binary),'rows':rows,'inputs':{str(p.relative_to(ROOT)):sha(p) for p in inputs.rglob('*') if p.is_file()},'claim':'Six external normal authoring trials preserve exact native-trial object results through release CLI; legacy starter fixtures, test rewards and producer tuning are explicit test inputs.'}
(out/'SUMMARY.json').write_text(json.dumps(summary,indent=2)+'\n')
print('PASS release CLI validate/describe/simulate: six packs, exact object outcome parity')
