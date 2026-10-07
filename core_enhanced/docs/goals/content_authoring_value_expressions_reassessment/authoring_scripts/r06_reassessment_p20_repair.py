from pathlib import Path
import json,hashlib,copy
p=Path('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment')
x=json.loads((p/'P2G_20_REVIEW_001.json').read_text()); replacements={a['ref']['Artifact ID']:(a['ref']['Artifact ID']+'-B003' if a['ref']['Artifact Type'] in ['Review Evidence','Source Observation'] else a['ref']['Artifact ID'][:-3]+'003') for a in x['artifacts']}
def change(v):
 if isinstance(v,dict):return {k:change(z) for k,z in v.items()}
 if isinstance(v,list):return [change(z) for z in v]
 if isinstance(v,str):return replacements.get(v,v)
 return v
x=change(x)
body=(p/'CODE_REVIEW_001.ko.md').read_text().replace('Owner: `CRA-CAVER-P2G20-001`','Owner: `CRA-CAVER-P2G20-003`')
(p/'CODE_REVIEW_003.ko.md').write_text(body)
for a in x['artifacts']:
 q=a['payload']
 if a['ref']['Artifact Type']=='Source Observation' and q['Disposition']=='current':q['Disposition']='confirmed'
 if a['ref']['Artifact Type']=='Code Review Artifact':q['Review body']={'path':'CODE_REVIEW_003.ko.md','sha256':hashlib.sha256((p/'CODE_REVIEW_003.ko.md').read_bytes()).hexdigest()}
 if a['ref']['Artifact Type']=='Review Batch':q['Completeness Evidence']='P2G_20_VALIDATION_003.json; CODE_REVIEW_003.ko.md C01/D01–D10'
 for k in ['Review body selector','Current Actual selector','Implementation Delta/Proposed Implementation selector']:
  if k in q and isinstance(q[k],str):q[k]=q[k].replace('CODE_REVIEW_001','CODE_REVIEW_003')
(p/'P2G_20_REVIEW_003.json').write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
# Inspect all stage references, canonical source disposition, exact input/output and all accepted meanings.
arts=[]
for n in ['P2G_00_INITIAL_001.json','P2G_10_POLICY_001.json','P2G_20_SOURCE_PLAN_001.json','P2G_20_REVIEW_003.json']:arts+=json.loads((p/n).read_text())['artifacts']
index={a['ref']['Artifact ID']:a for a in arts};assert len(index)==len(arts)
def refs(v):
 if isinstance(v,dict):
  if 'Artifact Type' in v and 'Artifact ID' in v:yield v
  else:
   for z in v.values():yield from refs(z)
 elif isinstance(v,list):
  for z in v:yield from refs(z)
for a in x['artifacts']:
 for r in refs(a['payload']):assert r['Artifact ID'] in index,(a['ref'],r)
for a in x['artifacts']:
 t=a['ref']['Artifact Type'];q=a['payload']
 if t=='Source Observation':assert q['Disposition'] in ['confirmed','N/A']
 if t=='Policy Element Revision':
  old=index[q['Predecessor Policy Element Revision Ref']['Artifact ID']]['payload']
  for k in ['Policy statement','Decision Revision Ref','Observable Policy Acceptance']:assert q[k]==old[k]
 if t=='Code Review Binding':assert q['Working Revision Ref']['Artifact ID']=='WR-CAVER-P2G20-003'
valid=json.loads((p/'P2G_20_VALIDATION_001.json').read_text());valid['counts']['predecessor_source_members_checked']=len(json.loads((p/'P2G_20_SOURCE_EVIDENCE_001.json').read_text())['predecessor_direct_source_matches']);valid['semantic_evidence']={'path':'CODE_REVIEW_003.ko.md','sha256':hashlib.sha256((p/'CODE_REVIEW_003.ko.md').read_bytes()).hexdigest()};valid['correction']={'predecessor':'P2G_20_VALIDATION_001.json','defects':['Source Observation disposition incorrectly spelled current; P2G requires confirmed','summary predecessor source count shadowed by exit-check variable (19), actual source evidence contains270','candidate002 generation collapsed numbered Evidence/Observation IDs; unique ID assertion failed before current selection, all002 files retained' ],'disposition':'Preserve all 001 files; 001 Review Batch not eligible/current. Same valid P2G-10 input/source-plan, corrected new immutable batch 003 with newly issued observations/reviewed revisions/result/bindings. No Freeze existed; no post-freeze repair obligation.','current':'RB-CAVER-P2G20-003 / WR-CAVER-P2G20-003','fresh_checks':'all exact review payload references resolved, enum checked, acceptance bytes equal, result-basis bindings checked'}
(p/'P2G_20_VALIDATION_003.json').write_text(json.dumps(valid,ensure_ascii=False,indent=2)+'\n')
s=(p/'WORKING_POLICY.ko.md').read_text().replace('WR-CAVER-P2G20-001','WR-CAVER-P2G20-003').replace('RB-CAVER-P2G20-001','RB-CAVER-P2G20-003').replace('P2G_20_REVIEW_001','P2G_20_REVIEW_003').replace('CODE_REVIEW_001','CODE_REVIEW_003').replace('P2G_20_VALIDATION_001','P2G_20_VALIDATION_003');s+='\nP2G-20 초기 001의 observation enum/요약 건수 오류는 [003 검사 correction](P2G_20_VALIDATION_003.json)에 보존한다. 001 batch는 current가 아니며 003만 선택한다.\n';(p/'WORKING_POLICY.ko.md').write_text(s)
print('Corrected immutable P2G-20 batch003: exact refs/enums/27 acceptances pass; predecessor source matches270')
