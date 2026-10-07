"""Check this roadmap proposal's preservation, links and proposed ordering only."""
import difflib
import hashlib
import json
import re
import subprocess
import tarfile
import tempfile
from pathlib import Path
from urllib.parse import unquote

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
EVIDENCE = HERE / 'evidence'


def meta(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data),
            'sha256': hashlib.sha256(data).hexdigest()}


def links(text):
    return re.findall(r'\]\(([^)]+)\)', text)


def main():
    entry = json.loads((EVIDENCE / 'ENTRY.json').read_text())
    for row in entry['protected_files']:
        assert meta(ROOT / row['path']) == row, row['path']
    assert meta(ROOT / entry['archive']['path']) == entry['archive']
    before = {}
    with tarfile.open(ROOT / entry['archive']['path']) as archive:
        assert set(archive.getnames()) == {r['path'] for r in entry['documents_before']}
        for row in entry['documents_before']:
            data = archive.extractfile(row['path']).read()
            assert len(data) == row['bytes']
            assert hashlib.sha256(data).hexdigest() == row['sha256']
            before[row['path']] = data.decode()

    forward, reverse = [], []
    after = {name: (ROOT / name).read_text() for name in before}
    for name, old in before.items():
        new = after[name]
        forward.extend(difflib.unified_diff(old.splitlines(True), new.splitlines(True),
                                           fromfile='a/' + name, tofile='b/' + name))
        reverse.extend(difflib.unified_diff(new.splitlines(True), old.splitlines(True),
                                           fromfile='a/' + name, tofile='b/' + name))
    (EVIDENCE / 'DOCUMENT_CHANGES.patch').write_text(''.join(forward))
    with tempfile.TemporaryDirectory(prefix='mod-safety-roadmap-') as tmp:
        temp = Path(tmp)
        for name, old in before.items():
            path = temp / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(old)
        for patch, expected in [(forward, after), (reverse, before)]:
            result = subprocess.run(['patch', '--batch', '--fuzz=0', '-p1'],
                                    input=''.join(patch), cwd=temp, text=True, capture_output=True)
            assert result.returncode == 0, result.stdout + result.stderr
            for name, content in expected.items():
                assert (temp / name).read_text() == content

    docs = sorted(HERE.glob('*.md')) + [ROOT / n for n in before if n.endswith('.md')]
    count = 0
    for doc in docs:
        old_links = set(links(before.get(str(doc.relative_to(ROOT)), '')))
        for link in links(doc.read_text()):
            if link.startswith(('https://', 'http://', '#')) or link in old_links:
                continue
            path = (doc.parent / unquote(link.split('#')[0])).resolve()
            if path == EVIDENCE / 'VALIDATION.json':
                continue
            assert path.exists(), (doc, link)
            count += 1

    source = ROOT / ('core_enhanced/docs/roadmaps/content_authoring/reviews/'
                     '2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/GOALS.json')
    old = json.loads(source.read_text())
    order = []
    for node in old['recommended_order']:
        order.append(node)
        if node == 'R06':
            order.extend(['MS01', 'MS02', 'MS03'])
        if node == 'R18':
            order.extend(['MS04', 'MS05'])
        if node == 'R21':
            order.append('MS06-A')
    assert len(order) == len(set(order))
    assert [n for n in order if n.startswith('R')] == old['recommended_order']
    positions = {n: i for i, n in enumerate(order)}
    inherited = [e for e in old['new_relationships'] if not e.get('conditional_revalidation')]
    refresh = [e for e in old['new_relationships'] if e.get('conditional_revalidation')]
    added = [
        ('R02', 'MS03', 'functional'), ('R03', 'MS03', 'functional'),
        ('R04', 'MS01', 'reuse'), ('R05', 'MS01', 'reuse'),
        ('R06', 'MS01', 'recommended'), ('MS01', 'MS02', 'recommended'),
        ('MS02', 'MS03', 'recommended'), ('R06', 'MS03', 'feature_verification'),
        ('R02', 'MS04', 'functional'), ('R04', 'MS04', 'reuse'),
        ('R02', 'MS05', 'functional'), ('MS01', 'MS05', 'functional'),
        ('MS03', 'MS05', 'verification'), ('MS04', 'MS05', 'integration'),
        ('MS05', 'R19', 'new_scope_integration'),
        ('MS04', 'R20', 'verification'), ('MS05', 'R20', 'verification'),
        ('R19', 'MS06-A', 'verification'), ('R20', 'MS06-A', 'verification'),
        ('R21', 'MS06-A', 'verification'), ('MS06-A', 'R22', 'verification'),
    ]
    for edge in inherited:
        assert positions[edge['from']] < positions[edge['to']], edge['id']
    for a, b, kind in added:
        assert positions[a] < positions[b], (a, b, kind)

    report = {
        'scope': 'roadmap_document_validation_only',
        'protected_files_unchanged': len(entry['protected_files']),
        'documents_before_preserved': len(before),
        'forward_and_reverse_patch': 'PASS',
        'new_local_link_occurrences_checked': count,
        'original_r_relative_order': 'PASS',
        'inherited_forward_relationships_checked': len(inherited),
        'preserved_conditional_revalidation': [e['id'] for e in refresh],
        'added_relationships_checked': len(added),
        'added_relationships': [{'from': a, 'to': b, 'kind': k} for a, b, k in added],
        'proposed_default_order': order,
        'order_check_limits': 'Default placement only. Feature-profile deltas, R09 deferral and earlier mod admission are conditional design obligations in the review, not execution readiness certified by this check.',
        'ms06_b_evidence_owners': ['R22', 'R23', 'R24'],
        'documents': [meta(doc) for doc in docs],
        'new_product_test_or_performance_runs': 0,
        'product_implementation': False,
        'formal_plan_changed': False,
        'independent_audit': False,
    }
    (EVIDENCE / 'VALIDATION.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in
                     ('documents', 'added_relationships', 'proposed_default_order')}, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
