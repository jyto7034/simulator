"""Validate planning artifacts, preserved inputs and reversible document edits."""
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
ROOT = next(p for p in HERE.parents if (p / 'core_enhanced/Cargo.toml').is_file())
EVIDENCE = HERE / 'evidence'


def meta(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data),
            'sha256': hashlib.sha256(data).hexdigest()}


def links(text):
    return re.findall(r'\]\(([^)]+)\)', text)


def main():
    entry = json.loads((EVIDENCE / 'ENTRY.json').read_text())
    for row in entry['product_and_guard_files']:
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
    for name, old in before.items():
        new = (ROOT / name).read_text()
        forward.extend(difflib.unified_diff(old.splitlines(True), new.splitlines(True),
                                           fromfile='a/' + name, tofile='b/' + name))
        reverse.extend(difflib.unified_diff(new.splitlines(True), old.splitlines(True),
                                           fromfile='a/' + name, tofile='b/' + name))
    (EVIDENCE / 'DOCUMENT_CHANGES.patch').write_text(''.join(forward))
    with tempfile.TemporaryDirectory(prefix='safety-plan-doc-check-') as tmp:
        temp = Path(tmp)
        for name, old in before.items():
            path = temp / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(old)
        for patch, expected in [(forward, {n: (ROOT / n).read_text() for n in before}),
                                (reverse, before)]:
            result = subprocess.run(['patch', '--batch', '--fuzz=0', '-p1'],
                                    input=''.join(patch), cwd=temp, text=True, capture_output=True)
            assert result.returncode == 0, result.stdout + result.stderr
            for name, text in expected.items():
                assert (temp / name).read_text() == text

    handoffs = ROOT / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs'
    docs = sorted(HERE.glob('*.md')) + [handoffs / 'R06_COST_POLICY_UPDATE_20260927.ko.md']
    docs += [ROOT / name for name in before]
    count = 0
    for doc in docs:
        old_links = set(links(before.get(str(doc.relative_to(ROOT)), '')))
        for link in links(doc.read_text()):
            if link.startswith(('https://', 'http://', '#')) or link in old_links:
                continue
            path = (doc.parent / unquote(link.split('#')[0])).resolve()
            if path == EVIDENCE / 'VALIDATION.json':
                continue  # this invocation generates the validation receipt
            assert path.exists(), (str(doc), link)
            count += 1
    report = {
        'scope': 'planning_document_validation_only',
        'product_and_guard_files_unchanged': len(entry['product_and_guard_files']),
        'original_documents_preserved': len(before),
        'forward_and_reverse_document_patch': 'PASS',
        'new_local_link_occurrences_checked': count,
        'frozen_target_currentness': json.loads((EVIDENCE / 'CURRENTNESS.log').read_text()),
        'documents': [meta(doc) for doc in docs],
        'new_product_tests_or_performance_runs': 0,
        'product_implementation': False,
        'independent_design_audit': False,
    }
    (EVIDENCE / 'VALIDATION.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ('documents', 'frozen_target_currentness')},
                     ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
