"""Verify the precise documentation delta and relevant-input baseline reuse."""
import hashlib
import json
from pathlib import Path
import re
import sys

C = Path(__file__).resolve().parents[1]
D = C.parents[3]
S = D / 'execution/s2'
sys.path.insert(0, str(D / 'tools'))
import run

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def write(name, value):
    with (C / name).open('x') as output:
        json.dump(value, output, indent=2)
        output.write('\n')

old = json.loads((S / 'target/s2_001/MANIFEST.json').read_text())
source = run.identity()
previous = {r['path']: r for r in old['product_source']['files']}
current = {r['path']: r for r in source['files']}
assert previous.keys() == current.keys()
changed = [p for p in current if current[p] != previous[p]]
guide = 'game_server/config/observability/README.md'
assert changed == [guide], changed
import tarfile
with tarfile.open(S / 'target/s2_001/SOURCE_AFTER.tar.gz') as archive:
    before = archive.extractfile(guide).read().decode()
old_link = 'execution/s2/METRICS_AND_STORAGE.md'
new_link = 'execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md'
assert before.count(old_link) == 1
after = (run.ROOT / guide).read_text()
assert after == before.replace(old_link, new_link)
links = []
for p in [run.ROOT / guide, C / 'METRICS_AND_STORAGE.md']:
    for target in re.findall(r'\]\(([^)]+)\)', p.read_text()):
        if '://' in target or target.startswith('#'):
            continue
        resolved = (p.parent / target.split('#')[0]).resolve()
        assert resolved.is_file(), (p, target)
        links.append({'from': str(p.relative_to(run.ROOT)), 'target': target,
                      'resolved': str(resolved.relative_to(run.ROOT)), 'sha256': sha(resolved)})
text = (C / 'METRICS_AND_STORAGE.md').read_text()
assert 'stale or duplicate queued summary' not in text
assert 'queued summary whose owner is no longer registered' in text
assert 'ignored without adding a histogram sample or incrementing that counter' in text
coverage = json.loads((S / 'COVERAGE.json').read_text())
aud = D / 'post_audit/s2_20261004_001'
assert 'Verdict: Complete / High' in (aud / 'COMPLETION_AUDIT.md').read_text()
for name, binding in coverage['receipt_bindings'].items():
    p = D / 'evidence' / (name + '.json')
    assert sha(p) == binding['receipt_sha256']
    assert sha(p.with_suffix('.log')) == binding['log_sha256']
reused = [r for r in source['files'] if r['path'] != guide]
assert reused == [r for r in old['product_source']['files'] if r['path'] != guide]
baseline_digest = hashlib.sha256(json.dumps(reused, sort_keys=True).encode()).hexdigest()
refs = {str(p.relative_to(D)): sha(p) for p in [
    S / 'COVERAGE.json', S / 'PREDECESSOR_IMPACT.json',
    S / 'target/s2_001/MANIFEST.json',
    *[aud / n for n in ('COMPLETION_AUDIT.md', 'CONDITION_REVIEW.json',
                        'PREDECESSOR_REVIEW.json', 'SOURCE_REVIEW.json',
                        'END_CURRENTNESS.json', 'EXTRA_INPUTS.json')]]}
write('COVERAGE.json', {
    'goal': 'GOAL-CRSO-S2', 'phase': 'PHASE-CRSO-S2', 'finding': 'N01',
    'status': 'documentation corrected and self-verified; independent GEX-20 pending',
    'source_sha256': source['sha256'], 'previous_source_sha256': old['product_source']['sha256'],
    'changed_product_inputs': changed, 'unchanged_other_input_count': len(reused),
    'unchanged_other_input_sha256': baseline_digest,
    'runtime_reexecution': False,
    'reuse_reason': 'The sole product-input delta is one Markdown hyperlink. Every executable, test, fixture, config, Cargo and other input byte is exact. Previous release-only cfg(test) delta remains separately justified by the prior independent audit.',
    'acceptance': [{'policy_ref': a, 'disposition': 'exact prior semantic and execution evidence reused; no runtime or policy delta'} for a in coverage['acceptance']],
    'validations': [{'id': v['id'], 'disposition': 'exact prior source/selector/input/raw evidence reused; documentation-only delta'} for v in coverage['validations']],
    'required_dimensions': coverage['required_dimensions'],
    'runtime_receipts': coverage['receipt_bindings'], 'prior_evidence_refs': refs,
    'documentation_links': links, 'original_finding': 'post_audit/s2_20261004_001/COMPLETION_AUDIT.md',
    'actual_branch': 'game_server/src/observability/mod.rs Worker::consume: absent owner increments expired; duplicate/out-of-order returns without histogram or counter update',
    'source_branch_sha256': sha(run.ROOT / 'game_server/src/observability/mod.rs'),
})
impact = json.loads((S / 'PREDECESSOR_IMPACT.json').read_text())
assert len(impact['inherited_conditions']) == 88
write('PREDECESSOR_IMPACT.json', {
    'source_sha256': source['sha256'], 'changed': changed,
    'prior_impact_path': str((S / 'PREDECESSOR_IMPACT.json').relative_to(D)),
    'prior_impact_sha256': sha(S / 'PREDECESSOR_IMPACT.json'),
    'prior_independent_review_sha256': sha(aud / 'PREDECESSOR_REVIEW.json'),
    'conditions': [{'id': row['id'], 'disposition': 'current via exact prior executable-input and semantic-review reuse'} for row in impact['inherited_conditions']],
    'independent_audit': 'pending documentation delta/currentness assessment',
})
print(json.dumps({'passed': True, 'changed': changed, 'unchanged_inputs': len(reused),
                  'reusable_runtime_receipts': len(coverage['receipt_bindings']),
                  'inherited_conditions': 88, 'source_sha256': source['sha256']}, indent=2))
