"""Frozen sentence contracts and minimum execution roles; no selector wildcard PASS."""
from evidence_common import HERE, read, ref

CONTRACT_PATH = HERE / 'tools/finalization_contracts.json'
REQUIRED_ROLES = {
    **{'integrated_' + n:'suite' for n in ('core_default','core_fixtures','core_observation',
                                          'host_default','host_observation','host_measurement')},
    **{'integrated_' + n:'check' for n in ('fmt','check','event_log','clippy_core','clippy_host')},
    'integrated_heap':'focused', 'host_measurement_heap':'focused',
    'measurement_support':'execution', 'release_build':'build', 'debug_build':'build',
    **{n:'execution' for n in ('playback_actual','lifecycle_actual','host_actual','playback_prepare',
                              'pc_prepare','pc_gameplay','pc_cost','r05_prepare','r05_summary','r05_dependencies')},
    'pc':'focused', 'r05':'r05',
    **{'public_' + n:'execution' for n in ('prepared_host','prepared_boundary','input_validation',
        'defaults','oracle','trace','relics','cli','profile','boundary','api','starter_schema','schema',
        'independent_area','continuation','rewind','object_schema','objects','object_combat',
        'guards','allocator_guards')},
    **{'fault_' + n:('build' if n.startswith('build_') else 'execution')
       for n in ('build_on','none','detached','queue_full','on','websocket','build_off','off')},
}
REQUIRED_ARTIFACTS = set(('release_binding','debug_binding','playback_actual','lifecycle_actual',
    'host_actual','pc_isolate','pc_raw','pc_summary','pc_cost','r05_raw','r05_summary','r05_proof','r05_dependencies',
    'input_validation','public_relics','public_cli','fault','guards','allocator_guards'))


def contracts():
    value = read(CONTRACT_PATH)
    frozen = HERE / 'P2G_30_FREEZE_001.json'
    phase = HERE / 'P2G_40_PHASE_PLAN_001.json'
    assert value['freeze'] == ref(frozen) and value['phase_plan'] == ref(phase)
    assert value['goal_set'] == ref(HERE / 'P2G_50_GOAL_SET.ko.md')
    assert value['code_review'] == ref(HERE / 'CODE_REVIEW_001.ko.md')
    original = read(frozen)['artifacts'][0]['payload']['Observable Policy Acceptance Set']
    assert len(value['groups']) == len(original) == 16
    for group, row in zip(value['groups'], original):
        assert group['policy_ref'] == row['Policy Element Revision Ref']
        assert [s['text'] for s in group['sentences']] == row['Observable Policy Acceptance']
        assert len(group['sentences']) == 2
    assert len(value['dimensions']) == 10 and len(value['nodes']) == 3 and len(value['seams']) == 16
    original_phase = read(phase)['artifacts'][0]['payload']
    assert [r['id'] for r in value['nodes']] == original_phase['phases'][0]['nodes']
    for actual, expected in zip(value['seams'], original_phase['integration']):
        assert all(actual[k] == v for k, v in expected.items())
    goal_text = (HERE / 'P2G_50_GOAL_SET.ko.md').read_text()
    for row in value['dimensions']:
        assert f"| {row['id']} | required | {row['seam_contract']} | {row['completion_requirement']} |" in goal_text
    return value
