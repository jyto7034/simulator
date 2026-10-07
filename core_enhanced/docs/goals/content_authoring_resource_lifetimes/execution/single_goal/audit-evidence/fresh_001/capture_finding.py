import json
from verify_currentness import ROOT, OUT, digest

specs = [
    ('core_enhanced/src/authoring/mod.rs', 123, 174),
    ('core_enhanced/src/authoring/control.rs', 215, 254),
    ('core_enhanced/src/authoring/control.rs', 310, 354),
    ('core_enhanced/src/authoring/control.rs', 536, 552),
    ('core_enhanced/src/game/data/content_pack/mod.rs', 114, 145),
    ('core_enhanced/src/game/data/mod.rs', 857, 928),
    ('core_enhanced/src/game/data/mod.rs', 951, 977),
    ('core_enhanced/src/game/resource/mod.rs', 127, 149),
    ('core_enhanced/src/game/data/skill_data.rs', 133, 159),
    ('core_enhanced/src/authoring/tests.rs', 23, 64),
    ('core_enhanced/src/bin/author_content.rs', 143, 153),
]
excerpts=[]
for path,start,end in specs:
    p=ROOT/path
    lines=p.read_text().splitlines()
    excerpts.append({'path':path,'source_sha256':digest(p),'first_line':start,'last_line':end,
        'lines':[{'line':i,'text':lines[i-1]} for i in range(start,end+1)]})
r={
 'finding':'F-R05-001', 'status':'required source correction', 'verdict':'Not Complete',
 'method':'Independent source-level deterministic control-flow and error-flow proof; no runtime panic reproduction was executed.',
 'entry':'public authoring::validate with a valid nonempty resource catalog and valid nonzero finite ExecutionLimits.max_work',
 'witness':[
  'Let P be cumulative work after raw final validation and constructor index checks, immediately before the repeated ResourceCatalog::validate charge in GameDataBase::new.',
  'Choose max_work=P, or any admitted value in [P,P+n-1] for n>0 resource definitions. The successful default-load prefix fixes P deterministically; all preceding charges fit and the next charge does not.',
  'ResourceCatalog::validate returns ResourceDefinitionError::Control after authoring::control::charge latches authoring_work_limit; GameDataBase::new calls expect on that error.',
  'The panic escapes content_pack publication and control::run_with_provenance, which directly invokes operation() and has no panic conversion; the promised Result<AuthoringContent, AuthoringError> does not return a typed failure.',
  'A second repeated fallible scan, SkillDatabase::validate_resource_references, has the same expect error boundary. It charges even skills with no typed resource operations.',
  'Cancellation becoming visible at either repeated scan also returns an error into expect. This is a source-path consequence; no timing-based cancellation experiment is claimed.'
 ],
 'existing_test_gap':'controlled_external_load_uses_the_same_token_and_exact_generated_work_boundary checks full-load work and full-load minus one in GeneratedSetup; it does not exhaust work inside these earlier FinalValidation constructor scans. Existing eight typed execution limits target first-cost, terminal and Complete, not this publication interval.',
 'execution_limitations':'Current author_content CLI uses AuthoringLimits::default and offers no arbitrary load-work budget argument. Its --help usage was inspected (exit 2); no pack run or panic reproducer was invoked. New source, fixture or builds were prohibited in this separated audit.',
 'owner':'R05 GEX-10, GS-CARL-P2G50-001 / GOAL-CARL-01 / PHASE-CARL-01; AG-CARL-01 (WP-CARL-03 publication/constructor), WP-CARL-04 focused durable regression.',
 'required_correction':'Preserve finite validation and cancellation semantics while propagating constructor/publication errors through the typed public result, or otherwise remove the invalid fallibility assumption with equivalent strict admission. Add a nonzero valid external-pack focused regression at the constructor scan budget boundary; verify no unwind, typed phase/site/code, no publication or replacement, prior generation unchanged and subsequent valid attempt succeeds.',
 'source_excerpts':excerpts
}
(OUT/'FINDING_F_R05_001.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({'finding':r['finding'],'source_excerpt_count':len(excerpts),'runtime_reproduced':False}))
