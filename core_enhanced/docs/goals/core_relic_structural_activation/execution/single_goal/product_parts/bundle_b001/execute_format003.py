#!/usr/bin/env python3
"""Collect the authorized two-file production-format delta and rustfmt checks."""
import datetime
import difflib
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile

ROOT = Path('/mnt/f/work/simulator')
B = Path(__file__).resolve().parent
OUT = B / 'u09_product_bundle_b_format003'
PATHS = ['core_enhanced/src/game/battle/core/effective_skill.rs',
         'core_enhanced/src/game/battle/core/sim.rs']


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def ref(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data),
            'sha256': hashlib.sha256(data).hexdigest()}


def write(name, value):
    with (OUT / name).open('x') as stream:
        stream.write(json.dumps(value, indent=2) + '\n')


assert not OUT.exists()
prior_path = B / 'u09_product_bundle_b_format_check002/RESULT.json'
prior = json.loads(prior_path.read_text())
expected_source = prior['source_after']
assert prior['source_unchanged'] and prior['diff_paths'] == PATHS and not prior['error_paths']
cfg_path = B.parent.parent / 'test_parts/bundle_cfg001/CLOSED.json'
assert ref(cfg_path)['sha256'] == '2050b31c541e5066c52b872fcd4c5d11dc40ab899c88c34322aad9a3190fef75'


def source():
    files = [ref(ROOT / row['path']) for row in expected_source['files']]
    return {'files': files,
            'sha256': hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()}


before = source()
assert before == expected_source
preimages = {name: (ROOT / name).read_text() for name in PATHS}
# These are exactly the four production whitespace hunks read and accepted
# by Root from format_check002/06.stdout and 10.stdout. No cfg parser is used.
patches = {
    PATHS[0]: [
        ('        let selected = source.relic_effective().map(|binding| binding.execution())\n',
         '        let selected = source\n            .relic_effective()\n            .map(|binding| binding.execution())\n'),
        ('                    if matches!(definition.recipients, crate::game::relic::structural::authoring::PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease { .. })) {\n',
         '                    if matches!(definition.recipients, crate::game::relic::structural::authoring::PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease { .. }))\n                {\n'),
    ],
    PATHS[1]: [
        ('            match self.requires_explicit_structural_selection(&skill, caster_instance_id, &effective_source) {\n',
         '            match self.requires_explicit_structural_selection(\n                &skill,\n                caster_instance_id,\n                &effective_source,\n            ) {\n'),
        ('                if self.requires_explicit_structural_selection(&skill, caster_instance_id, &self.runtime_fact_source)? {\n',
         '                if self.requires_explicit_structural_selection(\n                    &skill,\n                    caster_instance_id,\n                    &self.runtime_fact_source,\n                )? {\n'),
    ],
}
expected_text = dict(preimages)
for name, changes in patches.items():
    for old, new in changes:
        assert expected_text[name].count(old) == 1
        expected_text[name] = expected_text[name].replace(old, new)
env = os.environ.copy()
env['PATH'] = '/home/blast/.cargo/bin:' + env['PATH']
env['PYTHONDONTWRITEBYTECODE'] = '1'
rustfmt = '/home/blast/.cargo/bin/rustfmt'
version = subprocess.check_output([rustfmt, '--version'], env=env, text=True).strip()
assert version == prior['rustfmt_version']
OUT.mkdir()
started = utc()
write('BEFORE.json', {'actual_before_utc': started, 'source': before,
                     'source_writers_before_command': 0,
                     'authorized_source_writer_during_format': 1,
                     'other_writers': 0, 'other_runners': 0,
                     'authorization': 'Root approved only format_check002 production hunks at effective_skill322/338 and sim483/4737, with no automatic test formatting.',
                     'prior_format_check': ref(prior_path), 'cfg_closed': ref(cfg_path),
                     'recovery_preimage': ref(B / 'CHANGED_AFTER.tar.gz'),
                     'paths': PATHS, 'collector': ref(Path(__file__).resolve())})


