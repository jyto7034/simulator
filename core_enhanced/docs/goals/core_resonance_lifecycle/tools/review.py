#!/usr/bin/env python3
"""Publish U05 complete review002 after the owner clarifications."""
import copy,datetime,json,subprocess
from initialize import HERE,ROOT,artifact,write,sha
from source_plan_v2 import GROUPS

def row(p):
    p=ROOT/p
    return {'path':str(p.relative_to(ROOT)),'sha256':sha(p),'bytes':p.stat().st_size}

def main():
    old=[]
    for filename in ['P2G_00_INITIAL_001.json','P2G_10_POLICY_001.json','P2G_10_POLICY_002.json','P2G_20_SOURCE_PLAN_001.json','P2G_20_SOURCE_PLAN_002.json']:
        old+=json.loads((HERE/filename).read_text())['artifacts']
    before=next(a for a in old if a['ref']['Artifact ID']=='WR-CRSL-003')
    elements=[next(a for a in old if a['ref']==r) for _,r in sorted(before['payload']['Current Policy Element Revision Map'].items())]
    plan=next(a for a in old if a['ref']['Artifact ID']=='RSP-CRSL-002');requirements=plan['payload']['Requirement Set']
    manifest=json.loads((HERE/'evidence/REVIEW_SOURCE_MANIFEST_002.json').read_text())
    for r in manifest['files']:assert sha(ROOT/r['path'])==r['sha256'],r['path']
    assert sha(HERE/'evidence/REVIEW_SOURCE_002.tar.gz')==manifest['archive_sha256']
    body=row(str((HERE/'CODE_REVIEW_002.ko.md').relative_to(ROOT)));baseline=row(str((HERE/'evidence/PREDECESSOR_BASELINE_001.json').relative_to(ROOT)))
    arts=[];evidence=[];observations=[];stamp=datetime.datetime.now(datetime.timezone.utc).isoformat()
    for requirement in requirements:
        key=requirement['Requirement ID']; applicable=requirement['Applicability/Necessity']=='required'
        locations=requirement['Required identity/location constraint']['locations'] if applicable else []
        subset=[r for r in manifest['files'] if r['path'] in locations]
        ev=artifact('Review Evidence',f'REV-CRSL-{key}-002',{
            'Repository/environment identity':{'repository':str(ROOT),'git_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'axis':requirement['Source type/role']},
            'Source Set':subset,'Exact Source Archive':row(str((HERE/'evidence/REVIEW_SOURCE_002.tar.gz').relative_to(ROOT))),
            'Observation command/method':'Bounded read-only sed/rg forward/reverse source/owner/consumer review; exact source capture. Official inventory lexical only; predecessor367 product hashes freshly compared.',
            'Observed result':{'body':body,'selector':f'Current Actual {key}, TD01–11, Element Coverage'} if applicable else requirement['Validation requirement'],
            'Observation timestamp':stamp,'Supported claim':requirement['Required claim/boundary'],'Limitations':'Static workspace current/delta/design only; no new execution or production-complete claim' if applicable else requirement['Validation requirement'],
            'Preimplementation Baseline':baseline,'Runtime tests executed':[]})
        obs=artifact('Source Observation',f'SO-CRSL-{key}-002',{
            'Evaluated Working Revision Ref':before['ref'],'Review Scope/Source Requirement Plan Ref':plan['ref'],'Requirement ID':key,
            'Observed source identity/location/version':subset,'Disposition':'confirmed' if applicable else 'N/A','Observation Evidence Ref Set':[ev['ref']],
            'Limitation or N/A reason':'Workspace current actual/delta; implementation validation pending' if applicable else requirement['Validation requirement'],'Observation timestamp':stamp})
        arts.extend([ev,obs]);evidence.append(ev);observations.append(obs)
    basis=artifact('Reviewed Source Basis','RSB-CRSL-002',{'Selected Source Set':[
        {'Source ID':r['Requirement ID'],'Source Observation Ref':o['ref'],'Review Evidence Ref':e['ref'],'Supported claim':r['Required claim/boundary'],'Disposition':o['payload']['Disposition']}
        for r,o,e in zip(requirements,observations,evidence)]})
    def required(element):
        n=int(element['payload']['Stable Element ID'].split('-')[-1]);return [key for key,nums,*_ in GROUPS if n in nums]
    review=artifact('Code Review Artifact','CRA-CRSL-002',{'Input Working Revision Ref':before['ref'],'Review Scope/Source Requirement Plan Ref':plan['ref'],'Reviewed Source Basis Ref':basis['ref'],'Review Evidence Ref Set':[a['ref'] for a in evidence],'Review body':body,
        'Element review mapping':[{'Reviewed Policy Element Revision Ref':e['ref'],'Approved Target':e['payload']['Policy Statement'],'Decision Revision Ref':e['payload']['Decision Revision Ref'],'Approved Observable Policy Acceptance':e['payload']['Observable Policy Acceptance'],'Required Claim/Boundary Set':required(e)} for e in elements],
        'Current Actual':'CODE_REVIEW_002.ko.md Current Actual RQ01–07','Implementation Delta':'Current Actual last column and TD01–10','Proposed Implementation':'TD01–10 owner/state/input/result/order/failure/cutover/alternatives; exact GEX design still pending','Validation Seams':'Element Coverage 01–14','Findings/Unknowns':'IF01–03 implementation; TD11 technical; ED01 evidence limitation','Review Outcome':'14 policy meanings preserved; material finding0 and required source gap0; no production mutation'})
    reviewed=[]
    for e in elements:
        p=copy.deepcopy(e['payload']);p.update({'Predecessor Policy Element Revision Ref':e['ref'],'Intrinsic status':'code_reviewed'})
        reviewed.append(artifact('Policy Element Revision',e['payload']['Stable Element ID'].replace('PE-','PER-')+'-REVIEWED-002',p))
    wp=copy.deepcopy(before['payload']);wp.update({'Predecessor Working Revision Ref':before['ref'],'Current Policy Element Revision Map':{e['payload']['Stable Element ID']:e['ref'] for e in reviewed}})
    working=artifact('Working Revision','WR-CRSL-004',wp);bindings=[];candidates=[]
    for previous,new in zip(elements,reviewed):
        keys=required(previous);n=previous['payload']['Stable Element ID'].split('-')[-1]
        selected=[ev['ref'] for r,ev in zip(requirements,evidence) if r['Requirement ID'] in keys]
        binding=artifact('Code Review Binding',f'CRB-CRSL-{n}-002',{'Working Revision Ref':working['ref'],'Bound Working Revision Ref':working['ref'],'Policy Element Revision Ref':new['ref'],'Code Review Artifact Ref':review['ref'],'Review Path':'policy_revision_review','Predecessor Code Review Binding Ref':None,
            'Target Current-Element-Map Match Evidence':{'element':new['payload']['Stable Element ID'],'exact_ref_equal':True},'P2G-20 Review Scope/Source Requirement Plan Ref':plan['ref'],'Reviewed Source Basis Ref':basis['ref'],'Review Evidence Ref Set':selected,'Covered Required Claim/Boundary Set':keys,
            'Evaluated applicable active Change/Obligation/Closure Ref Set':[],'Applicable Policy Impact Closure Canonical Union':[]})
        candidate=artifact('Candidate Review Envelope',f'CRE-CRSL-{n}-002',{'Candidate Batch ID':'CAND-CRSL-002','Input Working Revision Ref':before['ref'],'Reviewed Element/Revision':previous['ref'],'Review Path':'policy_revision_review','Candidate Code Review Artifact Ref Set':[review['ref']],'Candidate Review Evidence Ref Set':selected,'Proposed code_reviewed Revision Payload':new['payload'],'Candidate Binding Payload':binding['payload'],'Candidate Result':'accepted-for-complete-result','Abort Reason':None,'Lifecycle Disposition':'non-current'})
        bindings.append(binding);candidates.append(candidate)
    batch=artifact('Review Batch','RB-CRSL-002',{'Candidate Batch Outcome':'ready-for-complete-result','Input Working Revision Ref':before['ref'],'Candidate Review Envelope Ref Set':[a['ref'] for a in candidates],
        'Review Scope/Source Requirement Plan Ref':plan['ref'],'Source Observation Ref Set':[a['ref'] for a in observations],'Code Review Artifact Ref Set':[review['ref']],'Review Evidence Ref Set':[a['ref'] for a in evidence],'Reviewed Source Basis Ref':basis['ref'],
        'Produced code_reviewed Policy Element Revision Ref Set':[a['ref'] for a in reviewed],'Result Working Revision Ref':working['ref'],'Result Current code_reviewed Policy Element Revision Ref Set':[a['ref'] for a in reviewed],'Result-basis Code Review Binding Ref Set':[a['ref'] for a in bindings],
        'Batch Completeness Evidence':{'source_requirements':9,'required_confirmed':7,'not_applicable_with_reason':2,'elements':14,'required_gaps':[],'new_material_policy_findings':[],'one_result_successor':True,'static_review_only':True},'Applicable P2G-20 Obligation Key Set':[],'Complete Result Validation':'passed','Abort Evidence':None})
    arts.extend([basis,review,*candidates,*reviewed,working,*bindings,batch]);refs={a['ref']['Artifact ID']:a['ref'] for a in old+arts}
    def check(v):
        if isinstance(v,dict):
            if set(v)=={'Artifact Type','Artifact ID','Content Digest'}:assert refs[v['Artifact ID']]==v,v
            else:
                for x in v.values():check(x)
        elif isinstance(v,list):
            for x in v:check(x)
    check(arts)
    for a in arts:assert artifact(a['ref']['Artifact Type'],a['ref']['Artifact ID'],a['payload'])==a
    for previous,new,binding in zip(elements,reviewed,bindings):
        for field in ['Policy Statement','Decision Revision Ref','Observable Policy Acceptance']:assert previous['payload'][field]==new['payload'][field]
        assert binding['payload']['Bound Working Revision Ref']==working['ref']
        assert binding['payload']['Policy Element Revision Ref']==working['payload']['Current Policy Element Revision Map'][new['payload']['Stable Element ID']]
    assert len(reviewed)==len(bindings)==14
    assert {r['Requirement ID'] for r in requirements}=={o['payload']['Requirement ID'] for o in observations}
    write('P2G_20_REVIEW_002.json',{'artifacts':arts})
    write('P2G_20_RESULT_002.json',{'stage':'P2G-20','status':'complete','Input Working Revision Ref':before['ref'],'Result Working Revision Ref':working['ref'],'Review Batch Ref':batch['ref'],'Review Scope/Source Requirement Plan Ref':plan['ref'],
        'checks':{'exact_refs_digests':True,'150_source_files_unchanged':True,'all14_acceptance_preserved_bound':True,'required7_confirmed':True,'N_A2_supported':True,'new_material_findings':0,'required_source_gaps':0},'production_mutation':False,'next':'P2G-30'})
    p=HERE/'WORKING_POLICY.ko.md';s=p.read_text().replace('`WR-CRSL-003`','`WR-CRSL-004`').replace('현재 P2G-10 완료, 다음은 P2G-20 정식 코드 검토다.','현재 P2G-20 완료, 다음은 P2G-30 정책 동결이다.')
    s+='\n현재 [검토 본문](CODE_REVIEW_002.ko.md), [완전 batch002](P2G_20_REVIEW_002.json), [검증](P2G_20_RESULT_002.json): 150 exact source, required7 confirmed/N·A2, 14 code_reviewed와 result binding. 제품 구현/실행 성공은 아직 아니다.\n';p.write_text(s)
    print('U05 P2G20 complete:14 exact code-reviewed/bindings;150 sources;7 required+2 N/A;no production changes')
if __name__=='__main__':main()
