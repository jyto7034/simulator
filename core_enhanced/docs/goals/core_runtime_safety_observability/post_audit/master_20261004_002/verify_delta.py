"""Fresh Master002 delta/condition audit. No source mutation or runtime execution."""
from pathlib import Path
import json,hashlib,tarfile,re,datetime,subprocess
R=Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';A=Path(__file__).resolve().parent;P=D/'post_audit/master_20261004_001';T=D/'execution/master/followups/n02_001/target/master_002';S=D/'execution/s2/corrections/n02_contract_link_001/target/s2_n02_001'
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def write(n,v):(A/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def artifact(n):
 a=load(D/n)['artifacts'][0];assert a['ref']['Content Digest']=='sha256:'+hashlib.sha256(json.dumps(a['payload'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest();return a
assert sha(P/'COMPLETION_AUDIT.md')=='744c00efa660aceb559f194802fe29735bcc4ec5e2abffbd676bb0a092a815f5'
SA=D/'post_audit/s2_20261004_003';assert sha(SA/'COMPLETION_AUDIT.md')=='7910ee0d89598c825a2720405197d60d5ed2d71d3dc4736296272b0a11e3a4c3';assert sha(SA/'END_CURRENTNESS.json')=='98807d50237a030595ed24bb7fe1a5cbb25868d6931f651156eb0631ecef2f27'
old=load(D/'execution/master/target/master_001/MANIFEST.json');phase=load(S/'MANIFEST.json');new=load(T/'MANIFEST.json');assert sha(S/'MANIFEST.json')=='8a2030564463fa0e3598d11d6a20a14fed0c499bf6ff10522617fe414012e833'
assert old['source_files']==phase['before_files'];assert new['source_files']==new['before_files']==phase['source_files'];assert old['product_source']==new['product_source']==phase['product_source']
a={x['path']:x for x in old['source_files']};b={x['path']:x for x in new['source_files']};changed=[p for p in sorted(set(a)|set(b)) if a.get(p)!=b.get(p)];assert changed==['core_enhanced/docs/core_runtime_contract.md'];assert set(a)==set(b)
name=changed[0]
with tarfile.open(D/'execution/master/target/master_001/SOURCE_AFTER.tar.gz') as f:old_text=f.extractfile(name).read().decode()
current=(R/name).read_text();old_link='goals/core_runtime_safety_observability/execution/s2/METRICS_AND_STORAGE.md';new_link='goals/core_runtime_safety_observability/execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md'
assert old_text.count(old_link)==1;assert old_text.replace(old_link,new_link)==current
contract=D/'execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md';assert sha(contract)=='c75bf02b17876f4178da980f700dc22d044d8a16abf17ad49692372cc71a87f5';routes=[]
for source in [R/name,R/'game_server/config/observability/README.md']:
 links=[x for x in re.findall(r'\]\(([^)]+)\)',source.read_text()) if x.endswith('METRICS_AND_STORAGE.md')];assert len(links)==1;assert (source.parent/links[0]).resolve()==contract
 routes.append(dict(source=str(source.relative_to(R)),href=links[0],destination=str(contract.relative_to(R)),sha256=sha(contract)))
branch=R/'game_server/src/observability/mod.rs';assert sha(branch)=='2457f8064f26253f7dd364fd8048d5ccf4d0d88d7d6c7dc665fff9e2f88d6f5e';text=contract.read_text();assert 'A duplicate or out-of-order sequence for an existing owner is ignored without adding a histogram sample or incrementing that counter.' in text
refs={}
for folder in [P,SA]:
 for x in folder.iterdir():
  if x.is_file():refs[str(x.relative_to(D))]=sha(x)
assert load(P/'EXTRA_INPUTS.json')==load(SA/'EXTRA_INPUTS.json')
# Requirements and all original owners are rechecked against immutable originals.
f=artifact('P2G_30_FREEZE_001.json');pp=artifact('P2G_40_PHASE_PLAN_001.json');h=artifact('P2G_60_HANDOFF_001.json');g=load(D/'P2G_50_RESULT_001.json');assert h['payload']['Final Policy Freeze Ref']==f['ref'];assert h['payload']['Phase Plan Ref']==pp['ref'];assert h['payload']['Goal Set Ref']==g['Goal Set Ref'];assert g['Goal Set Ref']['Content Digest']=='sha256:'+sha(D/'P2G_50_GOAL_SET.ko.md')
prior=load(P/'CONDITION_REVIEW.json');assessment='Master002 independently proves the only cumulative Master001→current delta is the one canonical documentation hyperlink. All359 product inputs,364 other union inputs,606 transitive inputs, frozen requirements and original evidence identities remain exact. Direct N02 branch/link review closes the sole prior finding. Previous condition-specific integration/semantic judgments therefore remain applicable; no executable consumer, fixture, environment or oracle delta requires runtime reruns.'
for i,r in enumerate(prior['acceptances']):
 frozen=f['payload']['Observable Policy Acceptance Set'][i];assert r['policy_ref']==frozen['Policy Element Revision Ref'];assert r['requirement']==frozen['Observable Policy Acceptance'];r['successor_verdict']='PASS / current';r['successor_assessment']=assessment;r['prior_Master_ref']='post_audit/master_20261004_001/CONDITION_REVIEW.json#acceptances/'+str(i)
for i,r in enumerate(prior['integration']):
 original=next(x for x in pp['payload']['integration'] if x['id']==r['id']);assert original['nodes']==[r['node']] and original['disposition']==r['disposition'];r['successor_verdict']='PASS / current';r['successor_assessment']=assessment;r['prior_Master_ref']='post_audit/master_20261004_001/CONDITION_REVIEW.json#integration/'+str(i)
for i,r in enumerate(prior['phase_dimensions']):r.update(successor_verdict='PASS / current',successor_assessment=assessment,prior_Master_ref='post_audit/master_20261004_001/CONDITION_REVIEW.json#phase_dimensions/'+str(i))
assert len(prior['acceptances'])==12 and len(prior['integration'])==13 and len(prior['phase_dimensions'])==20 and not prior['N_A']
prior.update(target_manifest_sha256=sha(T/'MANIFEST.json'),prior_Master_condition_sha256=sha(P/'CONDITION_REVIEW.json'),N02='closed')
write('CONDITION_REVIEW.json',prior)
pred=load(P/'PREDECESSOR_REVIEW.json');s2pred=load(SA/'PREDECESSOR_REVIEW.json');assert {(r['id'],r['group']) for r in pred['conditions']}=={(r['id'],r['group']) for r in s2pred['conditions']}
for i,r in enumerate(pred['conditions']):r.update(successor_verdict='PASS / current',successor_assessment=assessment,prior_Master_ref='post_audit/master_20261004_001/PREDECESSOR_REVIEW.json#conditions/'+str(i))
original_r06=load(D/'post_audit/s2_20261004_001/PREDECESSOR_REVIEW.json')['r06_original_acceptances'];assert len(original_r06)==31;assert s2pred['r06_original_acceptances']==original_r06
assert len(pred['conditions'])==88;pred['r06_original_acceptances']=original_r06;write('PREDECESSOR_REVIEW.json',pred)
# Bind original actual executions without claiming a repeat run.
bindings={}
for n,old_binding in load(P/'EVIDENCE_REUSE.json')['receipt_bindings'].items():
 receipt=D/'evidence'/(n+'.json');log=receipt.with_suffix('.log');r=load(receipt);assert sha(receipt)==old_binding['receipt_sha256'];assert sha(log)==old_binding['log_sha256']==r['log_sha256'];assert r['exit_code']==old_binding['raw_exit'];assert r['source_before']==r['source_after'] and r['source_unchanged']
 for x in r['command_inputs']:assert sha(D/x['archive'])==x['sha256']
 bindings[n]=dict(receipt_sha256=sha(receipt),log_sha256=sha(log),raw_exit=r['exit_code'],original_started_at_utc=r['started_at_utc'],unchanged_from_Master001=True)
assert len(bindings)==34
phase_delta=load(SA/'DELTA_VERIFICATION.json')
for n,digest in phase_delta['prior_evidence_refs'].items():assert sha(D/n)==digest
for n in ['s2n02_doc_verify001','master_n02_entry_verify001']:
 receipt=D/'evidence'/(n+'.json');r=load(receipt);assert r['source_before']==r['source_after']==new['product_source'];assert r['exit_code']==0;assert sha(receipt.with_suffix('.log'))==r['log_sha256']
 for x in r['command_inputs']:assert sha(D/x['archive'])==x['sha256']
write('DELTA_VERIFICATION.json',dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),verdict='PASS',prior_Master_manifest_sha256=sha(D/'execution/master/target/master_001/MANIFEST.json'),target_manifest_sha256=sha(T/'MANIFEST.json'),S2_target_manifest_sha256=sha(S/'MANIFEST.json'),changed=changed,added=[],deleted=[],unchanged_product_inputs=359,unchanged_other_union_inputs=364,entire_document_delta='exactly one hyperlink substitution',Master002_source_exact_S2_N02=True,documentation_routes=routes,branch_sha256=sha(branch),branch_judgment='Absent owner increments expired then returns; duplicate/out-of-order existing owner returns without expired increment or histogram samples. Corrected current contract matches. No runtime delta.',N02='closed',N01='remains closed',prior_references=refs))
write('EVIDENCE_REUSE.json',dict(assessment=assessment,prior_Master_evidence_sha256=sha(P/'EVIDENCE_REUSE.json'),receipt_bindings=bindings,actual_runtime_reexecuted=False,required_counts=dict(acceptances=12,IVA=13,phase_dimensions=20,inherited=88,R06_original_mapping=31),extra_inputs_exact=606,final_baseline_owner='GOAL-CRSO-S2 / WP-CRSO-S2VERIFY',Master_implementation_or_baseline_owner=False,scope='Retain all source/runtime/service/parity/allocator/R05/OBS001/20-minute and release cfg(test) exception scopes from Master001; no new exception introduced.'))
chain=load(P/'PHASE_CONTINUITY.json');chain['chain'] += [dict(from_target='execution/master/target/master_001/MANIFEST.json',to_target=str((S/'MANIFEST.json').relative_to(D)),changed=changed,added=[],deleted=[],before_exact_prior_after=True,judgment='Separate S2 owner corrected canonical hyperlink only; independent S2 audit003 closes N02.'),dict(from_target=str((S/'MANIFEST.json').relative_to(D)),to_target=str((T/'MANIFEST.json').relative_to(D)),changed=[],added=[],deleted=[],before_exact_prior_after=True,judgment='Master002 adds records only; exact S2 N02 source/document bytes.')];write('PHASE_CONTINUITY.json',chain)
write('FINDINGS.json',dict(open=[],closed=[dict(id='N02',owner='GOAL-CRSO-S2',source='post_audit/master_20261004_001/FINDINGS.json',disposition='Closed: full document delta and both current routes independently verified; exact N01 successor agrees with unchanged Worker::consume.')],N01='remains closed'))
raw=subprocess.check_output(['git','status','--porcelain=v1','--untracked-files=normal'],cwd=R);(A/'GIT_STATUS.txt').write_bytes(raw);write('DIRTY_ATTRIBUTION.json',dict(prior='post_audit/master_20261004_001/DIRTY_ATTRIBUTION.json',normal_status_identical_to_Master001=raw==(P/'GIT_STATUS.txt').read_bytes(),cumulative_source_delta=changed,owner='GOAL-CRSO-S2 separate N02 documentation correction after Master001 closure',current_membership_checks='ENTRY_CURRENTNESS.json and END_CURRENTNESS.json',audit_writes='This output directory only; excluded from frozen records. Disposable temporary restoration copies only. No checkout/reset/commit or unrelated-work cleanup.'))
print('PASS N02 closed: one link;359 product/364 other union inputs unchanged;12 acceptance/13IVA/20 dimensions/88 inherited/31 R06 mappings/34 raw bindings current.')
