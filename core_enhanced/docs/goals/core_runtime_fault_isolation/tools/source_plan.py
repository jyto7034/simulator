#!/usr/bin/env python3
import json
from initialize import HERE, ROOT, artifact, write

D='core_enhanced/docs/goals/core_runtime_safety_observability'
GROUPS=[
 ('RQ01',[1,2,3,4,6],'typed runtime producer→application prepare/commit→public error/latch boundary',[
  'core_enhanced/src/application.rs','core_enhanced/src/game/behavior.rs','core_enhanced/src/game/world/application.rs','core_enhanced/src/game/world.rs','core_enhanced/src/game/world/combat.rs','core_enhanced/src/game/world/resource_runtime.rs','core_enhanced/src/game/world/admin.rs','core_enhanced/src/game/world/admin/commands.rs','core_enhanced/src/game/battle/runtime_control.rs','core_enhanced/src/game/resource/store.rs','core_enhanced/src/game/value_expr/mod.rs','core_enhanced/src/game/battle/core/value_expr_runtime.rs','core_enhanced/src/game/battle/core/damage_runtime.rs','core_enhanced/src/game/battle/core/sustained.rs'],
  'typed categories and producing routes; live outer owner vs candidate rollback; precommit and public projection error distinction; all public mutation ports and classification exclusions'),
 ('RQ02',[2,5,6,7,10],'real actor/socket→manual/admin/tick/detach/reconnect/late callback/quit→new actor',[
  'game_server/src/main.rs','game_server/src/lib.rs','game_server/src/game/player_game_actor/mod.rs','game_server/src/game/player_game_actor/handlers.rs','game_server/src/game/player_game_actor/messages.rs','game_server/src/game/player_game_actor/session.rs','game_server/src/game/player_game_actor/state.rs','game_server/src/game/load_balance_actor/mod.rs','game_server/src/game/load_balance_actor/handlers.rs','game_server/src/game/load_balance_actor/messages.rs'],
  'source flow of early management handlers, tick scheduling, replacement/detach, actor lifetime, actual ws consumer and safe fault fallback'),
 ('RQ03',[3,6,8,9,10],'valid active/profile bytes→writer gates→explicit Quit/new owner/Continue',[
  'game_server/src/persistence/active_run_store.rs','game_server/src/persistence/player_profile_store.rs','game_server/src/game/player_game_actor/handlers.rs','game_server/src/game/player_game_actor/mod.rs','game_server/src/game/load_balance_actor/handlers.rs','core_enhanced/src/game/world/active_run.rs','core_enhanced/src/game/world/node_flow.rs','core_enhanced/src/application.rs'],
  'original store bytes and shared local-install ownership; stop/management/export/reset gates; existing loads_used and checkpoint continue preserved, no new free load or deletion'),
 ('RQ04',[4,5,6],'typed bounded fault/read-model→host snapshot→diagnostic failure',[
  'core_enhanced/src/application.rs','core_enhanced/src/game/world/snapshot.rs','core_enhanced/src/game/behavior.rs','core_enhanced/src/authoring/projection.rs','core_enhanced/src/authoring/diagnostic.rs','core_enhanced/src/authoring/trace.rs','core_enhanced/src/authoring/control.rs','game_server/src/game/player_game_actor/state.rs','game_server/src/game/player_game_actor/messages.rs'],
  'bound fault metadata without copying unbounded cause/site strings; ordinary query failure not terminal; minimum borrowed fault query remains independent of diagnostic serialization'),
 ('RQ05',[1,3,6,12],'speculative authoring candidate/trial/preview and private runtime retry',[
  'core_enhanced/src/authoring/trial.rs','core_enhanced/src/authoring/control.rs','core_enhanced/src/authoring/query.rs','core_enhanced/src/authoring/amount_query.rs','core_enhanced/src/application.rs','core_enhanced/src/game/battle/core/mod.rs','core_enhanced/src/bin/author_content.rs'],
  'candidate owner publication and prefix retry/cancellation unchanged; public accepted application fault lifetime separate from private rollback operation'),
 ('RQ06',[2,3,4,11,12],'U02 attempt/commit observation→terminal cleanup→worker/selection/exporter',[
  'core_enhanced/src/runtime_observation.rs','core_enhanced/src/game/world/application.rs','core_enhanced/src/application.rs','game_server/src/observability/mod.rs','config/runtime_safety.ron','game_server/config/observability.ron',D+'/execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md'],
  'cleanup without fake commit; failed attempt work/reason retained; selected/current clearing and ring/window bounds; disabled/enabled/collector-failure parity'),
 ('RQ07',[1,4,5,7,8,9,10,12],'existing genuine public failure/host fixtures→new end-to-end validation seams',[
  'core_enhanced/src/game/battle/core/boss_action_tests.rs','core_enhanced/src/runtime_safety.rs','game_server/src/test_support.rs','game_server/examples/observe_runtime.rs',D+'/tools/public_profile_probe.py',D+'/tools/run.py','core_enhanced/Cargo.toml','game_server/Cargo.toml','Cargo.toml','Cargo.lock'],
  'reuse actual limit and legal runtime-expression failure fixtures, baseline test commands/public transport; identify required new behavior tests without implementation in P2G20'),
 ('RQ08',[12],'100 predecessor condition applicability and exact audited U02 baseline',[
  D+'/post_audit/master_20261004_002/COMPLETION_AUDIT.md',D+'/post_audit/master_20261004_002/END_CURRENTNESS.json',D+'/execution/master/followups/n02_001/target/master_002/MANIFEST.json',D+'/execution/master/followups/n02_001/COVERAGE.json','core_enhanced/docs/roadmaps/core_first/execution/u02_closure_20261004_001/RESULT.ko.md','core_enhanced/docs/roadmaps/core_first/execution/u02_closure_20261004_001/PUBLICATION_RECORD.json'],
  'reuse precise source-bound old evidence and warnings; classify terminal-observation/public-live-retry impacts, unchanged parser/authority/config/private flow applicability; no current-source claims from old executions'),
]

def main():
    source=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
    working=next(a for a in source if a['ref']['Artifact Type']=='Working Revision')
    rows=[]
    for key,elements,claim,paths,validation in GROUPS:
        assert all((ROOT/p).is_file() for p in paths),(key,[p for p in paths if not (ROOT/p).is_file()])
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':[working['payload']['Current Policy Element Revision Map'][f'PE-CRFI-{n:02}'] for n in elements],'Required claim/boundary':claim,'Source type/role':'Workspace Actual; exact frozen predecessor evidence, actual local host source','Required identity/location constraint':{'repository':str(ROOT),'locations':paths,'dirty_basis':'Preserve all existing changes; capture exact bytes of actually reviewed files'},'Applicability/Necessity':'required','Applicability condition':None,'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['I01','I02','I03','I04']},'Validation requirement':validation})
    plan=artifact('Review Scope/Source Requirement Plan','RSP-CRFI-001',{'Bound Input Working Revision Ref':working['ref'],'Requirement Set':rows,'Currentness Boundary':'Workspace source/local host only. No remote production or Unity compatibility claim. GEX must execute genuine public negative paths.','Candidate Publication':'non-current until complete P2G-20 batch'})
    write('P2G_20_SOURCE_PLAN_001.json',{'artifacts':[plan]})
    print('P2G-20 candidate source plan: 8 groups, all12 elements; formal review pending')

if __name__=='__main__':main()
