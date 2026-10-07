"""Verify execution input provenance; never execute a receipt or award a verdict.

Legacy flat bindings keep their original checks. Composite v1 derives each
boundary from immutable snapshots. Its union is only a membership summary.
"""
import datetime
import hashlib
import json
import math
from pathlib import Path
import re
import tarfile

COMPOSITE = 'U08_COMPOSITE_EXECUTION_BINDING_V1'
SNAPSHOT = 'U08_EXECUTION_INPUT_SNAPSHOT_V1'
FINALIZERS = {'evidence_common.py', 'select_evidence.py', 'execution_binding.py',
              'build_predecessor_impact.py', 'finalize_adaptation.py', 'coverage.py',
              'coverage_contracts.py', 'finalize_gex.py', 'finalization_paths.py',
              'freeze_current.py', 'finalization_contracts.json'}
TRANSITIVE = {
    'core_enhanced/docs/goals/core_independent_objects/tools/prepare_public_inputs.py',
    'core_enhanced/docs/goals/core_runtime_safety_observability/tools/public_api_cli_probe.py',
    'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/compare_pc_v32.py'}


# Closed historical boundaries from the approved composite design.
HISTORICAL_SNAPSHOTS = {'007': {'before': {'path': 'core_enhanced/docs/goals/core_relic_numeric_variants/evidence/execution_harness_binding007/BEFORE.json', 'bytes': 1522218, 'sha256': '3331e5cd407de8d586f1ca7820fa3a1fd667dbdeab8f457ad7b0be50063803bf'}, 'after': {'path': 'core_enhanced/docs/goals/core_relic_numeric_variants/evidence/execution_harness_binding007/AFTER.json', 'bytes': 1561021, 'sha256': '562e8a0267c45a9cf45eccb990fff55159c2dd69c7e04b38a738b7896b1b4ce7'}}, '008': {'before': {'path': 'core_enhanced/docs/goals/core_relic_numeric_variants/evidence/execution_harness_binding008/BEFORE.json', 'bytes': 1692887, 'sha256': '666ed18bc5b72c0280daa37d412a005271e5838bf992cca7591371acf27f1e75'}, 'after': {'path': 'core_enhanced/docs/goals/core_relic_numeric_variants/evidence/execution_harness_binding008/AFTER.json', 'bytes': 1640843, 'sha256': '15655d4ddb2fbf27279cb4bccc0ad1c1e71d7c46d42bb8aa145a54746f0893bf'}}}


def instant(value):
    assert isinstance(value, str) and re.fullmatch(
        r'\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?(?:Z|\+00:00)', value), 'UTC timestamp required'
    return datetime.datetime.fromisoformat(value.replace('Z', '+00:00'))


def end(row):
    duration = row['duration_seconds']
    assert type(duration) in (int, float) and math.isfinite(duration) and duration >= 0
    return instant(row['started_at_utc']) + datetime.timedelta(seconds=duration)


def nonempty(value):
    assert isinstance(value, str) and value.strip() and 'TODO' not in value


class Reader:
    def __init__(self, root):
        self.root = Path(root).resolve()
        self._names = {}

    def name(self, value, *, absolute=False):
        p = Path(value)
        assert '..' not in p.parts
        assert absolute or (not p.is_absolute() and p.as_posix() == value)
        if value not in self._names:
            p = (self.root / p).resolve()
            assert p.is_relative_to(self.root)
            self._names[value] = p.relative_to(self.root).as_posix()
        return self._names[value]

    def check(self, row):
        name = self.name(row['path'])
        assert type(row['bytes']) is int and row['bytes'] >= 0
        assert re.fullmatch('[a-f0-9]{64}', row['sha256'])
        path = self.root / name
        data = path.read_bytes()
        assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'], name
        return path

    def read(self, row):
        def unique(pairs):
            result = {}
            for key, value in pairs:
                assert key not in result, 'Duplicate JSON key'
                result[key] = value
            return result
        return json.loads(self.check(row).read_text(), object_pairs_hook=unique)

    def rows(self, rows, *, live=True):
        assert isinstance(rows, list) and rows
        names = []
        for row in rows:
            assert set(row) == {'path', 'bytes', 'sha256'}
            names.append(self.name(row['path']))
            assert type(row['bytes']) is int and row['bytes'] >= 0
            assert re.fullmatch('[a-f0-9]{64}', row['sha256'])
            assert Path(row['path']).name not in FINALIZERS, 'Finalizer in execution inputs'
            if live:
                self.check(row)
        assert len(set(names)) == len(names), 'Duplicate input path'
        assert rows == sorted(rows, key=lambda r: Path(r['path'])), 'Input order differs'
        return {r['path']: r for r in rows}

    def archive(self, value, rows):
        # The current fixed adapters expose an exact flat archive for every set.
        assert value['exact_membership'] is True
        assert value['member_count'] == len(rows) and value['members'] == rows
        expected = {r['path']: r for r in rows}
        seen = set()
        with tarfile.open(self.check(value['artifact'])) as archive:
            for member in archive:
                assert member.isfile() and member.name in expected and member.name not in seen
                data = archive.extractfile(member).read()
                row = expected[member.name]
                assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256']
                seen.add(member.name)
        assert seen == expected.keys()

    def external_archive(self, value):
        # Historical archives intentionally bind only reviewed required members.
        rows = value['members']
        expected = {r['path']: r for r in rows}
        assert rows and len(expected) == len(rows) == value['member_count']
        seen = set()
        with tarfile.open(self.check(value['artifact'])) as archive:
            for member in archive:
                if member.name not in expected:
                    assert value['exact_membership'] is False
                    continue
                assert member.isfile() and member.name not in seen
                data = archive.extractfile(member).read()
                row = expected[member.name]
                assert len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256']
                seen.add(member.name)
        assert seen == expected.keys()


