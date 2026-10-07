import json,hashlib,datetime
from pathlib import Path
R=Path('/mnt/f/work/simulator'); O=Path(__file__).resolve().parent; S=O.parent.parent
j=lambda p:json.loads(p.read_text())
def identity(p):return {'path':str(p.relative_to(R)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size}
a=j(O/'currentness_entry.json');b=j(O/'currentness_closure.json'); e=j(O/'evidence_review.json');c=j(O/'consumer_review.json')
assert all(x['pass'] for x in [a,b,e,c])
for key in ['r02','r01','r01_outputs','r01_registered_count','r01_verdict','target_self_pass']:assert a[key]==b[key],key
assert identity(S/'AUDIT_TARGET_005.json')['sha256']=='a090b1856086f1c98c5d1630bb4464deb28b244eaed950514ce1132300fda57b'
values=[j(R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence/PREDECESSOR_PACK_002'/n) for n in ['consume_a1.stdout','consume_b.stdout','consume_a2.stdout']]
assert [v['run_snapshot']['resources']['enkephalin'] for v in values]==[500,733,500]
final={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'command':'python3 '+str(Path(__file__).relative_to(R)),'entry_closure_identical':True,'all_verifiers_pass':True,'actual_startup_currency':[500,733,500],'target_self_current':True,'pass':True}
(O/'closure_comparison.json').write_text(json.dumps(final,indent=2)+'\n')
files=[S/'COMPLETION_AUDIT_004.md']+sorted(p for p in O.rglob('*') if p.is_file() and p.name!='AUDIT_OUTPUTS.json')
registry={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'goal_set_ref':'GS-CAPG-P2G50-001','goal_id':'GOAL-CAPG-01','record_id':'ATR-CAPG-R02-005','reviewer':'/root/r02_r04_fresh_audit; separate same-model agent, no fork or implementation participation','verdict':'Complete / High','currentness':'current / unchanged','target':str((S/'AUDIT_TARGET_005.json').relative_to(R)),'target_sha256':'a090b1856086f1c98c5d1630bb4464deb28b244eaed950514ce1132300fda57b','entry_closure_identical':True,'manifest_members':4194,'required_absences':2,'external_inventories':39,'external_files':2515,'default_binaries':3,'required_R01_outputs_verified':13,'full_owner_tests_reused':1361,'original_R02_test_rows':72,'fresh_gameplay_executions':0,'source_evidence_checks':len(e['checks']),'consumer_checks':len(c['checks']),'stream_identity_checks':c['stream_hashes'],'registered_output_count':len(files),'files':[identity(p) for p in files],'self_exclusion':'Registry excludes itself from recursive identity list; all registered outputs verified after writing.'}
(O/'AUDIT_OUTPUTS.json').write_text(json.dumps(registry,indent=2)+'\n')
for f in registry['files']:assert identity(R/f['path'])==f
print(json.dumps({'report':identity(S/'COMPLETION_AUDIT_004.md'),'registry':identity(O/'AUDIT_OUTPUTS.json'),'outputs':len(files),'verdict':registry['verdict'],'pass':True},indent=2))
