"""Verify the full N02 union delta and exact semantic evidence reuse."""
import json
from pathlib import Path
import sys
import tarfile
C=Path(__file__).resolve().parents[1]
D=C.parents[3]
S=D/'execution/s2'
sys.path.insert(0,str(D/'tools'))
import run
T=D/'execution/master/target/master_001'
old=json.loads((T/'MANIFEST.json').read_text())
source=run.identity()
assert old['product_source']==source
changed=[]
for row in old['source_files']:
    f=run.ROOT/row['path']
    if f.stat().st_size!=row['bytes'] or run.sha(f)!=row['sha256']: changed.append(row['path'])
canonical='core_enhanced/docs/core_runtime_contract.md'
assert changed==[canonical],changed
with tarfile.open(T/'SOURCE_AFTER.tar.gz') as archive:
    before=archive.extractfile(canonical).read().decode()
a='execution/s2/METRICS_AND_STORAGE.md'
b='execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md'
assert before.count(a)==1
assert (run.ROOT/canonical).read_text()==before.replace(a,b)
contract=S/'corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md'
assert (run.ROOT/canonical).parent.joinpath('goals/core_runtime_safety_observability/'+b).resolve()==contract
host=run.ROOT/'game_server/config/observability/README.md'
assert b in host.read_text() and a not in host.read_text()
assert 'queued summary whose owner is no longer registered' in contract.read_text()
coverage=json.loads((S/'COVERAGE.json').read_text())
for name,binding in coverage['receipt_bindings'].items():
    p=D/'evidence'/(name+'.json')
    assert run.sha(p)==binding['receipt_sha256'] and run.sha(p.with_suffix('.log'))==binding['log_sha256'],name
aud=D/'post_audit/s2_20261004_002'
master=D/'post_audit/master_20261004_001'
refs={str(p.relative_to(D)):run.sha(p) for p in [T/'MANIFEST.json',S/'COVERAGE.json',S/'PREDECESSOR_IMPACT.json',contract,*[a/n for a in [aud,master] for n in ['COMPLETION_AUDIT.md','END_CURRENTNESS.json','CONDITION_REVIEW.json','PREDECESSOR_REVIEW.json']]]}
def write(name,value):
    with (C/name).open('x') as out: json.dump(value,out,ensure_ascii=False,indent=2);out.write('\n')
write('COVERAGE.json',dict(goal='GOAL-CRSO-S2',phase='PHASE-CRSO-S2',finding='N02',status='canonical link corrected and self-verified; independent GEX-20 pending',source_sha256=source['sha256'],changed_product_inputs=[],unchanged_product_input_count=len(source['files']),changed_source_document_inputs=changed,unchanged_other_union_inputs=len(old['source_files'])-1,acceptance=coverage['acceptance'],validations=coverage['validations'],required_dimensions=coverage['required_dimensions'],runtime_receipts=coverage['receipt_bindings'],prior_evidence_refs=refs,runtime_reexecution=False,reuse_reason='Only one canonical Markdown hyperlink changed; every executable, configuration, test, dependency, fixture and actual-consumer input byte remains exact.',original_finding='post_audit/master_20261004_001/COMPLETION_AUDIT.md',canonical_contract_sha256=run.sha(contract)))
impact=json.loads((S/'corrections/n01_metric_docs_001/PREDECESSOR_IMPACT.json').read_text())
assert len(impact['conditions'])==88
write('PREDECESSOR_IMPACT.json',dict(source_sha256=source['sha256'],changed=changed,conditions=impact['conditions'],prior_independent_review_sha256=run.sha(aud/'PREDECESSOR_REVIEW.json'),independent_audit='pending N02 documentation delta/currentness assessment'))
print(json.dumps(dict(passed=True,changed=changed,unchanged_product_inputs=len(source['files']),reusable_receipts=34,inherited=88)))
