"""Actual measurement binary samples over exact input and executable identities."""
import json
import os
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
from run import ROOT, HERE, identity, sha

label, inputs_label, *reuse_labels = sys.argv[1:]
assert label.isdigit() and inputs_label.isdigit()
inputs = HERE / 'evidence' / ('playback_inputs_' + inputs_label)
intake = json.loads((inputs / 'INPUTS.json').read_text())
source = identity()
assert source['sha256'] == intake['source_sha256']
assert all(sha(ROOT / path) == digest for path, digest in intake['files'].items())
output = HERE / 'evidence' / ('playback_samples_' + label)
output.mkdir()
binary = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / '.cache/d07-runtime-amount-product-target')) / 'debug/measure_runtime'
binary_sha = sha(binary)
reuse = None
if reuse_labels:
    assert len(reuse_labels) == 1 and reuse_labels[0].isdigit()
    prior_receipt = HERE / 'evidence' / ('playback_actual' + reuse_labels[0] + '.json')
    receipt = json.loads(prior_receipt.read_text())
    assert receipt['source_unchanged'] and receipt['source_before'] == source
    reuse = HERE / 'evidence' / ('playback_samples_' + reuse_labels[0])
rows = []
for specification in intake['rows']:
    name = specification['name']
    manifest = Path(specification['path'])
    assert sha(manifest) == specification['sha256']
    reused = reuse is not None and (reuse / (name + '.jsonl')).exists()
    sample_folder = reuse if reused else output
    sample = sample_folder / (name + '.jsonl')
    command = [str(binary), str(manifest), str(sample)]
    if reused:
        result = SimpleNamespace(returncode=0, stdout=(sample_folder / (name + '.stdout')).read_bytes(), stderr=(sample_folder / (name + '.stderr')).read_bytes())
    else:
        result = subprocess.run(command, cwd=ROOT, capture_output=True)
        for suffix, payload in [('stdout', result.stdout), ('stderr', result.stderr)]:
            with (sample_folder / (name + '.' + suffix)).open('xb') as stream:
                stream.write(payload)
    assert result.returncode == 0, (name, result.stderr[-2000:])
    report = json.loads(result.stdout.splitlines()[-1])
    assert report['summary_written'] and not report['summary_missing']
    assert report['summary']['driver_errors'] == 0, (name, report)
    records = [json.loads(line) for line in sample.read_text().splitlines()]
    header, footer = records[0], records[-1]
    assert header['record'] == 'header' and footer['record'] == 'summary'
    assert header['manifest_sha256'] == specification['sha256']
    assert header['binary_sha256'] == binary_sha
    assert header['source_sha256_supplied'] == source['sha256']
    assert header['process_heap']['instrumented'] if 'process_heap' in header else True
    assert footer['process_heap']['instrumented']
    assert footer['process_heap']['live_requested_bytes'] > 0
    assert footer['process_rss']['status'] == 'available'
    assert footer['process_window'] is not None
    assert footer['process_window']['baseline']['instrumented']
    assert footer['process_window']['end']['instrumented']
    window = footer['process_window']
    baseline = window['baseline']['live_requested_bytes']
    end = window['end']['live_requested_bytes']
    peak = window['peak_requested_bytes']
    assert peak is not None and peak >= max(baseline, end), (name, window)
    assert window['live_delta_bytes'] == end - baseline
    assert window['retiring_hooks_at_end'] == 0 and not window['boundary_updates_unavailable']
    assert sample.stat().st_size <= 16 * 1024 * 1024
    if name.startswith('normal_') or name == 'profile_isolation':
        expected_rejections = [(2, 'MissingBattleStateAuthority', 'missing_battle_state_authority'),
                               (13, 'ActionUnavailable', 'context_irrelevant'),
                               (15, 'InvalidAction', 'invalid_action'), (16, 'InvalidAction', 'invalid_action')]
        for index in range(len(header['sessions'])):
            actual_rejections = [(r['step'], r['outcome']['error_kind'], r['outcome']['error_code']) for r in records
                                 if r['record'] == 'step' and r['session'] == index and r['outcome']['error_kind'] is not None]
            assert actual_rejections == expected_rejections, (name, index, actual_rejections)
        assert footer['rejected_steps'] == 4 * len(header['sessions']), (name, footer)
    if name.startswith('objects_'):
        assert footer['rejected_steps'] == 0, (name, footer)
        assert any(r.get('outcome', {}).get('resolved_command_sha256') for r in records)
        assert any(r['record'] == 'native_operation' and r['summary']['retained'] and r['summary']['retained']['accepted_total_bytes'] > 0 for r in records)
    if name == 'queue_full':
        assert footer['queue_counters']['dropped'] == 76
    isolation = None
    if name == 'profile_isolation':
        manifest_value = json.loads(manifest.read_text())
        expected_sessions = header['sessions']
        assert len(expected_sessions) == 2
        assert [s['selected_hard'] for s in expected_sessions] == [16 * 1024 * 1024, 32 * 1024 * 1024]
        checks = [dict(steps=0, native=0, committed=0, prepared=0) for _ in expected_sessions]
        def check_identity(value, index):
            assert value is not None, (name, index, 'missing identity')
            expected = expected_sessions[index]
            assert expected['effective_profile'] == manifest_value['sessions'][index]['expected_effective_profile_sha256']
            for field, digest in [('effective_profile_sha256', expected['effective_profile']), ('raw_profile_sha256', expected['raw_profile']), ('content_sha256', expected['content']['semantic_sha256'])]:
                assert bytes(value[field]).hex() == digest, (name, index, field, value)
            assert value['run_seed'] == expected['seed']
            assert value['selected_profile_hard_bytes'] == expected['selected_hard']
            assert value['profile_warning_bytes'] == expected['nominal_warning']
        for record in records:
            if record['record'] == 'step':
                index = record['session']
                check_identity(record['identity'], index)
                checks[index]['steps'] += 1
                if record['committed'] is not None:
                    current = record['committed']['current']
                    if current['identity'] is None:
                        assert record['committed']['commits'] == 0 and current['owner_extent'] == 'unmeasured'
                    else:
                        check_identity(current['identity'], index)
                        checks[index]['committed'] += 1
            elif record['record'] == 'native_operation':
                native = record['summary']
                index = (native['session'] - 1) % 16
                assert index < 2
                check_identity(native['started_identity'], index)
                checks[index]['native'] += 1
                if native['retained'] is not None:
                    assert native['retained']['owner_hard_bytes'] == expected_sessions[index]['selected_hard']
                    assert native['retained']['profile_warning_bytes'] == expected_sessions[index]['nominal_warning']
                for field in ['prepared', 'committed']:
                    if native[field] is not None:
                        check_identity(native[field]['identity'], index)
                        checks[index][field] += 1
        assert all(c['steps'] > 0 and c['native'] > 0 and c['committed'] > 0 for c in checks), checks
        isolation = {'passed': True, 'sessions': expected_sessions, 'checks': checks}
    rows.append({'name': name, 'command': command, 'exit_code': result.returncode, 'manifest_sha256': sha(manifest),
                 'execution': 'exact current prior binary output revalidated' if reused else 'executed in this receipt',
                 'prior_execution_receipt': str(prior_receipt) if reused else None,
                 'sample_path': str(sample), 'sample_sha256': sha(sample), 'stdout_sha256': sha(sample_folder / (name + '.stdout')),
                 'stderr_sha256': sha(sample_folder / (name + '.stderr')), 'summary': footer,
                 'profile_isolation': isolation,
                 'final_states': [r for r in records if r['record'] == 'final_state'],
                 'step_results': [r['outcome'] for r in records if r['record'] == 'step']})
    print(json.dumps({'name': name, 'exit_code': result.returncode, 'accepted_steps': footer['accepted_steps'], 'rejected_steps': footer['rejected_steps'], 'raw': footer['raw_operations_seen'], 'heap_live': footer['process_heap']['live_requested_bytes'], 'heap_window_baseline': baseline, 'heap_window_end': end, 'heap_window_peak': peak, 'profile_isolation': isolation}), flush=True)
