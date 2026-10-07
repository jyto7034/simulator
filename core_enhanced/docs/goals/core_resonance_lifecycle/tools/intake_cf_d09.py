"""Rebind an externally migrated document tree without editing frozen records."""
import datetime
import difflib
import json
import re
import tarfile
from pathlib import Path
import run

R, H = run.ROOT, run.HERE
S = H / 'execution/single_goal'
C = S / 'corrections/cf_d09_001'
D = R / 'core_enhanced/docs/roadmaps/core_first/execution/rulebook_path_migration_20261004_001'
A = H / 'post_audit/u05_20261004_001'
T = S / 'target/u05_001'
sha = run.sha


def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


assert sha(T / 'MANIFEST.json') == '66d7053f2d2c747d189cf2c31fdd4baea800c2900e4b3e8f741f07f4ad5cd99a'
assert sha(A / 'COMPLETION_AUDIT.md') == 'f89dd050d8343f911d4532ace8330a87d9035e66842a7129cc2f86e935ae994f'
assert 'Audit closed.' in (A / 'COMPLETION_AUDIT.md').read_text()
manifest = json.loads((T / 'MANIFEST.json').read_text())
product = run.identity()
assert product == manifest['product_source']
# Existing U05 execution/P2G records are preserved, not rewritten in place.
records = json.loads((T / 'RECORDS.json').read_text())
assert all(sha(R / row['path']) == row['sha256'] for row in records)
migration = json.loads((D / 'EVIDENCE.json').read_text())
assert sha(D / 'BEFORE.tar.gz') == migration['before_archive_sha256']
assert len(migration['documents']) == 46
assert all(sha(R / row['after_path']) == row['after_sha256'] for row in migration['documents'])
C.mkdir(parents=True, exist_ok=False)
diffs, body_checks, old_text = [], [], {}
with tarfile.open(D / 'BEFORE.tar.gz') as archive:
    for row in migration['documents']:
        data = archive.extractfile(row['before_path']).read()
        import hashlib
        assert hashlib.sha256(data).hexdigest() == row['before_sha256']
        old = data.decode()
        new = (R / row['after_path']).read_text()
        old_text[row['before_path']] = old
        diffs.extend(difflib.unified_diff(old.splitlines(keepends=True), new.splitlines(keepends=True),
            fromfile=row['before_path'], tofile=row['after_path']))
        if '/game_rules_next/' in row['before_path']:
            transformed = old.replace('../game_rulebook.md#', '../archive/game_rulebook_legacy_2026-10-04.md#')
            transformed = transformed.replace('../game_rulebook_next.md', '../game_rulebook.md')
            for label in ('모드 규칙', '파편 획득', '모노리스 정책', '숙달 소탕', 'BossOmen 정책'):
                transformed = transformed.replace('[현행 ' + label + ']', '[과거 ' + label + ']')
            assert transformed == new, row['before_path']
            body_checks.append({'before': row['before_path'], 'after': row['after_path'],
                'passed': True, 'allowed': 'Exact parent-link relocation and named historical-comparison link/label changes only.'})
old = old_text['core_enhanced/docs/game_rulebook_next.md'].split('## 어떤 게임인가', 1)[1]
new = (R / 'core_enhanced/docs/game_rulebook.md').read_text().split('## 어떤 게임인가', 1)[1]
assert old.replace('game_rules_next/', 'game_rules/') == new
body_checks.append({'before': 'game_rulebook_next.md', 'after': 'game_rulebook.md',
    'passed': True, 'allowed': 'Body from 어떤 게임인가 is identical after only detailed-rule directory relocation.'})
old = old_text['core_enhanced/docs/game_rulebook.md'].split('## 게임 개요', 1)[1]
new = (R / 'core_enhanced/docs/archive/game_rulebook_legacy_2026-10-04.md').read_text().split('## 게임 개요', 1)[1]
without_link_targets = lambda text: re.sub(r'(?<=\])\([^)]*\)', '(LINK)', text)
assert without_link_targets(old) == without_link_targets(new)
body_checks.append({'before': 'game_rulebook.md', 'after': 'archive/game_rulebook_legacy_2026-10-04.md',
    'passed': True, 'allowed': 'Historical body is identical except Markdown link destinations rebased for archive location.'})
