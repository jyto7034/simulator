from release_gate import require
require('release')
from pathlib import Path
"""Current release CLI: normal creation, source withdrawal, actual attack/destruction."""
import json,subprocess,sys
from initialize import ROOT,HERE
from run import identity,sha
label=sys.argv[1];inputs=Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_independent_objects')/'evidence/public_object_combat_inputs_001';out=HERE/('evidence/public_object_combat_'+label);out.mkdir()
binary=ROOT/'.cache/d07-runtime-amount-product-target/release/author_content';source=identity();rows=[]
for case in ['ice','small','actor']:
 fixture=inputs/case;r=subprocess.run([str(binary),'simulate',str(fixture/'content'),str(fixture/'selection.json'),str(fixture/'scenario.json')],cwd=ROOT,capture_output=True)
 (out/(case+'.stdout')).write_bytes(r.stdout);(out/(case+'.stderr')).write_bytes(r.stderr);assert r.returncode==0,(case,r.stderr[-1000:])
 values=[json.loads(l)['payload'] for l in r.stdout.splitlines() if l.startswith(b'{')];steps={s['index']:s for s in values if s.get('kind')=='step'}
 def update(n):return steps[n]['public_result']['payload']['battle_update']
 def checkpoint(n):return update(n)['checkpoint']
 obj=checkpoint(27)['independent_objects']['active'][0];oid=obj['object_id'];creator=obj['source']['creator'];assert obj['created_at_ms']==1101
 assert checkpoint(28)['independent_objects']['active']==checkpoint(27)['independent_objects']['active']
 assert all(u['unit_instance_id']!=creator for u in checkpoint(28)['units'])
 assert update(28)['server_battle_time_ms']==1700
 events=update(31)['events_delta']['events'];damage=[e for e in events if e['event']['type']=='HpChanged' and e['event'].get('source_instance_id')==oid and e['event']['delta']<0];removed=[e for e in events if e['event']['type']=='IndependentObjectRemoved' and e['event']['receipt']['object_id']==oid]
 assert all(e['time_ms']>1700 for e in damage+removed)
 if case=='ice':
  assert obj['max_health']==3 and obj['defense']==0 and not obj['can_attack'] and not obj['can_receive_heal']
  hits=[e for e in events if e['event']['type']=='HpChanged' and e['event'].get('target_instance_id')==oid]
  assert [e['event']['hp_after'] for e in hits]==[2,1,0]
  assert len(removed)==1 and removed[0]['event']['receipt']['reason']=='destroyed'
  assert not checkpoint(31)['independent_objects']['active']
  assert not any(e['event']['type']=='UnitDied' and oid in json.dumps(e) for e in events)
  enemy=removed[0]['event']['receipt']['source_actor'];assert any(u['unit_instance_id']==enemy and u['position']=={'x':4,'y':6} for u in checkpoint(31)['units'])
 elif case=='small':
  assert len(damage)==1 and damage[0]['event']['raw_damage']==24 and damage[0]['event']['final_damage']==25 and damage[0]['event']['damage_source']=='Ability'
  assert len(removed)==1 and removed[0]['event']['receipt']['reason']=='attack_consumed' and damage[0]['seq']<removed[0]['seq']
  assert not checkpoint(31)['independent_objects']['active']
 else:
  assert len(damage)==1 and damage[0]['event']['final_damage']==10 and damage[0]['event']['damage_source']=='BasicAttack'
  assert any(e['event']['type']=='AttackStart' and e['event'].get('attacker_instance_id')==oid for e in events)
  assert checkpoint(31)['independent_objects']['active'][0]['object_id']==oid
 rows.append({'case':case,'object':obj,'damage':damage,'removals':removed,'stdout_sha256':sha(out/(case+'.stdout'))})
assert source==identity()
summary={'passed':True,'source':source,'binary_sha256':sha(binary),'rows':rows,'input_sha256':{str(p.relative_to(ROOT)):sha(p) for p in inputs.rglob('*') if p.is_file()},'claim':'Normal external pack/owned fragment/equip/preview-focus/cast/withdraw/clock commands reach actor BasicAttack, trap ability damage+consumption, and swept wall contact+damage+destruction+enemy route resumption after creator withdrawal.','fixture_limits':'Explicit legacy starter/reward/readiness fixtures; actor is custom authored; ice wall HP3/defense0 only in this external fixture to observe destruction promptly. Official wall HP120/defense10 separately verified without tuning. No production catalog or numeric limits changed.'}
(out/'SUMMARY.json').write_text(json.dumps(summary,indent=2)+'\n');print('PASS current release public actor/trap/wall combat after creator withdrawal')
