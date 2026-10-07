"""Fail-closed hashes, executed inputs, exact deltas and nonzero U08 evidence.

Semantic non-impact is a reviewed claim, never inferred from filenames or a
boolean compatibility flag. The exact changed rows and evidence remain visible.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re
import tarfile
from run import ROOT, HERE, identity, sha
from finalization_paths import S, C, paths

if not __debug__:
    raise RuntimeError('Finalization validation requires Python without -O')


def read(path):
    return json.loads(Path(path).read_text())


def write(path, value):
    with Path(path).open('x') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def resolve(name):
    p = Path(name)
    p = p if p.is_absolute() else ROOT / p
    p = p.resolve()
    assert p.is_relative_to(ROOT), p
    return p


def ref(path):
    path = resolve(path)
    return {'path': str(path.relative_to(ROOT)), 'bytes': path.stat().st_size,
            'sha256': sha(path)}


def check_ref(row):
    assert set(('path', 'sha256', 'bytes')) <= row.keys(), row
    assert ref(row['path']) == {k: row[k] for k in ('path', 'bytes', 'sha256')}, row['path']
    return resolve(row['path'])


def nonempty(value):
    assert isinstance(value, str) and value.strip() and 'TODO' not in value, value
    return value


def source_valid(value):
    rows = value['files']
    assert rows and len({r['path'] for r in rows}) == len(rows)
    # run.identity() sorts Path objects before constructing these exact rows.
    # Do not normalize/reorder the record: its digest owns the original order.
    assert rows == sorted(rows, key=lambda r: Path(r['path']))
    assert digest(rows) == value['sha256']
    for row in rows:
        assert set(row) == {'path', 'sha256', 'bytes'} and row['bytes'] >= 0
        assert re.fullmatch('[a-f0-9]{64}', row['sha256'])
        assert not Path(row['path']).is_absolute() and '..' not in Path(row['path']).parts


def passed_tests(text, *, single_threaded=False):
    """Parse completed libtest blocks without promoting ignored/missing tests.

