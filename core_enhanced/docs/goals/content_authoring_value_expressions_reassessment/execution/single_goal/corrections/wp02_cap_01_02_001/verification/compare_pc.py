import json
from pathlib import Path

here = Path(__file__).parent
raw = {name: json.loads((here / f'pc_{name}_raw.json').read_text())['rows']
       for name in ('before', 'after')}
summaries = {name: json.loads((here / f'pc_{name}_summary.json').read_text())['summary']
             for name in ('before', 'after')}


def key(case):
    family = case['family']
    if family == 'battle':
        family = ('literal_battle_public_step' if case['chunk'] else
                  'damage_burst_private_transaction')
    return (family, case['n'], case.get('chunk'), case.get('sources'),
            case.get('reads'), case.get('grouped'))


assert all(len(rows) == 576 for rows in raw.values())
assert all(len(cases) == 36 for cases in summaries.values())
before = {key(case): case for case in summaries['before']}
after = {key(case): case for case in summaries['after']}
assert before.keys() == after.keys()
assert all(case['samples'] == 11 and case['allocation_samples'] == 3
           for cases in summaries.values() for case in cases)

rows = []
for identity, current in after.items():
    prior = before[identity]
    old_success, new_success = not prior['errors'], not current['errors']
    if old_success and new_success and identity[0] in (
            'literal_battle_public_step', 'damage_burst_private_transaction'):
        assert prior['example']['after_log_sha256'] == current['example']['after_log_sha256']
        assert prior['example']['hp_loss'] == current['example']['hp_loss']
    if identity[1] == 500 and identity[0] in (
            'literal_battle_public_step', 'damage_burst_private_transaction'):
        assert new_success and current['example']['units'] == 512
    if identity[:4] == ('damage_burst_private_transaction', 300, 0, 1):
        assert new_success and current['example']['hp_loss'] == 300
    rows.append({
        'case': identity,
        'before': {'completed': old_success, 'faults': prior['errors'],
                   'p50_ms': prior['elapsed_ms']['p50'],
                   'requested_allocation_sum_max': prior['requested_bytes']['max']},
        'after': {'completed': new_success, 'faults': current['errors'],
                  'p50_ms': current['elapsed_ms']['p50'],
                  'requested_allocation_sum_max': current['requested_bytes']['max'],
                  'units': current['example'].get('units'),
                  'hp_loss': current['example'].get('hp_loss')},
        'completion_time_ratio': (current['elapsed_ms']['p50'] / prior['elapsed_ms']['p50']
                                  if old_success and new_success else None),
    })

assert sum(not case['before']['completed'] for case in rows) == 11
assert sum(not case['after']['completed'] for case in rows) == 5
result = {
    'conditions': 36,
    'time_samples_per_condition': 11,
    'allocation_samples_per_condition': 3,
    'warmup_rows_per_series': 36,
    'before_rejected_conditions': 11,
    'after_rejected_conditions': 5,
    'allocation_metric': 'sum of requested alloc/realloc bytes; not simultaneous peak or RSS',
    'simultaneous_peak': 'not measured by PC fixture',
    'rss': 'not measured by PC fixture',
    'rows': rows,
}
(here / 'pc_comparison.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({key: value for key, value in result.items() if key != 'rows'}, indent=2))
