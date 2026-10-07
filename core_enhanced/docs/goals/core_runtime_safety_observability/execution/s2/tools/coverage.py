"""Close self-verification only from current nonzero receipts and exact reuse."""
from pathlib import Path
import copy,hashlib,json,re,sys
D=Path(__file__).resolve().parents[3];S=D/'execution/s2';C=S/'compatibility/current_001';R=D.parents[3]
sys.path.insert(0,str(D/'tools'));import run
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
write=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
source=run.identity();bindings={};logs={}
def receipt(name, allowed=(0,), reuse=False):
 p=D/'evidence'/f'{name}.json';r=json.loads(p.read_text());log=p.with_suffix('.log');assert r['exit_code'] in allowed,(name,r['exit_code']);assert r['source_unchanged'];assert sha(log)==r['log_sha256']
 differences=[]
 if r['source_after']!=source:
  old={x['path']:x for x in r['source_after']['files']};differences=[x['path'] for x in source['files'] if old.get(x['path'])!=x]
  assert reuse and differences==['core_enhanced/src/game/battle/core/tests/amount_order.rs'],(name,differences)
  assert '#[cfg(test)]' in (R/'core_enhanced/src/game/battle/core/mod.rs').read_text() # tests module omitted from release
 bindings[name]={'receipt_sha256':sha(p),'log_sha256':sha(log),'raw_exit_code':r['exit_code'],'source_sha256':r['source_after']['sha256'],'reuse_test_only_delta':differences}
 logs[name]=log.read_text();return r
suite_names=['core_default','core_fixtures','core_observation','host_default','host_observation']
suites={}
for name in suite_names:
 key='s2_final_'+name+'001';receipt(key)
 results=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',logs[key]);assert results
 assert not re.search(r'test result: FAILED|^error:',logs[key],re.M)
 suites[name]={'receipt':key,'passed':sum(int(a) for a,b,c in results),'failed':sum(int(b) for a,b,c in results),'ignored':sum(int(c) for a,b,c in results)}
 assert suites[name]['passed']>0
for name in ['fmt','check','event_log','clippy_core','clippy_host','release','heap']:
 receipt('s2_final_'+name+'001')
assert '1 passed; 0 failed' in logs['s2_final_heap001']
for name in ['prepared_public','prepared_boundary','observed_release','default_parity','public_oracle','public_trace','public_cli','public_profile','public_boundary','public_api','pc_prepare','pc','pc_compare','pc_immediate','r05_compare']:
 receipt('s2compat_'+name+'_001')
receipt('s2compat_r05_001',(0,101))
receipt('s2_services_live006')
receipt('s2_services_render007')
receipt('s2_services_verify001')
receipt('s2_parity_cost002',reuse=True)
for mode in ['on','off']:receipt('s2_driver_release_'+mode+'002',reuse=True)
core_receipts=['s2_final_core_default001','s2_final_core_fixtures001','s2_final_core_observation001']
host_receipts=['s2_final_host_default001','s2_final_host_observation001']
def verify_selectors(selectors,receipts):
 for selector in selectors:
  for name in receipts:assert f'test {selector} ... ok' in logs[name],(selector,name)
old=json.loads((D/'execution/s1/corrections/f02_persistence_entry_001/COVERAGE.json').read_text())
old_impact=json.loads((D/'execution/s1/corrections/f02_persistence_entry_001/PREDECESSOR_IMPACT.json').read_text())
validations=[]
for v in old['validations']:
 if v['id']=='V12':continue
 verify_selectors(v['core_selectors'],core_receipts);verify_selectors(v['host_selectors'],host_receipts)
 v=copy.deepcopy(v);v['suite_receipts']=core_receipts+host_receipts;v['direct_receipts']=['s2compat_default_parity_001','s2compat_public_profile_001','s2compat_public_boundary_001','s2compat_public_api_001','s2compat_prepared_public_001','s2compat_prepared_boundary_001'];v['status']='current S2 inherited selectors executed; independent applicability audit pending';validations.append(v)
obs_core=[line.split('test ',1)[1].split(' ... ')[0] for line in logs['s2_final_core_observation001'].splitlines() if line.startswith('test ') and 'runtime_observation' in line and line.endswith(' ... ok')]
obs_host=[line.split('test ',1)[1].split(' ... ')[0] for line in logs['s2_final_host_observation001'].splitlines() if line.startswith('test ') and ('observability::tests::' in line or 'runtime_observation' in line) and line.endswith(' ... ok')]
assert len(obs_core)>=6 and len(obs_host)>=7
validations += [
 {'id':'V09','requirement':'Attempt/commit distinction, actual transient projectile peak, returned rejection once, prepared discard and TLS separation, finite vocabulary, selection/end/dedup','core_selectors':obs_core,'host_selectors':obs_host,'receipts':['s2_final_core_observation001','s2_final_host_observation001','s2_services_live006']},
 {'id':'V10','requirement':'Feature off/on/absent/disconnected/full queue/stalled retirement/failed exporter gameplay parity; finite queue/buckets/registry/ring and actual allocator idle maintenance; malformed observation config leaves actual host usable','receipts':['s2_parity_cost002','s2_final_heap001','s2_final_host_observation001','s2compat_public_profile_001'],'storage':'METRICS_AND_STORAGE.md','cost':'COST_AND_PARITY.md'},
 {'id':'V11','requirement':'Actual local standard exporter→Prometheus→Grafana datasource and browser wall/simulation charts, rejection reasons, two sessions, selection switch, end/drop cleanup and exporter restart across process runs','receipts':['s2_services_live006','s2_services_render007','s2_services_verify001'],'evidence':'evidence/services/live006','prior_failure_and_restart_history':['s2_services_live001','s2_services_live002','s2_services_live003','s2_services_live004','s2_services_live005']},
 {'id':'V12','requirement':'Full current Core/default/fixtures/observation and host, fmt/Clippy/check/release, current inherited public U01/R06/R07/PC36/R05 and current S1 F01/F02 boundaries','receipts':list(bindings),'impact':'PREDECESSOR_IMPACT.json'}]