With --nocapture a serial test's print starts immediately after `... ` and
libtest writes its final `ok` on a later line. Such a split is unambiguous only
for a one-test block or an explicitly serial command. Parallel ambiguous output
is refused; the caller must select a readable existing execution, never rewrite
its raw log or infer selector success from the summary alone.
    """
    block = None
    pending = None
    blocks = []
    outcomes = []
    start = re.compile(r'^running (\d+) tests?$')
    test = re.compile(r'^test (.+?) \.\.\.\s?(.*)$')
    summary = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;')

    def finish(name, status):
        assert block is not None
        assert name not in {row['selector'] for row in block['outcomes']}, 'Duplicate selector in one libtest block'
        row = {'selector':name, 'status':status, 'block':len(blocks)}
        block['outcomes'].append(row)
        outcomes.append(row)

    for line in text.splitlines():
        match = start.fullmatch(line)
        if match:
            assert block is None and pending is None, 'Nested or interrupted libtest block'
            block = {'running':int(match[1]), 'outcomes':[]}
            continue
        match = summary.match(line)
        if match:
            assert block is not None and pending is None, 'Unfinished selector before libtest summary'
            counts = dict(zip(('passed','failed','ignored','measured'), map(int, match.groups()[1:])))
            actual = {k:sum(row['status'] == status for row in block['outcomes'])
                      for k, status in [('passed','ok'),('failed','FAILED'),('ignored','ignored'),('measured','measured')]}
            assert actual == counts, ('Selector outcomes do not match summary', actual, counts)
            assert sum(counts.values()) == block['running']
            assert (match[1] == 'ok') == (counts['failed'] == 0)
            blocks.append(dict(block, counts=counts, summary_status=match[1]))
            block = None
            continue
        match = test.match(line)
        if match:
            assert block is not None and pending is None, 'Ambiguous interleaved test output'
            name, tail = match.groups()
            if tail in ('ok','FAILED'):
                finish(name, tail)
            elif re.fullmatch(r'ignored(?:, .*)?', tail):
                finish(name, 'ignored')
            elif tail.startswith('bench:'):
                finish(name, 'measured')
            else:
                assert block['running'] == 1 or single_threaded, 'Parallel split statuses are not attributable'
                pending = name
            continue
        if pending is not None and line in ('ok','FAILED','ignored'):
            finish(pending, line)
            pending = None
    assert block is None and pending is None, 'Interrupted libtest output'
    counts = {key:sum(b['counts'][key] for b in blocks) for key in ('passed','failed','ignored','measured')}
    return {'selectors':sorted({r['selector'] for r in outcomes if r['status'] == 'ok'}),
            'outcomes':outcomes, 'counts':counts, 'blocks':len(blocks)}


def archive_exact(path, rows):
    expected = {r['path']: r for r in rows}
    assert len(expected) == len(rows)
    seen = set()
    with tarfile.open(path) as archive:
        for member in archive:
            assert member.isfile() and member.name in expected and member.name not in seen, member.name
            row = expected[member.name]
            assert member.size == row['bytes']
            h = hashlib.sha256()
            with archive.extractfile(member) as stream:
                while chunk := stream.read(1024 * 1024):
                    h.update(chunk)
            assert h.hexdigest() == row['sha256'], member.name
            seen.add(member.name)
    assert seen == set(expected)


def delta(before, after):
    old = {r['path']: r for r in before['files']}
    new = {r['path']: r for r in after['files']}
    return [{'path': name, 'before': old.get(name), 'after': new.get(name)}
            for name in sorted(old.keys() | new.keys()) if old.get(name) != new.get(name)]


def currentness(executed, current, reuse):
    source_valid(executed)
    if executed == current:
        assert reuse is None, 'Exact source must not carry an unnecessary reuse exception'
        return {'kind': 'exact', 'source_sha256': current['sha256']}
    assert reuse is not None, 'Historical source requires an explicit per-receipt review'
    proof_path = check_ref(reuse)
    proof = read(proof_path)
    assert proof['source_before'] == executed and proof['source_after'] == current
    changes = delta(executed, current)
    assert proof['delta'] == changes and changes
    analyses = proof['changed_input_analyses']
    assert {a['path'] for a in analyses} == {a['path'] for a in changes}
    assert len(analyses) == len(changes)
    required = proof['required_inputs']
    assert required and proof['executed_selectors']
    before_rows = {r['path']: r for r in executed['files']}
    after_rows = {r['path']: r for r in current['files']}
    for row in required:
        assert row == before_rows.get(row['path']) == after_rows.get(row['path']), row['path']
    for row in analyses:
        nonempty(row['reason_for_no_effect_on_this_execution'])
        assert row['evidence']
        for item in row['evidence']:
            check_ref(item)
    nonempty(proof['reviewer'])
    nonempty(proof['claim_scope'])
    check_ref(proof['review_record'])
    return {'kind': 'reviewed_exact_delta', 'proof': ref(proof_path), 'delta': changes,
            'claim_scope': proof['claim_scope'], 'executed_selectors': proof['executed_selectors']}


def json_at(value, pointer):
    assert pointer == '' or pointer.startswith('/')
    for component in pointer.split('/')[1:]:
        component = component.replace('~1', '/').replace('~0', '~')
        value = value[int(component)] if isinstance(value, list) else value[component]
    return value


MEASUREMENT_SUPPORT_COMMAND = ['cargo', 'run', '--jobs=1', '-p', 'game_server',
    '--features', 'runtime-measurement', '--example', 'measurement_support', '--offline', '--locked']
MEASUREMENT_SUPPORT_FORMAT = 'measurement_support_stdout_json_line'


def measurement_support_value(receipt_value, raw):
    """Read the one process_heap JSON line from this exact successful example.

