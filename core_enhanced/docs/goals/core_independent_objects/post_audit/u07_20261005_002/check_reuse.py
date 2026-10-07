import json,pathlib,hashlib,tarfile,re,difflib,datetime
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_independent_objects';S=H/'execution/single_goal';E=H/'evidence';C=S/'compatibility/current_001';A=pathlib.Path(__file__).parent;O=A.parent/'u07_20261005_001'
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();digest=lambda b:hashlib.sha256(b).hexdigest()
m=load(S/'target/u07_002/MANIFEST.json');old=load(O/'CONDITION_JUDGMENTS.json');c=load(S/'COVERAGE_002.json');p=load(S/'PREDECESSOR_IMPACT_002.json')
assert old['acceptances']==c['acceptances']
assert [(x['id'],x['requirement']) for x in old['validations']]==[(x['id'],x['requirement']) for x in c['validations']]
assert [(x['id'],x['requirement'],x.get('original_meaning_basis')) for x in old['predecessor_conditions']]==[(x['id'],x['requirement'],x.get('original_meaning_basis')) for x in p['conditions']]
assert [(x['acceptance'],x['expected'],x['iva']) for x in old['r06_acceptances']]==[(x['acceptance'],x['expected'],x['iva']) for x in p['r06_acceptances']]
assert {x['dimension']:x['validation_refs'] for x in old['dimensions']}==c['required_dimensions']
assert sha(O/'AUDIT_CLOSURE.json')=='68d6c1dc6efff4ad5d6d3299f7c9225f783d258551f1ca57c14190c9746c3801'
prior_hashes={x['path']:x['sha256'] for g in load(O/'SOURCE_REVIEW.json')['direct_source_review'] for x in g['files']}
changed=[p for p,h in prior_hashes.items() if sha(R/p)!=h]
assert set(changed)=={'core_enhanced/src/game/battle/core/build.rs','core_enhanced/src/game/battle/core/movement/blocking.rs','core_enhanced/src/game/battle/core/object_creation.rs','core_enhanced/src/game/battle/core/object_evolution.rs'}
correction=E/'f01_correction_001';record=load(correction/'RECORD.json');red=load(correction/'RED.json');rc=load(H/red['receipt'])
assert sha(H/red['receipt'])==red['receipt_sha256'] and rc['source_before']==rc['source_after']==red['source'] and rc['source_unchanged'] and rc['exit_code']==101
assert sha(E/'f01_reproduce_003.log')==rc['log_sha256']
assert 'test result: FAILED. 0 passed; 6 failed;' in (E/'f01_reproduce_003.log').read_text()
for name,h in record['proofs'].items():assert sha(correction/name)==h
with tarfile.open(S/'target/u07_001/SOURCE_AFTER.tar.gz') as prior,tarfile.open(correction/'PREIMAGES.tar.gz') as pre:
 for f in pre.getmembers():assert pre.extractfile(f).read()==prior.extractfile(f.name).read()
 for path,h in red['production_paths_unchanged'].items():assert digest(prior.extractfile(path).read())==h
with tarfile.open(correction/'PRODUCT_AFTER.tar.gz') as ar:
 for f in ar.getmembers():assert ar.extractfile(f).read()==(R/f.name).read_bytes()
with tarfile.open(correction/'RED_TESTS.tar.gz') as ar:
 ids={x['path']:x['sha256'] for x in red['source']['files']}
 for f in ar.getmembers():assert digest(ar.extractfile(f).read())==ids[f.name]
withdrawn=E/'f01_correction_002';record2=load(withdrawn/'RECORD.json');assert not record2['production_change'] and record2['source_before']==record2['source_after']==m['product_source']
for name,h in record2['proofs'].items():assert sha(withdrawn/name)==h
for x in record2['experimental_receipts']:
 assert sha(R/x['path'])==x['sha256'];v=load(R/x['path']);assert v['exit_code']!=0
 assert sha((R/x['path']).with_suffix('.log'))==v['log_sha256']
with tarfile.open(withdrawn/'PREIMAGES.tar.gz') as ar:
 for f in ar.getmembers():assert ar.extractfile(f).read()==(R/f.name).read_bytes()
assert 'test result: FAILED.' in (E/'f01_retired_red_005.log').read_text()
ad=load(C/'ADAPTATION_FINAL_003.json')
for row in ad['files']:assert sha(R/row['source'])==row['source_sha256']
labels=C/'evidence/harness_label_delta_005';bindings=load(labels/'EXECUTED_BINDINGS.json')
for x in bindings['rows']:
 for key,path in [('current',R/x['script']),('receipt',R/x['executed_receipt']),('executed',R/x['executed_script_archive'])]:assert sha(path)==x[key+'_sha256']
 before=(R/x['executed_script_archive']).read_text();after=(R/x['script']).read_text()
 diff=list(difflib.unified_diff(before.splitlines(),after.splitlines()))
 changed_lines=[s for s in diff if s[:1] in '+-' and not s.startswith(('+++','---'))]
 assert len(changed_lines)==3 and '+import re' in changed_lines
 assert all(('import re' in s or 'assert ' in s) for s in changed_lines)
 assert 'after_003' in before and x['actual_input_label']=='after_003'
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exact_condition_meanings':{'u07':20,'dimensions':10,'predecessor':161,'r06':31},'prior_review_paths':len(prior_hashes),'rereviewed_changed_paths':changed,'additional_source_delta':'SOURCE_DELTA.json','first_closure_preserved':True,'f01_red_normal_failures':6,'f01_red_original_production_hashes':red['production_paths_unchanged'],'f01_correction_after_exact_current':True,'withdrawn_investigation_exact_full_restore':True,'withdrawn_failures_not_pass':5,'predecessor_adaptation_source_hashes':len(ad['files']),'label_equivalence_verified':2,'errors':[],'reuse_rule':'Reuse prior condition-level semantic review only for byte-identical owners; current affected-consumer receipts verified for every linked condition. New production callers, accounting, cleanup, tests and shared withdrawal influence directly re-reviewed.'}
(A/'REUSE_VERIFICATION.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps(out))
