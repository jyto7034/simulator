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
('RQ01',[1,2,3],'deployment and all positive resonance writers/readers',
 [K+x for x in ['build.rs','commands.rs','resonance_runtime.rs','damage_runtime.rs','tactical_stance.rs','field_work.rs','sim.rs','types.rs','mod.rs','movement/engine.rs']]+[B+'types.rs',C+'game/employee.rs',C+'game/ability.rs'],
 'trace actual spawn/regen/attack/damage/Rest/effect dispatch and gate; floor, cadence, cap, CC and no bank seams'),
('RQ02',[4,6,7,8,9,10,11],'normal input→validation→pending start→cancel/corrosion and target ownership',
 [K+x for x in ['commands.rs','corrosion.rs','targeting.rs','target_usefulness.rs','death_runtime.rs','queue.rs','movement/lifecycle.rs','tactical_relocation.rs','tactical_swap.rs']]+[W+'combat.rs',W+'battle_capability.rs',C+'game/behavior.rs',C+'application.rs'],
 'follow public manual/corrosion commands, preview/focus, paused acceptance, consume/full recheck, CC/target cancellation and accepted tile'),
('RQ03',[3,4,5,6,7,11,12],'cast/control/Flow/buff/channel/native carrier/action facts versus actual use lifetime',
 [K+'skill_runtime/'+x for x in ['mod.rs','cast.rs','flow.rs','projectile.rs','area.rs']]+[K+x for x in ['sustained.rs','summon_runtime.rs','triggers.rs','resource_runtime.rs','authoring_observation.rs','runtime_facts.rs']]+[B+'buffs.rs',B+'runtime_facts.rs',B+'runtime_control.rs',B+'event_log.rs',C+'game/events/combat.rs'],
 'map separate control terminal/effects settled/buff removal and cleanup; all release/interrupt/death/withdrawal completion paths'),
('RQ04',[1,2,3,5,8,10,12],'public authored/compiled schema→strict publication→all current definitions',
 [C+'game/data/'+x for x in ['skill_data.rs','skill_fragment_data.rs','live_ron.rs','mod.rs','validation.rs','corroded_employee_data.rs','summoned_unit_data.rs']]+[C+'authoring/view.rs',C+'authoring/amount.rs']+[str(p.relative_to(ROOT)) for p in sorted((ROOT/D).rglob('*.ron'))],
 'inventory all skills and legacy gain/lock fields; Flow/UseBuff/channel/creation shapes; public loader errors and effect dependency seams'),
('RQ05',[4,8,9,12,13],'public query/serialization and host command/tick/reconnect consumer',
 [W+x for x in ['snapshot.rs','state.rs','action_capability.rs','battle_capability.rs','application.rs']]+[C+'game/combat_preview/mod.rs',C+'application.rs']+[S+'game/player_game_actor/'+x for x in ['mod.rs','handlers.rs','messages.rs','state.rs','session.rs']]+['game_server/docs/unity_bridge_protocol.md','core_enhanced/docs/skill_target_contract.md','core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md'],
 'normal public input/query→host serialization/reconnect; observably distinguish no resonance/hardCC/use lock/pending without client rule reconstruction'),
('RQ06',[1,4,5,6,9,11,12,13,14],'U04 battle snapshot/Continue/checkpoint actual persistence',
 [W+x for x in ['active_run.rs','active_run/continuation.rs','node_flow.rs','combat.rs']]+[S+'persistence/'+x for x in ['active_run_store.rs','player_profile_store.rs','mod.rs']],
 'find actual save format and validation; private battle snapshot graph; unsupported original preservation and U04 rollback boundary'),
('RQ07',list(range(1,15)),'current acceptance tests/public RON/host verification and inherited proof',
 [str(p.relative_to(ROOT)) for p in sorted((ROOT/'core_enhanced/tests').rglob('*')) if p.is_file()]+[W+'tests/combat.rs',W+'tests/checkpoint_validation.rs',K+'boss_action_cancellation_tests.rs',S+'test_support.rs','Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml',
 'core_enhanced/docs/goals/core_result_persistence/post_audit/u04_20261004_002/COMPLETION_AUDIT.md',
 'core_enhanced/docs/goals/core_result_persistence/post_audit/u04_20261004_002/EXIT_CURRENTNESS.json',
 'core_enhanced/docs/goals/core_result_persistence/post_audit/u04_20261004_002/AUDIT_ARTIFACTS.json',
 'core_enhanced/docs/goals/core_result_persistence/execution/single_goal/target/u04_003/MANIFEST.json'],
 'reuse exact prerequisite evidence under impact classification; identify existing normal public and deterministic lifecycle regression seams'),
]

def main():
    source=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
    working=next(a for a in source if a['ref']['Artifact Type']=='Working Revision')
    rows=[];paths=set()
    for key,elements,claim,locations,validation in GROUPS:
        missing=[p for p in locations if not (ROOT/p).is_file()]
        assert not missing,(key,missing)
        paths.update(locations)
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':[working['payload']['Current Policy Element Revision Map'][f'PE-CRSL-{n:02}'] for n in elements],
          'Required claim/boundary':claim,'Source type/role':'Workspace Actual','Required identity/location constraint':{'repository':str(ROOT),'locations':locations,'dirty_basis':'Exact actual bytes; preserve pre-existing changes'},
          'Applicability/Necessity':'required','Applicability condition':None,'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['I01','I02','I03','I04','I05','I06','I07']},'Validation requirement':validation})
    for key,role,reason in [('RQ08','Released Actual','No external release/build compatibility target in U05; local release verification belongs to GEX'),('RQ09','Deployed Actual','No remote deployment or Unity implementation scope; actual local host source is required in RQ05/06')]:
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':list(working['payload']['Current Policy Element Revision Map'].values()),'Required claim/boundary':role,
          'Source type/role':role,'Required identity/location constraint':None,'Applicability/Necessity':'not_applicable','Applicability condition':None,
          'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['O03']},'Validation requirement':reason})
    plan=artifact('Review Scope/Source Requirement Plan','RSP-CRSL-001',{'Bound Input Working Revision Ref':working['ref'],'Requirement Set':rows,
        'Currentness Boundary':'Local workspace and reachable Core/host normal routes. No remote/Unity/deployed claim. Candidate only until complete review batch.'})
    write('P2G_20_SOURCE_PLAN_001.json',{'artifacts':[plan]})
    (HERE/'evidence').mkdir(exist_ok=True)
    with tarfile.open(HERE/'evidence/REVIEW_SOURCE_001.tar.gz','x:gz') as archive:
        for p in sorted(paths):archive.add(ROOT/p,arcname=p,recursive=False)
    write('evidence/REVIEW_SOURCE_MANIFEST_001.json',{'plan_ref':plan['ref'],'files':[{'path':p,'sha256':sha(ROOT/p)} for p in sorted(paths)],'archive_sha256':sha(HERE/'evidence/REVIEW_SOURCE_001.tar.gz')})
    with (HERE/'evidence/GIT_STATUS_INTAKE.txt').open('x') as stream:subprocess.run(['git','status','--short','--untracked-files=normal'],cwd=ROOT,stdout=stream,check=True)
    print(f'U05 P2G-20 candidate source plan: {len(rows)} requirements, {len(paths)} captured sources; review not yet complete')
if __name__=='__main__':main()
