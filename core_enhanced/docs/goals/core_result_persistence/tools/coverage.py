"""Bind each U04 and inherited acceptance to actual immutable current executions."""
import copy,hashlib,json,re
from pathlib import Path
import run
R,H=run.ROOT,run.HERE
S=H/'execution/single_goal';E=H/'evidence'
selection=json.loads((S/'EVIDENCE_SELECTION.json').read_text())
current=run.identity();bindings={};sha=run.sha

def receipt(name,allowed=(0,)):
 p=E/(name+'.json');v=json.loads(p.read_text())
 assert v['exit_code'] in allowed and v['source_unchanged'],name
 assert v['source_before']==current==v['source_after'],name
 assert v['log_sha256']==sha(E/(name+'.log')),name
 bindings[name]={'receipt_sha256':sha(p),'log_sha256':v['log_sha256'],'source_sha256':current['sha256'],'raw_exit_code':v['exit_code']}
 return v

def write(path,value):
 with path.open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')

suites=selection['suites'];passed={};counts={}
for name in suites:
 receipt(name);t=(E/(name+'.log')).read_text()
 passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',t,re.M))
 totals=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',t)
 counts[name]={key:sum(int(row[i]) for row in totals) for i,key in enumerate(['passed','failed','ignored'])}
 assert counts[name]['passed']>0 and counts[name]['failed']==0,name
core=set.intersection(*(passed[n] for n in suites[:3]));host=passed[suites[3]]&passed[suites[4]]
observed=passed[suites[2]]|passed[suites[4]]

def select(fragments,pool):
 result=set()
 for fragment in fragments:
  found={s for s in pool if fragment in s};assert found,fragment;result.update(found)
 return sorted(result)

contracts={
'V01':(['u04_live_terminal_commits','failed_live_defense_battle_consumes_node','boss_kind_combat_defeat_consumes_node','standard_completion_normal_boss_defeat','final_boss_defeat_with_valid_checkpoint'],['u04_real_terminal_host_tick'],[]),
'V02':(['u04_choose_one_two','u04_choose_one_zero','u04_pending_continue_rejects','u04_terminal_choice'],['u04_conditional_intent'],[]),
'V03':(['u04_continue_unfinished_battle','retreat_battle_restores_exact','two_retreats_consume','event_success_action_starts_combat_and_retreat','final_boss_retreat'],[],[]),
'V04':(['u04_each_unfinished_interactive_node','u04_unfinished_event_combat','u04_unfinished_sweep_continue','u04_continue_unfinished_battle'],[],[]),
'V05':(['u04_continue_before_first_savepoint','u04_live_terminal_commits','u04_terminal_choice'],['u04_two_restored_actors'],[]),
'V06':(['trauma_recovery_supports_flat_and_ceil','save_point_reduces_living_trauma','r09_savepoint_recovery_precedes','u04_savepoint_load','load_run_checkpoint_is_limited_to_three'],['u04_real_process_crash'],[]),
'V07':(['combat_result_commits_attempt_conditions','combat_defeat_commits_attempt_incapacitation','repeated_incapacitation_increments','death_prevent_consumes','dead_employee_cannot_be_revived','consumable_modifier_survives','u04_live_terminal_commits'],['u04_real_terminal_host_tick'],[]),
'V08':(['native_fault_preserves_last_root','u04_prepared_host_tick_fault','trusted_reconciliation_origins'],['real_tick_fault_blocks','new_owner_continue_keeps'],[]),
'V09':([],['u04_real_process_crash','u04_intent_io_failure','u04_real_terminal_host_tick','u04_admin_prepared_drop'],[]),
'V10':(['u04_choose_one_two','u04_savepoint_load'],['u04_conditional_intent','u04_two_restored_actors'],[]),
'V11':(['current_v10_','u04_confirmed_receipt_rejects','u04_pending_continue_rejects','u04_shared_savepoint_encoding'],['u04_incompatible_current_backup','u04_protected_profile'],[]),
'V12':(['public_application_checkpoint_port','u04_prepared_command_and_host_tick'],['u04_real_terminal_host_tick','u04_admin_prepared_drop'],[]),
'V13':([],[],[]),
'V14':(['checkpoint_validation::','u04_','run_checkpoint_rewinds_only_checkpoint','checkpoint_load_restores_floor_anchor'],['u04_real_process_crash'],[]),
'V15':(['owned_clone_shape_complete_world','u04_prepared_command_and_host_tick','u04_shared_savepoint_import_charges'],[],['u04_prepared_terminal_tick_observes','terminal_observation_keeps_failure','actual_queue_batch_dedup']),
}
compat_names=['prepared_public','prepared_boundary','observed_release','default_parity','public_oracle','public_trace','public_cli','public_profile','public_boundary','public_api','pc_prepare','pc','pc_compare','pc_immediate','r05','r05_compare']
compat={name:selection['compatibility_overrides'].get(name,'compat_'+name+selection['compatibility_suffix']) for name in compat_names}
for name in selection['checks']:
 receipt(name)
 if name.startswith('integrated_heap'):
  assert re.search(r'test result: ok\. 1 passed; 0 failed;', (E/(name+'.log')).read_text()), 'required ignored heap probe did not execute exactly once'
