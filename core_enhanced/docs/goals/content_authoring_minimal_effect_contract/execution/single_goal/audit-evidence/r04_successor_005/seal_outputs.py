import pathlib,json,hashlib,datetime
R=pathlib.Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal';O=B/'audit-evidence/r04_successor_005'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def j(p):return json.loads(p.read_text())
entry=j(O/'entry_currentness.json');closure=j(O/'closure_currentness.json');ev=j(O/'evidence_review.json');rt=j(O/'runtime_outputs.json')
assert entry['passed'] and closure['passed'] and ev['passed'] and rt['passed']
assert entry['checked']==closure['checked'] and entry['inventories']==closure['inventories']
assert entry['target_sha256']==closure['target_sha256']==sha(B/'AUDIT_TARGET_005.json')=='631c3aa01a8ef7b3303bc9b979e572c81fdb0e11e2d41915112baff98e548af2'
files=[B/'COMPLETION_AUDIT_005.md']+[p for p in sorted(O.rglob('*')) if p.is_file() and p.name!='AUDIT_OUTPUTS.json']
records=[{'path':str(p.relative_to(R)),'sha256':sha(p),'bytes':p.stat().st_size} for p in files]
d={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'goal_set_ref':'GS-CAMEC-P2G50-001','goal_id':'GOAL-CAMEC-01','record_id':'ATR-CAMEC-R01-005','reviewer':'/root/r01_r04_fresh_audit; separate same-model agent, no fork or implementation participation','verdict':'Complete / High','currentness':'current / unchanged','target':str((B/'AUDIT_TARGET_005.json').relative_to(R)),'target_sha256':closure['target_sha256'],'entry_closure_identical':True,'manifest_members':3785,'required_absences':2,'external_inventories':31,'external_files':2235,'named_default_binaries':3,'full_owner_tests_reused':1361,'fresh_gameplay_executions':0,'retained_output_checks':len(rt['checks']),'registered_output_count':len(records),'files':records,'self_exclusion':'This manifest is excluded from its own recursive digest list; all named output identities are verified after writing.'}
p=O/'AUDIT_OUTPUTS.json';p.write_text(json.dumps(d,indent=2)+'\n')
for m in j(p)['files']:assert sha(R/m['path'])==m['sha256'] and (R/m['path']).stat().st_size==m['bytes']
print(json.dumps({'verdict':d['verdict'],'currentness':d['currentness'],'registered_outputs':len(records),'manifest_sha256':sha(p),'report_sha256':sha(B/'COMPLETION_AUDIT_005.md')},indent=2))
