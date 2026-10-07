"""Compare all 576 U05/U06 rows after the explicit inline action storage costs.

Damage is omitted from canonical serialization, so its clone-shape visits and
native accounting are unchanged. Allocation bytes retain the new inline enum.
No workload, typed error, numeric ceiling or allocation count is normalized.
"""
import copy,hashlib,json,re,tarfile
from pathlib import Path
ROOT=Path('/mnt/f/work/simulator')
C=Path(__file__).resolve().parent
BEFORE=ROOT/'core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal/compatibility/current_001/evidence/pc_raw_current_003.json'
AFTER=C/'evidence/pc_raw_current_001.json'
LOG=ROOT/'core_enhanced/docs/goals/core_battle_starter_tactics/evidence/compat_pc_001.log'
OUTPUT=C/'evidence/PC_IMMEDIATE_ACCOUNTED_U06.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
layout_lines=re.findall(r'U06_LAYOUT (\{[^\n]+\})',LOG.read_text())
assert len(layout_lines)==1
layout=json.loads(layout_lines[0]);assert layout['basic_action_bytes']==40,layout
archive=ROOT/'core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal/target/u05_002/SOURCE_AFTER.tar.gz'
def declaration(text,name):
 return text.split('pub struct '+name+' {',1)[1].split('\n}',1)[0]
with tarfile.open(archive) as old_source:
 unit_path='core_enhanced/src/game/battle/core/types.rs'
 basic_path='core_enhanced/src/game/data/abnormality_data.rs'
 old_unit=old_source.extractfile(unit_path).read().decode()
 old_basic=old_source.extractfile(basic_path).read().decode()
 assert declaration(old_unit,'RuntimeUnit')==declaration((ROOT/unit_path).read_text(),'RuntimeUnit')
 current_basic=declaration((ROOT/basic_path).read_text(),'BasicAttackDef')
 action_field='\n    #[serde(default, skip_serializing_if = "BasicAttackAction::is_damage")]\n    pub action: BasicAttackAction,'
 assert current_basic.count(action_field)==1
 assert current_basic.replace(action_field,'')==declaration(old_basic,'BasicAttackDef')
 weapon_path='core_enhanced/src/game/data/equipment_data.rs'
 old_weapon=old_source.extractfile(weapon_path).read().decode()
 current_weapon=declaration((ROOT/weapon_path).read_text(),'WeaponCombatProfile')
 weapon_field='\n    #[serde(\n        default,\n        skip_serializing_if = "super::abnormality_data::BasicAttackAction::is_damage"\n    )]\n    pub action: super::abnormality_data::BasicAttackAction,'
 assert current_weapon.count(weapon_field)==1
 assert current_weapon.replace(weapon_field,'')==declaration(old_weapon,'WeaponCombatProfile')
 for path,name in [('core_enhanced/src/game/battle/types.rs','UnitCombatProfile'),('core_enhanced/src/game/battle/scenario.rs','ScenarioUnitSpawn')]:
  assert declaration(old_source.extractfile(path).read().decode(),name)==declaration((ROOT/path).read_text(),name)
def without_time(path):
 value=json.loads(path.read_text())
 for row in value['rows']:
  for stage in ('measurement','setup'):
   if stage in row:del row[stage]['elapsed_ns']
 return value
before,after=map(without_time,(BEFORE,AFTER))
assert before['schema']==after['schema']==1 and before['seed']==after['seed']==17
assert len(before['rows'])==len(after['rows'])==576
normalized=copy.deepcopy(after);costs=[]
for index,(old,new) in enumerate(zip(before['rows'],normalized['rows'])):
 family=old['family'];assert family==new['family'];changes=[]
 if family in ('literal_battle_public_step','damage_burst_private_transaction'):
  units=old['enemies']+old['allies'];assert units==old['units'] and old['allies']==12
  capacity=4
  while capacity*7//8<units:capacity*=2
  slots=10 if family=='literal_battle_public_step' and old['chunk_ms']==100 else 1
  assert len(new['native_usage'])==slots
  # RuntimeUnit gains one40-byte action. Setup grows4+8+...+capacity;
  # each measured transaction clones capacity buckets. No tactical states.
  # spawn_scenario_group clones each group's Vec<ScenarioUnitSpawn> once.
  # Its inline TestFixture UnitCombatProfile contains both BasicAttackDef
  # and Option<WeaponCombatProfile>: two40-byte actions, including None's
  # storage. Thus that existing setup allocation grows80 bytes/spawn.
  for stage,delta in [('setup',40*(2*capacity-4)+80*units),('measurement',40*capacity*slots)]:
   if new[stage]['allocation_observer']:
    raw=new[stage]['requested_bytes_sum'];assert raw>=delta
    new[stage]['requested_bytes_sum']-=delta
    changes.append({'field':stage+'.requested_bytes_sum','before':raw-delta,'after':raw,'delta':delta})
   else:assert new[stage]['requested_bytes_sum'] is None
 else:assert family in ('live_adapter_repetition_synthetic','kernel_resolved_aggregate_synthetic')
 assert old==new,(index,family,{k:(old.get(k),new.get(k)) for k in old.keys()|new.keys() if old.get(k)!=new.get(k)})
 costs.append({'row':index,'family':family,'warmup':old['warmup'],'sample':old['sample'],'allocation_observer':old['measurement']['allocation_observer'],'changes':changes})
assert before==normalized
sources=['core_enhanced/src/game/battle/core/types.rs','core_enhanced/src/game/data/abnormality_data.rs','core_enhanced/src/game/data/equipment_data.rs','core_enhanced/src/game/battle/types.rs','core_enhanced/src/game/battle/scenario.rs','core_enhanced/src/game/battle/core/build.rs','core_enhanced/src/game/battle/core/mod.rs','core_enhanced/src/authoring/clone_shape.rs']
result={'passed':True,'rows':576,'before':{'path':str(BEFORE.relative_to(ROOT)),'sha256':sha(BEFORE)},'after':{'path':str(AFTER.relative_to(ROOT)),'sha256':sha(AFTER)},'claim':'Every non-timing field equals immediate U05 after only declared inline action allocation-byte increases:40 bytes/runtime-unit bucket and80 bytes/initial spawn-vector element. Actual allocated bytes are larger; no unchanged-cost claim.','excluded':['setup.elapsed_ns','measurement.elapsed_ns'],'unchanged_gates':'Original gameplay and typed refusal oracle, native usage, allocation request counts, public guards and R05 ceilings remain unchanged.','source_basis':{p:sha(ROOT/p) for p in sources},'predecessor_source_archive':{'path':str(archive.relative_to(ROOT)),'sha256':sha(archive),'declaration_checks':['RuntimeUnit, UnitCombatProfile and ScenarioUnitSpawn fields are byte-identical','BasicAttackDef and WeaponCombatProfile fields are byte-identical after removing only their new inline action fields']},'compiler_layout':layout,'layout_log_sha256':sha(LOG),'per_row_cost_changes':costs}
with OUTPUT.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print('PASS 576 immediate U05 rows after explicit40-byte inline action storage cost; original gates unchanged')
