from pathlib import Path
import json,hashlib
R=Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal';E=B/'evidence';O=B/'audit-evidence/fresh_001'
def read(p):return json.loads(p.read_text())
out={};p=E/'INVALID_EXTERNAL_002';cases=read(p/'cases.json');ds=[]
for c in cases:
 name=c['name'];api=read(p/'api'/f'{name}.json');d=api['diagnostic']
 for i in [0,1]:assert read(p/f'{name}_{i}.stderr')==api and not (p/f'{name}_{i}.stdout').read_bytes()
 assert d['phase']==c['phase'] and d['code']==c['code'] and d.get('pack_id')==c['pack']
 if c['source']:assert d['source_file']==c['source']
 for source in c['related']:assert any(x['source_file']==source for x in d['related_sources'])
 ds.append({'name':name,'phase':d['phase'],'code':d['code'],'source':d['source_file'],'position':d.get('position'),'pack':d.get('pack_id'),'expected':d.get('expected'),'actual':d.get('actual')})
assert len(ds)==13;out['actual_diagnostic_parity']=ds
p=E/'REPLACEMENT_EXTERNAL_004';d=read(p/'RESULT.json');assert d['failed_attempts']==54 and d['successful_replacements']==3
for mode in ['off','on','gap']:
 for ours,theirs in [('a_continued','a'),('b_replaced','b')]:assert (p/mode/(ours+'.ndjson')).read_bytes()==(E/'COMBAT_EXTERNAL_002'/f'{theirs}_{mode}/receipts.ndjson').read_bytes()
out['replacement']={k:d[k] for k in ['failed_attempts','acquisition_failures','replacement_failures','successful_replacements','a_continuation_replays','b_completed_replays']}
old=read(R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_003.json');directories=[x['directory'] for x in old['external_actual']];assert len(directories)==7 and all(not Path(x).exists() for x in directories);out['expired_old_external_actual']=directories
for name in ['entry','closure']:assert read(O/f'currentness_{name}.json')['currentness']=='current / unchanged'
out['currentness_verified']='entry and closure all three targets unchanged'
(O/'supplemental_review.json').write_text(json.dumps(out,indent=2)+'\n');print('Verified 13 actual diagnostic envelope pairs,54 replacement failures and6 continuation bytes,7 expired historical directories.')