Cargo warnings stay in the immutable raw log. No counter, unavailable state,
number or timestamp is normalized, and a partial or ambiguous run is refused.
This parser does not execute the example or award current-source acceptance.
    """
    assert receipt_value['command'] == MEASUREMENT_SUPPORT_COMMAND, 'Wrong measurement support example/feature/command'
    assert receipt_value['exit_code'] == 0 and receipt_value['source_unchanged'] is True
    assert receipt_value['source_before'] == receipt_value['source_after']
    source_valid(receipt_value['source_before'])
    assert receipt_value['toolchain']['rustc'] and receipt_value['toolchain']['cargo']
    assert datetime.datetime.fromisoformat(receipt_value['started_at_utc']).tzinfo is not None
    import math
    assert math.isfinite(receipt_value['duration_seconds']) and receipt_value['duration_seconds'] >= 0
    assert hashlib.sha256(raw).hexdigest() == receipt_value['log_sha256'], 'Raw log differs from receipt'
    assert raw.endswith(b'\n'), 'println output is incomplete'
    lines = raw.decode('utf-8').splitlines()
    documents = []
    running = []

    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            assert key not in result, 'Duplicate JSON member'
            result[key] = value
        return result

    def invalid_constant(value):
        raise AssertionError('Nonfinite JSON constant: ' + value)

    for index, line in enumerate(lines, 1):
        match = re.fullmatch(r'\s+Running `([^`]+)`', line)
        if match:
            running.append((index, match[1]))
        text = line.strip()
        if not text:
            continue
        try:
            value = json.loads(text, object_pairs_hook=unique_object, parse_constant=invalid_constant)
        except json.JSONDecodeError:
            assert not text.startswith(('{', '[')), 'Malformed JSON-like stdout line'
        else:
            documents.append((index, line, value))
    assert len(documents) == 1, 'Expected exactly one JSON stdout line'
    index, line, value = documents[0]
    assert isinstance(value, dict), 'process_heap emits a JSON object'
    assert len(running) == 1 and running[0][0] < index, 'Missing/ambiguous actual example launch'
    binary = Path(running[0][1])
    if not binary.is_absolute():
        binary = Path(receipt_value['cwd']) / binary
    expected = Path(receipt_value['environment']['CARGO_TARGET_DIR']) / 'debug/examples/measurement_support'
    assert binary.resolve() == expected.resolve(), 'Raw launch is not the bound measurement support example'
    assert all(not tail.strip() for tail in lines[index:]), 'Unexpected output after the sole JSON result'
    assert value['instrumented'] is False
    assert value['live_requested_bytes'] is None and value['lifetime_peak_requested_bytes'] is None
    return {'value': value, 'line_number': index,
            'line_sha256': hashlib.sha256(line.encode('utf-8')).hexdigest(),
            'meaning': 'No installed allocator: instrumented=false and live/peak unavailable (null), never measured zero.'}


def assertion(row):
    path = check_ref(row['artifact'])
    format_name = row.get('format', 'json_document')
    extraction = None
    if format_name == MEASUREMENT_SUPPORT_FORMAT:
        receipt_path = check_ref(row['receipt'])
        assert path == receipt_path.with_suffix('.log'), 'Assertion must bind the receipt own raw log'
        extraction = measurement_support_value(read(receipt_path), path.read_bytes())
        value = extraction.pop('value')
        extraction.update(format=format_name, receipt=ref(receipt_path), raw_log=ref(path))
    else:
        assert format_name == 'json_document', 'Unknown assertion format'
        value = read(path)
    actual = json_at(value, row['pointer'])
    assert 'equals' in row and actual == row['equals'], (path, row['pointer'], actual)
    nonempty(row['meaning'])
    return dict(row, observed=actual, **({'extraction':extraction} if extraction is not None else {}))


def measurement_support_assertions(spec, checks):
    """Require the selected role's own receipt/raw and all three fixed states."""
    assert spec['id'] == 'measurement_support' and spec['kind'] == 'execution' and checks
    assert all(row.get('format') == MEASUREMENT_SUPPORT_FORMAT for row in checks)
    assert all(row['artifact'] == spec['raw_log'] and row['receipt'] == spec['receipt'] for row in checks), 'measurement_support must assert its own selected receipt and raw'
    required = {'/instrumented':False, '/live_requested_bytes':None, '/lifetime_peak_requested_bytes':None}
    for pointer, expected in required.items():
        assert any(row['pointer'] == pointer and row['equals'] is expected for row in checks), pointer


