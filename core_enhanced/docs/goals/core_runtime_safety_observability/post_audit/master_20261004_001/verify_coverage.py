"""Master integration mapping, cumulative continuity and evidence applicability checks."""
from pathlib import Path
import json,hashlib,re,subprocess,datetime
R=Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';A=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def write(n,v):(A/n).write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
def artifact(n):
 a=load(D/n)['artifacts'][0];assert a['ref']['Content Digest']=='sha256:'+hashlib.sha256(json.dumps(a['payload'],sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()).hexdigest();return a
f=artifact('P2G_30_FREEZE_001.json');p=artifact('P2G_40_PHASE_PLAN_001.json');h=artifact('P2G_60_HANDOFF_001.json');g=load(D/'P2G_50_RESULT_001.json');md=(D/'P2G_50_GOAL_SET.ko.md').read_text()
assert h['payload']['Final Policy Freeze Ref']==f['ref'];assert h['payload']['Phase Plan Ref']==p['ref'];assert p['payload']['Bound Final Policy Freeze Ref']==f['ref'];assert h['payload']['Goal Set Ref']==g['Goal Set Ref'];assert g['Goal Set Ref']['Content Digest']=='sha256:'+sha(D/'P2G_50_GOAL_SET.ko.md')
assert g['member_ids']==['GOAL-CRSO-MASTER','GOAL-CRSO-S1','GOAL-CRSO-S2']
c=load(D/'execution/master/COVERAGE.json');old=load(D/'post_audit/s2_20261004_001/CONDITION_REVIEW.json');n01=load(D/'post_audit/s2_20261004_002/CONDITION_REVIEW.json')
judgments=[
'One immutable embedded38-number profile remains the operational default; observation configuration is downstream and absent from content/safety identities. Original ceilings and typed legal ranges preserved. Current API/CLI/host defaults and S2 release payload evidence apply.',
'Safety loader rejects explicit invalid input before load_game_data publishes content; it does not use the optional diagnostic-config failure fallback. Bounded parsing and public invalid-host evidence remain exact.',
'ContentGeneration holds immutable database/profile; admission identity pairs original content identity with effective safety hash. Separate host observation RON and recorder never enter it. Raw/effective and concurrent generation evidence applies.',
'Current preparation uses the same generation before first native work and current-host restore. Read-only prepared checkpoint is persisted before live-root move; failed actual store writes leave save bytes/root unchanged. No settings imported as trusted save authority.',
'Public wrapper body and native teardown preserve same nested allowance/first fault, typed errors and rollback. Active-battle restore refuses before revision mutation; observed discard is diagnostic only. Actual API/CLI/host and opaque capability cases apply.',
'Original selected parse/compile/evaluate/query limits and semantic identity are unaffected by observation. Current named selectors and exact public/PC expression oracles cover shared consumers; no alternate evaluator or reconstructed query introduced.',
'History settings/import/count/bytes/span owner remains original; observation reads cached fields and uses already-required reconciliation. Cursor/resync and long-battle history tests remain current; span does not become gameplay timeout.',
'Native totals copied only after existing budget take; insertion/reconciliation hooks sample numeric peaks. Outer guard records actual returned fault once, and live-root address protects committed state. Scrape reads standard recorder, not Core.',
'Bounded standard recorder batches use synchronous upkeep plus deferred-retirement fence independently of scrape. Captured real Prometheus/Grafana/browser data and exact seven-mode release parity are applicable to unchanged production inputs.',
'Finite labels/handles/buckets,1024 queue,128 batch,300 ring and O(live sessions) weak registry are explicit. Queued loss cannot erase atomic counters/current state or game events. Real two-session selection and production sample vocabulary remain applicable.',
'Pending capture detaches TLS; discard retains attempts/peaks with no commit or invented fault. Counters/current ownership is independent of summary consumption. Sequence dedup and absent-owner expiry now have accurate canonical definitions; end/drop clears aggregates/selection.',
'S2 is designated final executable owner and has current nonzero complete suites/public/cost/consumer evidence. Source exceptions are precisely Markdown-only N01 and earlier cfg(test) test addition for three release receipts. Raw failures, warnings and inherited cancellation oracles remain explicit.'
]
accept=[]
for i,(r,previous,successor,j) in enumerate(zip(f['payload']['Observable Policy Acceptance Set'],old['acceptances'],n01['acceptances'],judgments),1):
 assert previous['policy_ref']==r['Policy Element Revision Ref'];assert previous['requirement']==r['Observable Policy Acceptance'];assert successor['policy_ref']==previous['policy_ref'];assert successor['successor_verdict']=='PASS / current'
 accept.append(dict(policy_ref=r['Policy Element Revision Ref'],requirement=r['Observable Policy Acceptance'],primary_phase='PHASE-CRSO-S1' if i<=7 else 'PHASE-CRSO-S2',primary_goal='GOAL-CRSO-S1' if i<=7 else 'GOAL-CRSO-S2',final_evidence_owner='GOAL-CRSO-S2',verdict='PASS',master_judgment=j,prior_semantic_ref='post_audit/s2_20261004_001/CONDITION_REVIEW.json#acceptances/'+str(i-1),N01_currentness_ref='post_audit/s2_20261004_002/CONDITION_REVIEW.json#acceptances/'+str(i-1)))
assert len(accept)==12
integration=[]
validations={r['id']:r for r in old['validations']}
for r in p['payload']['integration']:
 v=re.search(r'IVA-CRSO-(V\d+)-(S[12])',r['id']);assert v;vid,phase=v.groups();assert r['disposition']=='assigned_execution_node';assert r['nodes']==['WP-CRSO-'+phase+'VERIFY'];assert r['phases']==['PHASE-CRSO-'+phase];assert validations[vid]['verdict']=='PASS';assert r==next(x['frozen_assignment'] for x in c['integration'] if x['frozen_assignment']['id']==r['id'])
 integration.append(dict(id=r['id'],original_goal='GOAL-CRSO-'+phase,node=r['nodes'][0],disposition=r['disposition'],seam_locator=r['locator'],current_validation_ref='post_audit/s2_20261004_001/CONDITION_REVIEW.json#'+vid,final_execution_owner='GOAL-CRSO-S2/WP-CRSO-S2VERIFY',verdict='PASS',master_assessment='Existing durable Phase seam remains available; source-bound S2 final executions revalidate shared consumers, N01 changes no executable input, and Master adds no source delta. Original owner is retained; Master owns no implementation seam.'))
assert len(integration)==13 and len({r['id'] for r in integration})==13
# Extract both original Phase-specific dimension matrices, not only a shared dimension list.
dimensions=[]
for phase in ['S1','S2']:
 section=md.split('## Phase Goal Member `GOAL-CRSO-'+phase+'`')[1].split('## Phase Goal Member')[0].split('## Policy–Phase–Goal')[0]
 for name,seam,requirement in re.findall(r'^\| ([a-z_]+) \| required \| ([^|]+) \| ([^|]+) \|$',section,re.M):
  dimensions.append(dict(phase='PHASE-CRSO-'+phase,dimension=name,disposition='required',original_seam=seam.strip(),completion_requirement=requirement.strip(),verdict='PASS',evidence=('post_audit/s1_20261004_003/COMPLETION_AUDIT.md#Full S1 acceptance and validation' if phase=='S1' else 'post_audit/s2_20261004_001/CONDITION_REVIEW.json#required_dimensions'),successor_assessment='S2 final source/execution and independent full-delta review cover original dimension, not just happy path. Exact N01→Master inputs and Master cross-Phase authority/storage/persistence review preserve applicability.'))
assert len(dimensions)==20 and all(sum(r['phase']=='PHASE-CRSO-'+p for r in dimensions)==10 for p in ['S1','S2'])
write('CONDITION_REVIEW.json',dict(goal='GOAL-CRSO-MASTER',acceptances=accept,integration=integration,phase_dimensions=dimensions,N_A=[],all_required_covered=True))
paths=['execution/s1/corrections/f02_persistence_entry_001/target/s1_f02_001/MANIFEST.json','execution/s2/target/s2_001/MANIFEST.json','execution/s2/corrections/n01_metric_docs_001/target/s2_n01_001/MANIFEST.json','execution/master/target/master_001/MANIFEST.json'];manifests=[load(D/x) for x in paths];continuity=[]
for i in range(1,4):
 before,after=manifests[i-1],manifests[i];ob={x['path']:x for x in before['source_files']};nb={x['path']:x for x in after['source_files']};delta=[x for x in sorted(set(ob)|set(nb)) if ob.get(x)!=nb.get(x)]
 assert after['before_files']==before['source_files']
 assert len(delta)==[33,1,0][i-1]
 if i==1:assert set(delta)=={x['path'] for x in load(D/'post_audit/s2_20261004_001/SOURCE_REVIEW.json')['changed_paths']}
 if i==2:assert delta==['game_server/config/observability/README.md']
 continuity.append(dict(from_target=paths[i-1],to_target=paths[i],from_manifest_sha256=sha(D/paths[i-1]),to_manifest_sha256=sha(D/paths[i]),changed=delta,added=sorted(set(nb)-set(ob)),deleted=sorted(set(ob)-set(nb)),before_exact_prior_after=True,judgment=['S2 full-delta source/condition/execution review re-establishes S1 semantics; historical S1 source is different.','Only canonical hyperlink corrected; N01 independent audit proves runtime/evidence non-impact.','Exact current source bytes; Master documents/evidence only.'][i-1]))
write('PHASE_CONTINUITY.json',dict(chain=continuity,initial_baseline_owner='GOAL-CRSO-S1',initial_baseline='execution/s1/baseline/CURRENTNESS.json; exact audited U01 Core input reuse and separately preserved failed preimplementation server baseline',final_baseline_owner='GOAL-CRSO-S2/WP-CRSO-S2VERIFY',Master_baseline_owned=False))
current={x['path']:x for x in manifests[-1]['product_source']['files']};bindings={};logs={}
for name,b in c['receipt_bindings'].items():
 receipt=D/'evidence'/(name+'.json');r=load(receipt);log=receipt.with_suffix('.log');assert sha(receipt)==b['receipt_sha256'];assert sha(log)==b['log_sha256']==r['log_sha256'];assert r['exit_code']==b['raw_exit_code'];assert r['source_before']==r['source_after'] and r['source_unchanged'];prior={x['path']:x for x in r['source_after']['files']};delta=[x for x in sorted(set(prior)|set(current)) if prior.get(x)!=current.get(x)]
 assert set(delta)==set(b['reuse_test_only_delta']+['game_server/config/observability/README.md'])
 for inp in r['command_inputs']:assert sha(D/inp['archive'])==inp['sha256']
 logs[name]=log.read_text();bindings[name]=dict(receipt_sha256=sha(receipt),log_sha256=sha(log),raw_exit=r['exit_code'],started_at_utc=r['started_at_utc'],command=r['command'],toolchain=r['toolchain'],source_delta_to_Master=delta,reuse='Valid: N01 doc hyperlink cannot affect execution; named cfg(test) addition affects no release binary. Original execution date/result unchanged.')
assert len(bindings)==34
# Recheck nonzero selector presence; prior audited semantic interpretation is conditionally reused.
for v in old['validations']:
 for x in v['verified_selectors']:
  assert x['receipts']
  for n in x['receipts']:assert 'test '+x['selector']+' ... ok' in logs[n]
pre=load(D/'post_audit/s2_20261004_001/PREDECESSOR_REVIEW.json');latest=load(D/'post_audit/s2_20261004_002/PREDECESSOR_REVIEW.json');inherited=[]
for i,r in enumerate(pre['conditions']):
 assert r['verdict']=='PASS';assert (r['id'],r['group'])==(latest['conditions'][i]['id'],latest['conditions'][i]['group'])
 for x in r['current_selector_evidence']:
  for n in x['receipts']:assert 'test '+x['selector']+' ... ok' in logs[n]
 inherited.append(dict(id=r['id'],group=r['group'],requirement=r['requirement'],prior_semantic_ref='post_audit/s2_20261004_001/PREDECESSOR_REVIEW.json#conditions/'+str(i),successor_ref='post_audit/s2_20261004_002/PREDECESSOR_REVIEW.json#conditions/'+str(i),verdict='PASS / current',master_assessment='No additional product delta after assessed S2 successor. Exact evidence/source/dependency inputs verified; direct Master native/commit/profile/observation/persistence review preserves the shared assumptions of this original condition. Original row-specific selectors remain nonzero; X09/X10 use full baseline/restoration, not invented named tests.'))
assert len(inherited)==88 and len(pre['r06_original_acceptances'])==31
write('PREDECESSOR_REVIEW.json',dict(conditions=inherited,r06_original_acceptance_mapping_ref='post_audit/s2_20261004_001/PREDECESSOR_REVIEW.json#r06_original_acceptances',r06_original_acceptance_mappings=31,group_counts={group:sum(x['group']==group for x in inherited) for group in set(x['group'] for x in inherited)}))
refs={}
for audit in ['s1_20261004_003','s2_20261004_001','s2_20261004_002']:
 for x in (D/'post_audit'/audit).glob('*'):
  if x.is_file():refs[str(x.relative_to(D))]=sha(x)
prior_ev=load(D/'post_audit/s2_20261004_001/EVIDENCE_VERIFICATION.json')
for name,s in prior_ev['suites'].items():
 receipt=load(D/'execution/s2/COVERAGE.json')['suites'][name]['receipt'];totals=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',logs[receipt])];assert [sum(x[i] for x in totals) for i in range(3)]==[s['passed'],s['failed'],s['ignored']]
write('EVIDENCE_REUSE.json',dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),receipt_bindings=bindings,prior_audit_references=refs,verified_suite_counts=prior_ev['suites'],mechanical_methods_reviewed=['s2 audit001 verify_evidence.py','s2 audit001 verify_public.py','s2 audit001 recheck_r05.py','s2 audit001 dependency verification and direct pinned-source assessment'],reused_actual_results=['35 release payload/timing comparisons','251 actual browser datasource frames and4197 metrics lines','PC576 non-timing parity','R05 preserved raw101 authorized peak-only warning','8 prepared privacy cases; observed two blocked writes then204→213','actual default/profile/API/authority/CLI and5 cancellation+trace public oracles','dependency archive/source correspondence and finite-storage analysis'],no_runtime_suites_reexecuted=True,reuse_reason='Requirements and final production/test/config/fixture/dependency identities are exact after audited N01 documentation-only change; no new integration discrepancy invalidated prior actual runs. Master reviews cross-Phase contract and required baseline ownership, not manufacturing a new executable baseline.'))
write('UPSTREAM_BASIS.json',dict(freeze=f['ref'],phase_plan=p['ref'],goal_set=g['Goal Set Ref'],handoff=h['ref'],all_composite_links_exact=True,phase_count=2,work_package_count=len(p['payload']['work_packages']),integration_count=len(integration),acceptance_count=len(accept),dimension_count=len(dimensions)))
print('PASS: exact upstream links;12 acceptances;13 owned IVA;20 required Phase dimensions;34 receipt/log/input bindings;88 inherited rows;33→1→0 path continuity.')
