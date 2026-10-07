"""U08 P2G-30 exact acceptance projection and binding closure."""
import json
from initialize import HERE,ROOT,artifact,write,sha

def main():
    assert json.loads((HERE/'P2G_20_RESULT_001.json').read_text())['status']=='complete'
    arts=json.loads((HERE/'P2G_20_REVIEW_001.json').read_text())['artifacts']
    def one(kind):return next(a for a in arts if a['ref']['Artifact Type']==kind)
    wr,batch,review,basis=[one(k) for k in ['Working Revision','Review Batch','Code Review Artifact','Reviewed Source Basis']]
    es=sorted([a for a in arts if a['ref']['Artifact Type']=='Policy Element Revision'],key=lambda a:a['payload']['Stable Element ID'])
    bs=[a for a in arts if a['ref']['Artifact Type']=='Code Review Binding']
    assert len(es)==len(bs)==16 and not wr['payload']['Open Policy Question Ref Set']
    assert [e['ref'] for e in es]==batch['payload']['Result Current code_reviewed Policy Element Revision Ref Set']==list(wr['payload']['Current Policy Element Revision Map'].values())
    mapping=review['payload']['Element review mapping']
    for e,b,m in zip(es,bs,mapping):
        assert e['payload']['Intrinsic status']=='code_reviewed'
        assert b['payload']['Bound Working Revision Ref']==wr['ref'] and b['payload']['Policy Element Revision Ref']==e['ref']
        assert b['payload']['Reviewed Source Basis Ref']==basis['ref']
        assert b['payload']['Covered Required Claim/Boundary Set']==m['Required Claim/Boundary Set']
        assert e['payload']['Decision Revision Ref'] in wr['payload']['Decision Revision Ref Set']
        assert e['payload']['Observable Policy Acceptance']==m['Approved Observable Policy Acceptance']
    manifest=json.loads((HERE/'evidence/REVIEW_SOURCE_MANIFEST_003.json').read_text())
    for r in manifest['files']:assert sha(ROOT/r['path'])==r['sha256'],r['path']
    assert sha(HERE/'CODE_REVIEW_001.ko.md')==review['payload']['Review body']['sha256']
    f=artifact('Final Policy Freeze','FPF-CRNV-001',{
      'Working Revision Ref':wr['ref'],'Review Batch Ref':batch['ref'],'Predecessor Freeze':None,
      'Ordered code_reviewed Policy Element Revision Ref Set':[e['ref'] for e in es],
      'Decision Revision Ref Set':[e['payload']['Decision Revision Ref'] for e in es],
      'Observable Policy Acceptance Set':[{'Policy Element Revision Ref':e['ref'],'Observable Policy Acceptance':e['payload']['Observable Policy Acceptance']} for e in es],
      'Current Code Review Binding Ref Set':[b['ref'] for b in bs],'Reviewed Source Basis Ref':basis['ref'],
      'Binding Completeness Evidence':review['ref'],'Applicable Change/Obligation Basis':[],'Resolved Prerequisite Evidence':[],
      'Known Unknowns':'No required policy/source gap. Exact helpers/container/API/codec choice and measured costs remain GEX work in TD01–06. Official individual penalties,prices,stock/frequency and expanded activities remain named roadmap owners;representative fixture values are not final balance.',
      'Compatibility/Migration Disposition':'Preserve immutable content/baseplan authority,old equipment/fragment/source/Snapshot-Live/fault semantics,strict default8MiB/PC guards and U07M measurement. Add relic copy/slots/variant and distinct run resources to all candidate/save/entry/death/newrun roots. Legacy fragment_dust bytes/meaning unchanged;derived effective plans never trusted from save. Explicit codec/empty-old-state adaptation preserves supported old inputs and rejects malformed present graph without overwriting source. No new live-battle disk resume or retroactive failure waiver.',
      'Lifecycle Disposition':'current'})
    current={'Working Revision Ref':wr['ref'],'Final Policy Freeze Ref':f['ref'],'Phase Plan Ref':None,'Goal Set Ref':None,'Handoff Artifact Ref':None,'Post-Freeze Change Record Ref Set':[],'Authoritative Change Obligation Declaration Ref Set':[]}
    write('P2G_30_FREEZE_001.json',{'artifacts':[f]})
    write('P2G_30_RESULT_001.json',{'stage':'P2G-30','status':'complete','Final Policy Freeze Ref':f['ref'],
      'Freeze Membership Set':[{'Final Policy Freeze Ref':f['ref'],'Policy Element Revision Ref':e['ref']} for e in es],
      'P2G-30 Resolution Record Ref Set':[],'current_basis':current,'All Applicable Change Obligations Resolved':True,
      'checks':{'exact16_members':True,'acceptance_projection_exact':True,'binding_coverage_complete':True,'121_sources_current':True,'no_open_question':True},
      'Current Planning Freeze Selection':f['ref'],'next':'P2G-40'})
    p=HERE/'WORKING_POLICY.ko.md';p.write_text(p.read_text()+'\n현재 planning Freeze는 [FPF-CRNV-001](P2G_30_FREEZE_001.json),exact16member/decision/acceptance/binding. [P2G-30 결과](P2G_30_RESULT_001.json). 다음 P2G-40.\n')
    print('U08 P2G30 complete:16 exact policy members/decisions/acceptances/bindings;121current sources')

if __name__=='__main__':main()
