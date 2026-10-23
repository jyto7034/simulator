"""Recheck historical raw observations without writing historical audit outputs.

The U16-2 build-binding assertion uses its archived source, not later U16 code.
Current continuity is separately recorded in PREDECESSOR_DIFF and the review.
"""
from pathlib import Path
import contextlib, hashlib, json, sys, traceback
A=Path(__file__).resolve().parent
R=next(p for p in A.parents if (p/'game_server/Cargo.toml').exists())
base=R/'core_enhanced/docs/goals'
scripts=[
 ('u161',base/'core_support_node_activities/execution/single_goal/audit/independent001/recheck_evidence.py'),
 ('u162',base/'core_basic_relic_rewards/execution/single_goal/audit/independent001/recheck_evidence.py'),
 ('u163',base/'core_dealer_minigames/execution/single_goal/audit/independent001/recheck.py'),
 ('u164',base/'core_stress_combat/execution/single_goal/independent_audit004/check_evidence.py'),
]
results=[]
for name,p in scripts:
    code=p.read_text(); adjustments=[]
    if name=='u162':
        old='==current.read_text()'; new='==after[str(current.relative_to(R))].decode()'
        assert code.count(old)==1; code=code.replace(old,new)
        adjustments.append('historical prepared-binary cfg(test) locator checked against archived U16-2 source')
    if name in ['u163','u164']:
        old="(A/'EVIDENCE_RECHECK.json')" if name=='u163' else "(A / 'EVIDENCE_CHECK.json')"
        assert code.count(old)==1;code=code.replace(old,"(AUDIT_OUT / '"+name+"_RECHECK.json')")
        adjustments.append('write redirected to current audit directory')
    sys.argv=[str(p),str(A/(name+'_RECHECK.json'))]
    result={'name':name,'validator':str(p.relative_to(R)),'validator_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'adjustments':adjustments,'runtime_rerun':False}
    with (A/(name+'_recheck.log')).open('x') as log,contextlib.redirect_stdout(log),contextlib.redirect_stderr(log):
        try:
            exec(compile(code,str(p),'exec'),{'__file__':str(p),'__name__':'__main__','AUDIT_OUT':A})
            result['passed']=True
        except Exception:
            traceback.print_exc();result['passed']=False
    results.append(result);print(json.dumps(result),flush=True)
(A/'HISTORY_RECHECK.json').write_text(json.dumps(results,indent=2)+'\n')
assert all(x['passed'] for x in results)
