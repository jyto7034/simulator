import json,hashlib,datetime,tarfile,tomllib,collections
from pathlib import Path
H=Path('core_enhanced/docs/goals/core_relic_numeric_variants'); A=H/'post_audit/u08_current_013_pass002'; D=H/'evidence/audit_f01_paused_probe001/attempt003'
C=json.loads((H/'evidence/finalizer_ready013/CONFIG_READY_013.json').read_text()); O=json.loads((H/'evidence/finalizer_ready012/CONFIG_READY_012.json').read_text());bad=[];refs={};assertions=0

def check(ok,label):
 if not ok: bad.append(label)
def rd(p):return json.loads(Path(p).read_text())
def ref(r):
 p=r['path'];b=Path(p).read_bytes();check(len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256'],['ref',p]);refs[p]=r;return b
def pointer(x,p):
 for k in p.lstrip('/').split('/') if p else []:
  k=k.replace('~1','/').replace('~0','~');x=x[int(k)] if isinstance(x,list) else x[k]
 return x
unchanged=[]
for k in ['source','receipts','execution_harness_binding','performance_environments','functional_quiescence','artifacts','predecessor_source_delta','predecessor_conditions']:
 check(C[k]==O[k],['reused_changed',k]);unchanged.append(k)
oldclaims=0;newclaims=0
for fam in ['sentences','dimensions','nodes','seams']:
 check(C[fam].keys()==O[fam].keys(),['scope_keys',fam])
 for k,v in O[fam].items():
  check(v.get('text')==C[fam][k].get('text'),['scope_text',fam,k])
  old=v['claims'];new=C[fam][k]['claims'];check(new[:len(old)]==old,['old_claim_rewritten',fam,k]);oldclaims+=len(old);newclaims+=len(new)-len(old)

def walk(v):
 global assertions
 if isinstance(v,dict):
  if {'artifact','pointer','equals'}<=v.keys():
   b=ref(v['artifact'])
   if v.get('format') not in (None,'json_document'):x=next(json.loads(s) for s in b.decode().splitlines() if s.startswith('{') and 'instrumented' in s)
   else:x=json.loads(b)
   check(pointer(x,v['pointer'])==v['equals'],['assertion',v['artifact']['path'],v['pointer']]);assertions+=1
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
walk(C)
r=rd(D/'focus.json');check(r['source_before']==r['source_after']==C['source'],'supplemental_source');check(r['exit_code']==0,'supplemental_exit')
raw=(D/'focus.log').read_bytes();check(hashlib.sha256(raw).hexdigest()==r['log_sha256'],'supplemental_raw_hash')
docs=[json.loads(s) for s in raw.decode().splitlines() if s.startswith('{')];summary=[json.loads(s.split(' ',1)[1]) for s in raw.decode().splitlines() if s.startswith('U08_PAUSED_PUBLIC_PASS ')]
S=rd(D/'runtime/SUMMARY.json');T=rd(D/'runtime/TRANSCRIPT.json');check(summary==[S],'raw_summary');check([x for x in docs if 'operation'in x]==T,'raw_transcript')
check(collections.Counter(x['operation'] for x in T)=={'command':20,'admin':3,'host_tick':57},'operation_counts')
commands=[x for x in T if x['operation']=='command'];check([x['source_command_id'] for x in commands]==[f'paused-probe-command-{i}' for i in range(1,21)],'command_ids')
pi=next(i for i,x in enumerate(T) if x.get('input',{}).get('type')=='pause_battle');ri=next(i for i,x in enumerate(T) if x.get('input',{}).get('type')=='resume_battle')
check(len([x for x in T[pi+1:ri] if x.get('input',{}).get('type')=='activate_skill'])==2,'paused_accepted_two')
paused=[x for x in T[pi+1:ri] if x['operation']=='host_tick'];check(len(paused)==5 and all(x['milliseconds']==100 and x['payload'] is None for x in paused),'paused_ticks')
for x in T[pi:ri]:
 if x['payload'] and 'battle_update'in x['payload']:
  cp=x['payload']['battle_update']['checkpoint'];check(cp['battle_time_ms']==5000,'paused_time');check(sorted([[u['unit_instance_id'],u['stats']['current_health']] for u in cp['units']])==S['frozen_hp'],'paused_hp')
for k in ['damage_mutations','damage_strikes','damaging_strikes','damage_hp_lost','latest_damage','latest_nonzero_damage']:
 check(k in S['facts_before_acceptance']['summary'] and k in S['facts_after_acceptance']['summary'] and S['facts_before_acceptance']['summary'][k]==S['facts_after_acceptance']['summary'][k],['acceptance_damage',k])
check(S['facts_before_acceptance']['summary']['resource_spent']==20 and S['facts_after_acceptance']['summary']['resource_spent']==22,'legitimate_resonance_cost')
events={}
for x in T:
 for e in (x['payload']or{}).get('battle_update',{}).get('events_delta',{}).get('events',[]):
  check(e['seq'] not in events or events[e['seq']]==e,'event_conflict');events[e['seq']]=e
check([events[i]for i in sorted(events)]==S['events'] and set(events)==set(range(1,27)),'events_exact')
amounts=[]
for i,(flat,pct,variant) in enumerate([(20,20,'Normal'),(40,50,'Corrupt')]):
 expected=(100+flat)*(100+pct)//100;hit=S['observed'][i];src=hit['source'];binding=src['relic_numeric'];amounts.append(expected)
 check(hit['requested_damage']==hit['hp_before']-hit['hp_after']==expected and hit['hp_before']>expected and hit['time_ms']==5001,['independent_hp',i])
 check(src['actor']==binding['actor']==S['actors'][i] and binding['employee_uuid']==S['employees'][i] and binding['generation']==S['generation'],['binding_owner',i])
 check(binding['contributions']==[{'copy_uuid':S['relics'][i],'definition_id':'u08_trial_blend','effect_id':'damage','flat':flat,'percent':pct,'variant':variant}],['contributions',i])
 check(src['fragment']==binding['fragment']=={'copy_uuid':S['fragments'][i],'definition_id':'u08_trial_fragment','form_id':'base','skill_id':'u08_trial_damage'},['fragment',i])
 check(src['cast']['skill_id']=='u08_trial_damage' and src['cast']['caster_instance_id']==S['actors'][i] and src['authored']['source_file']=='official/data/skills/base.ron' and src['authored']['origin']=='Original',['original_source',i])
 check(hit['modifier_sources']==[] and all(v==0 for v in hit['unattributed_modifiers'].values()),['no_external_modifiers',i])
 check(len(hit['segments'])==1 and hit['segments'][0]['source']==src and hit['segments'][0]['raw_damage']==hit['segments'][0]['calculated_damage']==expected,['segments',i])
 actual=[x for x in docs if x.get('stage')=='actual_skill_hit' and x['employee_index']==i];check(len(actual)==1 and actual[0]['fact']==hit and actual[0]['ability_event_seq']==S['ability_event_sequences'][i],['raw_hit',i])
 start=events[S['start_sequences'][i]];cast=events[S['ability_event_sequences'][i]]
 check(start['event']['type']=='ManualCastStart' and start['time_ms']==5000 and start['event']['caster_instance_id']==S['actors'][i],['accepted_event',i]);check(cast['event']['type']=='AbilityCast' and cast['cause']['seq']==start['seq'] and cast['time_ms']==5001,['cast_cause',i])
 hp=[e for e in events.values() if e['event']['type']=='HpChanged' and e['event']['source_instance_id']==S['actors'][i]];check(len(hp)==1,['one_hp',i]);hp=hp[0];step=events[hp['cause']['seq']]
 check(step['event']['type']=='AbilityStepTriggered' and step['cause']['seq']==cast['seq'] and hp['event']['hp_before']==hit['hp_before'] and hp['event']['hp_after']==hit['hp_after'] and hp['event']['target_instance_id']==hit['target'],['hp_causality',i])
check(S['observed'][1]['hp_after']==S['observed'][0]['hp_before']==999790 and S['observed'][0]['hp_after']==999646,'chronological_210_then_144')
check(S['observed'][0]['source']['cast']['cast_seq']!=S['observed'][1]['source']['cast']['cast_seq'] and S['observed'][0]['source']['action_ordinal']!=S['observed'][1]['source']['action_ordinal'],'distinct_cast_actions')
prepared=rd(D/'PREPARED_MANIFEST.json');check(r['command'][-1]==hashlib.sha256((D/'PREPARED_MANIFEST.json').read_bytes()).hexdigest(),'manifest_consumed')
start=datetime.datetime.fromisoformat(r['started_at_utc']);finish=start+datetime.timedelta(seconds=r['duration_seconds']);snaps=[]
for prefix in ['PRIMARY','BASE']:
 b=rd(D/(prefix+'_BEFORE_V1.json'));a=rd(D/(prefix+'_AFTER_V1.json'));check(b['files']==a['files'] and b['source']==a['source']==C['source'],['snapshot_identity',prefix]);check(datetime.datetime.fromisoformat(b['verified_at_utc'])<start<finish<datetime.datetime.fromisoformat(a['verified_at_utc']),['timely',prefix]);check(b['active_input_writers']==a['active_input_writers']==0,['writers',prefix]); ar=b['archive'];ref(ar['artifact']);expected={x['path']:x for x in ar['members']};check(expected=={x['path']:x for x in b['files']},['snapshot_archive_members',prefix])
 with tarfile.open(ar['artifact']['path']) as tf:
  members=[x for x in tf if x.isfile()];check(len(members)==len(expected) and {x.name for x in members}==set(expected),['archive_exact',prefix])
  for m in members:
   content=tf.extractfile(m).read();row=expected[m.name];check(len(content)==row['bytes'] and hashlib.sha256(content).hexdigest()==row['sha256'],['archive_member',m.name])
 for row in b['files']:ref(row)
 snaps.append(len(b['files']))
root=tomllib.loads(Path('Cargo.lock').read_text())['package'];lock=tomllib.loads((D/'inputs/crate/Cargo.lock').read_text())['package'];key=lambda x:(x['name'],x['version'],x.get('source'),x.get('checksum'));basis={key(x)for x in root};nonprobe=[x for x in lock if x['name']!='u08_paused_probe'];check(len(lock)==140 and len(nonprobe)==139 and all(key(x)in basis for x in nonprobe),'root_dependency_basis')
binary=next(x for x in docs if x.get('stage')=='binary');ref(binary);check('Finished `dev` profile' in raw.decode() and 'Compiling game_core' in raw.decode(),'actual_debug_build');check('--offline'in next(x for x in docs if x.get('stage')=='build')['command'] and '--locked'in next(x for x in docs if x.get('stage')=='build')['command'],'locked_build')
new=C['supplemental_evidence'];vr=rd(new['actual_validator_receipt']['path']);boundary=rd(new['actual_boundary']['path']);check(vr['source_before']==vr['source_after']==C['source'] and vr['exit_code']==0 and vr['command']==boundary['actual_command'],'actual_validator');check(vr['log_sha256']==new['actual_validator_raw']['sha256'],'validator_raw');check(finish<datetime.datetime.fromisoformat(vr['started_at_utc']),'producer_before_validator')
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':C['source']['sha256'],'unchanged_reviewed_sections':unchanged,'old_claims_preserved':oldclaims,'added_claims':newclaims,'all_pointer_assertions_checked':assertions,'unique_refs_checked':len(refs),'raw_operations':len(T),'raw_event_count':len(events),'computed_damage':amounts,'chronological_damage':[210,144],'paused_ticks':len(paused),'snapshot_counts':snaps,'locked_package_count':len(lock),'original_functional_roles':50,'supplemental_in_original_performance_quiescence':False,'limitations':['Full paused RuntimeFactView and retained employee0 Arc are demonstrated by exact executed fail-closed source plus exit0, not independently serialized per-tick/final payloads.','Employee0 retained Arc acquired after both hits; not the chronologically first hit.','Active reconnect is public query; this probe does not prove socket detach/reattach.'],'failures':bad};(A/'SUCCESSOR_INDEPENDENT.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