by_name = {r['name']: r for r in rows}
for profile in ['default', 'dev16', 'dev32']:
    expected = by_name['normal_' + profile + '_none']
    for mode in ['normal', 'detached', 'full']:
        current = by_name['normal_' + profile + '_' + mode]
        assert current['step_results'] == expected['step_results'], (profile, mode)
        assert current['final_states'] == expected['final_states'], (profile, mode)
for mode in ['normal', 'detached', 'full']:
    assert by_name['queue_' + mode]['final_states'] == by_name['queue_none']['final_states']
# Actual profile identity changes are outside gameplay snapshots; inspect the
# original public result hashes separately instead of erasing arbitrary fields.
for case in ['actor', 'small', 'ice']:
    expected = by_name['objects_' + case + '_default']
    for profile in ['dev16', 'dev32']:
        current = by_name['objects_' + case + '_' + profile]
        assert current['step_results'] == expected['step_results'], (case, profile)
        assert [r['public_snapshots_sha256'] for r in current['final_states']] == [r['public_snapshots_sha256'] for r in expected['final_states']], (case, profile)
# Identity mismatch runs zero operations and never creates the proposed output.
mismatch_output = output / 'mismatch.jsonl'
mismatch_command = [str(binary), str(inputs / 'mismatch.json'), str(mismatch_output)]
mismatch = subprocess.run(mismatch_command, cwd=ROOT, capture_output=True)
with (output / 'mismatch.stdout').open('xb') as stream: stream.write(mismatch.stdout)
with (output / 'mismatch.stderr').open('xb') as stream: stream.write(mismatch.stderr)
assert mismatch.returncode != 0 and not mismatch_output.exists()
assert b'profile mismatch' in mismatch.stderr and b'absent-pack' not in mismatch.stderr
assert sha(binary) == binary_sha and source == identity()
assert all(sha(ROOT / path) == digest for path, digest in intake['files'].items())
with (output / 'SUMMARY.json').open('x') as stream:
    json.dump({'passed': True, 'source_sha256': source['sha256'], 'binary_sha256': binary_sha,
               'input_manifest_sha256': sha(inputs / 'INPUTS.json'), 'rows': rows,
               'mismatch': {'command': mismatch_command, 'exit_code': mismatch.returncode, 'output_absent': True,
                            'stderr_sha256': sha(output / 'mismatch.stderr')},
               'claim': 'Actual binary normal public playback, selected profile gate and raw resolved commands; allocator process window/RSS separate from native accounting. 26 finite cases, inherited object fixture choices explicit.'}, stream, indent=2)
print('PASS actual playback binary and finite identities/observer modes/profile parity')
