"""Capture U08's exact immutable inputs before any product mutation."""
import datetime
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
from run import HERE, ROOT, identity, sha


def write(path, value):
    with path.open('x') as stream:
        stream.write(value if isinstance(value, str) else json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def rows(paths):
    return [{'path': str(p.relative_to(ROOT)), 'sha256': sha(p), 'bytes': p.stat().st_size}
            for p in sorted(set(paths))]


def capture(directory, name, paths):
    members = rows(paths)
    write(directory / (name + '.json'), members)
    with tarfile.open(directory / (name + '.tar.gz'), 'x:gz') as archive:
        for row in members:
            archive.add(ROOT / row['path'], arcname=row['path'], recursive=False)
    with tarfile.open(directory / (name + '.tar.gz'), 'r:gz') as archive:
        assert len(archive.getmembers()) == len(members)
        for row in members:
            assert archive.extractfile(row['path']).read() == (ROOT / row['path']).read_bytes()
    return members


def main():
    label = sys.argv[1] if len(sys.argv) == 2 else '002'
    assert label.isdigit()
    e = HERE / ('evidence/gex00_' + label)
    e.mkdir()
    before = identity()
    assert len(before['files']) == 398
    assert before['sha256'] == 'f2144e9a4c5ddeb41245799294a75b41de9d843018cb2166f711b7f07821cda8'
    write(e / 'PRODUCT_BEFORE.json', before)
    product = capture(e, 'PRODUCT_SOURCE_BEFORE', [ROOT / r['path'] for r in before['files']])
    write(e / 'PROCESSES_BEFORE.txt', subprocess.check_output(['ps', '-eo', 'pid,ppid,stat,etime,args'], text=True))
    active = []
    for directory in Path('/proc').iterdir():
        if not directory.name.isdigit():
            continue
        try:
            name = (directory / 'comm').read_text().strip()
            if name in ('cargo', 'rustc') and (directory / 'cmdline').read_bytes():
                active.append(int(directory.name))
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            pass
    assert not active, active
    write(e / 'GIT_STATUS_BEFORE.txt', subprocess.check_output(['git', 'status', '--porcelain=v1', '--untracked-files=all'], cwd=ROOT, text=True))
    for filename, command in [('WORKTREE_BEFORE.patch', ['git', 'diff', '--binary']),
                              ('INDEX_BEFORE.patch', ['git', 'diff', '--cached', '--binary']),
                              ('INDEX_MEMBERS_BEFORE.txt', ['git', 'ls-files', '--stage']),
                              ('HEAD_BEFORE.txt', ['git', 'rev-parse', 'HEAD'])]:
        write(e / filename, subprocess.check_output(command, cwd=ROOT, text=True))
    index = Path(subprocess.check_output(['git', 'rev-parse', '--git-path', 'index'], cwd=ROOT, text=True).strip())
    index = index if index.is_absolute() else ROOT / index
    shutil.copyfile(index, e / 'INDEX_BEFORE')
    dirty_paths = subprocess.check_output(['git', 'diff', '--name-only', '-z', 'HEAD'], cwd=ROOT).decode().split('\0')
    capture(e, 'TRACKED_DIRTY_WORKTREE', [ROOT / p for p in dirty_paths if p and (ROOT / p).is_file()])
    write(e / 'TRACKED_DELETED_BEFORE.json', [p for p in dirty_paths if p and not (ROOT / p).exists()])
    old = ROOT / 'core_enhanced/docs/goals/core_runtime_measurement_profiles'
    harness_paths = set()
    for base in (old / 'tools', old / 'execution/single_goal/compatibility/current_001',
                 ROOT / 'core_enhanced/tools/runtime_architecture_final_validation',
                 ROOT / 'core_enhanced/tools/runtime_architecture_preservation'):
        harness_paths.update(p for p in base.rglob('*') if p.is_file()
                             and not {'evidence', '__pycache__', 'target'}.intersection(p.relative_to(base).parts))
    harness_paths.add(ROOT / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_eep_f01_f03_001/evidence/pc_probe.rs')
    harness = capture(e, 'HARNESS_INPUT_BEFORE', harness_paths)
    docs = set(HERE.glob('*.json')) | set(HERE.glob('*.md')) | set((HERE / 'execution/single_goal').glob('*.md'))
    docs.update((ROOT / 'core_enhanced/docs').glob('*.md'))
    docs.update((ROOT / 'core_enhanced/docs/skills').glob('*.md'))
    docs.update((ROOT / 'game_server/docs').glob('*.md'))
    docs.update(p for p in (ROOT / 'game_server/config/observability').glob('*.md'))
    for relative in ('core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md',
                     'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md'):
        docs.add(ROOT / relative)
    canonical = capture(e, 'CANONICAL_DOCS_BEFORE', docs)
    refs = [HERE / 'P2G_60_HANDOFF_001.json', HERE / 'P2G_50_GOAL_SET.ko.md', HERE / 'P2G_30_FREEZE_001.json',
            HERE / 'P2G_40_PHASE_PLAN_001.json', HERE / 'P2G_40_VALIDATION_RECEIPT_001.json', HERE / 'CODE_REVIEW_001.ko.md',
            HERE / 'WORKING_POLICY.ko.md', HERE / 'PREDECESSOR_INTAKE_001.json',
            ROOT / 'core_enhanced/docs/roadmaps/core_first/execution/u08_start_20261006_001/PUBLICATION_RECORD.json',
            old / 'execution/single_goal/target/u07m_001/MANIFEST.json',
            old / 'post_audit/u07m_20261006_001/AUDIT_CLOSURE.json',
            old / 'execution/single_goal/EVIDENCE_SELECTION.json']
    assert sha(refs[-3]) == '50c3c228d2c8f0915ab6133014bcf7783bb44d3b8880a8ba7c784ffe130a5733'
    assert sha(refs[-2]) == 'a9fac11bb702a2e484daeb8c34ee04e39d3c30d553cb1c996611c1b7aca4dbb8'
    assert sha(refs[-4]) == '65c08c64eb819b24c037605a218cf007ef2bfea120d5b6ec62bb1a9f711a913f'
    selected = json.loads(refs[-1].read_text())
    assert len(selected['receipt_bindings']) == 54
    for row in selected['receipt_bindings']:
        assert sha(ROOT / row['path']) == row['sha256']
    frozen = json.loads((HERE / 'evidence/REVIEW_SOURCE_MANIFEST_003.json').read_text())
    source_rows = frozen['files'] if isinstance(frozen, dict) else frozen
    for row in source_rows:
        assert sha(ROOT / row['path']) == row['sha256']
    assert identity() == before
    write(e / 'GEX00.json', {
        'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'stage': 'GEX-00', 'run_mode': 'goal-run', 'goal_set': 'GS-CRNV-001', 'goal': 'GOAL-CRNV-01',
        'handoff': 'HA-CRNV-001', 'implementation_role': '/root/u07_implementation / requested GPT-6.1 Sol/xHigh',
        'authority': 'Existing continuous request and explicit root U08 implementation assignment2026-10-06.',
        'product_sha256': before['sha256'], 'product_count': len(product), 'own_compilers_active': active,
        'harness_inputs': len(harness), 'canonical_docs': len(canonical), 'git_index_sha256': sha(e / 'INDEX_BEFORE'),
        'archives': rows(e.glob('*.tar.gz')), 'refs': rows(refs), 'reuse_receipts': len(selected['receipt_bindings']),
        'baseline_rule': '54 exact independently closed predecessor receipts, not new relic execution evidence. Original targets and histories external; no recursive embedding.',
        'git_rule': 'Actual dirty/index/worktree bytes captured; no reset, clean, stash or user-file mutation.',
        'next': 'GEX-10 DEFINE/OWNERS then literal-off/Expr/basic/carry and early default PC.'})
    print(json.dumps({'gex00': 'complete', 'product': before['sha256'], 'files': len(product), 'harness': len(harness), 'docs': len(canonical)}))


if __name__ == '__main__':
    main()
