#!/usr/bin/env python3
"""Candidate review requirements and exact source capture; not a completed review."""
import json, subprocess, tarfile
from initialize import HERE, ROOT, artifact, write, sha
C='core_enhanced/src/'
B=C+'game/battle/'
K=B+'core/'
W=C+'game/world/'
S='game_server/src/'
D='game_resources/data/'
GROUPS=[
('RQ01',[4,5,6,19], 'strict basic action/weapon/fragment/employee authored data and compilation',
 [C+'game/data/'+x for x in ['abnormality_data.rs','equipment_data.rs','employee_data.rs','skill_data.rs','skill_fragment_data.rs','live_ron.rs','mod.rs','validation.rs','corroded_employee_data.rs','summoned_unit_data.rs','run_policy_data.rs','content_pack/schema.rs','content_pack/raw.rs','content_pack/canonical.rs']]+[C+'game/ability.rs',C+'authoring/clone_shape.rs',C+'authoring/view.rs',C+'authoring/compiled.rs']+[str(p.relative_to(ROOT)) for p in sorted((ROOT/D).rglob('*.ron'))],
 'normal loader/strict fields/profile conversion/authoring metadata; six actual candidate sets and skill capability; identify unsupported shapes and negative inputs'),
('RQ02',[4,5,7,8,14,19], 'basic action target acquisition/start/release/heal/effects/observation and deterministic ordering',
 [K+x for x in ['basic_attack.rs','targeting.rs','target_usefulness.rs','commands.rs','damage_runtime.rs','sim.rs','queue.rs','runtime_facts.rs','authoring_observation.rs','resonance_runtime.rs','skill_runtime/cast.rs','skill_runtime/mod.rs']]+[B+x for x in ['event_log.rs','enums.rs','types.rs','runtime_facts.rs','tile_range.rs','buffs.rs']],
 'actual reachable action target and payload; damage versus ApplyHeal; death/damage/heal bucket priority; no damage proc for pureheal; source/clone/retained extent'),
('RQ03',[1,2,3,4,6,15], 'public roster/grants/deployment/withdrawal/TP and six sets',
 [W+x for x in ['combat.rs','helpers.rs','grant_state.rs','headquarters.rs','reward.rs','maintenance.rs','battle_capability.rs','action_capability.rs']]+[C+'game/employee.rs',C+'application.rs',C+'game/behavior.rs',K+'build.rs'],
 'normal StartRun/recruit grant and equipment resolution; active employee identity/condition priority; concurrent8 versus attempt participants; TP and roster query'),
('RQ04',[7,8,9,10,11,12,15], 'stance/relocation/swap/work state, validation, mutation, timers and query',
 [K+x for x in ['tactical_stance.rs','tactical_relocation.rs','tactical_swap.rs','field_work.rs','types.rs','mod.rs','triggers.rs','sustained.rs','death_runtime.rs']]+[W+'combat.rs',W+'battle_capability.rs',B+'scenario.rs'],
 'both-unit preflight/candidate rollback; rest/timer/CC/fieldwork reconcile ordering and complete-once fact; preview and actual share authority'),
('RQ05',[8,13,14,15], 'continuous movement/blocking geometry, state preservation and target exception',
 [K+'movement/'+x for x in ['blocking.rs','planner.rs','engine.rs','types.rs','lifecycle.rs','path.rs','steering.rs','rapier_backend.rs']]+[K+'targeting.rs',K+'basic_attack.rs',K+'sim.rs',B+'scenario.rs'],
 'trace actual segment motion→block acquisition/maintenance/release→tile/body projection→basic target range; contact sweep and stable capacity tie-breaks'),
('RQ06',[15,16,17,18,19], 'Core command/query focus/settings/playback and actual host route/reconnect',
 [W+x for x in ['state.rs','snapshot.rs','application.rs','combat.rs','battle_capability.rs','action_capability.rs']]+[C+'game/combat_preview/mod.rs',C+'game/behavior.rs',C+'application.rs',K+'corrosion.rs']+[S+'game/player_game_actor/'+x for x in ['mod.rs','handlers.rs','messages.rs','state.rs','session.rs']]+['game_server/docs/unity_bridge_protocol.md'],
 'sole playback/restore owner, nesting/Pause/settings/terminal projection; normal host input/serialize/tick/reconnect; preserved corrosion host-clock exception'),
('RQ07',[3,5,10,12,16,17,18,19], 'clone/save/rollback/Continue/SavePoint/fault and dynamic retained extent',
 [W+x for x in ['active_run.rs','active_run/continuation.rs','node_flow.rs','resource_runtime.rs','authoring_observation.rs']]+[K+x for x in ['resource_runtime.rs','types.rs','runtime_snapshot.rs'] if (ROOT/(K+x)).is_file()]+[B+'runtime_control.rs',C+'authoring/clone_shape.rs']+[S+'persistence/'+x for x in ['active_run_store.rs','player_profile_store.rs','mod.rs']],
 'actual writer/reader/independent copy and typed failure boundaries; new fields captured in budget and fault isolation, no invented active-battle disk persistence'),
('RQ08',list(range(1,21)), 'required existing regression/public authoring/headless/host/probe inputs and predecessor conditions',
 [W+'tests/'+x for x in ['combat.rs','equipment.rs','snapshots_and_start.rs','checkpoint_validation.rs','r09_combat.inc.rs','u04_continuation.inc.rs']]+[str(p.relative_to(ROOT)) for p in sorted((ROOT/'core_enhanced/tests').glob('*.rs'))]+[S+'test_support.rs',S+'game/player_game_actor/handlers/tests/fault.rs',S+'game/player_game_actor/handlers/tests/durable.rs','Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml',
 'core_enhanced/docs/skill_target_contract.md','core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md',
 'core_enhanced/docs/goals/core_resonance_lifecycle/post_audit/u05_20261004_002/COMPLETION_AUDIT.md',
 'core_enhanced/docs/goals/core_resonance_lifecycle/post_audit/u05_20261004_002/AUDIT_CLOSURE.json',
 'core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal/target/u05_002/MANIFEST.json'],
 'valid existing fixtures and actual selectors/commands; preserve 141 inherited and separate31 R06, original oracles and approved R05 exception; no execution success claimed'),
]