def load_config():
    parser = argparse.ArgumentParser()
    parser.add_argument('--config', required=True)
    args = parser.parse_args()
    config_path = resolve(args.config)
    cfg = read(config_path)
    assert cfg['schema'] == 1 and cfg['status'] == 'ready_for_execution', 'Prepared template is not executable'
    assert cfg['goal'] == 'GOAL-CRNV-01'
    cfg['_ref'] = ref(config_path)
    current = identity()
    source_valid(current)
    assert cfg['source'] == current, 'Config must bind exact current source membership and bytes'
    assert cfg['tool_inputs']
    for row in cfg['tool_inputs']:
        check_ref(row)
    own = {str(p.relative_to(ROOT)) for p in HERE.joinpath('tools').glob('*.py')
           if p.name in {'run.py','evidence_common.py','select_evidence.py','build_predecessor_impact.py',
                        'finalize_adaptation.py','coverage.py','coverage_contracts.py',
                        'finalize_gex.py','finalization_paths.py','freeze_current.py'}}
    own.add(str((HERE / 'tools/finalization_contracts.json').relative_to(ROOT)))
    assert own <= {r['path'] for r in cfg['tool_inputs']}, 'Bind every finalizer dependency'
    for row in cfg['basis']:
        check_ref(row)
    required_basis = {str((HERE / n).relative_to(ROOT)) for n in (
        'P2G_30_FREEZE_001.json','P2G_40_PHASE_PLAN_001.json','P2G_50_GOAL_SET.ko.md',
        'P2G_60_HANDOFF_001.json','CODE_REVIEW_001.ko.md','PREDECESSOR_INTAKE_001.json',
        'evidence/gex00_002/GEX00.json')}
    assert required_basis <= {r['path'] for r in cfg['basis']}, 'Exact frozen basis and actual GEX00_002 are required'
    names = paths(cfg)
    return cfg, current, names


