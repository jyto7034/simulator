"""Successor required source for the discovered outside-native clone boundary."""
import json
import tarfile
from initialize import ROOT, HERE, artifact, write, sha
from source_plan import GROUPS

GROUPS[1][3].append('core_enhanced/src/game/resource/store.rs')

def main():
    old=json.loads((HERE/'P2G_20_SOURCE_PLAN_001.json').read_text())['artifacts'][0]
    payload=old['payload']
    payload['Predecessor Review Scope/Source Requirement Plan Ref']=old['ref']
    payload['Requirement Set'][1]['Required identity/location constraint']['locations']=GROUPS[1][3]
    payload['Successor Reason']='RQ02 adds actual resource clone helper: outside-native persistence capture is not a fallback8MiB refusal; it has no native clone admission. Preserve existing scope and report measurement scope explicitly. Snapshot outside-native allocation fallback is a separate actual dev-profile seam.'
    new=artifact('Review Scope/Source Requirement Plan','RSP-CMRP-002',payload)
    paths=sorted({p for g in GROUPS for p in g[3]})
    for p in paths: assert (ROOT/p).is_file(),p
    write('P2G_20_SOURCE_PLAN_002.json',{'artifacts':[new]})
    with tarfile.open(HERE/'evidence/REVIEW_SOURCE_002.tar.gz','x:gz') as archive:
        for p in paths: archive.add(ROOT/p,arcname=p,recursive=False)
    write('evidence/REVIEW_SOURCE_MANIFEST_002.json',{'plan_ref':new['ref'],'files':[{'path':p,'sha256':sha(ROOT/p),'bytes':(ROOT/p).stat().st_size} for p in paths],'archive_sha256':sha(HERE/'evidence/REVIEW_SOURCE_002.tar.gz')})
    print(f'P2G20 source successor: {len(paths)} exact files; semantic review pending')

if __name__=='__main__': main()
