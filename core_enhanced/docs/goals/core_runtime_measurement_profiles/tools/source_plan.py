"""Capture U07-M actual sources before publishing the formal static review."""
import subprocess
import tarfile
import json
from initialize import ROOT, HERE, artifact, write, sha

C = 'core_enhanced/src/'
B = C + 'game/battle/'
K = B + 'core/'
W = C + 'game/world/'
S = 'game_server/src/'
P = 'core_enhanced/docs/goals/core_independent_objects/'
GROUPS = [
 ('RQ01', [1,2,6,10], 'trusted profile parser, immutable identity and publication authority',
  [C+x for x in ['runtime_safety.rs','runtime_safety/tests.rs','game/data/mod.rs','game/data/validation.rs','authoring/query.rs','authoring/trial.rs','authoring/mod.rs','application.rs']]
  + [S+'bootstrap.rs',S+'main.rs','config/runtime_safety.ron'],
  'strict omitted/default canonical compatibility; explicit finite dev selection; production identity consumer versus output-only hash; no content/save authority'),
 ('RQ02', [1,2,3,4,6,8], 'native operation owner, all retained admission/checks and nested/rollback boundaries',
  [B+'runtime_control.rs',B+'runtime_facts.rs']
  + [K+x for x in ['runtime_facts.rs','sim.rs','mod.rs','types.rs','resource_runtime.rs','queue.rs','object_creation.rs','object_evolution.rs','movement/blocking.rs']]
  + [W+x for x in ['application.rs','resource_runtime.rs','node_flow.rs','active_run.rs','combat.rs']]
  + [C+'authoring/control.rs',K+'value_expr_runtime.rs'],
  'default 8MiB validation remains; trusted policy only selects retained ceiling; direct owner/result checks agree; work/clone/expression unchanged; nonrefundable admission and sticky failure'),
 ('RQ03', [3,4,5,7,8], 'fixed operation metrics and bounded existing Core/host observation transport',
  [C+'runtime_observation.rs',C+'application/runtime_observation_tests.rs',S+'observability/mod.rs',S+'observability/tests.rs',S+'lib.rs',S+'game/load_balance_actor/handlers.rs',
   'game_server/config/observability.ron','game_server/config/observability/README.md',
   'core_enhanced/tools/runtime_architecture_final_validation/src/public_guards.rs'],
  'seed/request/accepted growth/extent distinction; ring/queue/drop bounds; allocator hook reuse limits and optional feature scope; host process RSS only; no gameplay feedback'),
 ('RQ04', [1,3,6,7,8], 'normal application, actual host session, clone, checkpoint/Continue and fault consumers',
  [C+'application.rs',W+'snapshot.rs',W+'active_run/continuation.rs',C+'authoring/scenario.rs',C+'authoring/resolve.rs',C+'bin/consume_content_pack.rs',C+'game/data/content_pack/mod.rs']
  + [S+'game/player_game_actor/'+x for x in ['handlers.rs','state.rs','session.rs','messages.rs','mod.rs','handlers/tests/fault.rs','handlers/tests/durable.rs']]
  + [S+'persistence/'+x for x in ['active_run_store.rs','player_profile_store.rs','mod.rs']],
  'normal execution and current-host restore; immutable live profile clone; failure save protection; bounded measurement replay mismatch rejected before mutation'),
 ('RQ05', [2,3,4,5,6,7,8,9], 'existing normal fixtures, public measurement/probe seams and affected regressions',
  [W+'tests/'+x for x in ['combat.rs','checkpoint_validation.rs','u06_starter.rs','u04_continuation.inc.rs']]
  + [K+'tests/u07_block_lifecycle.inc.rs', 'core_enhanced/tools/authoring_trial_api_probe/examples/runtime_application.rs',
   'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs',
   'core_enhanced/tools/runtime_architecture_preservation/src/main.rs']
  + [str(p.relative_to(ROOT)) for p in sorted((ROOT/'core_enhanced/tests').glob('*.rs'))]
  + ['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml'],
  'nonzero normal Core/host and U07 objects/lifecycle samples; compare default historical PC/R05 guards and separate development input; tests remain implementation owner work'),
 ('RQ06', [9,10], 'exact predecessor closure, final evidence selectors and registered followup handoff',
  [P+'post_audit/u07_20261005_002/AUDIT_CLOSURE.json',P+'post_audit/u07_20261005_002/COMPLETION_AUDIT.md',
   P+'execution/single_goal/target/u07_002/MANIFEST.json',P+'execution/single_goal/PLAN.md',
   'core_enhanced/docs/roadmaps/core_first/execution/u07_memory_budget_followup_20261005_001/FOLLOWUP.ko.md',
   'core_enhanced/docs/core_runtime_contract.md','core_enhanced/docs/data_loading_contract.md'],
  '181 predecessor plus31 R06 applicability and10 dimensions; historical failures immutable; whole-goal independent closure; U30-C/P own final tuning'),
]

