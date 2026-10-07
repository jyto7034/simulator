"""Independent source, selector and transitive evidence verification; read-only product."""
from pathlib import Path
import json,hashlib,re,tarfile
R=Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_fault_isolation'
C=D/'execution/single_goal/corrections/f02_origin_001';T=C/'target/u03_003';A=Path(__file__).parent
P=R/'core_enhanced/docs/goals/core_runtime_safety_observability'
def j(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def w(n,x):(A/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
m=j(T/'MANIFEST.json');cov=j(C/'COVERAGE.json');pred=j(C/'PREDECESSOR_IMPACT.json')
members={str((R/x['path']).resolve()) for x in j(T/'MEMBERSHIP.json')}
extra=set();logtexts={};verified=[]
def use(p):
 p=Path(p);p=p if p.is_absolute() else R/p
 assert p.is_file(),str(p)
 if str(p.resolve()) not in members:extra.add(p.resolve())
 return p
for name,b in cov['receipt_bindings'].items():
 receipt=use(D/'evidence'/f'{name}.json');log=use(receipt.with_suffix('.log'));x=j(receipt)
 assert sha(receipt)==b['receipt_sha256']
 assert sha(log)==x['log_sha256']==b['log_sha256']
 assert x['source_before']==x['source_after']==m['product_source']
 assert x['source_unchanged'] and x['exit_code']==b['raw_exit_code']
 assert x['exit_code']==(101 if name=='f02_compat_r05_001' else 0)
 assert sha(use(D/'tools/run.py'))==x['runner_sha256']
 for ci in x['command_inputs']:
  assert sha(use(ci['path']))==ci['sha256']==sha(use(D/ci['archive']))
 logtexts[name]=log.read_text()
 verified.append(dict(receipt=name,receipt_sha256=sha(receipt),log_sha256=sha(log),raw_exit=x['exit_code'],source_before_after_exact=True,command_inputs_exact=True))
assert len(verified)==36
for name,counts in cov['suite_counts'].items():
 counts_actual=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',logtexts[name])]
 assert {k:sum(x[i] for x in counts_actual) for i,k in enumerate(['passed','failed','ignored'])}==counts
 # A deliberately isolated allocator test is the only ignored ordinary host observation row.
 if counts['ignored']:
  assert counts['ignored']==1 and 'observability::tests::' in logtexts[name]
assert re.search(r'test result: ok\. 1 passed; 0 failed;',logtexts['f02_final_heap001'])
selector_rows=[]
for v in cov['validations']:
 for group,suites in [('core_selectors',['f02_final_core_default001','f02_final_core_fixtures001','f02_final_core_observation001']),('host_selectors',['f02_final_host_default001','f02_final_host_observation001']),('observed_selectors',['f02_final_core_observation001','f02_final_host_observation001'])]:
  for selector in v[group]:
   matched=[s for s in suites if f'test {selector} ... ok' in logtexts[s]]
   assert matched,(v['id'],selector)
   if group!='observed_selectors':assert len(matched)==len(suites),(v['id'],selector,matched)
   selector_rows.append(dict(validation=v['id'],selector=selector,matched_suites=matched))
core_suites=['f02_final_core_default001','f02_final_core_fixtures001','f02_final_core_observation001']
for row in pred['conditions']:
 for s in row.get('selectors',[]):
  assert all(f'test {s} ... ok' in logtexts[n] for n in core_suites),(row['id'],s)
 # U02 selectors are independently enumerated below from original reviewed evidence.
assert len(pred['conditions'])==100 and len(pred['r06_acceptances'])==31
assert len(set(str(x['id']) for x in pred['conditions']))==100
oldrev=j(use(P/'post_audit/master_20261004_002/PREDECESSOR_REVIEW.json'))
assert [x['id'] for x in pred['conditions'][:88]]==[x['id'] for x in oldrev['conditions']]
oldmap=oldrev['r06_original_acceptances']
assert len(oldmap)==31
assert [{k:r[k] for k in ['acceptance','expected','iva']} for r in oldmap]==[{k:r[k] for k in ['acceptance','expected','iva']} for r in pred['r06_acceptances']]
u02_selectors=[]
for row in pred['u02_validation_selector_binding']:
 for group,names in [('core_selectors',core_suites),('host_selectors',['f02_final_host_default001','f02_final_host_observation001']),('observed_selectors',['f02_final_core_observation001','f02_final_host_observation001'])]:
  for selector in row.get(group,[]):
   matches=[n for n in names if f'test {selector} ... ok' in logtexts[n]]
   assert matches,(row['id'],selector)
   u02_selectors.append(dict(validation=row['id'],selector=selector,matched_suites=matches))
# Old transitive data includes original fixtures, source consumers, scripts, assertions,
# prerequisite audits and tool dependencies, not only the current summary receipts.
oldextra=j(use(P/'post_audit/master_20261004_002/EXTRA_INPUTS.json'));old_diff=[]
for r in oldextra:
 p=use(r['path']);h=sha(p)
 if h!=r['sha256']:old_diff.append(dict(path=str(p),prior_sha256=r['sha256'],current_sha256=h))
assert sorted(Path(x['path']).name for x in old_diff)==['libgame_core-2dd464a0adea236d.rlib','libgame_server-f38ff124b64e7a19.rlib']
reuse=j(C/'SERVICE_REUSE.json')
for r in reuse['unchanged_service_inputs']:
 p=use(R/r['path']);assert sha(p)==r['sha256'] and p.stat().st_size==r['bytes']
for key,hkey in [('reused_audit','reused_audit_sha256'),('reused_service_evidence','reused_service_evidence_sha256'),('reused_master_source_review','reused_master_source_review_sha256')]:assert sha(use(R/reuse[key]))==reuse[hkey]
# Explicitly retain the reviewed inherited semantic rows and the tools producing
# current compatibility data (historical old cached Core/server objects are not reused).
for audit in ['master_20261004_001','master_20261004_002','s2_20261004_001','s2_20261004_002']:
 for name in ['COMPLETION_AUDIT.md','PREDECESSOR_REVIEW.json','CONDITION_REVIEW.json','SOURCE_REVIEW.json','EVIDENCE_REUSE.json','EVIDENCE_VERIFICATION.json']:
  p=P/'post_audit'/audit/name
  if p.is_file():use(p)
for p in (P/'tools').glob('*.py'):use(p)
summary=j(D/'evidence/final_fault_f02_001_SUMMARY.json')
for p,h in summary['external_input_files'].items():assert sha(use(R/p))==h
off=use(R/'.cache/d07-runtime-amount-product-target/release/examples/fault_runtime');assert sha(off)==summary['off_binary_sha256']
modes={}
for mode in ['none','detached','queue_full','on','off']:
 lines=[json.loads(x) for x in logtexts['final_fault_f02_001_'+mode].splitlines() if x.startswith('{')]
 modes[mode]=next(x['result'] for x in lines if 'result' in x)
assert all(x==modes['off'] for x in modes.values())
assert modes['off']['repeat_count']==64 and modes['off']['first_error']['code']=='battle_runtime_work_limit'
on=[json.loads(x) for x in logtexts['final_fault_f02_001_on'].splitlines() if x.startswith('{')]
before=next(x['before_observation'] for x in on if 'before_observation' in x)
after=next(x['after_observation'] for x in on if 'after_observation' in x)
assert before['sessions'][0]['commits']==after['sessions'][0]['commits']==1
assert after['selected_session'] is None and after['selected_battle'] is None
assert after['sessions'][0]['current']=={'battle':None,'usage':[0]*9}
assert after['ring'][-1]['boundary']=='ended'
w('EVIDENCE_BINDINGS.json',dict(verified=verified,validation_selector_bindings=selector_rows,u02_selector_bindings=u02_selectors,suite_counts=cov['suite_counts'],isolated_allocator_executed=True,five_mode_payload_equality=True,ended_selection_no_commit=True,scope='Mechanical bindings only. Condition-specific semantic verdicts are separately recorded.'))
w('TRANSITIVE_CLOSURE.json',dict(prior_inputs_checked=len(oldextra),prior_inputs_unchanged=len(oldextra)-len(old_diff),expected_rebuilt_objects=old_diff,unchanged_service_inputs=len(reuse['unchanged_service_inputs']),current_off_binary_sha256=sha(off),historical_on_binary='Replaced by declared final off build; historical on command/source/hash and actual outputs remain frozen. No assertion that historical on bytes survive at the reused path.',scope='All prior external evidence inputs were checked; current command scripts and archived inputs plus service boundary and extra semantic documents included. Current source-dependent evidence is independently bound to successor source, not reused as if unchanged.'))
w('EXTRA_INPUTS.json',[dict(path=str(p),bytes=p.stat().st_size,sha256=sha(p)) for p in sorted(extra)])
print('36 receipts exact; all V selectors and 100-condition named selectors nonzero; five fault modes exact; transitive extra inputs',len(extra),flush=True)
