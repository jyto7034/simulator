"""Capture the exact pre-mutation baseline; does not build or test products."""
import datetime
import json
from pathlib import Path
import subprocess
import tarfile
from run import HERE, ROOT, identity, sha


def write(path, value):
    with path.open('x') as stream:
        stream.write(value if isinstance(value, str) else json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def main():
    e = HERE / 'evidence/gex00_001'
    e.mkdir(exist_ok=True)
    before = identity()
    assert len(before['files']) == 389
    assert before['sha256'] == '886d427882319b3147260f394f1204acf903af654f772709bdaaa045f1c62146'
    if (e / 'PRODUCT_BEFORE.json').exists():
        assert json.loads((e / 'PRODUCT_BEFORE.json').read_text()) == before
    else:
        write(e / 'PRODUCT_BEFORE.json', before)
    with tarfile.open(e / 'SOURCE_BEFORE.tar.gz', 'r:gz' if (e / 'SOURCE_BEFORE.tar.gz').exists() else 'x:gz') as archive:
        for row in before['files']:
            if archive.mode == 'r':
                assert archive.extractfile(row['path']).read() == (ROOT / row['path']).read_bytes()
            else:
                archive.add(ROOT / row['path'], arcname=row['path'], recursive=False)
    processes = subprocess.check_output(['ps', '-eo', 'pid,ppid,stat,etime,args'], text=True)
    if not (e / 'PROCESSES_BEFORE.txt').exists():
        write(e / 'PROCESSES_BEFORE.txt', processes)
    active = []
    for directory in Path('/proc').iterdir():
        if not directory.name.isdigit():
            continue
        try:
            name = (directory / 'comm').read_text().strip()
            command = (directory / 'cmdline').read_bytes()
            if name in ('cargo', 'rustc') and command and (str(ROOT).encode() in command or (directory / 'cwd').resolve() == ROOT):
                active.append(int(directory.name))
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            pass
    assert not active, active
    if not (e / 'GIT_STATUS_BEFORE.txt').exists():
        write(e / 'GIT_STATUS_BEFORE.txt', subprocess.check_output(['git', 'status', '--short'], cwd=ROOT, text=True))
    old = ROOT / 'core_enhanced/docs/goals/core_independent_objects'
    compatibility = old / 'execution/single_goal/compatibility/current_001'
    paths = set()
    for base in (old / 'tools', compatibility):
        for path in base.rglob('*'):
            relative = path.relative_to(base)
            if path.is_file() and not {'evidence', '__pycache__', 'target'}.intersection(relative.parts):
                paths.add(path)
    for folder in ('runtime_architecture_final_validation', 'runtime_architecture_preservation'):
        base = ROOT / 'core_enhanced/tools' / folder
        paths.update(path for path in base.rglob('*') if path.is_file() and 'target' not in path.relative_to(base).parts)
    paths.add(ROOT / 'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_eep_f01_f03_001/evidence/pc_probe.rs')
    rows = [{'path': str(path.relative_to(ROOT)), 'sha256': sha(path), 'bytes': path.stat().st_size} for path in sorted(paths)]
    if (e / 'HARNESS_INPUT_BEFORE.json').exists():
        assert json.loads((e / 'HARNESS_INPUT_BEFORE.json').read_text()) == rows
    else:
        write(e / 'HARNESS_INPUT_BEFORE.json', rows)
    with tarfile.open(e / 'HARNESS_INPUT_BEFORE.tar.gz', 'r:gz' if (e / 'HARNESS_INPUT_BEFORE.tar.gz').exists() else 'x:gz') as archive:
        for row in rows:
            if archive.mode == 'r':
                assert archive.extractfile(row['path']).read() == (ROOT / row['path']).read_bytes()
            else:
                archive.add(ROOT / row['path'], arcname=row['path'], recursive=False)
    refs = [HERE / 'P2G_60_HANDOFF_001.json', HERE / 'P2G_50_GOAL_SET.ko.md', HERE / 'CODE_REVIEW_001.ko.md',
            ROOT / 'core_enhanced/docs/roadmaps/core_first/execution/u07m_start_20261005_001/PUBLICATION_RECORD.json',
            ROOT / 'core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md',
            ROOT / 'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md',
            old / 'execution/single_goal/target/u07_002/MANIFEST.json',
            old / 'post_audit/u07_20261005_002/AUDIT_CLOSURE.json']
    assert identity() == before
    write(e / 'GEX00.json', {
        'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'stage': 'GEX-00', 'run_mode': 'goal-run', 'goal_set': 'GS-CMRP-001', 'goal': 'GOAL-CMRP-01',
        'handoff': 'HA-CMRP-001', 'implementation_role': '/root/u07_implementation / requested GPT-6.1 Sol/xHigh',
        'authority': 'Existing continuous request; explicit root implementation assignment on2026-10-05. P2G alone is not authorization.',
        'execution_directory': str((HERE / 'execution/single_goal').relative_to(ROOT)),
        'roles': {name: 'execution/single_goal/' + name for name in ('PLAN.md', 'EXPERIMENTS.md', 'EXPERIMENT_NOTES.md', 'STATE_INVENTORY.md')},
        'product_sha256': before['sha256'], 'product_count': len(before['files']), 'own_compilers_active': active,
        'source_archive_sha256': sha(e / 'SOURCE_BEFORE.tar.gz'), 'harness_archive_sha256': sha(e / 'HARNESS_INPUT_BEFORE.tar.gz'),
        'harness_input_members': len(rows), 'refs': [{'path': str(p.relative_to(ROOT)), 'sha256': sha(p)} for p in refs],
        'reuse': 'Exact completed U07 source and audit baseline; no initial full test repetition. Frozen predecessors are external exact references, never recursively embedded.',
        'next': 'GEX-10 PROFILE parser/private entry/local snapshot admission and focused boundaries, then ACCOUNTING/default PC before expensive integration.'})
    print(json.dumps({'product_sha256': before['sha256'], 'files': len(before['files']), 'harness_inputs': len(rows), 'gex00': 'complete'}))


if __name__ == '__main__':
    main()
