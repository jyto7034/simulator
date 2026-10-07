"""Freeze one U08 audit target, restoring both directions only in isolation.

This is an execution tool. Preparing or syntax-checking it proves no restoration.
The original workspace and every previous target remain untouched.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
from evidence_common import (ROOT, HERE, S, read, write, ref, check_ref, digest,
    load_config, result_base, selected, load_stage, archive_exact, identity, nonempty)
from run import sha


def rows(root):
    return [{'path':str(p.relative_to(root)), 'sha256':sha(p), 'bytes':p.stat().st_size}
            for p in sorted(root.rglob('*')) if p.is_file()]


def extract(path, destination, expected):
    archive_exact(path, expected)
    with tarfile.open(path) as archive:
        for member in archive:
            output = destination / member.name
            assert output.resolve().is_relative_to(destination.resolve())
            output.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, output.open('xb') as target:
                shutil.copyfileobj(source, target)


def pack(destination, base, entries):
    with tarfile.open(destination, 'x:gz') as archive:
        for row in entries:
            path = base / row['path']
            assert sha(path) == row['sha256'] and path.stat().st_size == row['bytes']
            archive.add(path, arcname=row['path'], recursive=False)
    archive_exact(destination, entries)


def main():
    cfg, current, names = load_config()
    assert cfg['freeze_execution_mode'] == 'direct_unwrapped'
    pid = os.getppid()
    while pid > 1:
        directory = Path('/proc') / str(pid)
        try:
            command = (directory / 'cmdline').read_bytes().split(b'\0')
            assert str(HERE / 'tools/run.py').encode() not in command, 'Run freeze directly; an active run.py log cannot be frozen'
            status = (directory / 'status').read_text()
            pid = int(next(line.split(':',1)[1] for line in status.splitlines() if line.startswith('PPid:')))
        except (FileNotFoundError, ProcessLookupError):
            break
    selected(cfg, current, names)
    gex = load_stage('gex', cfg, current, names)
    coverage = load_stage('coverage', cfg, current, names)
    assert gex['coverage'] == ref(names['coverage'])
    assert gex['self_awarded_independent_verdict'] is False
    assert gex['goal_status'] == 'Not independently Complete'
    handoff_path = ROOT / cfg['documentation_handoff_path']
    handoff = read(handoff_path)
    assert handoff['source_sha256'] == current['sha256'] and handoff['gex'] == ref(names['gex'])
    assert handoff['coverage'] == ref(names['coverage'])
    assert handoff['active_input_writers'] == handoff['active_functional_runners'] == 0
    docs = {r['path']:r for r in handoff['documents']}
    required_docs = {str((S / n).relative_to(ROOT)) for n in ('PLAN.md','EXPERIMENT_NOTES.md','EXPERIMENTS.md','STATE_INVENTORY.md')}
    assert required_docs <= docs.keys()
    for row in docs.values():
        check_ref(row)
    nonempty(handoff['reviewer'])
    target = names['target']
    assert not target.exists(), 'Never reuse or replace a target label'
    # Three exact GEX00_002 before-images own distinct portions of restoration.
    # Goal tools/evidence are separately included in RECORDS, not invented as
    # absent source preimages at an earlier checkpoint.
    baseline = HERE / 'evidence/gex00_002'
    before_by_path = {}
    baselines = []
    for stem in ('PRODUCT_SOURCE_BEFORE','CANONICAL_DOCS_BEFORE','HARNESS_INPUT_BEFORE'):
        manifest = baseline / (stem + '.json')
        archive = baseline / (stem + '.tar.gz')
        entries = read(manifest)
        archive_exact(archive, entries)
        for row in entries:
            assert row['path'] not in before_by_path or before_by_path[row['path']] == row
            before_by_path[row['path']] = row
        baselines.append({'manifest':ref(manifest), 'archive':ref(archive), 'entries':entries})
    # Match run.identity()/rows(): Path component ordering differs from string
    # ordering for e.g. application/fault.rs versus application.rs. Original
    # baseline lists and their digests are read unchanged; these are new union
    # manifests for the explicitly scoped restoration archives.
    before = [before_by_path[p] for p in sorted(before_by_path, key=Path)]
    scope = set(before_by_path) | {r['path'] for r in current['files']}
    after = [ref(ROOT / p) for p in sorted(scope, key=Path) if (ROOT / p).is_file()]
    # refs and source rows use the same fields; key order never changes identity.
    expected_after = [{k:r[k] for k in ('path','sha256','bytes')} for r in after]
    external = []
    assert cfg['external_archives'], 'Keep predecessor target archives as exact external references'
    for item in cfg['external_archives']:
        path = check_ref(item['archive'])
        manifest = read(check_ref(item['members_manifest']))
        from evidence_common import json_at
        entries = json_at(manifest, item['members_pointer'])
        archive_exact(path, entries)
        external.append(dict(item, member_count=len(entries), exact_member_bytes_verified=True))
    previous_root = ROOT / 'core_enhanced/docs/goals/core_runtime_measurement_profiles/execution/single_goal/target/u07m_001'
    required_external = {str((previous_root / n).relative_to(ROOT)) for n in
                         ('SOURCE_BEFORE.tar.gz','SOURCE_AFTER.tar.gz','RECORDS.tar.gz','REFERENCES.tar.gz')}
    assert required_external <= {r['archive']['path'] for r in external}
    for previous in target.parent.glob('u08_*'):
        if previous.is_dir() and (previous / 'MANIFEST.json').is_file():
            needed = {str(p.relative_to(ROOT)) for p in previous.glob('*.tar.gz')}
            assert needed <= {r['archive']['path'] for r in external}
    target.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory(prefix='u08-isolated-restore-') as directory:
        root = Path(directory)
        b, a = root / 'before', root / 'after'
        b.mkdir(); a.mkdir()
        for original in baselines:
            temporary = root / ('preimage_' + str(len(list(root.glob('preimage_*')))))
            temporary.mkdir()
            extract(ROOT / original['archive']['path'], temporary, original['entries'])
            for row in original['entries']:
                destination = b / row['path']
                destination.parent.mkdir(parents=True, exist_ok=True)
                if destination.exists():
                    assert sha(destination) == row['sha256']
                else:
                    shutil.copyfile(temporary / row['path'], destination)
        assert rows(b) == before
        for row in expected_after:
            destination = a / row['path']
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / row['path'], destination)
        assert rows(a) == expected_after
        pack(target / 'SOURCE_BEFORE.tar.gz', b, before)
        pack(target / 'SOURCE_AFTER.tar.gz', a, expected_after)
        for label, left, right in [('FORWARD','before','after'), ('REVERSE','after','before')]:
            command = ['git','diff','--no-index','--binary','--src-prefix=a/','--dst-prefix=b/', '--',left,right]
            result = subprocess.run(command, cwd=root, capture_output=True)
            assert result.returncode in (0,1), result.stderr.decode(errors='replace')
            with (target / (label + '.patch')).open('xb') as stream:
                stream.write(result.stdout)
        for label, expected in [('FORWARD',expected_after), ('REVERSE',before)]:
            patch = target / (label + '.patch')
            assert patch.stat().st_size > 0, 'U08 source delta must be explicit'
            command = ['git','apply','--binary','-p2','--whitespace=nowarn',str(patch)]
            result = subprocess.run(command, cwd=b, capture_output=True)
            with (target / ('RESTORE_' + label + '.log')).open('xb') as stream:
                stream.write(result.stdout + result.stderr)
            assert result.returncode == 0, result.stderr.decode(errors='replace')
            assert rows(b) == expected, 'Isolated restoration membership or bytes mismatch'
    write(target / 'RESTORE_VERIFICATION.json', {'before_sha256':digest(before),
        'after_sha256':digest(expected_after), 'forward_membership_and_bytes_match':True,
        'reverse_membership_and_bytes_match':True, 'isolated_only':True,
        'workspace_reverse_patch_applied':False, 'baseline':'evidence/gex00_002',
        'patch_method':'git diff --no-index --binary; git apply -p2 in isolated before directory'})
    record_paths = sorted(p for p in HERE.rglob('*') if p.is_file()
                          and not {'target','post_audit','__pycache__'} & set(p.relative_to(HERE).parts))
    record_rows = [ref(p) for p in record_paths]
    pack(target / 'RECORDS.tar.gz', ROOT, record_rows)
    write(target / 'RECORDS.json', record_rows)
    references = {r['path']:r for r in cfg['references']}
    for row in cfg['basis'] + cfg['tool_inputs'] + list(docs.values()):
        references[row['path']] = row
    references[handoff['gex']['path']] = handoff['gex']
    references[ref(handoff_path)['path']] = ref(handoff_path)
    for item in external:
        references[item['members_manifest']['path']] = item['members_manifest']
    for row in references.values():
        check_ref(row)
        assert row['path'] not in {r['archive']['path'] for r in external}, 'Do not recursively embed predecessor archives'
    reference_rows = [references[p] for p in sorted(references)]
    pack(target / 'REFERENCES.tar.gz', ROOT, reference_rows)
    write(target / 'REFERENCES.json', reference_rows)
    write(target / 'EXTERNAL_ARCHIVES.json', external)
    # Read VCS state; do not mutate the index, tracked files or user work.
    def git(*args):
        return subprocess.check_output(['git',*args], cwd=ROOT)
    tracked = set(git('ls-files','-z').decode().split('\0'))
    dirty = set(git('diff','--name-only','-z','HEAD').decode().split('\0'))
    membership = {}
    for group, entries in [('source',expected_after),('records',record_rows),('references',reference_rows),
                           ('external_archive',[r['archive'] for r in external])]:
        for row in entries:
            item = membership.setdefault(row['path'], dict(row, groups=[],
                kind='tracked/dirty' if row['path'] in dirty else 'tracked/clean' if row['path'] in tracked else 'untracked'))
            assert item['sha256'] == row['sha256'] and item['bytes'] == row['bytes']
            item['groups'].append(group)
    write(target / 'MEMBERSHIP.json', [membership[p] for p in sorted(membership)])
    with (target / 'GIT_STATUS.txt').open('xb') as stream:
        stream.write(git('status','--porcelain=v2','--untracked-files=all'))
    assert identity() == current
    assert all(ref(ROOT / r['path']) == r for r in after + record_rows + reference_rows)
    result = dict(result_base(cfg, current), goal_set='GS-CRNV-001', phase='PHASE-CRNV-01',
        state='Frozen GEX10 self-verification target; independent GEX20 pending',
        product_source=current, before_files=before, source_files=expected_after,
        before_sha256=digest(before), restoration_source_sha256=digest(expected_after),
        changed=[r['path'] for r in expected_after if before_by_path.get(r['path']) != r],
        added=sorted({r['path'] for r in expected_after} - set(before_by_path)),
        deleted=sorted(set(before_by_path) - {r['path'] for r in expected_after}),
        vcs_revision=git('rev-parse','HEAD').decode().strip(), repository=str(ROOT),
        selection=ref(names['selection']), evidence_map=ref(names['coverage']),
        predecessor_impact=ref(names['impact']), gex10=ref(names['gex']),
        documentation_handoff=ref(handoff_path), counts=coverage['counts'],
        records_manifest_sha256=sha(target/'RECORDS.json'),
        references_manifest_sha256=sha(target/'REFERENCES.json'),
        membership_manifest_sha256=sha(target/'MEMBERSHIP.json'),
        archives_and_patches=[{'path':p.name,'sha256':sha(p),'bytes':p.stat().st_size}
                              for p in sorted(target.iterdir()) if p.is_file()],
        exclusions=['No mutation to previous target/audit artifacts', 'Build outputs and __pycache__ are excluded',
                    'Earlier archives are external member-verified references',
                    'Source restoration has explicit product/canonical/historical-input scope; goal tools and evidence are frozen in RECORDS'],
        completion='PE-CRNV-16.2 remains pending exact independent Complete / High / open0 closure')
    write(target / 'MANIFEST.json', result)
    print(json.dumps({'target':str(target), 'product_source':current['sha256'],
                      'restoration_verified':True,'independent_audit':False}))


if __name__ == '__main__':
    main()
