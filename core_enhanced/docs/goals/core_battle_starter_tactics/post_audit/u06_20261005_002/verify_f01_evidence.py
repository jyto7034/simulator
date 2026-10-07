from pathlib import Path
import json,hashlib,re
R=Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_battle_starter_tactics';O=Path(__file__).parent;j=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();errors=[]
def ck(ok,msg):
 if not ok:errors.append(msg)
p=H/'evidence/f01_reproduce_001';s=j(p/'SUMMARY.json');raw=j(p/'stdout.json');ck(sha(p/'stdout.json')==s['raw_sha256'],'reproduction raw sha');ck(sha(p/'stderr.log')==s['build_log_sha256'],'reproduction build log sha');results={}
for key,v in raw.items():
 events=v['events'];starts=sum(x['event']['type']=='AttackStart' for x in events);heals=[x for x in events if x['event']['type']=='HpChanged' and x['event']['delta']>0];ck(starts==v['attack_starts'],'starts '+key);ck(len(heals)==v['positive_hp_events'],'heals '+key);ck({k:w for k,w in v.items() if k!='events'}==s['cases'][key],'summary '+key);ck(v['final_hp']==(heals[-1]['event']['hp_after'] if heals else 50),'final HP '+key);results[key]={'starts':starts,'positive_hp_events':len(heals),'final_hp':v['final_hp']}
p=H/'evidence/public_starter_schema_003';s=j(p/'SUMMARY.json');rows=[]
for row in s['rows']:
 d=p/row['case'];ck(sha(d/'stdout')==row['stdout_sha256'] and sha(d/'stderr')==row['stderr_sha256'],'schema raw hash '+row['case']);rawtext=(d/'stdout').read_text()+(d/'stderr').read_text();expect=row['expected_diagnostic'];ck(row['exit_code']==0 if expect is None else row['exit_code']!=0 and expect.lower() in rawtext.lower(),'schema result '+row['case']);rows.append({'case':row['case'],'exit_code':row['exit_code'],'expected_diagnostic':expect})
for path,digest in s['inputs'].items():ck(sha(R/path)==digest,'schema input '+path)
ck(sha(R/'.cache/d07-runtime-amount-product-target/release/author_content')==s['binary_sha256'],'fresh reader binary')
p=H/'evidence/f01_public_consumer_001.log';observed=re.findall(r'^F01 public (\w+): flat=(\d+), percent=(\d+), SP=(\d+), useful=(true|false), final_hp=(\d+)$',p.read_text(),re.M);ck(len(observed)==8,'public consumer case count')
for actor,flat,percent,sp,useful,hp in observed:ck(hp==('90' if useful=='true' else '50'),'public consumer HP '+actor)
heap=(H/'evidence/integrated_heap009.log').read_text();heaplines=[x for x in heap.splitlines() if any(k in x for k in ['sample','retained','held','idle','test result'])]
result={'errors':errors,'reproduction_independently_counted_events':results,'schema_cases':rows,'schema_inputs_verified':len(s['inputs']),'release_reader_sha256':s['binary_sha256'],'public_consumer_eight_observations':observed,'heap_raw_lines':heaplines,'fresh_release_build_command':j(H/'evidence/f01_public_release_build_001.json')['command']};(O/'F01_EVIDENCE_VERIFICATION.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