for name,key in compat.items():receipt(key,(0,101) if name=='r05' else (0,))
r05_summary_path=S/selection['r05_summary'];r05_summary=json.loads(r05_summary_path.read_text())
assert r05_summary['source_digest']==current['sha256'], 'R05 summary names a different source receipt'
r05_receipt=json.loads((E/(compat['r05']+'.json')).read_text())
assert r05_summary['raw_exit_code']==r05_receipt['exit_code']
assert r05_summary['raw_sha256']==sha(S/selection['compatibility_directory']/'evidence/r05_raw_current_002.json')
assert not r05_summary['other_hard_guard_failures']
fp=selection['fault_prefix']
for name in ['build_on','none','detached','queue_full','on','websocket','build_off','off']:receipt(fp+name)
fault=json.loads((E/(fp+'SUMMARY.json')).read_text());assert fault['passed'] and fault['source']==current
for key in ('continuation','rewind'):receipt(selection[key]);assert json.loads((E/selection[key+'_summary']).read_text())['passed']
public=[selection['continuation'],selection['rewind']]
seams={
'V01':public,'V02':public,'V03':[selection['continuation']],'V04':public,'V05':public,
'V06':[selection['rewind']],'V07':public,'V08':[fp+'websocket'],
'V09':public,'V10':public,'V11':[compat['public_boundary'],compat['public_api']],
'V12':public+[compat['prepared_public'],compat['public_api'],compat['public_cli']],
'V13':list(compat.values())+selection['checks'],
'V14':public+['public_guards003','public_guards_allocator002'],
'V15':[fp+n for n in ('none','detached','queue_full','on','off')]+[compat[n] for n in ('pc','pc_compare','pc_immediate','r05','r05_compare','prepared_public')]+['integrated_heap004','public_guards003','public_guards_allocator002'],
}
review=(H/'CODE_REVIEW_001.ko.md').read_text()
requirements={k:v.strip() for k,v in re.findall(r'^\| (V\d+) \| (.*?) \|$',review,re.M)}
validations=[dict(id=k,requirement=requirements[k],core_selectors=select(c,core),host_selectors=select(h,host),observed_selectors=select(o,observed),suite_receipts=suites,direct_receipts=seams[k],status='current self-verification; independent semantic audit pending') for k,(c,h,o) in contracts.items()]
freeze=json.loads((H/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
acceptances=copy.deepcopy(freeze['payload']['Observable Policy Acceptance Set'])
policy_seams={1:['V01','V02','V09'],2:['V01'],3:['V02','V10'],4:['V03'],5:['V03','V04'],6:['V04','V14'],7:['V05','V09'],8:['V06'],9:['V06','V10'],10:['V07'],11:['V08','V15'],12:['V09','V10','V12'],13:['V10'],14:['V11'],15:['V12','V13','V14','V15']}
for i,row in enumerate(acceptances,1):row.update(validation_refs=policy_seams[i],status='self-verified; independent audit pending')
assert len(acceptances)==15 and len(validations)==15
prior=R/'core_enhanced/docs/goals/core_runtime_fault_isolation/execution/single_goal/corrections/f02_origin_001'
impact=copy.deepcopy(json.loads((prior/'PREDECESSOR_IMPACT.json').read_text()))
prior_coverage=json.loads((prior/'COVERAGE.json').read_text())
renames={}
actor='game::player_game_actor::handlers::tests::'
profile='persistence::player_profile_store::tests::'
intent='persistence::player_profile_store::application_intent::tests::'
host_replacements={
 actor+'active_run_write_failure_does_not_rollback_or_skip_compendium_publish': ([actor+'application_preintent_write_failure_keeps_core_and_profile_unchanged',intent+'u04_intent_io_failure_before_and_after_acceptance_has_distinct_recovery'], 'Intentional U04 acceptance12 change: independent active/profile publication is replaced by one paired intent. Before acceptance neither Core nor files commit; after acceptance recovery retains the accepted result.'),
 actor+'compendium_write_failure_does_not_rollback_active_run_commit': ([actor+'accepted_application_profile_write_failure_commits_core_and_requires_recovery'], 'Accepted intent now owns the boundary: profile publication failure keeps accepted Core and marks recovery pending instead of treating independently saved active bytes as the commit witness.'),
 actor+'pending_standard_completion_blocks_continue_and_retries_in_process': ([actor+'pending_application_blocks_continue_and_retries_in_process'], 'Generalize the existing pending/retry gate from Standard-only completion to every accepted application intent.'),
 profile+'explicit_reset_clears_the_entire_successor_profile_and_all_recovery_artifacts': ([profile+'explicit_reset_preserves_retired_profile_and_journal_originals',profile+'missing_publish_load_backup_recovery_and_reset_use_real_files',actor+'clean_title_exposes_confirmable_full_compendium_reset_without_retry'], 'Intentional U04 acceptance14 protection: supported current profiles still reset completely through real file and title paths; unsupported retired profile/journal originals are now refused and preserved, not deleted.'),
 profile+'standard_completion_journal_closes_profile_and_active_run_after_restart': ([profile+'terminal_application_journal_closes_profile_and_active_run_after_restart',intent+'u04_real_process_crash_at_every_intent_and_aggregate_publication_stage'], 'The paired application journal retains terminal close and fresh-process recovery, broadening its producer scope and covering each real crash stage.'),
 profile+'standard_completion_publish_failure_exposes_neither_side': ([profile+'terminal_application_preintent_failure_exposes_neither_side'], 'Before generic intent acceptance neither aggregate is published; postacceptance recovery is separately exercised, not collapsed into this refusal case.'),
 profile+'standard_completion_retries_failed_active_run_invalidation_from_journal': ([profile+'terminal_application_retries_failed_active_invalidation_from_journal'], 'Generic accepted terminal tombstone recovery retains the original monotonic profile and no-live-run invariant; publication retries remain idempotent.'),
}
def map_selectors(selectors):
 result=[]
 for selector in selectors:
  result.extend(host_replacements[selector][0] if selector in host_replacements else [map_selector(selector)])
 return sorted(set(result))
def map_selector(s):
 n=s.replace('::current_v9_', '::current_v10_')
 if n!=s:renames[s]=n
 return n
# Changed publication/restore semantics are classified per inherited owner.
# Preserve every previous interpretation only inside predecessor_u03_row.
owner_impacts={
'A08': ('Boss request/release/settled routing and stale activation semantics remain inside the unchanged battle kernel. Terminal completion now feeds immediate world settlement; candidate failure and prepared discard still publish neither root nor observer end.', ['V01','V08','V12','V15']),
'A09': ('Corrosion start gate, forced pause and employee attempt costs remain battle-owned. Final enemy death now settles the employee/node once before acknowledgement; Continue retains that settlement, retreat restores pre-entry cost state, and SavePoint explicitly rewinds checkpoint-owned condition counters.', ['V01','V03','V06','V07']),
'A10': ('Flow scheduling, authored result order, targeting and expression arithmetic are unchanged. Live host ticks now prepare a root before accepting terminal persistence; rollback must preserve flow/RNG until acceptance and the compact result must not serialize runtime scheduling history.', ['V01','V08','V12','V14','V15']),
'A11': ('Channel/aura/source ownership and action gates remain battle-local. Terminal settlement discards runtime effects after collecting employee/resource results; retreat restores entry state; Continue serializes only durable owners. Existing exact source/lease selectors cover that boundary.', ['V01','V03','V07','V14','V15']),
'A12': ('Shield, HP-cost, parry, CC and death precedence stay in their existing kernel owners. Terminal preparation moves their durable employee/resource outcome into the confirmed root once; discard/fault retains prior root and explicit rewind uses checkpoint owners.', ['V01','V03','V06','V07','V08','V15']),
'IVA': ('R06 typed arithmetic, parser, amount binding and read-only evaluation contracts remain unchanged. U04 changes the caller lifetime around candidate preparation and terminal restoration; exact inherited selectors and PC576/public oracles rerun, with V15 retaining work/clone accounting.', ['V11','V12','V13','V15']),
'F': ('R07 exact-copy, source binding, grants, recipe and resource owners remain authoritative. Victory now grants unconditional assets before acknowledgement and claims use the existing grant transaction; current Continue and independent SavePoint preserve/rewind their respective copies without recreating IDs.', ['V01','V02','V06','V07','V10','V11']),
}
x_impacts={
'X01': ('External boss Flow public producer remains unchanged; its terminal output is consumed by immediate settlement and accepted host publication.', ['V01','V12','V13']),
'X02': ('Normal exact-copy grant/equip/channel inputs retain their source IDs; terminal/restore follows current receipt and independent checkpoint owners.', ['V01','V06','V07','V13']),
'X03': ('Multi-employee shield/cost/parry kernel behavior is unchanged; result and Continue preserve the committed condition/resource outcome.', ['V01','V07','V13']),
'X04': ('Corrosion with Flow/channel/pool remains kernel-owned; final result now commits once before display/ack and survives restart.', ['V01','V05','V07']),
'X05': ('Directly changed: result settlement, node completion, condition expiry and SavePoint recovery happen before display; V10 restores latest current state with an independent rewind root, retaining exact copies/resources and VN identities.', ['V01','V05','V06','V07','V11','V14']),
'X06': ('Directly changed: first two successful retreats overlay the pre-entry Continue root; disconnect/restart does not spend a retreat; third challenge defeat consumes node. RNG/assets restore, duplicate requests and explicit SavePoint branch replay remain separated.', ['V03','V04','V06','V10']),
'X07': ('Observation publication is deferred until prepared commit; discard must emit no end, observer mode does not change results, and compact persisted receipts do not reconstruct old event histories.', ['V08','V12','V15']),
'X08': ('New continuation codec and prepared payload are charged before allocation/copy; strict imports reject malformed references and unsupported versions without overwriting original roots/files.', ['V08','V09','V11','V15']),
'X09': ('Full current matrix and all16 public procedures are rebound. PC576 and R05 retain original raw evidence/approved peak-only warning; byte/latency guards and allocator probe retain original caps.', ['V13','V14','V15']),
'X10': ('U04 before/after captures actual dirty baseline and current added/deleted members, exact forward/reverse restoration and predecessor archive digests; independent completion remains pending.', ['V13','V14']),
}
u02_impacts={
1: ('Default profile38 numbers and gameplay/error oracles remain unchanged; generic preparation does not add gameplay caps.', ['V12','V13','V15']),
2: ('Safety file parser is unchanged; invalid profile still rejects before publication, separately from strict V10/V5 save decoding.', ['V11','V13']),
3: ('Runtime generation/profile identity stays immutable; continuation uses current validated content identity and host settings, not imported runtime authority.', ['V05','V11','V13']),
4: ('Fresh imports now restore V10 current and optional SavePoint under native/authoring admission. Equal SavePoint reference materializes an independent controlled clone; first-entry and current-host limit refusals retain original bytes.', ['V05','V06','V11','V15']),
5: ('API/CLI admission and cancellation stay unchanged; added generic command/tick/admin prepare scopes retain candidate refusal/rollback before durable intent acceptance.', ['V08','V09','V12','V13']),
6: ('Expression parse/compile/evaluate and legal typed ranges are unchanged. Prepared world metadata must honor surrounding work and clone accounting.', ['V13','V15']),
7: ('Active history cursor/window/statistics remain native bounded owners. V10 persists a compact confirmed result and honestly reports unavailable full historical events after restart.', ['V01','V05','V11','V14']),
8: ('Observer attempts remain distinct from committed results: prepared terminal discard emits no end; accepted commit publishes exactly once. Real first faults remain sticky and counters are not duplicated.', ['V08','V12','V15']),
9: ('Actual unchanged exporter/Prometheus/Grafana evidence is reused by file digests only. Changed command/tick/admin producers and five observer modes rerun with real terminal/fault lifetimes.', ['V08','V12','V15']),
10: ('Finite registry/ring/histogram storage and selection identity remain unchanged. Newly prepared terminal observation publishes at accepted root commit; selection/end cleanup and allocator idle maintenance rerun.', ['V12','V15']),
11: ('Result retries, stale actor claims and discarded preparations do not publish duplicate observer commits/end. Queue saturation affects observation only; first-fault and actual five-mode parity rerun.', ['V08','V09','V10','V12','V15']),
12: ('Full source-bound Core/host matrix, public API/CLI/profile, original defaults, PC576, R05 and prepare boundaries rerun. U01 cancellation and source restoration remain required; old failed attempts are retained.', ['V13','V14','V15']),
}
def inherited_impact(row):
 key=row['id']
 if key=='F09': return ('Directly changed storage contract: V10 current plus independent SavePoint replaces V9. Exact copy/form/owner/resource graph and original checksum/content mismatch refusals are retained under the mapped current_v10 selectors; completed result/choice and rewind roots add explicit U04 validation.', ['V05','V06','V11','V14'])
 if key=='F10': return ('Intentional policy-authorized difference under frozen U04 acceptance14: unsupported legacy saves are preserved/refused and semantic conversion compatibility is not required. Existing explicit V7/V8 helpers retain deterministic V9 conversion and original-byte protection; their V9 output is now refused by normal V10 Continue. The inherited tests explicitly assert that refusal, while copy/history conversion assertions remain. No claim that old conversions produce a current resumable run.', ['V11','V13'])
 if key.startswith('PE-CRSO-'): return u02_impacts[int(key.rsplit('-',1)[1])]
 if key.startswith('X'): return x_impacts[key]
 return owner_impacts['IVA' if key.startswith('IVA-') else 'F' if key.startswith('F') else key.split('-')[0]]
for row in impact['conditions']:
 row['predecessor_u03_row']=copy.deepcopy(row)
 if row.get('selectors'):
  row['selectors']=map_selectors(row['selectors'])
  assert set(row['selectors'])<=core,row['id']
 row['receipts']=suites
 row['current_impact'],row['current_seams']=inherited_impact(row)
 row['current_semantic_boundary']=row['current_impact']
 row.pop('current_fault_seams',None)
 row['directly_affected']=bool(row.get('public_owner_impact')) or row['id'] in ('X05','X06','X07','X08','X09','X10')
 row['evidence_refs']=suites+list(compat.values())+[fp+'SUMMARY.json']+public
 row['disposition']='current exact selector/source binding; independent applicability review pending'
for group in ('u02_validation_selector_binding',):
 for row in impact[group]:
  row['predecessor_u03_binding']=copy.deepcopy(row)
  for key,pool in (('core_selectors',passed[suites[2]]),('host_selectors',passed[suites[4]]),('observed_selectors',observed)):
   if key in row:
    row[key]=map_selectors(row[key]);assert set(row[key])<=pool,(row['id'],key)
  row['current_u04_impact']={
   'V01':u02_impacts[1][0], 'V02':u02_impacts[2][0], 'V03':u02_impacts[3][0],
   'V04':u02_impacts[4][0], 'V05':u02_impacts[5][0], 'V06':u02_impacts[6][0], 'V07':u02_impacts[7][0],
   'V08':'Actual stock host now accepts conditional paired intent before committing commands/ticks/admin; reconnect exposes the same confirmed result, stale revisions refuse claims, and V10 Continue preserves load monotonicity independently of SavePoint.',
   'V09':u02_impacts[8][0]+' '+u02_impacts[10][0], 'V10':u02_impacts[11][0], 'V11':u02_impacts[9][0], 'V12':u02_impacts[12][0],
  }[row['id']]
  row['suite_receipts']=suites;row['direct_receipts']=list(compat.values())
  row['current_changed_lifetime_evidence']=public+[fp+'SUMMARY.json','SERVICE_REUSE.json']
  row['status']='U04 current selector binding and changed prepared/result lifecycle; service-only reuse checked separately'
for row in impact['r06_acceptances']:
 assert row['iva'] and all(any(c['id']==key for c in impact['conditions']) for key in row['iva']),row
 row['status']='unchanged R06 semantic requirement; current mapped IVA selectors and PC576 executed; independent applicability review pending'
u03_impacts={
1:'Generic command/tick/admin preparation retains trusted origin classification; ordinary refusals and uncommitted candidates do not poison the published root.',
2:'First-fault gates precede all added generic preparation and Continue/export ports; no later command can replace the first reason or produce a confirmed receipt.',
3:'Failed native tick cannot install the newly introduced immediate settlement or reward entitlement; the previous confirmed Continue/root and failed-attempt costs remain authoritative.',
4:'Accepted durable intent precedes Core commit and projection; IO after acceptance remains recovery-pending, while projection/transport errors do not create native faults or roll back accepted results.',
5:'Stock WebSocket fault and reconnect use the same public first-fault identity, with bounded cause projection and unchanged root; no persisted normal receipt is invented.',
6:'Added prepared command/tick/admin ports check first-fault gates before mutation/export; ordinary active/noncombat valid calls continue through the same exclusive candidate owner.',
7:'Actual automatic tick fault remains sticky across reconnect/session replacement; scheduled callbacks stop and old sessions cannot mutate or accidentally persist a Continue update.',
8:'The new paired active/profile intent writer is bypassed by every fault-root gate, preserving both original files through later commands/detach/reconnect/stop. Other healthy actors remain writable.',
9:'Quit removes the fault owner; a fresh actor loads the prior valid V10 confirmed continuation with unchanged load use. It never serializes or resumes the faulted battle object.',
10:'Revision/CAS locks are per profile and active file; one fault actor cannot alter another healthy player, whose command/tick and durable publication tests still pass.',
11:'Deferred observer publication retains attempted work/failure on fault, ends selection once and cannot turn discarded terminal preparation into a commit/end. Five-mode native and WebSocket parity rerun.',
12:'100 inherited conditions plus the12 U03 acceptances are explicitly rebound; public native fault, actual stock host, retained R05 raw failure and source restoration constrain the completion claim.',
}
for i,row in enumerate(prior_coverage['acceptances'],1):
 linked=[v for v in prior_coverage['validations'] if v['id'] in row['validation_refs']]
 checks=[]
 for v in linked:
  for key,pool in (('core_selectors',core),('host_selectors',host),('observed_selectors',observed)):
   selectors=map_selectors(v.get(key,[]))
   assert set(selectors)<=pool,(i,v['id'],key)
   checks.append({'predecessor_validation':v['id'],'kind':key,'selectors':selectors,'current_receipts':suites})
 impact['conditions'].append({'id':f'PE-CRFI-{i:02d}','policy_ref':row['Policy Element Revision Ref'],'requirement':row['Observable Policy Acceptance'],'predecessor_validation_refs':row['validation_refs'],'current_selector_bindings':checks,'current_seams':['V08','V12','V13','V15'],'evidence_refs':suites+[fp+'SUMMARY.json',compat['prepared_public'],compat['public_api']],'current_impact':u03_impacts[i],'disposition':'current self-verification; independent semantic applicability pending'})
assert len(impact['conditions'])==112 and len(impact['r06_acceptances'])==31
impact['source_sha256']=current['sha256'];impact['independent_audit']=False
impact['u04_predecessor_path']=str(prior.relative_to(R));impact['u04_predecessor_impact_sha256']=sha(prior/'PREDECESSOR_IMPACT.json');impact['u04_predecessor_coverage_sha256']=sha(prior/'COVERAGE.json');impact['u04_renamed_selectors']=renames
impact['u04_changed_host_selector_dispositions']=[{'predecessor_selector':old,'current_selectors':new,'meaning':meaning,'current_receipts':suites[3:],'additional_u04_seams':['V09','V10','V11','V12']} for old,(new,meaning) in host_replacements.items()]
impact['u04_semantic_changes']=['Immediate settlement replaces ack settlement; ordinary defeat consumes the node.','Two successful retreats overlay unfinished pre-entry Continue; explicit SavePoint rewinds separately.','Current V10 continuation replaces V9 SavePoint-only export; strict validators and old bytes retained/refused. Frozen acceptance14 permits retirement of legacy semantics; historical explicit V7/V8 converters still return V9, which normal Continue now refuses (F10).','Latest V5 paired application intent and CAS replace independent active/profile writes and Standard-only journal.','Equal SavePoint codec sharing stays separate from mutable gameplay ownership; old export byte cap remains.']
service_path=R/'core_enhanced/docs/goals/core_runtime_fault_isolation/execution/single_goal/SERVICE_REUSE.json'
service=copy.deepcopy(json.loads(service_path.read_text()))
for row in service['unchanged_service_inputs']:assert sha(R/row['path'])==row['sha256'],row['path']
for key in ('reused_audit','reused_service_evidence','reused_master_source_review'):assert sha(R/service[key])==service[key+'_sha256']
service['u04_predecessor_record_sha256']=sha(service_path);service['changed_producer']='Generic prepared command/tick/admin defers observer publication until accepted Core root commit; discarded terminal preparations do not end live observation. Collector/exporter/dashboard/service inputs above remain exact.'
service['current_validation']=suites+[fp+'SUMMARY.json','integrated_heap004',selection['continuation']]
service['reuse_limit']='Only unchanged actual service plumbing evidence is reused; changed result/prepare/fault producer lifetime is re-executed on U04 source. No new deployed service claim.'
write(S/'SERVICE_REUSE.json',service)
dimensions={'success':['V01','V02','V05','V07'],'invalid_input':['V02','V08','V11'],'failure_rollback':['V03','V04','V09','V11'],'retry_idempotency':['V01','V02','V09','V10'],'ordering_concurrency':['V09','V10'],'boundary_empty_max':['V02','V03','V06','V15'],'disconnect_restart_persistence':['V04','V05','V06','V08','V09'],'live_data':['V01','V12','V13'],'producer_consumer':['V01','V09','V12','V15'],'legacy_fallback_absence':['V08','V11','V13']}
result=dict(goal='GOAL-CRRP-01',goal_set='GS-CRRP-001',source_sha256=current['sha256'],status='GEX-10 self-verification complete; independent GEX-20 pending',frozen_policy_ref=freeze['ref'],acceptances=acceptances,validations=validations,required_dimensions=dimensions,suite_counts=counts,receipt_bindings=bindings,public_fault_summary='../../evidence/'+fp+'SUMMARY.json',compatibility=selection['compatibility_adaptation'],r05_summary={'path':selection['r05_summary'],'sha256':sha(r05_summary_path)},state_inventory='STATE_INVENTORY.md',unchanged_observation_service_reuse='SERVICE_REUSE.json',limitations=['Actual local Linux Core/stock WebSocket/file subprocess behavior; no Windows crash, remote deployed population or Unity UI claim.','Original R05 raw failure and only approved concurrent-peak warning retained; no latency claim from allocation-instrumented runs.','OBS001 and20minute stress exclusion remain inherited; unchanged service reuse is explicitly byte-bound.','Independent completion verdict belongs solely to separate implementation-nonparticipant audit.'])
write(S/'PREDECESSOR_IMPACT.json',impact);write(S/'COVERAGE.json',result)
print(json.dumps({'current':current['sha256'],'validations':len(validations),'acceptances':len(acceptances),'inherited':len(impact['conditions']),'r06':len(impact['r06_acceptances']),'counts':counts}))
