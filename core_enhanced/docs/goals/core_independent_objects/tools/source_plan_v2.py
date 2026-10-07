#!/usr/bin/env python3
"""Extend the candidate plan for discovered authoritative consumers; preserve001."""
import copy,json,tarfile
from initialize import HERE,ROOT,artifact,write,sha
from source_plan import GROUPS as INITIAL
GROUPS=copy.deepcopy(INITIAL)
EXTRA={'RQ03':['core_enhanced/src/game/battle/core/basic_heal.rs'],
       'RQ04':['core_enhanced/src/game/battle/stat_pipeline.rs'],
       'RQ06':['core_enhanced/src/game/world/battle_interaction.rs']}
GROUPS=[(k,e,c,paths+EXTRA.get(k,[]),v) for k,e,c,paths,v in GROUPS]
def main():
    old=json.loads((HERE/'P2G_20_SOURCE_PLAN_001.json').read_text())['artifacts'][0]
    payload=copy.deepcopy(old['payload'])
    for r in payload['Requirement Set']:
        if r['Requirement ID'] in EXTRA:
            r['Required identity/location constraint']['locations']+=EXTRA[r['Requirement ID']]
    payload['Predecessor Candidate Plan Ref']=old['ref']
    payload['Extension reason']='Reverse consumer search found the basic-Heal capability, authoritative stat mutation, and shared World interaction gate. Supplemental static review; no policy change or product mutation.'
    plan=artifact('Review Scope/Source Requirement Plan','RSP-CIOB-002',payload)
    paths=sorted({p for _,_,_,ps,_ in GROUPS for p in ps})
    assert all((ROOT/p).is_file() for p in paths)
    write('P2G_20_SOURCE_PLAN_002.json',{'artifacts':[plan]})
    with tarfile.open(HERE/'evidence/REVIEW_SOURCE_002.tar.gz','x:gz') as archive:
        for p in paths:archive.add(ROOT/p,arcname=p,recursive=False)
    write('evidence/REVIEW_SOURCE_MANIFEST_002.json',{'plan_ref':plan['ref'],'files':[{'path':p,'sha256':sha(ROOT/p),'bytes':(ROOT/p).stat().st_size} for p in paths],'archive_sha256':sha(HERE/'evidence/REVIEW_SOURCE_002.tar.gz')})
    print(f'Candidate002: {len(paths)} captured sources; supplemental review pending')
if __name__=='__main__':main()
