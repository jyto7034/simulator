"""Strict immediate-U06 PC parity plus the measured empty-owner extent delta.

Only elapsed_ns is omitted. Keep original raw data, caps, workload and failed
runs. Unknown costs or refusal changes fail rather than gaining an exception.
"""
import argparse
import collections
import copy
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path('/mnt/f/work/simulator')
C = Path(__file__).resolve().parent
H = ROOT / 'core_enhanced/docs/goals/core_independent_objects'
sys.path.insert(0, str(H / 'tools'))
from run import identity

BEFORE = ROOT / 'core_enhanced/docs/goals/core_battle_starter_tactics/execution/single_goal/compatibility/current_001/evidence/pc_raw_current_001.json'
FAULT = re.compile(r'BattleRuntimeFault\(BattleRuntimeFault \{ code: (\w+), site: "([^"]+)", limit: Some\((\d+)\), requested: Some\((\d+)\) \}\)')
CONDITION_FIELDS = ('family', 'allies', 'enemies', 'units', 'chunk_ms', 'burst_sources',
                    'source_equivalent', 'target_equivalent', 'reads_per_expr',
                    'members', 'repetitions', 'grouped_per_tile')
LIVE_FAMILIES = {'literal_battle_public_step', 'damage_burst_private_transaction',
                 'live_adapter_repetition_synthetic'}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(test, detail):
    if not test:
        raise AssertionError(detail)


def compare(before, after, layout):
    require(layout['runtime_source_bytes'] == 136, ('runtime source layout', layout))
    require(layout['object_state_bytes'] == 152, ('object owner layout', layout))
    require(layout['placement_domain_option_bytes'] == 48, ('placement option layout', layout))
    delta = layout['object_state_bytes'] + layout['placement_domain_option_bytes']
    require(delta == 200, ('unreviewed layout delta', delta))
    require({k: v for k, v in before.items() if k != 'rows'} ==
            {k: v for k, v in after.items() if k != 'rows'} == {'schema': 1, 'seed': 17}, 'header')
    require(len(before['rows']) == len(after['rows']) == 576, 'row count')
    costs, repeats, conditions = [], {}, collections.Counter()
    for index, (old, new) in enumerate(zip(before['rows'], after['rows'])):
        left, right = copy.deepcopy(old), copy.deepcopy(new)
        changes = []
        for stage in ('setup', 'measurement'):
            if stage in left:
                require(stage in right, (index, stage, 'missing'))
                left[stage].pop('elapsed_ns')
                right[stage].pop('elapsed_ns')
            else:
                require(stage not in right, (index, stage, 'added'))
        if 'native_usage' in left:
            require(old['family'] in LIVE_FAMILIES, (index, 'unknown native family'))
            old_usage, new_usage = left['native_usage'], right['native_usage']
            flat = bool(old_usage) and isinstance(old_usage[0], int)
            olds = [old_usage] if flat else old_usage
            news = [new_usage] if flat else new_usage
            require(len(olds) == len(news), (index, 'native operation count'))
            for operation, (x, y) in enumerate(zip(olds, news)):
                require(len(x) == len(y) == 3 and x[:2] == y[:2]
                        and y[2] == x[2] + delta,
                        (index, 'native work/clone/retained', operation, x, y))
            changes.append({'field': 'native_usage', 'before': old_usage, 'after': new_usage,
                            'expected_per_operation_delta': [0, 0, delta]})
            right['native_usage'] = copy.deepcopy(old_usage)
        if 'retained_extent_at_end' in left:
            require(old['family'] in LIVE_FAMILIES and
                    right['retained_extent_at_end'] == left['retained_extent_at_end'] + delta,
                    (index, 'retained extent', left['retained_extent_at_end'], right['retained_extent_at_end']))
            changes.append({'field': 'retained_extent_at_end', 'before': left['retained_extent_at_end'],
                            'after': right['retained_extent_at_end'], 'delta': delta})
            right['retained_extent_at_end'] = left['retained_extent_at_end']
        if left['error'] is None:
            require(right['error'] is None, (index, 'new refusal', right['error']))
        else:
            x, y = FAULT.fullmatch(left['error']), FAULT.fullmatch(right['error'] or '')
            require(x is not None and y is not None, (index, 'typed error grammar'))
            require(x.groups()[:3] == y.groups()[:3] ==
                    ('RetainedByteLimit', 'value_expr_evaluation_retained', '8388608'),
                    (index, 'code/site/cap', x.groups(), y.groups()))
            require(int(y[4]) == int(x[4]) + delta, (index, 'requested delta', x[4], y[4]))
            changes.append({'field': 'error.requested', 'before': int(x[4]), 'after': int(y[4]),
                            'delta': delta, 'code': y[1], 'site': y[2], 'cap': int(y[3])})
            right['error'] = left['error']
        require(left == right, (index, old['family'],
                {k: [left.get(k), right.get(k)] for k in left.keys() | right.keys() if left.get(k) != right.get(k)}))
        key = tuple((k, old[k]) for k in CONDITION_FIELDS if k in old)
        conditions[key] += 1
        repeated = copy.deepcopy(new)
        for stage in ('setup', 'measurement'):
            if stage in repeated:
                repeated[stage].pop('elapsed_ns')
        repeated.pop('sample')
        repeated.pop('warmup')
        repeat_key = (key, old['measurement']['allocation_observer'])
        encoded = json.dumps(repeated, sort_keys=True)
        require(repeats.setdefault(repeat_key, encoded) == encoded, (index, 'non-repeatable non-timing values'))
        costs.append({'row': index, 'family': old['family'], 'sample': old['sample'],
                      'warmup': old['warmup'], 'allocation_observer': old['measurement']['allocation_observer'],
                      'changes': changes})
    require(len(conditions) == 36 and set(conditions.values()) == {16}, ('condition repetitions', conditions))
    warmups = [r for r in after['rows'] if r['warmup'] and not r['measurement']['allocation_observer']]
    statuses = collections.Counter(r['error'] is None for r in warmups)
    require(statuses == {True: 31, False: 5}, ('conditions31/5', statuses))
    return {'passed': True, 'rows': 576, 'distinct_conditions': 36,
            'successful_conditions': 31, 'typed_refusal_conditions': 5,
            'expected_owner_extent_delta': delta, 'per_row_cost_changes': costs}


