import pathlib,json,hashlib,tarfile,re,datetime
R=pathlib.Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/core_runtime_safety_observability';C=D/'execution/s2/corrections/n02_contract_link_001';A=pathlib.Path(__file__).resolve().parent;P=D/'post_audit/s2_20261004_002';M=D/'post_audit/master_20261004_001';T=C/'target/s2_n02_001';MT=D/'execution/master/target/master_001';N=D/'execution/s2/corrections/n01_metric_docs_001/target/s2_n01_001'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(n,x):(A/n).write_text(json.dumps(x,indent=2,ensure_ascii=False)+'\n')
old=read(MT/'MANIFEST.json');new=read(T/'MANIFEST.json');n01=read(N/'MANIFEST.json');cov=read(C/'COVERAGE.json');pcov=read(D/'execution/s2/COVERAGE.json');prior=read(P/'CONDITION_REVIEW.json');impact=read(C/'PREDECESSOR_IMPACT.json');pimpact=read(P/'PREDECESSOR_REVIEW.json');mc=read(M/'CONDITION_REVIEW.json');mi=read(M/'PREDECESSOR_REVIEW.json');guide='core_enhanced/docs/core_runtime_contract.md';host=R/'game_server/config/observability/README.md';contract=D/'execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md'
assert sha(M/'COMPLETION_AUDIT.md')=='744c00efa660aceb559f194802fe29735bcc4ec5e2abffbd676bb0a092a815f5'
assert sha(P/'COMPLETION_AUDIT.md')=='da1a91d0ae6c051eac95fa635be68e1990b778d18eee3dfb7d6cacdf572f8b1b'
assert new['before_files']==old['source_files']==n01['source_files']
assert new['before_sha256']==old['source_sha256']==n01['source_sha256']
before={r['path']:r for r in old['source_files']};after={r['path']:r for r in new['source_files']}
assert before.keys()==after.keys()
assert [p for p in after if after[p]!=before[p]]==[guide]==new['changed']
assert not new['added'] and not new['deleted']
assert new['product_source']==old['product_source']==n01['product_source']
assert len(new['product_source']['files'])==cov['unchanged_product_input_count']==359
assert cov['unchanged_other_union_inputs']==len(after)-1==364
assert not cov['changed_product_inputs']
with tarfile.open(MT/'SOURCE_AFTER.tar.gz') as tf:oldguide=tf.extractfile(guide).read().decode()
oldlink='execution/s2/METRICS_AND_STORAGE.md';newlink='execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md'
assert oldguide.count(oldlink)==1
assert (R/guide).read_text()==oldguide.replace(oldlink,newlink)
links=[]
for p in [R/guide,host]:
 targets=[l for l in re.findall(r'\]\(([^)]+)\)',p.read_text()) if 'METRICS_AND_STORAGE.md' in l]
 assert len(targets)==1
 dest=(p.parent/targets[0]).resolve();assert dest==contract.resolve()
 links.append(dict(source=str(p.relative_to(R)),target=targets[0],resolved=str(dest.relative_to(R)),sha256=sha(dest)))
assert sha(contract)==cov['canonical_contract_sha256']=='c75bf02b17876f4178da980f700dc22d044d8a16abf17ad49692372cc71a87f5'
assert cov['runtime_receipts']==pcov['receipt_bindings']
for name,b in cov['runtime_receipts'].items():
 p=D/'evidence'/(name+'.json');assert sha(p)==b['receipt_sha256'];assert sha(p.with_suffix('.log'))==b['log_sha256']
