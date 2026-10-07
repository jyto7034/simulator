from pathlib import Path
import json,hashlib,os,re,datetime
R=Path('/mnt/f/work/simulator');A=Path(__file__).resolve().parent;G=A.parents[1];B=G/'execution/single_goal/corrections/f01_action_flow_20261004_001';T=B/'target/f01_action_flow_001';E=B/'evidence'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();load=lambda p:json.loads(Path(p).read_text())
# Recompute the complete membership at closure, independent of only-listed file checks.
m=load(T/'MANIFEST.json');names={r['path'] for r in load(G/'execution/single_goal/baseline/MANIFEST.json')['files'] if (R/r['path']).is_file()}
for folder in ['core_enhanced/src','core_enhanced/tests','game_resources/data','core_enhanced/tools/runtime_architecture_final_validation','core_enhanced/tools/runtime_architecture_preservation']:
 for parent,dirs,files in os.walk(R/folder):
  dirs[:]=[d for d in dirs if d not in ('target','__pycache__')]
  names.update(str((Path(parent)/f).relative_to(R)) for f in files if (Path(parent)/f).is_file())
names.add('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json')
rows=[[n,(R/n).stat().st_size,sha(R/n)] for n in sorted(names)];assert rows==m['source_files']
assert hashlib.sha256(json.dumps(rows,separators=(',',':')).encode()).hexdigest()==m['source_digest']
for group in ['RECORDS','REFERENCES']:
 for r in load(T/(group+'.json')):assert (R/r['path']).stat().st_size==r['bytes'] and sha(R/r['path'])==r['sha256']
assert sha(T/'MANIFEST.json')=='3a7bd83b904d1f8367590b7c2d3f4616a9cce8ca6a8a9cb0c7e0cc767d12b50a'
old=load(G/'reviews/r08_r12_20261002_001/AUDIT_TARGET.json')['current']['files'];before=load(B/'baseline/SOURCE_BEFORE.json')['files']
assert [r[0] for r in old]==[r[0] for r in before]
intake_delta=[a[0] for a,b in zip(old,before) if a!=b];assert intake_delta==['core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md']
# Recompute nine public-result and amount comparisons from raw logs.
def normalize(v):
 if isinstance(v,dict):
  d={k:normalize(x) for k,x in v.items()}
  if {'employees','physical_damage_unchanged','replaces_run_checkpoint'}<=d.keys():d['employees']=sorted(d['employees'],key=lambda e:e['employee_uuid'])
  return d
 if isinstance(v,list):return [normalize(x) for x in v]
 return v
parity=[]
for row in load(E/'cli_current/CLI_FINAL_SUMMARY_002.json')['comparisons']:
 tag=row['pack'];outputs=[]
 for suffix in ['off','on']:
  docs=[json.loads(x) for x in (E/'cli_current'/('cli_final_'+tag+'_'+suffix+'_002.log')).read_text().splitlines() if x.strip()]
  public=[normalize(d['payload']['public_result']) for d in docs if d.get('payload',{}).get('kind')=='step'];amount=docs[-4:]
  assert all(d.get('kind') in ('amount_preview','amount_explanation') for d in amount)
  assert any(d.get('status')=='evaluated' for d in amount)
  outputs.append((public,amount))
 assert outputs[0]==outputs[1]
 parity.append({'pack':tag,'public_results':len(outputs[0][0]),'amount_documents':4,'equal':True})
with (A/'CLI_PARITY_RECHECK.json').open('x') as f:json.dump(parity,f,indent=2);f.write('\n')
# Exact requirement coverage and final report links.
conditions=load(A/'CONDITION_REVIEW.json');assert len(conditions['conditions'])==55 and all(r['verdict']=='PASS' for r in conditions['conditions'])
impact=load(A/'R06_R07_IMPACT_REVIEW.json');assert len(impact['R06_acceptances'])==31 and len(impact['integration_conditions'])==33
for r in load(G/'reviews/r06_r07_20260930_001/AUDIT_CLOSURE.json')['reports']:assert sha(R/r['path'])==r['sha256']
paths=[p for p in A.iterdir() if p.is_file()]
closure={'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'goal':'GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001','verdict':'Complete / High','independent_reviewer':'/root/u01_f01_audit; implementation nonparticipant','target':str(T.relative_to(R)),'manifest_sha256':sha(T/'MANIFEST.json'),'source_digest':m['source_digest'],'source_count':len(rows),'source_membership_and_bytes_match':True,'records_and_references_match':True,'currentness':'current / unchanged','u00_prior_audit_to_correction_baseline_delta':intake_delta,'condition_count':55,'condition_failures':[],'r06_acceptances':31,'r06_integration':21,'r07_conditions':12,'blocking_findings':[],'historical_f01':'resolved in this new target; old Not Complete unchanged','residual_limitations':['R05 raw exit101 retained: authorized concurrent peak only +343587 bytes; all other hard guards pass.','Inherited OBS001 recovery-preview employee-array ordering only.','No20-minute test; later roadmap/Unity/deployment not audited.'],'successor_eligibility':'U02 may proceed under the continuous request after its own prerequisite/owner intake.','artifacts':[{'path':str(p.relative_to(R)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(paths)]}
with (A/'AUDIT_CLOSURE.json').open('x') as f:json.dump(closure,f,ensure_ascii=False,indent=2);f.write('\n')
for link in re.findall(r'\]\(([^)]+)\)',(A/'COMPLETION_AUDIT.md').read_text()):
 if '://' not in link:assert (A/link).exists(),link
print(json.dumps({k:v for k,v in closure.items() if k!='artifacts'},ensure_ascii=False,indent=2))
