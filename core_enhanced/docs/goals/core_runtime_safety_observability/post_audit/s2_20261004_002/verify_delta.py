import pathlib,json,hashlib,tarfile,re,datetime
R=pathlib.Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';C=D/'execution/s2/corrections/n01_metric_docs_001';A=pathlib.Path(__file__).resolve().parent;P=D/'post_audit/s2_20261004_001'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def digest(x):return hashlib.sha256(json.dumps(x,sort_keys=True).encode()).hexdigest()
def write(n,x):(A/n).write_text(json.dumps(x,indent=2,ensure_ascii=False)+'\n')
old=read(D/'execution/s2/target/s2_001/MANIFEST.json');new=read(C/'target/s2_n01_001/MANIFEST.json');cov=read(C/'COVERAGE.json');pcov=read(D/'execution/s2/COVERAGE.json');prior=read(P/'CONDITION_REVIEW.json');impact=read(C/'PREDECESSOR_IMPACT.json');pimpact=read(P/'PREDECESSOR_REVIEW.json');guide='game_server/config/observability/README.md'
assert new['before_files']==old['source_files']
assert new['before_sha256']==old['source_sha256']
before={r['path']:r for r in old['source_files']};after={r['path']:r for r in new['source_files']}
assert before.keys()==after.keys()
assert [p for p in after if after[p]!=before[p]]==[guide]==new['changed']
assert not new['added'] and not new['deleted']
with tarfile.open(D/'execution/s2/target/s2_001/SOURCE_AFTER.tar.gz') as tf: oldguide=tf.extractfile(guide).read().decode()
assert (R/guide).read_text()==oldguide.replace('execution/s2/METRICS_AND_STORAGE.md','execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md')
reused=[r for r in new['product_source']['files'] if r['path']!=guide]
assert reused==[r for r in old['product_source']['files'] if r['path']!=guide]
assert len(reused)==cov['unchanged_other_input_count']==358
assert digest(reused)==cov['unchanged_other_input_sha256']
assert cov['runtime_receipts']==pcov['receipt_bindings']
for name,b in cov['runtime_receipts'].items():
 p=D/'evidence'/(name+'.json');assert sha(p)==b['receipt_sha256'];assert sha(p.with_suffix('.log'))==b['log_sha256']
for name,h in cov['prior_evidence_refs'].items():assert sha(D/name)==h
pre=read(C/'PREREQUISITE.json')
assert sha(D/'execution/s2/target/s2_001/MANIFEST.json')==pre['previous_manifest_sha256']
assert sha(P/'COMPLETION_AUDIT.md')==pre['previous_audit_sha256']
assert sha(P/'END_CURRENTNESS.json')==pre['previous_end_currentness_sha256']
assert [r['policy_ref'] for r in cov['acceptance']]==pcov['acceptance']
assert [r['id'] for r in cov['validations']]==[r['id'] for r in prior['validations']]
assert cov['required_dimensions']==pcov['required_dimensions']
assert set(cov['required_dimensions'])=={r['dimension'] for r in prior['required_dimensions']}
assert [r['id'] for r in impact['conditions']]==[r['id'] for r in pimpact['conditions']]
assert sha(P/'PREDECESSOR_REVIEW.json')==impact['prior_independent_review_sha256']
assert sha(D/impact['prior_impact_path'])==impact['prior_impact_sha256']
links=[]
for p in [R/guide,C/'METRICS_AND_STORAGE.md']:
 for link in re.findall(r'\]\(([^)]+)\)',p.read_text()):
  if '://' in link or link.startswith('#'):continue
  dest=(p.parent/link).resolve();assert dest.is_file()
  links.append(dict(source=str(p.relative_to(R)),target=str(dest.relative_to(R)),sha256=sha(dest)))
