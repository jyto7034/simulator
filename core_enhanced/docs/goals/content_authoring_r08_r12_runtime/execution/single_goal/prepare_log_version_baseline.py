"""Capture unmodified R07 workload event bytes, then migrate only log envelope version."""
import hashlib,json,tarfile,tempfile,subprocess,sys
from pathlib import Path
H=Path(__file__).resolve().parent;E=H/'evidence';ROOT=H.parents[5];sha=lambda b:hashlib.sha256(b).hexdigest()
S=Path(tempfile.mkdtemp(prefix='r08-r12-log-golden-'));A=H/'baseline/SOURCE_BEFORE.tar.gz'
with tarfile.open(A) as t:t.extractall(S,filter='data')
M=json.loads((H/'baseline/MANIFEST.json').read_text())
for r in M['files']:assert sha((S/r['path']).read_bytes())==r['sha256']
# P01 baseline is a fixed input and was not in the source membership.
base=Path('/mnt/f/work/simulator/core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json')
(S/'core_enhanced/evidence').mkdir(exist_ok=True);(S/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json').write_bytes(base.read_bytes())
out=E/'r07_log_envelopes_001';out.mkdir()
p=S/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs';original=p.read_bytes();s=original.decode()
a='    let serialized = serde_json::to_vec(&core.event_log).expect("serialize event-heavy log");'
assert s.count(a)==1;s=s.replace(a,a+'\n    if sample == 0 { std::fs::write(std::path::Path::new(&std::env::var("R08_GOLDEN_LOGS").unwrap()).join("event.json"), &serialized).unwrap(); }')
a='    let elapsed_ns = started.elapsed().as_nanos();\n    let event_count = lane_results.iter().map(|(_, count, _)| count).sum();'
assert s.count(a)==1;s=s.replace(a,'    if sample == 0 { for (session, _, bytes) in &lane_results { std::fs::write(std::path::Path::new(&std::env::var("R08_GOLDEN_LOGS").unwrap()).join(format!("session_{session}.json")), bytes).unwrap(); } }\n'+a)
p.write_text(s)
(E/'r07_log_capture_runner_001.rs').write_text(s)
(E/'r07_log_capture_identity_001.json').write_text(json.dumps({'isolate':str(S),'baseline_source_digest':M.get('source_digest',M.get('digest')),'all_baseline_bytes_matched_before_instrumentation':True,'baseline_manifest_sha256':sha((H/'baseline/MANIFEST.json').read_bytes()),'instrumented_path':str(p.relative_to(S)),'original_sha256':sha(original),'instrumented_sha256':sha(p.read_bytes()),'purpose':'warmup-only raw event byte capture; no measurement claims'},indent=2)+'\n')
args=[sys.executable,str(H/'run.py'),'r07_log_capture_001','--cwd',str(S/'core_enhanced'),'--','env','CARGO_TARGET_DIR=/tmp/r08-r12-log-golden-target',f'R08_GOLDEN_LOGS={out}','cargo','run','--manifest-path','tools/runtime_architecture_final_validation/Cargo.toml','--bin','final_performance','--release','--features','allocation-observer','--offline','--locked','--','evidence/P01_PERFORMANCE_BASELINE.json',str(E/'r07_log_capture_measurements_001.json')]
code=subprocess.run(args).returncode
assert code in (0,101)
old=json.loads(base.read_text());new=json.loads(base.read_text());proof=[]
def migrate(p):
 b=p.read_bytes();assert b.startswith(b'{"version":31,');new=b'{"version":32,'+b[len(b'{"version":31,'):]
 assert len(b)==len(new);assert json.loads(b)['entries']==json.loads(new)['entries']
 proof.append({'path':str(p.relative_to(H)),'bytes':len(b),'original_sha256':sha(b),'v32_sha256':sha(new),'same_event_entries':True})
 return b,new
a,b=migrate(out/'event.json')
assert sha(a)==old['event_heavy']['raw_samples'][0]['event_log_sha256']
for x in new['event_heavy']['raw_samples']:assert x['event_log_sha256']==sha(a);x['event_log_sha256']=sha(b)
da=hashlib.sha256();db=hashlib.sha256()
for n in range(64):
 a,b=migrate(out/f'session_{n}.json')
 for d,v in [(da,a),(db,b)]:d.update(n.to_bytes(8,'little'));d.update(len(v).to_bytes(8,'little'));d.update(v)
assert da.hexdigest()==old['concurrent_sessions']['raw_samples'][0]['combined_event_log_sha256']
for x in new['concurrent_sessions']['raw_samples']:assert x['combined_event_log_sha256']==da.hexdigest();x['combined_event_log_sha256']=db.hexdigest()
p=E/'P01_LOG_V32_BASELINE_001.json';p.write_text(json.dumps(new,indent=2)+'\n')
(E/'LOG_VERSION_MIGRATION_001.json').write_text(json.dumps({'original_baseline_sha256':sha(base.read_bytes()),'migrated_baseline_sha256':sha(p.read_bytes()),'only_changed_fields':['event_heavy.raw_samples[*].event_log_sha256','concurrent_sessions.raw_samples[*].combined_event_log_sha256'],'workload_and_all_caps_preserved':True,'event_bytes_migrated_only_version_31_to_32':True,'captures':proof},indent=2)+'\n')
print('Captured 65 original logs; exact original hashes verified; only version envelope migrated.')