def controls(before, after, layout, failed):
    result = [{'name': 'actual_current_positive', 'passed': compare(before, after, layout)['passed']}]
    probes = []
    def mutate(name, row, action):
        value = copy.deepcopy(after)
        action(value['rows'][row])
        probes.append((name, value))
    mutate('new_refusal', 0, lambda r: r.update(error='new refusal'))
    mutate('gameplay_hp', 0, lambda r: r.update(hp_loss=r['hp_loss'] + 1))
    mutate('gameplay_log', 0, lambda r: r.update(after_log_sha256='0' * 64))
    mutate('native_work', 0, lambda r: r['native_usage'][0].__setitem__(0, r['native_usage'][0][0] + 1))
    mutate('native_clone', 0, lambda r: r['native_usage'][0].__setitem__(1, r['native_usage'][0][1] + 1))
    mutate('native_retained', 0, lambda r: r['native_usage'][0].__setitem__(2, r['native_usage'][0][2] + 1))
    mutate('retained_extent', 0, lambda r: r.update(retained_extent_at_end=r['retained_extent_at_end'] + 1))
    observed = next(i for i, r in enumerate(after['rows']) if r['measurement']['allocation_observer'])
    mutate('allocation_count', observed, lambda r: r['measurement'].update(allocation_requests=r['measurement']['allocation_requests'] + 1))
    mutate('allocation_bytes', observed, lambda r: r['measurement'].update(requested_bytes_sum=r['measurement']['requested_bytes_sum'] + 1))
    synthetic = next(i for i, r in enumerate(after['rows']) if 'last_expr_usage' in r)
    mutate('evaluator_work', synthetic, lambda r: r.update(total_expr_work=r['total_expr_work'] + 1))
    mutate('evaluator_usage', synthetic, lambda r: r['last_expr_usage'].update(work=r['last_expr_usage']['work'] + 1))
    refusal = next(i for i, r in enumerate(after['rows']) if r['error'])
    mutate('completion', refusal, lambda r: r.update(completed=r['completed'] + 1))
    for name, text, replacement in [
        ('error_code', 'RetainedByteLimit', 'WorkLimit'),
        ('error_site', 'value_expr_evaluation_retained', 'other_site'),
        ('error_cap', '8388608', '8388609'),
    ]:
        mutate(name, refusal, lambda r, text=text, replacement=replacement: r.update(error=r['error'].replace(text, replacement)))
    requested = FAULT.fullmatch(after['rows'][refusal]['error'])[4]
    mutate('error_requested', refusal, lambda r: r.update(error=r['error'].replace('requested: Some(' + requested + ')', 'requested: Some(' + str(int(requested) + 1) + ')')))
    probes.append(('actual_historical_pc001_regression', failed))
    for name, value in probes:
        try:
            compare(before, value, layout)
        except AssertionError as error:
            result.append({'name': name, 'rejected': True, 'first_failure': str(error)[:1500]})
        else:
            raise AssertionError(('negative control accepted', name))
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--label', default='004')
    args = parser.parse_args()
    require(args.label.isdigit(), 'label')
    after_path = C / f'evidence/pc_raw_current_{args.label}.json'
    log = H / f'evidence/compat_pc_{args.label}.log'
    receipt_path = H / f'evidence/compat_pc_{args.label}.json'
    receipt = json.loads(receipt_path.read_text())
    current = identity()
    require(receipt['exit_code'] == 0 and receipt['source_unchanged'] and
            receipt['source_before'] == receipt['source_after'] == current, 'current PC source')
    layouts = re.findall(r'U07_LAYOUT (\{[^\n]+\})', log.read_text())
    require(len(layouts) == 1, 'exact layout observation')
    layout = json.loads(layouts[0])
    before, after = json.loads(BEFORE.read_text()), json.loads(after_path.read_text())
    result = compare(before, after, layout)
    failed_path = C / 'evidence/pc_raw_current_001.json'
    result['controls'] = controls(before, after, layout, json.loads(failed_path.read_text()))
    sources = {
        'core_enhanced/src/game/battle/core/independent_objects.rs': 'IndependentObjectState::retained_extent includes152-byte empty owner; zero dynamic entries, no allocator allocation.',
        'core_enhanced/src/game/battle/core/mod.rs': 'new_from_database scenario_fact_binding_extent always adds48-byte Option<IndependentPlacementDomain>, includingNone; None dynamic extent0.',
        'core_enhanced/src/game/battle/core/runtime_facts.rs': 'retained_runtime_fact_extent includes independent owner and scenario extent, seeds native operation admission; empty scans debit0, not extra work.',
        'core_enhanced/src/game/battle/runtime_facts.rs': 'RuntimeFactSource136 matches U06; extended_source None for all legacyPC gameplay sources, no wrapper allocation.',
        'core_enhanced/src/game/battle/runtime_control.rs': 'seed_retained_extent/admit_retained_growth preserve operation cumulative admission and original8MiB cap.',
    }
    result.update({
        'before': {'path': str(BEFORE.relative_to(ROOT)), 'sha256': sha(BEFORE)},
        'after': {'path': str(after_path.relative_to(ROOT)), 'sha256': sha(after_path)},
        'source_sha256': current['sha256'], 'pc_receipt': {'path': str(receipt_path.relative_to(ROOT)), 'sha256': sha(receipt_path)},
        'compiler_layout': layout, 'layout_log_sha256': sha(log),
        'source_owner_proof': {p: {'sha256': sha(ROOT / p), 'reason': reason} for p, reason in sources.items()},
        'claim': 'All576 immediateU06 non-timing fields are exact except the measured200-byte empty-owner retained extent/admission and same200 requested delta in the original5 typed refusals. Work, clone, allocation count/bytes, evaluator usage/work, gameplay/facts/logs, completion and error code/site/cap are exact. Kernel synthetic costs remain exact. All repeated samples have identical non-timing observations within each observer condition. Unknown deltas/new refusals fail.',
        'excluded': ['setup.elapsed_ns', 'measurement.elapsed_ns'],
        'unchanged_gates': 'Original36/31/5 workload,8MiB cap, hard public numeric guards and inherited R05 peak-only warning; no new exception.',
    })
    output = C / f'evidence/PC_IMMEDIATE_ACCOUNTED_U07_{args.label}.json'
    with output.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k not in ('per_row_cost_changes', 'source_owner_proof', 'controls')}))
    print(f"PASS576 strict immediate parity; {len(result['controls']) - 1} negative controls rejected")


if __name__ == '__main__':
    main()