def result_base(cfg, current):
    return {'schema': 1, 'goal': cfg['goal'], 'source_sha256': current['sha256'],
            'config': cfg['_ref'], 'tool_inputs': cfg['tool_inputs'],
            'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'independent_audit': False}


def receipt(spec, current):
    path = check_ref(spec['receipt'])
    r = read(path)
    log = check_ref(spec['raw_log'])
    assert log == path.with_suffix('.log') and sha(log) == r['log_sha256']
    assert r['source_unchanged'] and r['source_before'] == r['source_after']
    allowed = (0, 101) if spec['kind'] == 'r05' else (0,)
    assert r['exit_code'] in allowed
    state = currentness(r['source_before'], current, spec.get('reuse'))
    source_archive = HERE / r['source_archive']['path']
    assert sha(source_archive) == r['source_archive']['sha256']
    archive_exact(source_archive, r['source_before']['files'])
    runner = HERE / 'evidence/runner_versions' / (r['runner_sha256'] + '.py')
    assert sha(runner) == r['runner_sha256']
    assert r['command'] == spec['command'] and r['toolchain'] and r['duration_seconds'] >= 0
    inputs = []
    for item in r['command_inputs']:
        archived = HERE / item['archive']
        assert sha(archived) == item['sha256']
        inputs.append(dict(item, archived_ref=ref(archived)))
    text = log.read_text(errors='replace')
    parsed = {'selectors':[], 'outcomes':[], 'counts':{'passed':0,'failed':0,'ignored':0,'measured':0}, 'blocks':0}
    if spec['kind'] in ('suite','focused'):
        command = r['command']
        serial = '--test-threads=1' in command or any(command[i:i+2] == ['--test-threads','1'] for i in range(len(command)))
        parsed = passed_tests(text, single_threaded=serial)
    selectors, count = parsed['selectors'], parsed['counts']['passed']
    if spec['kind'] in ('suite', 'focused'):
        assert selectors and count > 0 and parsed['counts']['failed'] == 0 and parsed['blocks'] > 0
        assert spec['expected_passed_count'] == count
        assert set(spec['required_selectors']) <= set(selectors)
        if spec['kind'] == 'focused':
            assert spec['required_selectors'], 'Focused execution must name the exact passed selector'
        if state['kind'] != 'exact':
            assert set(state['executed_selectors']) <= set(selectors)
    elif spec['kind'] == 'execution':
        assert spec['execution_assertions'], 'A zero-test wrapper needs actual output assertions'
    else:
        assert spec['kind'] in ('check', 'build', 'r05')
    checks = [assertion(row) for row in spec.get('execution_assertions', [])]
    if spec['id'] == 'measurement_support':
        measurement_support_assertions(spec, checks)
    return dict(ref(path), id=spec['id'], kind=spec['kind'], raw_log=ref(log),
                actual_source_sha256=r['source_before']['sha256'], currentness=state,
                exit_code=r['exit_code'], passed_count=count, selectors=selectors, libtest=parsed,
                command=r['command'], inputs=inputs, runner=ref(runner),
                source_archive=ref(source_archive), assertions=checks,
                started_at_utc=r['started_at_utc'], duration_seconds=r['duration_seconds'])


def selected(cfg, current, names):
    value = read(names['selection'])
    assert value['config'] == cfg['_ref'] and value['source_sha256'] == current['sha256']
    for row in value['receipt_bindings']:
        check_ref(row)
        check_ref(row['raw_log'])
        check_ref(row['runner'])
        check_ref(row['source_archive'])
        for item in row['inputs']:
            check_ref(item['archived_ref'])
    check_ref(value['execution_harness_binding'])
    for row in value['artifacts'].values():
        check_ref(row)
        for item in row['raw_files']:
            check_ref(item)
    for bundle in value['raw_bundles']:
        actual = [ref(p) for p in sorted((ROOT / bundle['root']).rglob('*')) if p.is_file()]
        assert actual == bundle['files']
    return value


def load_stage(name, cfg, current, names):
    value = read(names[name])
    assert value['config'] == cfg['_ref'] and value['source_sha256'] == current['sha256']
    return value


def evidence_claim(claim, selection):
    nonempty(claim['claim'])
    nonempty(claim['oracle_independence'])
    assert claim['oracle_evidence'] and claim['input_refs']
    for row in claim['oracle_evidence'] + claim['input_refs']:
        check_ref(row)
    assert isinstance(claim['limitations'], list)
    by_id = {r['id']: r for r in selection['receipt_bindings']}
    ids = claim['receipt_ids']
    assert ids and set(ids) <= by_id.keys()
    selectors = claim['selectors']
    pool = {s for name in ids for s in by_id[name]['selectors']}
    assert set(selectors) <= pool
    assertions = [assertion(row) for row in claim['assertions']]
    assert selectors or assertions, 'A generic receipt reference is not direct acceptance evidence'
    for name in ids:
        binding = by_id[name]
        assert binding['kind'] in ('suite', 'focused', 'execution', 'r05'), name
        if binding['currentness']['kind'] != 'exact':
            used = set(selectors) & set(binding['selectors'])
            assert used <= set(binding['currentness']['executed_selectors'])
    assert claim['expected_observed'], 'Name the behavior, oracle and observed effect'
    for item in claim['expected_observed']:
        nonempty(item['behavior'])
        nonempty(item['oracle'])
        nonempty(item['observed'])
        assert item.get('selector') in selectors or item.get('assertion_index') in range(len(assertions))
    return dict(claim, assertions=assertions,
                receipt_bindings=[{k: by_id[n][k] for k in ('id','path','sha256','raw_log','currentness')} for n in ids])
