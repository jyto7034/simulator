"""U08 source requirement successor including actual battle-profile construction and exact-byte capture, prior to formal review."""
import subprocess
import tarfile
import json
from initialize import ROOT, HERE, artifact, write, sha
C='core_enhanced/src/'
B=C+'game/battle/'
K=B+'core/'
W=C+'game/world/'
D=C+'game/data/'
P='core_enhanced/docs/goals/core_runtime_measurement_profiles/'
GROUPS=[
 ('RQ01',[1,5,6,9,14],'immutable authored schema,source addresses,validation,registration and numeric capability',
  [D+x for x in ['mod.rs','validation.rs','content_pack/raw.rs','content_pack/schema.rs','content_pack/resolver.rs','content_pack/mod.rs','skill_fragment_data.rs','equipment_data.rs','event_data.rs']]
  +[C+x for x in ['game/ability/authoring.rs','game/ability.rs','game/value_expr/catalog.rs','game/value_expr/mod.rs','game/value_expr/eval.rs','authoring/compiled.rs','authoring/definition.rs','authoring/resolve.rs']],
  'New relic definitions/variants are absent; EffectOrigin is attribution only. Actual typed numeric supported selectors and compiler provenance must be connected; no parse-only or unconditional closure fallback.'),
 ('RQ02',[1,2,3,4,10,11,13],'run copy/slot ownership and normal atomic grant,equip,corruption,recycle and Event authority',
  [C+x for x in ['game/resources/inventory.rs','game/resources/equipment_loadout.rs','game/resources/item_slot.rs','game/resources/mod.rs','game/skill_fragment.rs','game/reward.rs','game/employee.rs','game/behavior.rs','game/managers/uuid_manager.rs']]
  +[W+x for x in ['grant_state.rs','maintenance.rs','event_node.rs','shop.rs','reward.rs','admin/grants.rs','admin/commands.rs','application.rs','action_capability.rs']],
  'Current paired loadout and staged grant must include new owners in every producer/consumer. Legacy fragment dust stays distinct. Free Event must be authorized by actual current node/session, not arbitrary mode flag.'),
 ('RQ03',[5,6,7,8,9,13],'individual numeric plan identity,capture authority and all actual amount consumers',
  [K+x for x in ['value_expr_runtime.rs','sim.rs','mod.rs','types.rs','amount_observation.rs','queue.rs','corrosion.rs','skill_runtime/mod.rs','skill_runtime/cast.rs','skill_runtime/flow.rs','skill_runtime/projectile.rs','skill_runtime/area.rs']]
  +[B+x for x in ['runtime_facts.rs','skill_source.rs','damage.rs']]
  +[W+'combat.rs',W+'tactical_preview.rs',C+'game/stats.rs'],
  'Preserve generation-local catalog ptr identity and action/provider authority. Literal Damage with observation off and authored DamageExpr both require normal effective results. Source ending must not drop released numeric plan.'),
 ('RQ04',[1,3,4,10,11,12,13],'save schema,live clone,checkpoint,entry,Continue,new death and new run ownership',
  [W+x for x in ['state.rs','active_run.rs','active_run/continuation.rs','active_run/v8.rs','snapshot.rs','node_flow.rs','headquarters.rs','resource_runtime.rs','helpers.rs']]
  +[C+'application.rs','game_server/src/persistence/active_run_store.rs','game_server/src/game/player_game_actor/handlers.rs'],
  'All capture/restore/serialization/validation/clone/death/return consumers must include new run roots; source-compatible empty defaults cannot hide malformed present state; old save bytes and legacy assets preserved.'),
 ('RQ05',[6,7,8,9,14,15],'normal public queries,authoring CLI/trial,source and amount replay observations',
  [C+x for x in ['authoring/query.rs','authoring/amount_query.rs','authoring/amount.rs','authoring/preview.rs','authoring/trial.rs','authoring/scenario.rs','bin/consume_content_pack.rs','runtime_observation.rs']]
  +[W+'authoring_observation.rs',W+'authoring_lifecycle.rs',W+'admin/catalog.rs',W+'admin/fixtures.rs'],
  'preview_amount replays committed evaluation; PreviewSkillTarget is targeting. New pre-use variant/numeric query must not mislabel old ports. Normal trial/content identity and source origins reflect actual numeric execution.'),
 ('RQ06',[8,10,12,13,15,16],'admission,retained owners,CloneShape,rollback and inherited measurement',
  [C+'authoring/clone_shape.rs',C+'authoring/control.rs',C+'runtime_safety.rs',B+'runtime_control.rs',K+'runtime_facts.rs',C+'game/resource/store.rs',
   'game_server/src/runtime_measurement/mod.rs','config/runtime_safety.ron'],
  'New owned vectors/maps/Arc retention measured before allocation at real boundaries. Preserve default8MiB/PC and native cumulative admission,not just final extent. U07-M measurement remains applicable.'),
 ('RQ07',list(range(1,17)),'concrete validation seams,normal external inputs and current predecessor evidence',
  [W+'tests/'+x for x in ['r07_copies.rs','equipment.rs','checkpoint_validation.rs','u04_continuation.inc.rs','u06_starter.rs','u07_objects.rs','resource_lifetimes.rs']]
  +[C+'authoring/trial/tests/r07_copies.rs',C+'authoring/trial/tests/amount_completion.rs',K+'value_expr_runtime/tests/amount_replay.rs',K+'value_expr_runtime/tests/legacy_amount.rs',
   'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs','core_enhanced/tools/runtime_architecture_preservation/src/main.rs',
   'game_resources/data/equipments/base.ron','game_resources/data/skills/base.ron','game_resources/data/skill_fragments/base.ron','game_resources/data/events/story/base.ron',
   'Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml',
   P+'post_audit/u07m_20261006_001/AUDIT_CLOSURE.json',P+'execution/single_goal/target/u07m_001/MANIFEST.json',
   'core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/game_rules/growth-and-rewards.md'],
  '16 new policy acceptance groups,191 predecessor conditions+31 R06,10 dimensions; current normal path numeric/copy/cost/recovery fault inputs and whole-goal independent audit; preserve original PC/R05 and failures.')]