assert len(body_checks) == 12
(C / 'EXTERNAL_DOCUMENT_MIGRATION.diff').write_text(''.join(diffs))

exit_receipt = json.loads((A / 'CURRENTNESS_EXIT_INTERRUPTED.json').read_text())
assert len(exit_receipt['mismatches']) == 11 and not exit_receipt['artifact_mismatches'] and not exit_receipt['product_changed']
by_before = {row['before_path']: row for row in migration['documents']}
affected = []
for mismatch in exit_receipt['mismatches']:
    row = by_before[mismatch['path']]
    assert mismatch['expected_sha256'] == row['before_sha256']
    affected.append({**mismatch, 'mapped_current_path': row['after_path'],
        'mapped_current_sha256': row['after_sha256'],
        'disposition': 'Explicit external locator/authority-label migration; policy body and product behavior unchanged.'})

coverage = json.loads((S / 'COVERAGE.json').read_text())
receipt_checks = []
for name, binding in coverage['receipt_bindings'].items():
    p = H / 'evidence' / (name + '.json')
    assert sha(p) == binding['receipt_sha256']
    receipt = json.loads(p.read_text())
    assert receipt['source_before'] == product == receipt['source_after']
    assert sha(H / 'evidence' / (name + '.log')) == binding['log_sha256']
    for item in receipt['command_inputs']:
        assert sha(Path(item['path'])) == item['sha256']
        assert sha(H / item['archive']) == item['sha256']
    receipt_checks.append({'name': name, 'receipt_sha256': sha(p), 'source_and_command_inputs_unchanged': True})
assert len(receipt_checks) == 43
audit_files = {str(p.relative_to(R)): sha(p) for p in sorted(A.rglob('*')) if p.is_file() and '__pycache__' not in p.parts}
intake = {
    'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'finding': 'U05 audit001 F01; eleven current canonical bindings changed during external CF-D09 migration.',
    'authority': 'Existing continuous request plus external CF-D09/USER18 record; preserve migrated user documents and all original Goal/Freeze/audit bytes.',
    'external_migration': {'path': str(D.relative_to(R)), 'record_sha256': sha(D / 'RECORD.ko.md'),
        'evidence_sha256': sha(D / 'EVIDENCE.json'), 'before_archive_sha256': sha(D / 'BEFORE.tar.gz'),
        'documents': migration['documents']},
    'affected_frozen_members': affected, 'body_checks': body_checks,
    'full_external_diff_sha256': sha(C / 'EXTERNAL_DOCUMENT_MIGRATION.diff'),
    'source_sha256': product['sha256'], 'product_files': len(product['files']),
    'product_delta': [], 'policy_acceptance_delta': [],
    'preserved_original_execution_files': len(records),
    'current_rulebook': 'core_enhanced/docs/game_rulebook.md',
    'current_details': 'core_enhanced/docs/game_rules/',
    'historical_rulebook': 'core_enhanced/docs/archive/game_rulebook_legacy_2026-10-04.md',
    'locator_rule': 'Old frozen next paths refer to their archived original bytes. Current navigation uses this explicit map; no automatic fallback or duplicate old-path file is created.',
    'predecessor_audit': audit_files,
    'verdict': 'Documentation rebinding self-verification only; whole-Goal independent re-audit required.',
}
write(C / 'DOCUMENT_PATH_MIGRATION.json', intake)
write(C / 'EVIDENCE_REUSE.json', {'source_sha256': product['sha256'], 'receipts': receipt_checks,
    'coverage_sha256': sha(S / 'COVERAGE.json'), 'predecessor_impact_sha256': sha(S / 'PREDECESSOR_IMPACT.json'),
    'audit_evidence_sha256': sha(A / 'EVIDENCE_VERIFICATION.json'),
    'reason': 'Exact unchanged product, test/data and command inputs. Independently checked document body/locator changes do not invalidate execution outcomes. No product reruns or new latency claims.',
    'acceptances': 14, 'inherited_conditions': 127, 'r06_acceptances': 31})
