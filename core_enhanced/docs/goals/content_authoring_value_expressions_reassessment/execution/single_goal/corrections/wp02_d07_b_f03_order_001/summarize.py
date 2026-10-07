"""Validate fresh C07 receipts, independently check query rows, and bind reused evidence."""
import hashlib
import importlib.util
import json
import re
import tarfile
from pathlib import Path

ROOT=Path('/mnt/f/work/simulator')
HERE=Path(__file__).resolve().parent
PREV=HERE.parent/'wp02_d07_b_f03_aggregate_001'
def read(p): return json.loads(p.read_text())
def sha(b): return hashlib.sha256(b).hexdigest()
def save(p,v):
    with p.open('x') as f: json.dump(v,f,indent=2,ensure_ascii=False); f.write('\n')
def meta(p):
    b=p.read_bytes(); return {'path':str(p.relative_to(ROOT)),'bytes':len(b),'sha256':sha(b)}

def main():
    names=['c07_order_007','check_final_001','authoring_final_001','value_expr_final_001','all_targets_final_001','clippy_final_001','format_final_001']
    receipts={n:read(HERE/'evidence'/f'{n}.json') for n in names}
    source=receipts[names[0]]['source_after']
    for n,r in receipts.items():
        assert r['exit_code']==0 and r['test_failed']==0,n
        assert r['source_before']==r['source_after']==source,n
        assert sha((HERE/'evidence'/f'{n}.log').read_bytes())==r['log_sha256'],n
    for path,size,digest in source['files']:
        b=(ROOT/path).read_bytes(); assert len(b)==size and sha(b)==digest,path
    assert receipts[names[0]]['test_passed']==4
    assert receipts['all_targets_final_001']['test_passed']==1560
    text=(HERE/'evidence/c07_order_007.log').read_text()
    pairs=[json.loads(line.split('C07_QUERY ',1)[1]) for line in text.splitlines() if 'C07_QUERY ' in line]
    assert len(pairs)==7
    expected=[('unavailable',None,None),('evaluated',1,1),('evaluated',1,1),('evaluated',2,0),('unavailable',None,None),('evaluated',2,0),('evaluated',2,0)]
    for i,(pair,(status,ordinal,amount)) in enumerate(zip(pairs,expected)):
        for kind,doc in pair.items():
            assert doc['kind']==('amount_preview' if kind=='preview' else 'amount_explanation')
            assert doc['status']==status
            if status=='evaluated':
                assert doc['identity']['evaluation_ordinal']==ordinal
                assert doc['typed_base_amount']=={'value':amount,'unit':'Magnitude'}
                assert doc['trace']==[{'node_index':0,'member_identity':None,'value':{'value':amount,'unit':'Magnitude'}}]
                assert doc['amount_kind']==('damage' if ordinal==1 else 'heal')
            else:
                assert doc['reason']==('not_yet_evaluated' if i==0 else 'not_retained')
                assert 'identity' not in doc and 'narrowed_base_amount' not in doc
        assert pair['preview'].get('identity')==pair['explain'].get('identity')
    assert pairs[1]['preview']['identity']['battle_key']==pairs[3]['preview']['identity']['battle_key']
    assert pairs[3]['preview']==pairs[6]['preview']
    assert pairs[4]['preview']['selector']['evaluation_ordinal']==1
    order=re.findall(r'C07_ORDER ordinal=(\d+) effect=(\d+) target=(\S+) amount=(\S+) time=10',text)
    assert [int(r[0]) for r in order]==[3,9,18,21,27,36]
    projectiles=[json.loads(line.split('C07_PROJECTILE ',1)[1]) for line in text.splitlines() if 'C07_PROJECTILE ' in line]
    assert [(r['amount'],r['stop']) for r in projectiles]==[(0,'expiry'),(2,'expiry'),(0,'hit_limit'),(2,'hit_limit')]
    for r in projectiles:
        assert r['hits']==2 and r['empty_impacts']==0 and r['ordinal']==2 and r['open_deliveries']==0
        assert r['pending_casts']==(1 if r['stop']=='expiry' else 0)
    empty='C07_EMPTY_DYNAMIC ordinal=0 record=none resource=3 settled=true' in text
    assert empty
    save(HERE/'evidence/ORDER_MATRIX.json',{'source_digest':source['digest'],'log_sha256':receipts[names[0]]['log_sha256'],'query_pairs':pairs,'runtime_order':order,'projectile_cases':projectiles,'dynamic_empty':empty})
    spec=importlib.util.spec_from_file_location('freezer',HERE/'freeze.py'); f=importlib.util.module_from_spec(spec); spec.loader.exec_module(f)
    before=f.read_archive(HERE/'baseline/entry_001/SOURCE_BEFORE.tar.gz',f.BEFORE)
    after={p:(ROOT/p).read_bytes() for p in f.AFTER}
    proof=f.test_only_proof(before,after)
    assert [p for p in f.BEFORE if before[p]!=after[p]]==f.MODIFIED
    inherited_reuse=read(PREV/'evidence/EVIDENCE_REUSE.json')
    reuse={k:inherited_reuse[k] for k in ['mode','evidence','pc36','r05','limits','toolchain','environment_comparison','measured_source_digest','environment_applicability']}
    reuse['predecessor']=meta(PREV/'target/d07_b_f03_aggregate_001/MANIFEST.json')
    reuse['modified_inherited_paths']=f.MODIFIED
    reuse['added_cfg_test_children']=f.ADDED
    reuse['admission_ownership_runtime_policy_changes']=False
    reuse['current_source_digest']=source['digest'];reuse['predecessor_source_digest']=f.BASELINE['source']['digest']
    reuse['test_only_proof']=proof
    reuse['unchanged_inherited_files']=len(f.BEFORE)-len(f.MODIFIED)
    reuse['changes']='Only cfg(test) parent module/helper changes and two new test modules. The same public fixture helper is rerun by its unchanged legacy test. No product, loader, compiler, ownership, admission, or native cost path changed.'
    reuse.pop('external_workspace_addition',None)
    reuse['evidence'].extend([meta(PREV/'target/d07_b_f03_aggregate_001/MANIFEST.json'),meta(PREV/'evidence/EVIDENCE_REUSE.json')])
    for r in reuse['evidence']: assert meta(ROOT/r['path'])==r
    for r in receipts.values():assert r['toolchain']==reuse['toolchain']
    save(HERE/'evidence/EVIDENCE_REUSE.json',reuse)
    save(HERE/'evidence/FINAL_VALIDATION.json',{'state':'partial_C07_self_validation_complete_review_pending','source_count':source['file_count'],'source_digest':source['digest'],'fresh_validations':names,'results':{n:{k:r[k] for k in ['exit_code','test_passed','test_failed','elapsed_seconds','log_sha256']} for n,r in receipts.items()},'scope':{'new_tests':4,'public_query_pairs':7,'runtime_order_executions':6,'piercing_cases':4,'dynamic_empty_cases':1,'production_changed':False,'C08_C15_complete':False,'C03_OBS01_resolved':False,'C07_OBS01_resolved':False,'C07_OBS01_owner':'C09_per_user_decision','independent_review_pass':False,'F03_complete':False,'D07_B_complete':False},'reused':'EVIDENCE_REUSE.json','finding':'C07_OBS01.json'})
    print(json.dumps({'source_count':source['file_count'],'source_digest':source['digest'],'all_targets':receipts['all_targets_final_001']['test_passed'],'query_pairs':len(pairs)},indent=2))
if __name__=='__main__':main()