def _legacy(binding, current, bindings, required, reader):
    # Original single-interval checks, retained without a composite fallback.
    assert binding['source_sha256'] == current['sha256']
    prepared = reader.read(binding['prepared_manifest'])
    assert prepared['source'] == current, 'Fresh handoff must bind the final product/test source'
    prepared_files = [{k: r[k] for k in ('path', 'bytes', 'sha256')} for r in prepared['files']]
    before_tools, after_tools = binding['files_before'], binding['files_after']
    assert before_tools == after_tools == prepared_files and len(prepared_files) >= 45
    assert len({r['path'] for r in before_tools}) == len(before_tools)
    assert TRANSITIVE <= {r['path'] for r in before_tools}
    assert {reader.name(i['path'], absolute=True) for r in bindings if r['id'] in required
            for i in r['inputs'] if i['path'].endswith('.py')} <= {r['path'] for r in before_tools}
    assert not {Path(r['path']).name for r in before_tools} & FINALIZERS
    for row in before_tools:
        reader.check(row)
    assert binding['existing_harness_writers_active'] == 0
    nonempty(binding['recorded_by'])
    reader.check(binding['handoff_review'])
    bound = {r['id']: r for r in bindings}
    execution_refs = {r['id']: r['receipt'] for r in binding['receipts']}
    assert required <= execution_refs.keys() <= bound.keys()
    for role, expected in execution_refs.items():
        assert expected == {k: bound[role][k] for k in ('path', 'bytes', 'sha256')}
        assert instant(binding['verified_before_utc']) <= instant(bound[role]['started_at_utc'])
        assert end(bound[role]) <= instant(binding['verified_after_utc'])


