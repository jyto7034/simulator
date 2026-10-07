#!/usr/bin/env python3
"""Compare ALL recognized AST test spans even when opaque reviews also exist.

This successor intentionally does not call verify_cfg.compare(), whose early
review result prevented a byte verdict for these two files. Macro-container
evidence remains a separate manual001 artifact, never a replacement for this.
"""
import argparse, hashlib, json, pathlib, subprocess, tarfile

ROOT=pathlib.Path('/mnt/f/work/simulator')
OUT=pathlib.Path(__file__).resolve().parent
BUNDLE=OUT.parent.parent/'product_parts/bundle_b001'
PATHS=['core_enhanced/src/application.rs','core_enhanced/src/game/battle/core/types.rs']
def digest(b):return hashlib.sha256(b).hexdigest()
def ref(p):
    b=p.read_bytes();return {'path':str(p.relative_to(ROOT)),'bytes':len(b),'sha256':digest(b)}
def signatures(raw,row):
    assert row.get('parse_error') is None
    result=[];last=-1
    for s in row['spans']:
        a,b=s['start'],s['end'];assert 0<=a<=b<=len(raw) and a>=last;last=a
        text=raw[a:b];text.decode()
        result.append((s['kind'],s['context'],s['reasons'],text))
    return result
def comparison(before,after,a,b):
    old=signatures(before,a);new=signatures(after,b)
    rows=[]
    for i in range(max(len(old),len(new))):
        x=old[i] if i<len(old) else None;y=new[i] if i<len(new) else None
        rows.append({'index':i,'kind_before':x[0] if x else None,'kind_after':y[0] if y else None,'context_before':x[1] if x else None,'context_after':y[1] if y else None,'bytes_before':len(x[3]) if x else None,'bytes_after':len(y[3]) if y else None,'sha256_before':digest(x[3]) if x else None,'sha256_after':digest(y[3]) if y else None,'exact_bytes_equal':bool(x and y and x[3]==y[3]),'kind_context_reasons_equal':bool(x and y and x[:3]==y[:3]),'ordered_span_equal':x==y})
    return {'all_recognized_spans_compared':True,'before_span_count':len(old),'after_span_count':len(new),'ordered_context_and_exact_bytes_equal':old==new,'span_rows':rows}
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=pathlib.Path,required=True);parser.add_argument('--output',type=pathlib.Path,required=True);args=parser.parse_args();dest=args.output.parent
    verification_path=OUT/'runs/u09_cfg001_verify002/verification.json';v=json.loads(verification_path.read_text())
    raw_path=ROOT/v['parser_output']['path'];assert ref(raw_path)==v['parser_output'];original={x['id']:x for x in json.loads(raw_path.read_text())['files']}
    before_path=BUNDLE/'BEFORE.json';before=json.loads(before_path.read_text());expected={r['path']:r for r in before['source']['files']};old={};bindings={}
    for archive in before['recovery_archives']:
        assert ref(ROOT/archive['path'])==archive
        with tarfile.open(ROOT/archive['path'],'r:gz') as tar:
            for m in tar.getmembers():
                path=m.name.removeprefix('./')
                if path in PATHS:
                    old[path]=tar.extractfile(m).read();bindings[path]={'archive':archive,'member':m.name}
    current={p:(ROOT/p).read_bytes() for p in PATHS}
    actual=[]
    for p in PATHS:
        assert digest(old[p])==expected[p]['sha256']
        a=original['baseline:'+p];b=original['current:'+p]
        row=comparison(old[p],current[p],a,b)
        row.update(path=p,preimage=bindings[p],current=ref(ROOT/p),before_review_count=len(a['reviews']),after_review_count=len(b['reviews']),opaque_review_resolution='separate manual001 exact macro-container evidence; does not suppress this AST comparison')
        actual.append(row)
    fixture_path=OUT/'fixtures/mixed_opaque_and_normal.json';fixture=json.loads(fixture_path.read_text())
    inputs=[{'id':'static_before','source':fixture['before']},{'id':'static_after','source':fixture['after']}]
    mutations=[]
    for p in PATHS:
        row=original['current:'+p]
        candidates=[s for s in row['spans'] if s['kind']=='ItemMod' and s['context'][-1]=='mod:tests']
        assert len(candidates)==1
        selected=candidates[0];at=selected['end']-1;assert current[p][at:at+1]==b'}'
        insertion=b'\n    const CFG001_METADATA_ONLY_MUTATION: () = ();\n'
        mutated=current[p][:at]+insertion+current[p][at:]
        inputs.append({'id':'actual_mutation:'+p,'source':mutated.decode()})
        mutations.append({'path':p,'insertion_at':at,'insertion':insertion.decode(),'mutated_bytes':mutated,'selected_normal_test_module':selected})
    encoded=json.dumps({'files':inputs},ensure_ascii=False).encode()
    process=subprocess.run([str(args.binary)],input=encoded,capture_output=True)
    stdout=dest/'controls_parser.stdout.json';stderr=dest/'controls_parser.stderr';stdout.write_bytes(process.stdout);stderr.write_bytes(process.stderr)
    assert process.returncode==0
    parsed={x['id']:x for x in json.loads(process.stdout)['files']}
    controls=[]
    a=parsed['static_before'];b=parsed['static_after'];prior=fixture['before'].encode();changed=fixture['after'].encode()
    for label,other,other_row,expected_equal in [('static_same_input',prior,a,True),('static_normal_test_mutation_with_same_opaque_macro',changed,b,False)]:
        result=comparison(prior,other,a,other_row)
        assert a['reviews'] and other_row['reviews'],'control must retain the opaque review trigger'
        ma=a['macros'][0];mb=other_row['macros'][0];opaque_same=prior[ma['start']:ma['end']]==other[mb['start']:mb['end']]
        controls.append({'id':label,'expected_equal':expected_equal,'opaque_macro_exact_equal':opaque_same,'opaque_reviews_retained':True,'comparison':result,'pass':opaque_same and result['ordered_context_and_exact_bytes_equal']==expected_equal})
    for mutation in mutations:
        p=mutation['path'];row=parsed['actual_mutation:'+p]
        assert row['reviews'] and original['current:'+p]['reviews']
        result=comparison(current[p],mutation['mutated_bytes'],original['current:'+p],row)
        controls.append({'id':'actual_normal_cfg_module_mutation:'+p,'expected_equal':False,'opaque_reviews_retained':True,'mutation':{k:v for k,v in mutation.items() if k!='mutated_bytes'},'comparison':result,'pass':not result['ordered_context_and_exact_bytes_equal']})
    success=all(x['ordered_context_and_exact_bytes_equal'] for x in actual) and all(c['pass'] for c in controls)
    report={'schema':'u09_cfg_reviewed_files_ast_successor_v1','status':'pass' if success else 'fail','cause_for_successor':'verify002 early review return left normal AST span comparison unadjudicated in two files; manual001 covered only opaque macro containers','verification':ref(verification_path),'macro_container_evidence':ref(OUT/'runs/u09_cfg001_manual001/manual_evidence.json'),'parser_original_output':ref(raw_path),'control_fixture':ref(fixture_path),'parser_binary':ref(args.binary),'control_parser_input_bytes':len(encoded),'control_parser_input_sha256':digest(encoded),'control_parser_stdout':ref(stdout),'control_parser_stderr':ref(stderr),'control_parser_exit_code':process.returncode,'actual_file_comparisons':actual,'controls':controls,'controls_passed':sum(c['pass'] for c in controls),'controls_total':len(controls),'general_scanner_exit2_retained':True,'gameplay_source_writes':0,'functional_game_tests':0,'scope':'All recognized AST span kinds, reasons, context/order and exact bytes in both reviewed files, independently of the presence of opaque reviews. Macro bodies still rely on explicit manual001 container proof.'}
    args.output.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps({'status':report['status'],'actual_files':len(actual),'normal_ast_span_counts':[x['before_span_count'] for x in actual],'normal_ast_equal':[x['ordered_context_and_exact_bytes_equal'] for x in actual],'controls_passed':report['controls_passed'],'controls_total':len(controls)},indent=2))
    return 0 if success else 2
if __name__=='__main__':raise SystemExit(main())