def invoke(index, phase, command):
    stdout_path, stderr_path = OUT / f'{phase}{index:02d}.stdout', OUT / f'{phase}{index:02d}.stderr'
    start = utc()
    with stdout_path.open('xb') as stdout, stderr_path.open('xb') as stderr:
        run = subprocess.run(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
    return {'phase': phase, 'command': command, 'actual_started_at_utc': start,
            'actual_closed_at_utc': utc(), 'exit_code': run.returncode,
            'stdout': ref(stdout_path), 'stderr': ref(stderr_path)}


apply_rows = [invoke(index, 'apply', [rustfmt, '--edition', '2021', '--config',
                                   'skip_children=true', '--color', 'never', str(ROOT / name)])
              for index, name in enumerate(PATHS)]
after = source()
changed = [old['path'] for old, new in zip(before['files'], after['files']) if old != new]
matches_four_hunks = (changed == PATHS and all((ROOT / name).read_text() == expected_text[name] for name in PATHS))
diff_path = OUT / 'change.diff'
with diff_path.open('x') as stream:
    for name in PATHS:
        stream.writelines(difflib.unified_diff(preimages[name].splitlines(keepends=True),
                                             (ROOT / name).read_text().splitlines(keepends=True),
                                             fromfile='a/' + name, tofile='b/' + name))
check_rows = []
if matches_four_hunks and all(row['exit_code'] == 0 for row in apply_rows):
    check_rows = [invoke(index, 'check', [rustfmt, '--check', '--edition', '2021', '--config',
                                       'skip_children=true', '--color', 'never', str(ROOT / name)])
                  for index, name in enumerate(PATHS)]
final_source = source()
archive_path = OUT / 'FORMATTED_DELTA.tar.gz'
with tarfile.open(archive_path, 'x:gz') as archive:
    for name in PATHS:
        data = (ROOT / name).read_bytes()
        member = tarfile.TarInfo(name)
        member.size = len(data)
        member.mode = 0o644
        member.mtime = int(datetime.datetime.now(datetime.timezone.utc).timestamp())
        archive.addfile(member, io.BytesIO(data))
with tarfile.open(archive_path, 'r:gz') as archive:
    assert [member.name for member in archive.getmembers()] == PATHS
    for member in archive.getmembers():
        assert archive.extractfile(member).read() == (ROOT / member.name).read_bytes()
success = (matches_four_hunks and all(row['exit_code'] == 0 for row in apply_rows)
           and len(check_rows) == 2 and all(row['exit_code'] == 0 for row in check_rows)
           and after == final_source)
result = {'label': OUT.name, 'kind': 'authorized_production_whitespace_only_format',
          'actual_started_at_utc': started, 'actual_closed_at_utc': utc(),
          'exit_code': 0 if success else 1, 'cwd': str(ROOT), 'rustfmt_version': version,
          'edition': '2021', 'config': prior['config'],
          'source_before': before, 'source_after': final_source,
          'changed_paths': changed, 'matches_exact_four_authorized_hunks': matches_four_hunks,
          'applied_file_count': 2, 'apply_rows': apply_rows,
          'read_only_check_rows': check_rows, 'source_unchanged_during_read_only_recheck': after == final_source,
          'prior_38_checks_reused': [row for row in prior['rows'] if row['path'] not in PATHS],
          'prior_38_source_bytes_unchanged': all(old == new for old, new in zip(before['files'], final_source['files']) if old['path'] not in PATHS),
          'prior_format_check': ref(prior_path), 'cfg_closed': ref(cfg_path),
          'check012': ref(B / 'u09_product_bundle_b_check012.json'),
          'after_delta_archive': ref(archive_path), 'archive_members': [ref(ROOT / name) for name in PATHS],
          'archive_readback': 'exact two-member bytes/order', 'diff': ref(diff_path),
          'collector': ref(Path(__file__).resolve()),
          'source_writers_now': 0, 'runner_now': 0, 'other_runners': 0,
          'additional_cargo_runs': 0, 'cfg_verifier_reruns': 0,
          'visibility': 'Visible PID namespace and explicit parent sole ownership; no whole shared-host assertion'}
write('RESULT.json', result)
print(json.dumps({'result': ref(OUT / 'RESULT.json'), 'exit_code': result['exit_code'],
                  'source_after_sha256': final_source['sha256'], 'changed_paths': changed,
                  'matches_four_hunks': matches_four_hunks, 'read_only_checks': [r['exit_code'] for r in check_rows],
                  'diff': ref(diff_path), 'archive': ref(archive_path)}, indent=2))
raise SystemExit(result['exit_code'])
