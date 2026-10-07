"""Correct canonical revision field shapes, preserving all earlier publication bytes."""
import copy
import json
from initialize import HERE, artifact, write

old=json.loads((HERE/'P2G_20_REVIEW_001.json').read_text())['artifacts']
working=next(a for a in old if a['ref']['Artifact Type']=='Working Revision')
elements=[a for a in old if a['ref']['Artifact Type']=='Policy Element Revision']
policies=json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
decisions={a['ref']['Artifact ID']:a for a in policies if a['ref']['Artifact Type']=='Decision Revision'}
arts=[]; current={}; drs=[]
for e in elements:
 n=e['payload']['Stable Element ID'].split('-')[-1]
 d=decisions[e['payload']['Decision Revision Ref']['Artifact ID']]
 p=copy.deepcopy(d['payload']);p['Predecessor Decision Revision Ref']=d['ref']
 dr=artifact('Decision Revision',f'DR-CRSO-{n}-002',p)
 p=copy.deepcopy(e['payload']);p['Predecessor Policy Element Revision Ref']=e['ref']
 p['Decision Revision Ref']=dr['ref'];p.pop('Intrinsic Status');p['Intrinsic status']='decided'
 p['Observable Policy Acceptance']=[p['Observable Policy Acceptance']]
 per=artifact('Policy Element Revision',f'PER-CRSO-{n}-DECIDED-002',p)
 arts.extend([dr,per]);current[p['Stable Element ID']]=per['ref'];drs.append(dr['ref'])
p=copy.deepcopy(working['payload']);p['Predecessor Working Revision Ref']=working['ref'];p['Current Policy Element Revision Map']=current;p['Decision Revision Ref Set']=drs
wr=artifact('Working Revision','WR-CRSO-004',p);arts.append(wr)
write('P2G_10_POLICY_002.json',{'artifacts':arts})
write('P2G_10_RESULT_002.json',{'stage':'P2G-10','status':'complete','reason':'Canonical field shape correction discovered before P2G-30: Intrinsic status, acceptance array, conventional PER/DR IDs required by repository validator. No policy meaning/approval changed; earlier bytes preserved. P2G-20 fresh result binding required.', 'Predecessor Working Revision Ref':working['ref'],'Result Working Revision Ref':wr['ref'],'checks':{'acceptance_text_unchanged':True,'decision_meaning_unchanged':True,'all_12_decided':True},'next':'P2G-20 fresh review binding'})
v=(HERE/'WORKING_POLICY.ko.md').read_text().replace('`WR-CRSO-003`','`WR-CRSO-004`').replace('현재 P2G-20 완료, 다음은 P2G-30 정책 동결이다.','canonical 필드 형식 교정 successor P2G-10 완료, P2G-20 fresh binding 검증 중이다.')
(HERE/'WORKING_POLICY.ko.md').write_text(v)
print('Canonical shape successor created; policy meaning unchanged, fresh review pending')