def _snapshot_pair(spec, current, reader):
    assert set(spec) == {'id', 'adapter', 'before', 'after'}
    nonempty(spec['id'])
    before, after = reader.read(spec['before']), reader.read(spec['after'])
    adapter = spec['adapter']
    if adapter in ('u08_007_base', 'u08_008_base', 'u08_008_probe002'):
        recovery = adapter != 'u08_007_base'
        historical = '008' if recovery else '007'
        for phase in ('before', 'after'):
            assert spec[phase] == HISTORICAL_SNAPSHOTS[historical][phase], 'Historical snapshot identity changed'
        assert before['schema'] == ('U08_ACTUAL_RECOVERY_EXECUTION_BEFORE_008' if recovery else 'U08_ACTUAL_EXECUTION_BEFORE_007')
        assert after['schema'] == ('U08_ACTUAL_RECOVERY_AFTER_008_FAILED' if recovery else 'U08_ACTUAL_EXECUTION_AFTER_007')
        assert after['before'] == spec['before']
        start = instant(before['verified_before_utc'])
        finish = instant(after['actual_after_utc'] if recovery else after['verified_after_utc'])
        probe = adapter == 'u08_008_probe002'
        files_before = before['probe002115_before'] if probe else before['files_before']
        files_after = after['probe002115_after'] if probe else after['files_after']
        manifest_ref = before['probe002_manifest'] if probe else before['prepared_manifest']
        if not recovery:
            assert after['prepared_manifest'] == manifest_ref
        archive_before = before['probe002_archive_before'] if probe else before['flat_input_archive_before']
        archive_after = after['probe002_archive_after'] if probe else after['flat_input_archive_after']
        assert len(files_before) == (115 if probe else 1322), 'Historical membership cannot expand'
        review = before['root032'] if recovery else before['handoff_review']
    else:
        assert adapter == 'snapshot_v1', 'Unknown snapshot adapter'
        assert before['schema'] == after['schema'] == SNAPSHOT
        assert before['phase'] == 'before' and after['phase'] == 'after'
        assert after['before'] == spec['before']
        start, finish = instant(before['verified_at_utc']), instant(after['verified_at_utc'])
        files_before, files_after = before['files'], after['files']
        manifest_ref = before['prepared_manifest']
        assert after['prepared_manifest'] == manifest_ref
        archive_before, archive_after = before['archive'], after['archive']
        review = before['handoff_review']
    assert start < finish, 'Reversed or empty execution interval'
    assert before['source'] == after['source'] == current
    for value in (before, after):
        if 'source_sha256' in value:
            assert value['source_sha256'] == current['sha256']
        assert type(value['active_input_writers']) is int and value['active_input_writers'] == 0
        if 'existing_harness_writers_active' in value:
            assert value['existing_harness_writers_active'] == 0
    reader.check(review)
    external_before = before.get('external_inputs_before', before.get('external_inputs', []))
    external_after = after.get('external_inputs_after', after.get('external_inputs', []))
    if adapter != 'snapshot_v1':
        assert len(external_before) == 2, 'Original external archive boundary is required'
    assert external_before == external_after, 'External input archives changed'
    for value in external_before:
        reader.external_archive(value)
    assert files_before == files_after, 'Input bytes changed during execution'
    rows = reader.rows(files_before)
    manifest = reader.read(manifest_ref)
    assert manifest['files'] == files_before
    if adapter == 'u08_008_probe002':
        assert manifest['source_sha256'] == current['sha256']
    else:
        assert manifest['source'] == current
    reader.archive(archive_before, files_before)
    reader.archive(archive_after, files_after)
    assert archive_before['artifact'] == archive_after['artifact']
    return dict(id=spec['id'], start=start, end=finish, files=rows,
                before=spec['before'], after=spec['after'], review=review,
                external={r['artifact']['path']: r['artifact'] for r in external_before})


def _merge(target, rows):
    for name, row in rows.items():
        assert name not in target or target[name] == row, 'Conflicting common input bytes'
        target[name] = row