conditions=copy.deepcopy(old_impact['inherited_conditions']);assert len(conditions)==88
for row in conditions:
 verify_selectors(row['selectors'],core_receipts)
 row['receipts']=core_receipts if row['selectors'] else (['s2compat_pc_001','s2compat_r05_001','s2compat_public_oracle_001','s2compat_public_trace_001','s2compat_public_cli_001'] if row['id']=='X09' else ['target/s2_001/RESTORE_VERIFICATION.json','independent end-currentness'])
 row['disposition']='Current execution verified; restoration/semantic applicability requires independent GEX-20'
 row['current_impact']='Optional hooks at public attempt/actual commit/native budget/extent/insertion/history seams and host attachment. Feature-off and observed feature suites both execute inherited selectors; public old inputs, PC36/R05 and prepared persistence boundaries rerun.'
verify_selectors(old_impact['f01_selectors'],core_receipts)
impact={'predecessor_manifest_sha256':sha(D/'execution/s1/corrections/f02_persistence_entry_001/target/s1_f02_001/MANIFEST.json'),'predecessor_audit_sha256':sha(D/'post_audit/s1_20261004_003/COMPLETION_AUDIT.md'),'current_source_sha256':source['sha256'],'inherited_conditions':conditions,'r06_acceptances':old_impact['r06_acceptances'],'f01_selectors':old_impact['f01_selectors'],'input_to_seam':'INPUT_TO_SEAM_IMPACT.json','public_oracle':str((C/'evidence/public_after/SUMMARY.json').relative_to(S)),'public_trace':str((C/'evidence/public_after_trace/SUMMARY.json').relative_to(S)),'public_cli':str((C/'evidence/cli_current/CLI_FINAL_SUMMARY_002.json').relative_to(S)),'independent_audit':'pending; no renewed Complete/High claim'}
write(S/'PREDECESSOR_IMPACT.json',impact)
frozen=json.loads((D/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
acceptances=copy.deepcopy(frozen['payload']['Observable Policy Acceptance Set'][7:]);assert len(acceptances)==5
for a,vs in zip(acceptances,[['V09','V10'],['V10','V11'],['V09','V10','V11'],['V09','V10','V11'],['V10','V12']]):a['validation_refs']=vs
cost=json.loads((C/'evidence/R05_SUMMARY_CURRENT_002.json').read_text());assert not cost['other_hard_guard_failures']
assert json.loads((C/'evidence/PC_IMMEDIATE_PARITY.json').read_text())['passed']
parity=json.loads((S/'evidence/parity_cost002/RESULT.json').read_text());assert len(parity['rows'])==35
live=json.loads((S/'evidence/services/live006/MANIFEST.json').read_text());assert live['exit_code']==0
coverage={'goal':'GOAL-CRSO-S2','phase':'PHASE-CRSO-S2','source_sha256':source['sha256'],'frozen_policy_ref':frozen['ref'],'status':'GEX-10 self-verification complete; independent GEX-20 pending','acceptance':acceptances,'validations':validations,'required_dimensions':{'success':['V09','V11'],'invalid_input':['V10'],'failure_rollback':['V09','V10'],'retry_idempotency':['V09'],'ordering_concurrency':['V09','V10'],'boundary_empty_max':['V10'],'disconnect_restart_persistence':['V10','V11'],'live_data':['V10','V12'],'producer_consumer':['V11'],'legacy_fallback_absence':['V10']},'suites':suites,'receipt_bindings':bindings,'cost_evidence':cost,'limitations':['Local standard services and browser only; no remote/Unity claim.','Histogram drops/expired summaries are observable, never gameplay drops.','No per-battle RSS or new latency gate.','Prior raw failures are preserved and classified in EXPERIMENT_NOTES.md.','Formal restoration target and independent currentness/applicability are still required.']}
write(S/'COVERAGE.json',coverage)
print(json.dumps({'source_sha256':source['sha256'],'suites':suites,'bound_receipts':len(bindings),'inherited_conditions':len(conditions)},indent=2))
