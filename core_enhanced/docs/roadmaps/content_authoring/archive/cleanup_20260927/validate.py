"""Verify roadmap cleanup without changing product or historical evidence."""
import difflib
import hashlib
import json
import os
import re
import subprocess
import tarfile
import tempfile
from pathlib import Path
from urllib.parse import unquote

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / 'core_enhanced/Cargo.toml').is_file())
ROAD = ROOT / 'core_enhanced/docs/roadmaps/content_authoring'


def meta(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data),
            'sha256': hashlib.sha256(data).hexdigest()}


def links(text):
    return re.findall(r'\]\(([^)]+)\)', text)


def destination(doc, link):
    path, _, anchor = link.partition('#')
    return Path(os.path.normpath(str(doc.parent / unquote(path)))) if path else doc, unquote(anchor)


def anchors(text):
    result = set(re.findall(r'<a\s+(?:id|name)=["\']([^"\']+)', text))
    seen = {}
    for title in re.findall(r'^#{1,6}\s+(.+?)\s*#*$', text, re.M):
        slug = re.sub(r'[^\w\s-]', '', title.lower()).replace(' ', '-')
        index = seen.get(slug, 0)
        seen[slug] = index + 1
        result.add(slug if not index else f'{slug}-{index}')
    return result


def main():
    entry = json.loads((HERE / 'ENTRY.json').read_text())
    assert meta(ROOT / entry['archive']['path']) == entry['archive']
    for row in entry['protected_files']:
        assert meta(ROOT / row['path']) == row, row['path']
    before = {}
    with tarfile.open(ROOT / entry['archive']['path']) as archive:
        assert set(archive.getnames()) == {r['path'] for r in entry['documents_before']}
        for row in entry['documents_before']:
            data = archive.extractfile(row['path']).read()
            assert len(data) == row['bytes']
            assert hashlib.sha256(data).hexdigest() == row['sha256']
            before[row['path']] = data.decode()
    removed = set(entry['archived_from_active_tree'])
    for name in removed:
        assert name in before and not (ROOT / name).exists(), name

    after = {n: (ROOT / n).read_text() for n in before if n not in removed}
    new = [ROAD / 'history/2026-09-04.md', HERE / 'README.ko.md', HERE / 'validate.py']
    for path in new:
        after[str(path.relative_to(ROOT))] = path.read_text()
    forward, reverse = [], []
    for name in sorted(set(before) | set(after)):
        old, current = before.get(name, ''), after.get(name, '')
        forward.extend(difflib.unified_diff(old.splitlines(True), current.splitlines(True),
                       fromfile='a/' + name if name in before else '/dev/null',
                       tofile='b/' + name if name in after else '/dev/null'))
        reverse.extend(difflib.unified_diff(current.splitlines(True), old.splitlines(True),
                       fromfile='a/' + name if name in after else '/dev/null',
                       tofile='b/' + name if name in before else '/dev/null'))
    (HERE / 'DOCUMENT_CHANGES.patch').write_text(''.join(forward))
    with tempfile.TemporaryDirectory(prefix='roadmap-cleanup-') as tmp:
        temp = Path(tmp)
        for name, data in before.items():
            path = temp / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(data)
        for patch, expected in [(forward, after), (reverse, before)]:
            proc = subprocess.run(['patch', '--batch', '--fuzz=0', '-p1'], input=''.join(patch),
                                  cwd=temp, text=True, capture_output=True)
            assert proc.returncode == 0, proc.stdout + proc.stderr
            for name in set(before) | set(after):
                path = temp / name
                assert path.exists() == (name in expected), name
                if name in expected:
                    assert path.read_text() == expected[name], name

    docs = [ROOT / n for n in after if n.endswith('.md')]
    checked, fragments = 0, 0
    for doc in docs:
        old_links = set(links(before.get(str(doc.relative_to(ROOT)), '')))
        for link in links(doc.read_text()):
            if link.startswith(('https:', 'http:')):
                continue
            path, anchor = destination(doc, link)
            if path == HERE / 'VALIDATION.json':
                continue
            # Existing unmodified links remain the historical check's responsibility.
            if link in old_links:
                continue
            assert path.exists(), (str(doc.relative_to(ROOT)), link)
            checked += 1
            if anchor and path.suffix == '.md':
                assert anchor in anchors(path.read_text()), (str(doc.relative_to(ROOT)), link)
                fragments += 1

    status = ROAD / 'ROADMAP_STATUS.md'
    original = before[str(status.relative_to(ROOT))].split('## 6. 최근 기록', 1)[1].strip()
    def rebase(match):
        link = match.group(1)
        if link.startswith(('#', 'http:', 'https:')):
            return match.group(0)
        path, sep, anchor = link.partition('#')
        changed = os.path.relpath(os.path.normpath(str(ROAD / path)), ROAD / 'history')
        return '](' + changed + (sep + anchor if sep else '') + ')'
    expected_history = re.sub(r'\]\(([^)]+)\)', rebase, original)
    assert (ROAD / 'history/2026-09-04.md').read_text().endswith(expected_history + '\n')
    old_count = len(re.findall(r'^### ', original, re.M))
    s = status.read_text()
    assert '다음 개발 작업 | **잔여 D04' in s
    assert '| 개발 주작업 | R06' in s
    assert '현재 대상 | 사용자가 선택한 별도 로드맵' not in s
    source = ROAD / 'reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/GOALS.json'
    units = json.loads(source.read_text())['units']
    rows = [line for line in s.splitlines() if re.match(r'^\| R\d\d \|', line)]
    assert [line.split('|')[1].strip() for line in rows] == [u['id'] for u in units]
    for row, unit in zip(rows, units):
        cols = [col.strip() for col in row.split('|')]
        assert cols[4] == (', '.join(unit['implementation_dependencies']) or '—')
        assert cols[5] == (', '.join(unit['verification_dependencies']) or '—')

    report = {
        'scope': 'roadmap_cleanup_document_validation_only',
        'protected_files_unchanged': len(entry['protected_files']),
        'original_files_preserved_in_archive': len(before),
        'archived_files_removed_from_active_tree': len(removed),
        'history_entries_preserved_with_equivalent_links': old_count,
        'forward_and_reverse_patch': 'PASS',
        'new_or_relocated_local_links_checked': checked,
        'new_or_relocated_heading_links_checked': fragments,
        'r_order_and_dependency_rows_preserved': len(rows),
        'development_task': 'R06 / WP02 / remaining D04 design and implementation request',
        'status_lines_before': len(before[str(status.relative_to(ROOT))].splitlines()),
        'status_lines_after': len(s.splitlines()),
        'operating_rules_lines_after': len((ROAD / 'OPERATING_RULES.md').read_text().splitlines()),
        'documents': [meta(path) for path in docs],
        'product_build_test_or_performance_runs': 0,
        'frozen_target_currentness_rerun': False,
        'formal_goal_or_dependency_change': False,
    }
    (HERE / 'VALIDATION.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'documents'}, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