result = json.loads((S / 'GEX_10_RESULT.json').read_text())
result.update(recorded_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    status='Implementation unchanged; external document migration intake and current locator rebinding self-verified; successor independent GEX-20 pending',
    canonical_rulebook='../../../../game_rulebook.md', canonical_gameplay_detail='../../../../game_rules/combat.md',
    historical_rulebook='../../../../archive/game_rulebook_legacy_2026-10-04.md',
    document_authority_intake='corrections/cf_d09_001/DOCUMENT_PATH_MIGRATION.json',
    evidence_reuse='corrections/cf_d09_001/EVIDENCE_REUSE.json',
    predecessor_result={'path': 'GEX_10_RESULT.json', 'sha256': sha(S / 'GEX_10_RESULT.json')},
    predecessor_audit={'path': str((A / 'COMPLETION_AUDIT.md').relative_to(R)), 'sha256': sha(A / 'COMPLETION_AUDIT.md'), 'verdict': 'Not Complete / High'},
    next='Freeze target/u05_002 including current relocated canonical documents; independent impact-based whole-Goal GEX-20. U06 remains gated.')
write(S / 'GEX_10_RESULT_002.json', result)
write(S / 'CURRENT_EXECUTION.json', {'goal': 'GOAL-CRSL-01', 'current_result': 'GEX_10_RESULT_002.json',
    'current_plan': 'corrections/cf_d09_001/PLAN.md', 'coverage': 'COVERAGE.json',
    'target': 'target/u05_002', 'status': result['status'], 'prior_records_preserved': True})
(C / 'PLAN.md').write_text('# CF-D09 문서 경로 인수와 U05 F01 교정\n\n감사001의 Not Complete / High는 외부 문서 이동으로11개 binding이 달라진 F01 한 건이다. 제품 결함은 별도로 확인되지 않았다. 기존 연속 요청으로 이 좁은 교정을 수행한다.\n\n문서 이동46개의 원본·현재 hash와 상세정책10개/룰북/과거본문의 의미 동일성을 확인하고, 원래43개 실행의 소스·명령 입력 hash를 다시 결속했다. 제품368개와14승인 조건·127선행 조건·31 R06 의미는 변경하지 않았다. 동결 원문을 수정하지 않고 DOCUMENT_PATH_MIGRATION, EVIDENCE_REUSE와 GEX_10_RESULT_002를 후속 현재 기록으로 발행한다.\n\n새 target/u05_002에 현재 경로와 외부 문서 귀속, 원래 target/audit를 연결한다. 독립 영향 기반 재감사에서 전체 Goal Complete / High를 받아야 U06를 선택한다.\n')
(C / 'EXPERIMENTS.md').write_text('# 실제 확인\n\nDOCUMENT_PATH_MIGRATION.json의46개 원문/현재 hash,11개 감사 mismatch mapping,12개 본문 비교가 통과했다. EVIDENCE_REUSE.json의43개 receipt·log·command-input과368개 product membership/digest가 원래 target과 일치했다. 제품 재실행은 하지 않았다. 원래 성능 한도·실패 원본·R05 승인 경고를 그대로 유지한다.\n')
(C / 'EXPERIMENT_NOTES.md').write_text('# 관찰과 남은 판정\n\nCF-D08 때의 이름만으로 현재 policy authority를 추론하면 안 된다. CF-D09 이후 game_rulebook.md가 최신 정책이고 과거 룰북은 archive로 이동했다. 옛 next locator의 역사적 의미는 원래 archive/hash로 보존하고 새 현재 위치를 명시한다. 문서 교정의 자체 검증은 독립 전체 완료 판정이 아니며 새 감사 대상의 entry/exit currentness와 F01 closure가 남는다.\n')
assert run.identity() == product
print(json.dumps({'product_unchanged': True, 'documents': 46, 'affected': 11, 'body_checks': 12, 'reused_receipts': 43}))