def main():
    artifacts=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
    working=next(a for a in artifacts if a['ref']['Artifact Type']=='Working Revision')
    rows=[]; paths=set()
    for key, elements, claim, locations, validation in GROUPS:
        assert all((ROOT/p).is_file() for p in locations), (key,[p for p in locations if not (ROOT/p).is_file()])
        paths.update(locations)
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':[working['payload']['Current Policy Element Revision Map'][f'PE-CMRP-{n:02}'] for n in elements],
            'Required claim/boundary':claim,'Source type/role':'Workspace Actual',
            'Required identity/location constraint':{'repository':str(ROOT),'locations':locations,'dirty_basis':'Exact actual bytes; preserve existing changes'},
            'Applicability/Necessity':'required','Applicability condition':None,
            'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['I01','I02','I03','I04']},
            'Validation requirement':validation})
    for key,role,reason in [('RQ07','Released Actual','No externally released-version compatibility target; local release/public validation is GEX work.'),('RQ08','Deployed Actual','No deployed-service/Unity scope. Actual local host is required in RQ03/04; later platform sampling belongs to U30-C/P.')]:
        rows.append({'Requirement ID':key,'Exact Policy Element Revision Ref Set':list(working['payload']['Current Policy Element Revision Map'].values()),
            'Required claim/boundary':role,'Source type/role':role,'Required identity/location constraint':None,
            'Applicability/Necessity':'not_applicable','Applicability condition':None,
            'Policy/Scope Basis Reference':{'Working Revision Ref':working['ref'],'Scope selectors':['O02','O03']},'Validation requirement':reason})
    plan=artifact('Review Scope/Source Requirement Plan','RSP-CMRP-001',{'Bound Input Working Revision Ref':working['ref'],'Requirement Set':rows,
        'Currentness Boundary':'Local actual dirty workspace. U07 immutable baseline remains exact except published two roadmap status documents. Capture is not a completed semantic review.'})
    write('P2G_20_SOURCE_PLAN_001.json',{'artifacts':[plan]})
    (HERE/'evidence').mkdir(exist_ok=True)
    with tarfile.open(HERE/'evidence/REVIEW_SOURCE_001.tar.gz','x:gz') as archive:
        for p in sorted(paths): archive.add(ROOT/p,arcname=p,recursive=False)
    write('evidence/REVIEW_SOURCE_MANIFEST_001.json',{'plan_ref':plan['ref'],'files':[{'path':p,'sha256':sha(ROOT/p),'bytes':(ROOT/p).stat().st_size} for p in sorted(paths)],'archive_sha256':sha(HERE/'evidence/REVIEW_SOURCE_001.tar.gz')})
    with (HERE/'evidence/GIT_STATUS_INTAKE.txt').open('x') as stream:
        subprocess.run(['git','status','--short','--untracked-files=normal'],cwd=ROOT,stdout=stream,check=True)
    print(f'P2G20 candidate: {len(rows)} requirements/{len(paths)} sources, semantic review pending')

if __name__=='__main__': main()
