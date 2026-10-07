from pathlib import Path
import json,hashlib,datetime,re
R=Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal';O=D/'audit-evidence/r04_correction_005';report=D/'COMPLETION_AUDIT_005.md';registry=O/'AUDIT_OUTPUTS.json'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for n in ['currentness_entry.json','currentness_closure.json']:
 x=json.loads((O/n).read_text())
 assert x['R02']['currentness']==x['R01']['currentness']=='current / unchanged'
 assert not x['registry_errors']
for n,key in [('evidence_review.json','pass'),('correction_review.json','passed'),('consumer_review.json','pass'),('budget_review.json','passed'),('source_receipts.json','pass')]:assert json.loads((O/n).read_text())[key]
assert sha(D/'AUDIT_TARGET_006.json')=='3e57d1f541f7f1ac94ee9638c8cf2f6ff6821e2c81668350da698c2921f56ee1'
assert '**Verdict: Complete / High**' in report.read_text()
missing=[]
for ref in re.findall(r'\]\(([^)]+)\)',report.read_text()):
 if ref.startswith('http') or 'goal_execution_pipeline(' in ref or ref.endswith('AUDIT_OUTPUTS.json'):continue
 if not (D/ref.split('#')[0]).exists():missing.append(ref)
assert not missing,missing
paths=[report]+sorted(p for p in O.rglob('*') if p.is_file() and p!=registry)
x={'sealed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'goal_set_ref':'GS-CAPG-P2G50-001','goal_id':'GOAL-CAPG-01','phase_id':'PHASE-CAPG-01','target':'ATR-CAPG-R02-006','target_sha256':sha(D/'AUDIT_TARGET_006.json'),'reviewer':'/root/r02_r04_correction_fresh_audit','independence':'Fresh separate same-model agent; no implementation conversation fork or implementation participation; not human or different-model review','verdict':'Complete / High','currentness':'current / unchanged','fresh_gameplay_test_build_executions':0,'outputs':[{'path':str(p.relative_to(R)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in paths],'registry_self_identity':'Registry excludes its own recursive identity; final handoff supplies registry bytes/SHA256.'}
registry.write_text(json.dumps(x,indent=2)+'\n')
for row in x['outputs']:
 p=R/row['path'];assert sha(p)==row['sha256'] and p.stat().st_size==row['bytes']
print(json.dumps({'report':{'path':str(report.relative_to(R)),'bytes':report.stat().st_size,'sha256':sha(report)},'registry':{'path':str(registry.relative_to(R)),'bytes':registry.stat().st_size,'sha256':sha(registry)},'registered_outputs':len(x['outputs']),'all_registered_identities_verified':True},indent=2))
