"""Read existing raw evidence only; no gameplay execution or target mutation."""
from pathlib import Path
import hashlib
import json
import statistics

B = Path(__file__).resolve().parent
ROOT = next(p for p in B.parents if (p / 'core_enhanced/src/lib.rs').exists())
E = B.parent.parent / 'execution/single_goal'
P = E / 'product_parts'

def read(p): return json.loads(p.read_text())
def ref(p):
    data = p.read_bytes()
    return {'path': str(p.relative_to(ROOT)), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
def write(name, value):
    with (B / name).open('x') as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write('\n')

calibrations = []
limits = {'elapsed_ns': 6_000_000_000, 'native_work': 233_000_000,
          'requested_heap_peak_bytes': 1280 * 1024**2, 'rss_bytes': 1536 * 1024**2}
for run in ['normal_standalone_calibration006', 'normal_server_calibration003', 'full60_combined_calibration002']:
    rows, cases, builds, refs = [], {}, {}, []
    directory = P / run / 'calibration_raw'
    for repeat in range(3):
        parity = []
        for mode in ['Plain', 'Instrumented']:
            path = directory / f'{repeat}-{mode}.json'
            d = read(path)
            proc = read(path.with_suffix('.process.json'))
            assert d['result'] == 'complete'
            assert proc['exit_code'] == 0 and proc['child_reaped'] and proc['success']
            assert not proc['stream_overflow'] and proc['stream_error'] is None and proc['supervisor_error'] is None
            assert proc['binary_sha256_before'] == proc['binary_sha256_after'] == d['report']['build']['executable_sha256']
            report = d['report']
            builds[mode] = report['build']
            assert len(report['samples']) == 4
            refs.extend([ref(path), ref(path.with_suffix('.process.json'))])
            signature = [{k: c[k] for k in ['id', 'class', 'command_sha256', 'error_sha256', 'gameplay_sha256', 'checkpoint_sha256', 'accepted_steps', 'preparation']}
                         for c in report['driver']['cases']]
            parity.append(signature)
            for c in signature:
                cases[c['id']] = {'accepted_steps': c['accepted_steps'], 'preparation': None if c['preparation'] is None else
                    {'roster_count': len(c['preparation']['employee_ids']), 'deployed_count': c['preparation']['deployed_count']}}
            for sample in report['samples']:
                assert sample['sample_count'] == 1 and sample['dropped_samples'] == 0 and sample['missing_samples'] == 0
                assert not sample['counter_overflowed'] and not sample['window_uncertain']
                # Each worker reports local repeat zero. The parent lab assigns
                # the invocation repeat index when collecting paired reports.
                assert sample['repeat_index'] == 0
                for key, limit in limits.items():
                    if mode == 'Plain' and key in ['requested_heap_peak_bytes', 'rss_bytes']:
                        assert sample[key] is None  # Uninstrumented control, not missing measured data.
                        continue
                    assert sample[key] is not None and sample[key] <= limit, (run, mode, key, sample[key])
                if mode == 'Instrumented' and sample['native_accounting'] is not None:
                    retained = sample['native_accounting'].get('retained')
                    if retained:
                        assert not retained['aggregation_overflowed'] and not retained['cumulative_request_overflowed'] and not retained['owner_request_overflowed']
                rows.append({'mode': mode, **sample, 'invocation_repeat_index': repeat})
        assert parity[0] == parity[1], (run, repeat, 'plain/native semantic parity')
    assert builds['Plain']['source_compiler_sha256'] == builds['Instrumented']['source_compiler_sha256']
    assert builds['Plain']['dependency_graph_sha256'] == builds['Instrumented']['dependency_graph_sha256']
    stats = []
    for mode in ['Plain', 'Instrumented']:
        for phase in sorted({r['phase'] for r in rows}):
            selected = [r for r in rows if r['mode'] == mode and r['phase'] == phase]
            assert len(selected) == 3
            stats.append({'mode': mode, 'phase': phase, 'n': 3,
                          'metrics': {k: {'min': min(r[k] for r in selected), 'median': statistics.median(r[k] for r in selected),
                                          'max': max(r[k] for r in selected)} for k in limits if selected[0][k] is not None},
                          'unmeasured_metrics': [k for k in limits if selected[0][k] is None]})
    calibrations.append({'run': run, 'workers': 6, 'samples': len(rows), 'complete_parity': True,
                         'cases': cases, 'builds': builds, 'stats': stats, 'raw_refs': refs})

heap = []
for name in ['empty', 'normal', 'expanded']:
    for index in range(3):
        path = P / 'g11_heap_samples001' / f'{name}.{index}.json'
        d, proc = read(path), read(path.with_suffix('.process.json'))
        assert proc['exit_code'] == 0 and proc['fresh_process'] and proc['reaped'] and not proc['timed_out']
        assert proc['result_sha256'] == ref(path)['sha256']
        assert d['passed'] and d['error'] is None and len(d['stages']) == 12
        release = d['release']
        assert release['exact_requested_bytes_recovered']
        assert release['warm_generation_baseline']['live_requested_bytes'] == release['final_after_all_owners']['live_requested_bytes']
        for stage in d['stages']:
            h = stage['heap']
            assert stage['operation_ok'] and not h['boundary_updates_unavailable'] and h['retiring_hooks_at_end'] == 0
            assert h['baseline']['instrumented'] and h['end']['instrumented']
            assert not h['baseline']['counter_overflowed'] and not h['end']['counter_overflowed']
            if not stage['native_scope_measured']:
                assert stage['native_work'] is None and stage['native_clone_slots'] is None
        heap.append({'raw_ref': ref(path), 'process_ref': ref(path.with_suffix('.process.json')),
                     'case': d['case'], 'sample_index': index, 'stages': [
                         {'name': s['name'], 'wall_seconds': s['wall_seconds'], 'peak_requested_bytes': s['heap']['peak_requested_bytes'],
                          'live_delta_bytes': s['heap']['live_delta_bytes'], 'native_work': s['native_work']} for s in d['stages']],
                     'released_to_warm_baseline_bytes': release['final_after_all_owners']['live_requested_bytes']})

crashes = []
directory = P / 'normal_persistence001/evidence/return_loan_story_crashes'
for case in sorted(p for p in directory.iterdir() if p.is_dir()):
    report = read(case / 'RESULT.json')
    crash = read(case / 'crash/exit.json')
    assert crash['exit'] == 86 and crash['mode'] == 'crash'
    recovered = []
    for index in range(2):
        p = case / f'recover_{index}'
        assert read(p / 'exit.json')['exit'] == 0
        result = read(p / 'RESULT.json')
        assert result['profile'] == report['expected']['profile'] and result['checkpoint'] == report['expected']['checkpoint']
        recovered.append(result)
    assert recovered[0]['files'] == recovered[1]['files']
    if not report['accepted']:
        for step in ['retry', 'retry_reopen']:
            assert read(case / step / 'exit.json')['exit'] == 0
        retry, reopen = read(case / 'retry/RESULT.json'), read(case / 'retry_reopen/RESULT.json')
        assert retry['profile'] == reopen['profile'] and retry['checkpoint'] == reopen['checkpoint'] and retry['files'] == reopen['files']
    crashes.append({'case': case.name, 'crash_exit': 86, 'recoveries': 2, 'accepted': report['accepted'],
                    'exact_expected_profile_checkpoint': True, 'second_recovery_files_identical': True,
                    'unaccepted_retry_checked': not report['accepted'], 'case_ref': ref(case / 'RESULT.json')})
assert len(crashes) == 22

write('RAW_REVIEW.json', {'schema': 'u12.remaining-raw-review/v1', 'new_gameplay_runs': 0,
    'calibrations': calibrations, 'unchanged_criterion': limits, 'heap_samples': heap, 'crash_cases': crashes,
    'limits': ['Existing raw observations reaggregated independently, not newly measured performance.',
               '500/1000 roster are bounded diagnostic fixtures with eight deployed actors, not normal gameplay roster limits.',
               'Native work, process requested heap and OS RSS are distinct; missing native scope is null, not zero.',
               'Expanded vault represents repeated accumulation of valid returned copy states; no new tamper defense requirement.',
               'No deployment sizing, optimization or whole-suite rerun claim.']})
print(json.dumps({'calibrations': len(calibrations), 'samples_each': 24, 'heap_children': len(heap),
                  'crashes': len(crashes), 'raw_checks_pass': True}))
