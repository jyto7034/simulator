import pathlib,json,hashlib,datetime
R=pathlib.Path('/mnt/f/work/simulator');O=pathlib.Path(__file__).parent
D=O.parent.parent;report=D/'COMPLETION_AUDIT_004.md';target=D/'AUDIT_TARGET_004.json'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for n in ['entry.json','closure.json']:
 d=json.loads((O/n).read_text());assert not d['errors'];assert d['currentness']=='current / unchanged'
for n in ['evidence_review.json','correction_review.json','output_review.json','consumer_guard_review.json','supplemental_review.json']:
 assert json.loads((O/n).read_text())['passed'],n
assert sha(target)=='b59521416b76bec8487a9d9353645fc7a5e4bceb0b05818a179e78f2c1197cf1'
files=[report]+sorted(p for p in O.rglob('*') if p.is_file() and p.name!='AUDIT_OUTPUTS.json')
outputs=[{'path':str(p.relative_to(R)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in files]
d={'sealed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'FINAL','goal_set_ref':'GS-CATP-P2G50-001','goal_id':'GOAL-CATP-01','phase_id':'PHASE-CATP-01','target':'ATR-CATP-R03-004','target_sha256':sha(target),'reviewer':'/root/r03_r04_correction_fresh_audit','independence':'Separate same-model agent; no conversation fork or implementation participation; not human or different-model review','verdict':'Complete / High','currentness':'current / unchanged','fresh_gameplay_test_build_executions':0,'outputs':outputs,'registry_excludes_itself':True}
p=O/'AUDIT_OUTPUTS.json';p.write_text(json.dumps(d,indent=2)+'\n')
for row in outputs:
 f=R/row['path'];assert sha(f)==row['sha256'] and f.stat().st_size==row['bytes']
print(json.dumps({'registered_outputs':len(outputs),'report':outputs[0],'registry':{'path':str(p.relative_to(R)),'bytes':p.stat().st_size,'sha256':sha(p)},'target_sha256':sha(target),'verdict':d['verdict']},indent=2))