# Followed actual grant/copy/authoring/stat consumers during RQ01/RQ03/RQ07 review.
EXTRA = {
 'RQ01': ['core_enhanced/src/authoring/definition.rs','core_enhanced/src/authoring/resolve.rs','core_enhanced/src/authoring/preview.rs'],
 'RQ02': ['core_enhanced/src/game/battle/damage.rs'],
 'RQ03': ['core_enhanced/src/game/skill_fragment.rs','core_enhanced/src/game/skill_fragment/transition.rs','core_enhanced/src/game/reward.rs','core_enhanced/src/game/resources/equipment_loadout.rs','core_enhanced/src/game/recruitment.rs'],
 'RQ07': ['core_enhanced/src/game/world/tests/mod.rs'],
}
for key, _elements, _claim, locations, _validation in GROUPS:
    locations += EXTRA.get(key, [])

def main():
    source=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
    working=next(a for a in source if a['ref']['Artifact Type']=='Working Revision')
    rows=[];paths=set()
    for key,elements,claim,locations,validation in GROUPS:
        missing=[p for p in locations if not (ROOT/p).is_file()]
        assert not missing,(key,missing)
        paths.update(locations)
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':[working['payload']['Current Policy Element Revision Map'][f'PE-CBST-{n:02}'] for n in elements],
          'Required claim/boundary':claim,'Source type/role':'Workspace Actual','Required identity/location constraint':{'repository':str(ROOT),'locations':locations,'dirty_basis':'Exact actual bytes; preserve pre-existing changes'},
          'Applicability/Necessity':'required','Applicability condition':None,'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['I01','I02','I03','I04','I05','I06']},'Validation requirement':validation})
    for key,role,reason in [('RQ09','Released Actual','No external release/build compatibility target in U06; local release verification belongs to GEX'),('RQ10','Deployed Actual','No remote deployment or Unity implementation scope; actual local host source is required in RQ06/07')]:
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':list(working['payload']['Current Policy Element Revision Map'].values()),'Required claim/boundary':role,
          'Source type/role':role,'Required identity/location constraint':None,'Applicability/Necessity':'not_applicable','Applicability condition':None,
          'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['O03']},'Validation requirement':reason})
    plan=artifact('Review Scope/Source Requirement Plan','RSP-CBST-003',{'Bound Input Working Revision Ref':working['ref'],'Requirement Set':rows,
        'Currentness Boundary':'Local workspace and reachable Core/host normal routes. No remote/Unity/deployed claim. Candidate only until complete review batch.'})
    write('P2G_20_SOURCE_PLAN_003.json',{'artifacts':[plan]})
    (HERE/'evidence').mkdir(exist_ok=True)
    with tarfile.open(HERE/'evidence/REVIEW_SOURCE_003.tar.gz','x:gz') as archive:
        for p in sorted(paths):archive.add(ROOT/p,arcname=p,recursive=False)
    write('evidence/REVIEW_SOURCE_MANIFEST_003.json',{'plan_ref':plan['ref'],'files':[{'path':p,'sha256':sha(ROOT/p)} for p in sorted(paths)],'archive_sha256':sha(HERE/'evidence/REVIEW_SOURCE_003.tar.gz')})
    with (HERE/'evidence/GIT_STATUS_REVIEW_003.txt').open('x') as stream:subprocess.run(['git','status','--short','--untracked-files=normal'],cwd=ROOT,stdout=stream,check=True)
    print(f'U06 P2G-20 candidate source plan: {len(rows)} requirements, {len(paths)} captured sources; review not yet complete')
if __name__=='__main__':main()