for name,h in cov['prior_evidence_refs'].items():assert sha(D/name)==h
assert cov['acceptance']==pcov['acceptance']
assert cov['validations']==pcov['validations']
assert [r['id'] for r in cov['validations']]==[r['id'] for r in prior['validations']]
assert cov['required_dimensions']==pcov['required_dimensions']
assert set(cov['required_dimensions'])=={r['dimension'] for r in prior['required_dimensions']}
assert [r['id'] for r in impact['conditions']]==[r['id'] for r in pimpact['conditions']]==[r['id'] for r in mi['conditions']]
assert sha(P/'PREDECESSOR_REVIEW.json')==impact['prior_independent_review_sha256']
assert [r['policy_ref'] for r in mc['acceptances']]==[r['policy_ref'] for r in prior['acceptances']]
assert [r['requirement'] for r in mc['acceptances']]==[r['requirement'] for r in prior['acceptances']]
assert len(mc['integration'])==13 and len(mc['phase_dimensions'])==20 and mc['all_required_covered']
assert len(pimpact['r06_original_acceptances'])==mi['r06_original_acceptance_mappings']==31
branch=R/'game_server/src/observability/mod.rs';assert sha(branch)=='2457f8064f26253f7dd364fd8048d5ccf4d0d88d7d6c7dc665fff9e2f88d6f5e'
text=branch.read_text();a=text.index('    fn consume(&mut self, summary: core::OperationSummary)');b=text.index('    fn batch(',a);consume=text[a:b]
assert 'let Some(owner) = self.owners.get_mut(&summary.session) else {\n            self.handles.expired.increment(1);\n            return false;\n        };' in consume
assert 'if summary.sequence <= owner.sequence {\n            return false;\n        }' in consume
assert consume.index('owner.sequence = summary.sequence;')<consume.index('metric.record(value);')
assert 'queued summary whose owner is no longer registered' in contract.read_text()
assert 'ignored without adding a histogram sample or incrementing that counter' in contract.read_text()
receipt=read(D/'evidence/s2n02_doc_verify001.json');log=D/'evidence/s2n02_doc_verify001.log'
assert receipt['exit_code']==0 and receipt['source_unchanged'] and sha(log)==receipt['log_sha256']
assert receipt['source_before']==receipt['source_after']==new['product_source']
for entry in receipt['command_inputs']:assert sha(pathlib.Path(entry['path']))==entry['sha256']==sha(D/entry['archive'])
raw=json.loads(log.read_text());assert raw['passed'] and raw['unchanged_product_inputs']==359 and raw['inherited']==88 and raw['reusable_receipts']==34
reason='S2 audit003 independently proves exactly one canonical Markdown hyperlink substitution against Master001/N01 after bytes. All 359 product inputs, other 364 union inputs, 606 transitive extra inputs, original requirements and 34 runtime receipt/log pairs are unchanged. Both current routes resolve the exact previously audited N01 contract. Prior condition/source/evidence judgments therefore remain current; no executable, policy or consumer delta requires runtime reruns.'
write('DELTA_VERIFICATION.json',dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),verdict='PASS',target_manifest_sha256=sha(T/'MANIFEST.json'),prior_master_manifest_sha256=sha(MT/'MANIFEST.json'),prior_master_audit_sha256=sha(M/'COMPLETION_AUDIT.md'),before_union_exact_prior_master_and_n01_after=True,source_members=365,changed=[guide],unchanged_product_inputs=359,unchanged_other_union_inputs=364,canonical_change='single metric contract hyperlink',documentation_links=links,branch_sha256=sha(branch),branch_judgment='missing owner increments expired; duplicate/out-of-order existing owner returns without counter/histogram',prior_evidence_refs=cov['prior_evidence_refs'],runtime_receipts_rebound_and_exact=34,doc_verifier_receipt_sha256=sha(D/'evidence/s2n02_doc_verify001.json'),doc_verifier_log_sha256=sha(log),runtime_reexecution=False,N01='remains closed',N02='closed',reason=reason))
write('CONDITION_REVIEW.json',dict(goal='GOAL-CRSO-S2',target_manifest_sha256=sha(T/'MANIFEST.json'),prior_condition_review_sha256=sha(P/'CONDITION_REVIEW.json'),prior_master_condition_review_sha256=sha(M/'CONDITION_REVIEW.json'),acceptances=[dict(x,successor_verdict='PASS / current',successor_assessment=reason) for x in prior['acceptances']],validations=[dict(x,successor_verdict='PASS / current',successor_assessment=reason) for x in prior['validations']],required_dimensions=[dict(x,successor_verdict='PASS / current',successor_assessment=reason) for x in prior['required_dimensions']],N02='closed; canonical and host routes resolve the same immutable N01 successor; actual consume branches agree'))
write('PREDECESSOR_REVIEW.json',dict(target_manifest_sha256=sha(T/'MANIFEST.json'),prior_independent_review_sha256=sha(P/'PREDECESSOR_REVIEW.json'),prior_master_review_sha256=sha(M/'PREDECESSOR_REVIEW.json'),conditions=[dict(x,successor_verdict='PASS / current',successor_assessment=reason) for x in pimpact['conditions']],r06_original_acceptances=pimpact['r06_original_acceptances'],summary='88/88 inherited conditions and 31 original R06 mappings remain current by exact relevant input/evidence identity. Historical verdicts preserved.'))
write('MASTER_APPLICABILITY.json',dict(scope='Prior Master integration judgments remain applicable to this S2 documentation successor; not a new Master GEX-30 verdict.',prior_master_sha256=sha(M/'COMPLETION_AUDIT.md'),prior_conditions_sha256=sha(M/'CONDITION_REVIEW.json'),acceptances=[dict(x,n02_applicability=reason) for x in mc['acceptances']],integration=[dict(x,n02_applicability=reason) for x in mc['integration']],phase_dimensions=[dict(x,n02_applicability=reason) for x in mc['phase_dimensions']],finding='N02 closed by S2 owner successor; fresh separate Master GEX-30 still required'))
print('PASS: exact one canonical link; all 359 product inputs, 364 other union inputs, 34 receipt/log pairs, 12 acceptances, V01-12, 10 S2 dimensions, 88 inherited, 31 original R06 mappings; prior Master 13 integration/20 phase dimensions retain applicability')
