#!/usr/bin/env python3
"""Compare exact inherited test syntax bytes; never restore or write source."""
import argparse, collections, difflib, hashlib, json, pathlib, posixpath, re, subprocess, tarfile

ROOT = pathlib.Path('/mnt/f/work/simulator')
OUT = pathlib.Path(__file__).resolve().parent
BUNDLE = OUT.parent.parent / 'product_parts/bundle_b001'

def digest(data): return hashlib.sha256(data).hexdigest()
def ref(path):
    data=path.read_bytes()
    return {'path':str(path.relative_to(ROOT)),'bytes':len(data),'sha256':digest(data)}
def outer_spans(row):
    selected=[]
    for span in sorted(row.get('spans',[]),key=lambda s:(s['start'],-s['end'])):
        if not any(p['start']<=span['start'] and span['end']<=p['end'] for p in selected): selected.append(span)
    return selected
def fingerprint(source,row):
    raw=source.encode()
    return [(s['context'],s['kind'],raw[s['start']:s['end']].decode()) for s in outer_spans(row)]
def compare(before,after,a,b):
    # Deliberately no whole-file shortcut for controls: unresolved syntax is
    # observable even when a caller supplied identical strings.
    if a.get('parse_error') or b.get('parse_error') or a.get('reviews') or b.get('reviews'):
        return 'review'
    return 'unchanged' if fingerprint(before,a)==fingerprint(after,b) else 'changed'
def is_named_test(path):
    p=pathlib.PurePosixPath(path)
    return bool({'tests','fixtures'}.intersection(p.parts)) or p.name=='tests.rs' or p.name.endswith(('_tests.rs','_test.rs'))
def module_targets(path,module,available):
    p=pathlib.PurePosixPath(path)
    if module['path'] is not None:
        # Rust's literal path attribute resolves against the containing file
        # plus inline module directories. Non-mod-rs inline ambiguity is never
        # silently chosen when more than one candidate exists.
        bases=[p.parent.joinpath(*module['inline_modules'])]
        if module['inline_modules'] and p.name not in ('mod.rs','lib.rs','main.rs'):
            bases.append(p.parent.joinpath(p.stem,*module['inline_modules']))
        candidates=[posixpath.normpath(str(base/module['path'])) for base in bases]
    else:
        crate_entry=p.name in ('mod.rs','lib.rs','main.rs') or (len(p.parts)==3 and p.parts[1] in ('tests','examples')) or ('bin' in p.parts and p.parent.name=='bin')
        base=p.parent if crate_entry else p.parent/p.stem
        base=base.joinpath(*module['inline_modules'])
        candidates=[str(base/(module['name']+'.rs')),str(base/module['name']/'mod.rs')]
    return sorted(set(c for c in candidates if c in available))
def include_target(path,inc,available):
    literal=inc['literal_path']
    if literal is not None:
        target=posixpath.normpath(str(pathlib.PurePosixPath(path).parent/literal))
    else:
        # Recognize only this exact, bounded constant-expression form.
        match=re.fullmatch(r'concat ! \(env ! \("CARGO_MANIFEST_DIR"\) , ("(?:[^"\\]|\\.)*")\s*,?\)',inc['tokens'])
        if not match: return None
        crate=path.split('/')[0]
        target=posixpath.normpath(crate+json.loads(match.group(1)))
    return target if target in available else None
