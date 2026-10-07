"""Trace all predecessor conditions to fresh raw tests; do not turn test success into a blanket audit PASS."""
import json,re,copy
from pathlib import Path
import run
H=run.HERE;E=H/'evidence';P=run.OLD;PE=P/'evidence'
def write(path,value):
 with path.open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
frozen=json.loads((H/'AUDIT_TARGET.json').read_text())
assert run.identity()==frozen['current']
logs={}
for n in ['all_default_001','all_feature_001']:
 r=json.loads((E/(n+'.json')).read_text());assert r['exit_code']==0 and r['passed']==1677 and r['failed']==0 and r['source_unchanged']
 assert r['source_after']==frozen['current'] and run.sha(E/(n+'.log'))==r['log_sha256']
 logs[n]=set(re.findall(r'^test (\S+) \.\.\. ok$',(E/(n+'.log')).read_text(),re.M))
coverage=json.loads((P/'COVERAGE.json').read_text())
rows=[]; failed={'A08-03','A08-04','A10-04','A10-06'}
for c in coverage['conditions']:
 selectors=c['actual_test_selectors']
 for n,present in logs.items():assert all(s in present for s in selectors),(c['id'],n)
 disposition='existing_oracles_reexecuted; no additional finding in inspected scope'
 if c['id'] in failed:disposition='FAIL: F01 public boss cancellation counterexample overrides passing predecessor selectors'
 if c['id']=='X09':disposition='environment limitation resolved by unfiltered default/feature 1677 each; checks/CLI/cost evidence verified and reused'
 if c['id']=='X10':disposition='restoration/currentness PASS; all-mandatory-conditions conjunction NOT MET because F01 remains'
 rows.append(dict(id=c['id'],requirement=c['required_input_and_oracle'],legacy_acceptance=c['legacy_acceptance'],selectors=selectors,all_selectors_passed_in=list(logs),disposition=disposition,finding='F01' if c['id'] in failed or c['id']=='X10' else None,predecessor_receipts=c['actual_receipts']))
assert len(rows)==55
write(H/'CONDITION_REVIEW.json',dict(goal='GS-CARX-001 / GOAL-CARX-01',current_source=frozen['current']['digest'],verdict='Not Complete',condition_count=55,scope='55 conditions traced; existing test oracles are evidence, not independent per-condition full approval; F01 blocks the whole goal',conditions=rows))
impact=json.loads((P/'R06_R07_IMPACT_COVERAGE.json').read_text())
for group in ['R06_integration','R07_F01_F12']:
 for r in impact[group]:
  for n,present in logs.items():assert all(s in present for s in r['actual_selectors']),(group,r,n)
  r['audit_reexecution']=list(logs);r['audit_disposition']='all mapped selectors pass; no separate current Complete/High issued on rejected R08-R12 integration'
impact['audit_scope']='R06 31 acceptance -> 21 integration groups; R07 F01-F12. Historical Complete/High retained only for historical target. Shared action/flow lifetime correction must recheck R06/R07.'
impact['audit_current_source']=frozen['current']['digest'];impact['audit_verdict']='R08-R12 Not Complete / F01; prerequisite transfer not authorized'
write(H/'R06_R07_IMPACT_REVIEW.json',impact)
# Verify reusable command receipts against raw output and the product-identical frozen source.
names=['integrated_check_003','integrated_event_check_003','integrated_clippy_003','integrated_fmt_003']
summary=json.loads((PE/'CLI_FINAL_SUMMARY_002.json').read_text());names += [r['name'] for r in summary['commands']]
reused=[]
for name in names:
 p=PE/(name+'.json');r=json.loads(p.read_text())
 assert r['exit_code']==0 and r['source_before']['digest']==r['source_after']['digest']==frozen['frozen_digest']
 assert r['log_sha256']==run.sha(PE/(name+'.log'))
 reused.append(dict(receipt=str(p.relative_to(run.ROOT)),receipt_sha256=run.sha(p),raw_sha256=r['log_sha256'],command=r['command'],exit_code=0,toolchain=r['toolchain'],basis='same product/build/input bytes; ROADMAP_STATUS-only delta cannot alter these executions'))
write(H/'REUSED_EVIDENCE.json',dict(reused_commands=reused,not_rerun='check/event_log_checks/clippy/fmt and prior 45 CLI commands; prior raw bytes verified in frozen 1435 records',new_commands='all-target default/test-fixtures, socket test, release build and public cancellation probes',not_claimed='old raw all-target EPERM failures are not changed; new executions close the environment gap'))
# Independently recompute R05 original guards, allowing exactly the agreed concurrent-peak warning.
raw=json.loads((PE/'r05_raw_final_005.json').read_text());baseline=json.loads((PE/'P01_LOG_V32_BASELINE_001.json').read_text());original=json.loads((run.ROOT/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json').read_text())
a=copy.deepcopy(original);b=copy.deepcopy(baseline)
for family,key in [('event_heavy','event_log_sha256'),('concurrent_sessions','combined_event_log_sha256')]:
 for x,y in zip(a[family]['raw_samples'],b[family]['raw_samples']):x.pop(key);y.pop(key)
assert a==b
assert raw['tool_source_sha256']==run.sha(run.ROOT/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs')
prior=json.loads((run.ROOT/'core_enhanced/docs/goals/content_authoring_fragment_copies/execution/single_goal/evidence/r05_raw_integrated_003.json').read_text())
assert raw['workload_input_sha256']==prior['workload_input_sha256'] and raw['tool_source_sha256']==prior['tool_source_sha256']
assert raw['allocation_observer_enabled'] and raw['allocation_gate_applicable'] and not raw['latency_gate_applicable']
recalc={}
for family in ['event_heavy','concurrent_sessions']:
 w=raw[family];old=baseline[family]['raw_samples'];new=w['allocation_raw_samples'];assert len(old)==len(new)==5
 def maxima(samples):
  h=[s['heap_allocation'] for s in samples]
  return dict(operations=max(s['allocation_count']+s['reallocation_count'] for s in h),retained=max(s['retained_delta_bytes'] for s in h),peak=max(s['peak_delta_bytes'] for s in h))
 cap= maxima(old);actual=maxima(new)
 assert actual['operations']<=cap['operations'] and actual['retained']<=cap['retained'] and w['repeatable'] and w['deterministic_baseline_parity']
 if family=='event_heavy':assert actual['peak']<=cap['peak']
 else:assert cap['peak']==9030188
 recalc[family]=dict(caps=cap,actual=actual,peak_excess=max(0,actual['peak']-cap['peak']))
receipt=json.loads((PE/'r05_final_005.json').read_text());assert receipt['exit_code']==101
assert receipt['log_sha256']==run.sha(PE/'r05_final_005.log')
assert 'assertion failed: receipt.concurrent_sessions.allocation_retained_peak_guard_pass' in (PE/'r05_final_005.log').read_text()
write(E/'R05_RECHECK.json',dict(classification='reused raw, independently recomputed; warning concurrent peak only',raw_exit_code=101,raw_sha256=run.sha(PE/'r05_raw_final_005.json'),other_hard_failures=[],families=recalc))
print(json.dumps(dict(conditions=len(rows),new_suite_passed_each=1677,reused_command_receipts=len(reused),impact_groups=33,verdict='Not Complete',finding='F01')))