GROUPS[2][3].extend([C+'game/events/combat.rs', C+'game/combat_setup/player_spawns.rs', C+'game/combat_setup/mod.rs', B+'types.rs', B+'stat_pipeline.rs', K+'build.rs', K+'commands.rs', K+'authoring_observation.rs'])

def main():
    arts=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
    working=next(a for a in arts if a['ref']['Artifact Type']=='Working Revision')
    assert json.loads((HERE/'P2G_10_RESULT_001.json').read_text())['is_p2g10_complete']
    rows=[];paths=set()
    for key,elements,claim,locations,validation in GROUPS:
        missing=[p for p in locations if not (ROOT/p).is_file()]
        assert not missing,(key,missing)
        paths.update(locations)
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':[working['payload']['Current Policy Element Revision Map'][f'PE-CRNV-{n:02}'] for n in elements],
          'Required claim/boundary':claim,'Source type/role':'Workspace Actual',
          'Required identity/location constraint':{'repository':str(ROOT),'locations':locations,'dirty_basis':'Actual exact bytes;preserve all existing changes'},
          'Applicability/Necessity':'required','Applicability condition':None,
          'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['I01','I02','I03','I04']},'Validation requirement':validation})
    for key,role,reason in [('RQ08','Released Actual','No external released-version target. Local release/public validation is required later GEX evidence.'),('RQ09','Deployed Actual','No deployed service or Unity scope;current host transport/save consumers are included where affected. U26/U30-P own actual Unity.')]:
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':list(working['payload']['Current Policy Element Revision Map'].values()),
          'Required claim/boundary':role,'Source type/role':role,'Required identity/location constraint':None,'Applicability/Necessity':'not_applicable','Applicability condition':None,
          'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['O01','O03']},'Validation requirement':reason})
    plan=artifact('Review Scope/Source Requirement Plan','RSP-CRNV-002',{'Bound Input Working Revision Ref':working['ref'],'Requirement Set':rows,
      'Predecessor Source Plan':'RSP-CRNV-001;110file capture preserved. Add8 actual player-profile/build/stat/command/observation producer-consumer files before formal review;policy unchanged.', 'Currentness Boundary':'Local dirty workspace;U07-M prerequisite exact except published status documents. Source capture does not by itself complete semantic review.'})
    write('P2G_20_SOURCE_PLAN_002.json',{'artifacts':[plan]})
    (HERE/'evidence').mkdir(exist_ok=True)
    with tarfile.open(HERE/'evidence/REVIEW_SOURCE_002.tar.gz','x:gz') as archive:
        for p in sorted(paths):archive.add(ROOT/p,arcname=p,recursive=False)
    write('evidence/REVIEW_SOURCE_MANIFEST_002.json',{'plan_ref':plan['ref'],'files':[{'path':p,'sha256':sha(ROOT/p),'bytes':(ROOT/p).stat().st_size} for p in sorted(paths)],'archive_sha256':sha(HERE/'evidence/REVIEW_SOURCE_002.tar.gz')})
    with (HERE/'evidence/GIT_STATUS_REVIEW002.txt').open('x') as stream:
        subprocess.run(['git','status','--short','--untracked-files=normal'],cwd=ROOT,stdout=stream,check=True)
    print(f'P2G20 source capture:{len(rows)} requirements/{len(paths)} exact sources;formal semantic review pending')

if __name__=='__main__':main()