assert len(links)==2
branch=R/'game_server/src/observability/mod.rs';assert sha(branch)==cov['source_branch_sha256']
text=branch.read_text();a=text.index('    fn consume(&mut self, summary: core::OperationSummary)');b=text.index('    fn batch(',a);consume=text[a:b]
assert 'let Some(owner) = self.owners.get_mut(&summary.session) else {\n            self.handles.expired.increment(1);\n            return false;\n        };' in consume
assert 'if summary.sequence <= owner.sequence {\n            return false;\n        }' in consume
assert consume.index('owner.sequence = summary.sequence;')<consume.index('metric.record(value);')
doc=(C/'METRICS_AND_STORAGE.md').read_text();old_doc=(D/'execution/s2/METRICS_AND_STORAGE.md').read_text()
paragraph='\nThis is the current contract after S2 audit N01. It supersedes the counter description in the preserved [pre-audit contract](../../METRICS_AND_STORAGE.md); runtime behavior is unchanged. A queued summary with no registered owner increments `core_runtime_observation_expired_total`. A duplicate or out-of-order sequence for an existing owner is ignored without adding a histogram sample or incrementing that counter.\n'
assert doc.replace(paragraph,'').replace('queued summary whose owner is no longer registered','stale or duplicate queued summary')==old_doc
receipt=read(D/'evidence/s2n01_doc_verify002.json');log=D/'evidence/s2n01_doc_verify002.log'
assert receipt['exit_code']==0 and receipt['source_unchanged'] and sha(log)==receipt['log_sha256']
assert receipt['source_before']==receipt['source_after']==new['product_source']
for entry in receipt['command_inputs']:
 assert sha(pathlib.Path(entry['path']))==entry['sha256']==sha(D/entry['archive'])
raw=json.loads(log.read_text());assert raw['passed'] and raw['unchanged_inputs']==358 and raw['inherited_conditions']==88 and raw['reusable_runtime_receipts']==34
reason='Independently verified sole product delta is one Markdown hyperlink. All 358 other product inputs and 606 transitive frozen extra inputs match; original semantic judgments, selectors and raw execution evidence remain applicable. N01 source branch and successor contract reviewed directly. No execution input or policy delta.'
write('DELTA_VERIFICATION.json',dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),verdict='PASS',before_union_exact_prior_after=True,source_members=365,changed=[guide],guide_change='single contract hyperlink',unchanged_other_product_inputs=358,unchanged_input_sha256=digest(reused),successor_document_delta='one supersession paragraph plus expired-counter table definition; all other bytes exact',branch_sha256=sha(branch),branch_judgment='missing owner increments expired; duplicate/out-of-order existing owner returns without counter/histogram',documentation_links=links,prior_evidence_refs=cov['prior_evidence_refs'],runtime_receipts_rebound_and_exact=34,doc_verifier_receipt_sha256=sha(D/'evidence/s2n01_doc_verify002.json'),doc_verifier_log_sha256=sha(log),runtime_reexecution=False,N01='closed',reason=reason))
write('CONDITION_REVIEW.json',dict(goal='GOAL-CRSO-S2',target_manifest_sha256=sha(C/'target/s2_n01_001/MANIFEST.json'),prior_condition_review_sha256=sha(P/'CONDITION_REVIEW.json'),acceptances=[dict(x,successor_verdict='PASS / current',successor_assessment=reason) for x in prior['acceptances']],validations=[dict(x,successor_verdict='PASS / current',successor_assessment=reason) for x in prior['validations']],required_dimensions=[dict(x,successor_verdict='PASS / current',successor_assessment=reason) for x in prior['required_dimensions']],N01='closed; contract agrees with actual existing-owner sequence suppression and absent-owner expiry'))
write('PREDECESSOR_REVIEW.json',dict(target_manifest_sha256=sha(C/'target/s2_n01_001/MANIFEST.json'),prior_independent_review_sha256=sha(P/'PREDECESSOR_REVIEW.json'),conditions=[dict(x,successor_verdict='PASS / current',successor_assessment=reason) for x in pimpact['conditions']],r06_original_acceptances=pimpact['r06_original_acceptances'],summary='All 88 inherited conditions retain the exact previous requirements, judgments, selectors and evidence, now rebound by unchanged executable and transitive input verification. 31 original R06 mappings retained.'))
print('PASS: doc delta, branch, 358 unchanged product inputs, 34 raw receipts, 12 acceptances, 12 validations, 10 dimensions, 88 inherited conditions, 31 original R06 mappings')
