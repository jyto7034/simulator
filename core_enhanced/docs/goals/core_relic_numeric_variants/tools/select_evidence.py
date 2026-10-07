"""Select explicit executions and hash every declared raw/input bundle.

Run only after all functional workers and writers have stopped, then PC/R05
have completed. This tool does not execute tests or manufacture test verdicts.
"""
import datetime
from pathlib import Path
from evidence_common import (ROOT, HERE, check_ref, read, write, ref, load_config,
    result_base, receipt, assertion, identity, nonempty)
from coverage_contracts import REQUIRED_ROLES, REQUIRED_ARTIFACTS
from execution_binding import validate_execution_binding


def instant(value):
    return datetime.datetime.fromisoformat(value.replace('Z', '+00:00'))


def end(row):
    return instant(row['started_at_utc']) + datetime.timedelta(seconds=row['duration_seconds'])


def main():
    cfg, current, names = load_config()
    specs = cfg['receipts']
    assert len({s['id'] for s in specs}) == len(specs)
    by_id = {s['id']: s for s in specs}
    assert REQUIRED_ROLES.keys() <= by_id.keys(), sorted(REQUIRED_ROLES.keys() - by_id.keys())
    for role, kind in REQUIRED_ROLES.items():
        assert by_id[role]['kind'] == kind, role
        assert by_id[role].get('reuse') is None, 'Final required matrix must use exact current source'
    bindings = [receipt(spec, current) for spec in specs]
    bound = {row['id']: row for row in bindings}
    assert len({r['path'] for r in bindings}) == len(bindings), 'Do not count one receipt twice'
    harness_binding_path = check_ref(cfg['execution_harness_binding'])
    harness_binding = read(harness_binding_path)
    helper_ref = ref(HERE / 'tools/execution_binding.py')
    assert helper_ref in cfg['tool_inputs'], 'Bind the execution binding verifier itself'
    validate_execution_binding(harness_binding, current, bindings, REQUIRED_ROLES, ROOT)
    artifacts = {}
    for key, spec in cfg['artifacts'].items():
        path = check_ref(spec['ref'])
        producer = spec['producer_receipt_id']
        assert producer in bound and spec['checks'], key
        checks = [assertion(dict(row, artifact=spec['ref'])) for row in spec['checks']]
        raw = spec['raw_files']
        assert raw, 'Each selected result binds actual raw/input files'
        for row in raw:
            check_ref(row)
        artifacts[key] = dict(ref(path), producer_receipt_id=producer, checks=checks, raw_files=raw)
    assert REQUIRED_ARTIFACTS <= artifacts.keys(), sorted(REQUIRED_ARTIFACTS - artifacts.keys())
    bundles = []
    for bundle in cfg['raw_bundles']:
        root = ROOT / bundle['root']
        assert root.resolve().is_relative_to(ROOT) and root.is_dir()
        actual = [ref(p) for p in sorted(root.rglob('*')) if p.is_file()]
        assert actual == bundle['files'], 'Raw directory membership or bytes changed'
        assert actual
        bundles.append(bundle)
    assert bundles, 'Bind raw directory membership, not just summary booleans'
    bundled = {r['path'] for b in bundles for r in b['files']}
    assert {r['path'] for a in artifacts.values() for r in a['raw_files']} <= bundled
    # Fresh build receipts must precede all consumers. Main release and debug
    # executables retain exact hashes even when fault examples rebuild separately.
    binary_bindings = {}
    for profile, expected in [('release', {'author_content','package_official_content','consume_content_pack','game_server'}),
                              ('debug', {'author_content','measure_runtime','game_server'})]:
        value = read(ROOT / artifacts[profile + '_binding']['path'])
        assert value['passed'] is True and value['profile'] == profile
        assert value['source_sha256'] == current['sha256']
        build = bound[profile + '_build']
        assert Path(value['build_receipt']).resolve() == (ROOT / build['path']).resolve()
        assert value['build_receipt_sha256'] == build['sha256']
        assert {Path(p).name for p in value['binaries']} == expected
        assert len(value['binaries']) == len(expected)
        for path, digest in value['binaries'].items():
            from run import sha
            assert sha(Path(path)) == digest, 'Bound executable changed'
        consumers = cfg['binary_consumers'][profile]
        assert consumers
        for role in consumers:
            assert role in bound and end(build) <= instant(bound[role]['started_at_utc'])
        binary_bindings[profile] = value
    public_roles = {r for r in REQUIRED_ROLES if r.startswith('public_')}
    assert public_roles <= set(cfg['binary_consumers']['release'])
    assert {'playback_actual','lifecycle_actual','host_actual'} <= set(cfg['binary_consumers']['debug'])
    performance_roles = {'pc_prepare','pc','pc_gameplay','pc_cost','r05_prepare','r05','r05_summary','r05_dependencies'}
    assert set(cfg['performance_receipt_ids']) == performance_roles
    functional = [row for row in bindings if row['id'] not in performance_roles]
    assert functional
    for phase, role in [('pc','pc'), ('r05','r05')]:
        measured = bound[role]
        assert end(bound[phase + '_prepare']) <= instant(measured['started_at_utc'])
        assert all(end(row) <= instant(measured['started_at_utc']) for row in functional), 'Functional work overlaps measurement'
        environment = read(check_ref(cfg['performance_environments'][phase]))
        assert environment['source_sha256'] == current['sha256']
        assert environment['phase'] == phase and environment['own_cargo_rustc_active'] is False
        if phase == 'r05':
            assert environment['r05_target_existed_before_measurement'] is False
        nonempty(environment['timing_claim'])
        proof_path = check_ref(cfg['functional_quiescence'][phase])
        proof = read(proof_path)
        assert proof['source_sha256'] == current['sha256']
        assert proof['active_input_writers'] == proof['active_functional_runners'] == 0
        assert {r['path'] for r in proof['functional_receipts']} == {r['path'] for r in functional}
        for row in proof['functional_receipts']:
            check_ref(row)
        assert instant(proof['recorded_at_utc']) <= instant(measured['started_at_utc'])
        assert all(end(row) <= instant(proof['recorded_at_utc']) for row in functional)
    assert max(end(bound[r]) for r in ('pc','pc_gameplay','pc_cost')) <= instant(bound['r05']['started_at_utc']), 'Performance runs must be sequential'
    result = dict(result_base(cfg, current), target=cfg['target'], receipt_bindings=bindings,
                  artifacts=artifacts, raw_bundles=bundles, binary_bindings=binary_bindings,
                  performance_environments=cfg['performance_environments'],
                  functional_quiescence=cfg['functional_quiescence'],
                  execution_harness_binding=ref(harness_binding_path),
                  status='executed evidence selected; acceptance evaluation still pending')
    assert identity() == current
    write(names['selection'], result)
    print('Selected explicit U08 receipts, raw hashes and exact build/source bindings')


if __name__ == '__main__':
    main()
