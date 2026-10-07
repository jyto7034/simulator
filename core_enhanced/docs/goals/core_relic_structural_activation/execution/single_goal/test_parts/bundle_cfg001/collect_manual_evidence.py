#!/usr/bin/env python3
"""Narrow evidence for the two explicitly returned opaque-macro review cases.

The generic boundary checker stays fail-closed. This script proves whole parsed
macro containers equal; it never claims to parse or expand their child bodies.
"""
import argparse, hashlib, json, pathlib, tarfile

ROOT=pathlib.Path('/mnt/f/work/simulator')
OUT=pathlib.Path(__file__).resolve().parent
BUNDLE=OUT.parent.parent/'product_parts/bundle_b001'
EXPECTED={
    'core_enhanced/src/application.rs':('define_application_command_kinds',915,'6a9f64444e527ee3b1d2236e75e279d4145dd4ff9a4a6ba31d3ee86472da1916'),
    'core_enhanced/src/game/battle/core/types.rs':(None,1076,'f74b037a958a785cb7b2d68386dcfc460f680faace018c88058433318ed41e47'),
}
def digest(b):return hashlib.sha256(b).hexdigest()
def ref(p):
    b=p.read_bytes();return {'path':str(p.relative_to(ROOT)),'bytes':len(b),'sha256':digest(b)}
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=pathlib.Path,required=True);args=parser.parse_args()
    verification=OUT/'runs/u09_cfg001_verify002/verification.json';v=json.loads(verification.read_text())
    assert v['controls_passed']==v['controls_total']==26 and not v['differences']
    assert {x['path'] for x in v['manual_reviews']}==set(EXPECTED)
    before=json.loads((BUNDLE/'BEFORE.json').read_text());saved={};bindings={}
    for archive in before['recovery_archives']:
        assert ref(ROOT/archive['path'])==archive
        with tarfile.open(ROOT/archive['path'],'r:gz') as t:
            for member in t.getmembers():
                name=member.name.removeprefix('./')
                if name in EXPECTED:
                    saved[name]=t.extractfile(member).read();bindings[name]={'archive':archive,'member':member.name}
    rows=[]
    for issue in v['manual_reviews']:
        path=issue['path'];name,size,sha=EXPECTED[path];current=(ROOT/path).read_bytes()
        assert len(issue['manual_review_container_evidence'])==1
        evidence=issue['manual_review_container_evidence'][0];span=evidence['before'];old=saved[path][span['start']:span['end']]
        assert evidence['name']==name and len(old)==size and digest(old)==sha
        matched=[p for p in evidence['matching_after'] if current[p['start']:p['end']]==old]
        assert len(matched)==1
        rows.append({'path':path,'preimage':bindings[path],'macro_name':name,'before_span':span,'current_span':matched[0],'exact_bytes':size,'exact_sha256':sha,'container_text':old.decode(),'authored_test_bytes_within_container_unchanged':True,'scope':'Exact entire ItemMacro text; no child-body boundary or expanded macro semantics inferred','review_assessment':'Independent author read actual macro contents and containing declarations. Test-sensitive into_domain/template or target_traits field is contained entirely inside these identical bytes.'})
    raw=json.loads((OUT/'runs/u09_cfg001_verify002/parser.stdout.json').read_text())
    baseline=[x for x in raw['files'] if x['id'].startswith('baseline:')]
    counts={'baseline_rust_files_parsed':sum(x.get('parse_error') is None for x in baseline),'baseline_rust_files_not_parsed_but_whole_bytes_checked':sum(x.get('parse_error') is not None for x in baseline),'baseline_ast_test_positive_attributes':sum(sum(c['class'] in ('test_positive','test_mixed') for c in x.get('conditions',[])) for x in baseline),'baseline_ast_test_function_attributes':sum(sum(c['class']=='test_attribute' for c in x.get('conditions',[])) for x in baseline)}
    report={'schema':'u09_cfg_manual_container_evidence_v1','verification':ref(verification),'generic_checker_status_unchanged':'changes_or_manual_review_required; exit2 retained','not_a_blanket_opaque_macro_allowlist':True,'scope':'Two exact inherited/current parsed macro containers only; Root acceptance consumes this alongside the AST result.','cases':rows,'counts':counts,'all_external_test_fixture_paths_unchanged':all(r['status']=='unchanged' for r in v['source_rows'] if r['path'] in v['external_tests_and_fixtures']),'external_test_fixture_count':v['external_count'],'old_failed_artifacts_preserved':[ref(BUNDLE/name) for name in ['execute_format.py','collect_cfg_preservation.py','CFG_TEST_COLLECTION_FAILURE_001.txt','CFG_TEST_RESTORATION_001.json','CFG_TEST_PRESERVATION.json']],'restoration_actions':0,'gameplay_source_writes':0,'functional_game_tests':0}
    args.output.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps({'cases':len(rows),'exact_macro_containers':2,'authored_test_byte_changes':0,'external_test_fixture_count':v['external_count'],'counts':counts,'generic_checker_exit2_retained':True},indent=2))
    return 0
if __name__=='__main__':raise SystemExit(main())