def test_graph(sources,parsed):
    protected={p for p in sources if is_named_test(p)}
    reasons={p:['inherited tests/fixtures path'] for p in protected}
    reviews=[]
    changed=True
    while changed:
        changed=False
        for path,row in parsed.items():
            inherited=path in protected
            # A shared file can have separate public test and private production
            # registrations. Preserve the test registration node, not the whole
            # production module reachable through the other declaration.
            production_targets=set()
            for module in row.get('modules',[]):
                if not module['test_context']:
                    production_targets.update(module_targets(path,module,sources))
            for module in row.get('modules',[]):
                if not inherited and not module['test_context']: continue
                targets=module_targets(path,module,sources)
                if len(targets)!=1:
                    entry={'path':path,'kind':'external_module','module':module,'candidates':targets}
                    if entry not in reviews: reviews.append(entry)
                    continue
                target=targets[0]
                if not inherited and target in production_targets: continue
                if target not in protected:
                    protected.add(target);reasons[target]=[f'Rust module registration in {path}'];changed=True
            for inc in row.get('includes',[]):
                if not inherited and not inc['test_context']: continue
                target=include_target(path,inc,sources)
                if target is None:
                    entry={'path':path,'kind':'external_include','include':inc}
                    if entry not in reviews: reviews.append(entry)
                    continue
                if target not in protected:
                    protected.add(target);reasons[target]=[f"{inc['macro']} literal in {path}"];changed=True
    return protected,reasons,reviews

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=pathlib.Path,required=True);parser.add_argument('--output',type=pathlib.Path,required=True);args=parser.parse_args()
    dest=args.output.parent
    before_path=BUNDLE/'BEFORE.json';before=json.loads(before_path.read_text());base_rows={r['path']:r for r in before['source']['files']}
    saved={};bindings={}
    for archive in before['recovery_archives']:
        p=ROOT/archive['path'];assert ref(p)==archive
        with tarfile.open(p,'r:gz') as tar:
            for member in tar.getmembers():
                if member.isfile():
                    name=member.name.removeprefix('./');data=tar.extractfile(member).read()
                    saved[name]=data;bindings[name]={'archive':archive,'member':member.name,'bytes':len(data),'sha256':digest(data)}
    baseline={}
    for name,row in base_rows.items():
        assert name in saved and len(saved[name])==row['bytes'] and digest(saved[name])==row['sha256'],name
        baseline[name]=saved[name]
    freeze_path=BUNDLE/'u09_product_bundle_b_check012.json';freeze=json.loads(freeze_path.read_text());current={}
    for row in freeze['source_after']['files']:
        p=ROOT/row['path'];assert ref(p)==row,row['path'];current[row['path']]=p.read_bytes()
    fixture_catalog=json.loads((OUT/'fixtures/cases.json').read_text())
    cases=fixture_catalog['cases'];graph_cases=fixture_catalog.get('graph_cases',[])
    inputs=[]
    for version,sources in [('baseline',baseline),('current',current)]:
        inputs.extend({'id':version+':'+path,'source':raw.decode()} for path,raw in sources.items() if path.endswith('.rs'))
    for case in cases:
        inputs.extend({'id':f"control:{case['id']}:{version}",'source':case[version]} for version in ('before','after'))
    for case in graph_cases:
        inputs.extend({'id':f"graph:{case['id']}:{path}",'source':source} for path,source in case['sources'].items())
    snapshots=json.loads((OUT/'fixtures/HISTORICAL_SNAPSHOT_REFS.json').read_text());historical={}
    for snapshot in snapshots:
        data=(ROOT/snapshot['snapshot']).read_bytes();assert digest(data)==snapshot['source']['sha256']
        assert ref(ROOT/snapshot['receipt']['path'])==snapshot['receipt']
        name=pathlib.Path(snapshot['snapshot']).stem;historical[name]=data.decode();inputs.append({'id':'historical:'+name,'source':data.decode()})
    world='core_enhanced/src/game/world/application.rs'
    probe=current[world].decode().replace('Self::SetEnkephalinFixture => ApplicationOwner::TestFixture,','Self::SetEnkephalinFixture => ApplicationOwner::Admin,',1)
    assert probe!=current[world].decode();inputs.append({'id':'historical:real_test_arm_mutation','source':probe})
    encoded=json.dumps({'files':inputs},ensure_ascii=False).encode()
    command=[str(args.binary)]
    result=subprocess.run(command,input=encoded,capture_output=True)
    (dest/'parser.stdout.json').write_bytes(result.stdout);(dest/'parser.stderr').write_bytes(result.stderr)
    assert result.returncode==0,('metadata parser failed',result.returncode)
    payload=json.loads(result.stdout);parsed={row['id']:row for row in payload['files']}
    assert len(parsed)==len(inputs)
    controls=[]
    for case in cases:
        a=parsed[f"control:{case['id']}:before"];b=parsed[f"control:{case['id']}:after"]
        actual=compare(case['before'],case['after'],a,b)
        texts=[r[2] for r in fingerprint(case['before'],a)]
        exact=case.get('expected_before_outer_texts')
        controls.append({'id':case['id'],'expected':case['expected'],'actual':actual,'exact_boundary_expected':exact,'exact_boundary_observed':texts,'pass':actual==case['expected'] and (exact is None or exact==texts),'before_parse_error':a.get('parse_error'),'after_parse_error':b.get('parse_error'),'before_reviews':a.get('reviews',[]),'after_reviews':b.get('reviews',[])})
    for case in graph_cases:
        nodes={p:parsed[f"graph:{case['id']}:{p}"] for p in case['sources']}
        selected,_,issues=test_graph(case['sources'],nodes)
        controls.append({'id':case['id'],'expected_protected_paths':case['expected_protected_paths'],'actual_protected_paths':sorted(selected),'reviews':issues,'pass':sorted(selected)==case['expected_protected_paths'] and not issues})
    # Both exact historical product snapshots have the same test bytes. The
    # check011 product omission remains a real compile failure, not a test edit.
    for name,source,expected in [('check011_application',historical['check011_application'],'unchanged'),('check012_application',historical['check012_application'],'unchanged'),('real_test_arm_mutation',probe,'changed')]:
        a=parsed['baseline:'+world];b=parsed['historical:'+name]
        actual=compare(baseline[world].decode(),source,a,b)
        controls.append({'id':'exact_historical_'+name,'expected':expected,'actual':actual,'pass':actual==expected,'before_reviews':a.get('reviews',[]),'after_reviews':b.get('reviews',[])})
    base_parsed={p:parsed['baseline:'+p] for p in baseline if p.endswith('.rs')}
    external,reasons,graph_reviews=test_graph(baseline,base_parsed)
    rows=[];differences=[];reviews=[]
    for path in sorted(set(baseline)|set(current)):
        old=baseline.get(path);new=current.get(path)
        row={'path':path,'before_sha256':digest(old) if old is not None else None,'after_sha256':digest(new) if new is not None else None}
        if path in baseline: row['preimage']=bindings[path]
        if old==new:
            row.update(proof='exact_whole_file',status='unchanged',external_test_or_fixture=path in external)
        elif new is None:
            row.update(proof='missing_source',status='changed');differences.append({'path':path,'cause':'inherited source removed'})
        elif path in external:
            row.update(proof='registered_external_test_or_fixture',status='changed',reasons=reasons[path]);differences.append({'path':path,'cause':'external test/fixture bytes changed'})
        elif not path.endswith('.rs'):
            row.update(proof='non_rust_input_changed',status='review');reviews.append({'path':path,'cause':'changed non-Rust inherited input requires explicit classification'})
        elif old is None:
            b=parsed['current:'+path]
            if b.get('parse_error') or b.get('reviews'):
                row.update(proof='new_source_unresolved',status='review');reviews.append({'path':path,'parser':b})
            elif b['spans'] or is_named_test(path):
                row.update(proof='new_test_conditional_source',status='changed');differences.append({'path':path,'cause':'new test source not in inherited baseline'})
            else: row.update(proof='new_rust_source_no_test_attributes',status='unchanged')
        else:
            a=parsed['baseline:'+path];b=parsed['current:'+path]
            status=compare(old.decode(),new.decode(),a,b)
            row.update(proof='Rust_AST_owner_exact_bytes',status=status,before_outer_spans=len(outer_spans(a)),after_outer_spans=len(outer_spans(b)),before_condition_attributes=len(a.get('conditions',[])),after_condition_attributes=len(b.get('conditions',[])))
            if status=='review':
                containers=[]
                for issue in a.get('reviews',[]):
                    at=issue.get('start',-1)
                    for macro in a.get('macros',[]):
                        if macro['start']<=at<macro['end']:
                            matching=[m for m in b.get('macros',[]) if (m['name'],m['path'],m['context'])==(macro['name'],macro['path'],macro['context'])]
                            prior=old[macro['start']:macro['end']]
                            containers.append({'name':macro['name'],'kind':'exact_AST_macro_container_only_not_expanded_child_boundary','before':macro,'matching_after':matching,'before_bytes':len(prior),'before_sha256':digest(prior),'exact_container_equal':len(matching)==1 and prior==new[matching[0]['start']:matching[0]['end']]})
                reviews.append({'path':path,'before_parser':a,'after_parser':b,'manual_review_container_evidence':containers})
            if status=='changed':
                left=json.dumps(fingerprint(old.decode(),a),indent=2,ensure_ascii=False);right=json.dumps(fingerprint(new.decode(),b),indent=2,ensure_ascii=False)
                differences.append({'path':path,'cause':'protected Rust owner bytes/context changed','diff':''.join(difflib.unified_diff(left.splitlines(True),right.splitlines(True),fromfile='inherited',tofile='current'))})
        rows.append(row)
    # An unresolved external edge is explicitly reported. If the entire owning
    # file and every inherited candidate are unchanged, that is still not proof
    # about an arbitrary generated/out-of-baseline include; no success inference.
    reviews.extend({'cause':'external test registration/include resolution needs review',**r} for r in graph_reviews)
    ok=all(c['pass'] for c in controls) and not differences and not reviews
    counts=collections.Counter(row['proof'] for row in rows)
    report={'schema':'u09_cfg_preservation_successor_v1','status':'pass' if ok else 'changes_or_manual_review_required','baseline':ref(before_path),'baseline_count':len(baseline),'baseline_source_sha256':before['source']['sha256'],'frozen':ref(freeze_path),'current_count':len(current),'current_source_sha256':freeze['source_after']['sha256'],'parser_command':command,'parser_input_bytes':len(encoded),'parser_input_sha256':digest(encoded),'parser_output':ref(dest/'parser.stdout.json'),'parser_stderr':ref(dest/'parser.stderr'),'parser_binary':ref(args.binary),'parser_exit_code':result.returncode,'controls':controls,'controls_passed':sum(c['pass'] for c in controls),'controls_total':len(controls),'proof_counts':dict(counts),'external_tests_and_fixtures':sorted(external),'external_count':len(external),'source_rows':rows,'differences':differences,'manual_reviews':reviews,'limits':['Syntax-byte preservation only; not runtime or semantic equivalence.','No macro expansion or procedural-macro execution. Unowned test-sensitive opaque tokens require review.','test-sensitive cfg_attr is not expanded and requires review.','Unknown cfg operators, predicates exceeding16 non-test atoms, parser errors or unresolved external test edges never imply success.','AST separators belong to their actual Rust owner; no next-brace or keyword heuristic is used.','UTF-8 parser columns are converted to byte offsets; original comments/whitespace within a protected owner are compared exactly.','Cfg Boolean influence is evaluated symbolically over each non-test atom; positive and mixed test influence is protected, negative-only production cfg is reported separately.'],'gameplay_source_writes':0,'restoration_actions':0,'functional_game_tests':0,'superseded_failed_proof':ref(BUNDLE/'CFG_TEST_PRESERVATION.json')}
    args.output.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps({'status':report['status'],'controls_passed':report['controls_passed'],'controls_total':len(controls),'proof_counts':dict(counts),'external_count':len(external),'difference_count':len(differences),'manual_review_count':len(reviews),'failed_controls':[c['id'] for c in controls if not c['pass']]},indent=2))
    return 0 if ok else 2

if __name__=='__main__': raise SystemExit(main())