def _composite(binding, current, bindings, required, reader):
    fields = {'schema', 'source_sha256', 'recorded_by', 'handoff_review', 'receipts',
              'prepared_manifest', 'union_recorded_at_utc', 'intervals', 'gaps',
              'dependency_review', 'existing_harness_writers_active'}
    assert fields <= binding.keys() <= fields | {'execution_envelope'}, 'Unknown or mixed composite fields'
    assert binding['source_sha256'] == current['sha256']
    nonempty(binding['recorded_by'])
    reader.check(binding['handoff_review'])
    assert type(binding['existing_harness_writers_active']) is int and binding['existing_harness_writers_active'] == 0
    bound = {r['id']: r for r in bindings}
    assert len(bound) == len(bindings) and required <= bound.keys()
    refs = binding['receipts']
    assert len({r['id'] for r in refs}) == len(refs) and {r['id'] for r in refs} == bound.keys()
    for row in refs:
        assert set(row) == {'id', 'receipt'}
        assert row['receipt'] == {k: bound[row['id']][k] for k in ('path', 'bytes', 'sha256')}
        reader.check(row['receipt'])
    prepared = reader.read(binding['prepared_manifest'])
    assert set(prepared) == {'schema', 'source', 'files', 'recorded_at_utc'}
    assert prepared['schema'] == 'U08_EXECUTION_INPUT_UNION_V1' and prepared['source'] == current
    assert prepared['recorded_at_utc'] == binding['union_recorded_at_utc']
    union_time = instant(prepared['recorded_at_utc'])
    union = reader.rows(prepared['files'], live=False)
    assert TRANSITIVE <= union.keys(), 'Missing reviewed transitive inputs'
    dependency = reader.read(binding['dependency_review'])
    assert set(dependency) == {'schema', 'source_sha256', 'reviewer', 'review_record', 'roles'}
    assert dependency['schema'] == 'U08_EXECUTION_DEPENDENCIES_V1'
    assert dependency['source_sha256'] == current['sha256']
    nonempty(dependency['reviewer'])
    reader.check(dependency['review_record'])
    deps = {r['id']: r for r in dependency['roles']}
    assert len(deps) == len(dependency['roles']) and deps.keys() == bound.keys()
    intervals, all_files, assigned, input_ids = [], {}, set(), set()
    for interval in binding['intervals']:
        assert set(interval) == {'snapshot', 'input_sets', 'roles'}
        primary = _snapshot_pair(interval['snapshot'], current, reader)
        assert primary['id'] not in {i['id'] for i in intervals}
        sets = [primary] + [_snapshot_pair(s, current, reader) for s in interval['input_sets']]
        available = {}
        for item in sets:
            assert item['id'] not in input_ids, 'Duplicate input-set ID'
            input_ids.add(item['id'])
            _merge(available, item['files'])
            _merge(all_files, item['files'])
            assert item['end'] <= union_time, 'Union predates actual input observation'
        roles = interval['roles']
        assert roles and len(set(roles)) == len(roles) and not assigned.intersection(roles)
        assigned.update(roles)
        assert set(roles) <= bound.keys()
        for role in roles:
            row = bound[role]
            # receipt() owns the existing r05 expected-101 exception and raw checks.
            assert row['exit_code'] in ((0, 101) if row['kind'] == 'r05' else (0,))
            assert row['actual_source_sha256'] == current['sha256']
            start, finish = instant(row['started_at_utc']), end(row)
            assert primary['start'] <= start <= finish <= primary['end'], 'Role outside assigned interval'
            coverage = {}
            for item in sets:
                if item['start'] <= start and finish <= item['end']:
                    _merge(coverage, item['files'])
            dep = deps[role]
            assert set(dep) == {'id', 'inputs', 'reason'}
            nonempty(dep['reason'])
            dep_rows = reader.rows(dep['inputs'], live=False) if dep['inputs'] else {}
            for name, value in dep_rows.items():
                assert coverage.get(name) == value, 'Dependency lacks timely snapshot coverage'
            for item in row['inputs']:
                if item['path'].endswith('.py'):
                    name = reader.name(item['path'], absolute=True)
                    assert name in dep_rows, 'Python input absent from reviewed dependencies'
                    assert coverage.get(name) == dep_rows[name] and coverage[name]['sha256'] == item['sha256'], 'Python hash/time differs'
        primary['available'] = available
        intervals.append(primary)
    assert intervals and assigned == bound.keys(), 'Missing selected role'
    assert all_files == union, 'Union membership differs from actual input sets'
    gaps = binding['gaps']
    assert len(gaps) == len(intervals) - 1
    for prior, following, gap in zip(intervals, intervals[1:], gaps):
        assert prior['end'] <= following['start'], 'Overlapping or unordered intervals'
        assert set(gap) == {'from', 'to', 'after', 'before', 'started_at_utc', 'ended_at_utc',
                            'reason', 'authorization', 'added', 'removed'}
        assert gap['from'] == prior['id'] and gap['to'] == following['id']
        assert gap['after'] == prior['after'] and gap['before'] == following['before']
        assert instant(gap['started_at_utc']) == prior['end'] and instant(gap['ended_at_utc']) == following['start']
        nonempty(gap['reason'])
        assert gap['authorization'] == following['review'], 'Gap authorization must match actual next handoff'
        reader.check(gap['authorization'])
        old_external, new_external = prior['external'], following['external']
        assert old_external == new_external, 'External archive changes need a new reviewed schema'
        old, new = prior['available'], following['available']
        assert all(old[n] == new[n] for n in old.keys() & new.keys())
        assert gap['added'] == [new[n] for n in sorted(new.keys() - old.keys(), key=Path)]
        assert gap['removed'] == [old[n] for n in sorted(old.keys() - new.keys(), key=Path)]
    if 'execution_envelope' in binding:
        envelope = binding['execution_envelope']
        assert set(envelope) == {'earliest_before_utc', 'latest_after_utc', 'scope'}
        assert instant(envelope['earliest_before_utc']) == intervals[0]['start']
        assert instant(envelope['latest_after_utc']) == intervals[-1]['end']
        assert envelope['scope'] == 'interval_envelope_only_not_union_first_observation'


def validate_execution_binding(binding, current, bindings, required_roles, root):
    """Validate already receipt()-checked selections, with no external side effects."""
    assert __debug__, 'Validation requires Python without -O'
    reader = Reader(root)
    marker = binding.get('schema')
    if marker == COMPOSITE:
        _composite(binding, current, bindings, set(required_roles), reader)
    else:
        assert marker is None or type(marker) is int and marker == 1, 'Unknown execution binding schema'
        assert not {'intervals', 'input_sets', 'gaps', 'union_recorded_at_utc',
                    'execution_envelope', 'dependency_review', 'version'} & binding.keys(), 'Mixed legacy/composite binding'
        _legacy(binding, current, bindings, set(required_roles), reader)
