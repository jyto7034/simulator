#!/usr/bin/env python3
"""Extend the candidate plan for discovered authoritative consumers; preserve001."""
import copy,json,tarfile
from initialize import HERE,ROOT,artifact,write,sha
from source_plan_v2 import GROUPS as INITIAL
GROUPS=copy.deepcopy(INITIAL)
EXTRA={'RQ01':['core_enhanced/src/game/value_expr/catalog.rs'], 'RQ05':['core_enhanced/src/game/battle/core/value_expr_runtime.rs']}
GROUPS=[(k,e,c,paths+EXTRA.get(k,[]),v) for k,e,c,paths,v in GROUPS]
def main():
    old=json.loads((HERE/'P2G_20_SOURCE_PLAN_002.json').read_text())['artifacts'][0]
    payload=copy.deepcopy(old['payload'])
    for r in payload['Requirement Set']:
        if r['Requirement ID'] in EXTRA:
            r['Required identity/location constraint']['locations']+=EXTRA[r['Requirement ID']]
    payload['Predecessor Candidate Plan Ref']=old['ref']
    payload['Extension reason']='New opcode result-contract and released value-expression capture consumers need explicit coverage; no policy change or product mutation.'
    plan=artifact('Review Scope/Source Requirement Plan','RSP-CIOB-003',payload)
    paths=sorted({p for _,_,_,ps,_ in GROUPS for p in ps})
    assert all((ROOT/p).is_file() for p in paths)
    write('P2G_20_SOURCE_PLAN_003.json',{'artifacts':[plan]})
    with tarfile.open(HERE/'evidence/REVIEW_SOURCE_003.tar.gz','x:gz') as archive:
        for p in paths:archive.add(ROOT/p,arcname=p,recursive=False)
    write('evidence/REVIEW_SOURCE_MANIFEST_003.json',{'plan_ref':plan['ref'],'files':[{'path':p,'sha256':sha(ROOT/p),'bytes':(ROOT/p).stat().st_size} for p in paths],'archive_sha256':sha(HERE/'evidence/REVIEW_SOURCE_003.tar.gz')})
    print(f'Candidate003: {len(paths)} captured sources; supplemental review pending')
if __name__=='__main__':main()
