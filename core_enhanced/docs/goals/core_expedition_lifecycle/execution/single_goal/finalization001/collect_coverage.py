#!/usr/bin/env python3
"""Bind reviewed U12 acceptance mappings to actual, nonzero execution records.

No product evaluator is used as an oracle. This index is not an independent audit.
"""
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
E = HERE.parent
ROOT = next(p for p in HERE.parents if (p / 'core_enhanced/src/lib.rs').is_file())
P = E / 'product_parts'


def read(path):
    return json.loads(path.read_text())


def ref(path):
    data = path.read_bytes()
    return {'path': str(path.relative_to(ROOT)), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}


def write(path, data):
    with path.open('x') as stream:
        json.dump(data, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


RUNS = {
    'core91': 'final_core_regression001',
    'core72': 'final_coverage_regression001',
    'core13': 'final_coverage_regression002',
    'g11_capture': 'g11_core_functional001_01',
    'g11_boundaries': 'g11_core_functional002',
    'server_model': 'server_functional001',
    'tutorial_direct': 'public_tutorial_complete005',
    'expedition_direct': 'public_expedition_complete002',
    'defeat_abandon_direct': 'public_defeat003',
    'failure_direct': 'public_failure002',
    'normal_crash': 'normal_persistence001',
    'normal_return_previous': 'normal_return_cycle001',
    'normal_ack_previous': 'normal_ack002',
    'normal_local_return': 'normal_local_return001',
    'normal_local_failure': 'normal_local_failure001',
    'normal_local_failure_continue': 'normal_local_failure_continue001',
    'full60_previous': 'full60_combined_calibration001',
    'required_missing': 'full60_missing_required001',
    'criterion_stale': 'full60_stale_criterion001',
    'normal_return_current': 'normal_return_cycle002',
    'normal_ack_current': 'normal_return_boundaries003',
    'absent_copies': 'normal_return_boundaries003',
    'standalone_current_calibration': 'normal_standalone_calibration006',
    'standalone_current_adoption': 'normal_standalone_adoption002',
    'server_current_calibration': 'normal_server_calibration003',
    'server_current_adoption': 'normal_server_adoption003',
    'local_current_start': 'normal_local_start002',
    'local_current_continue': 'normal_local_continue002',
    'full60_current': 'full60_combined_calibration002',
    'full60_current_adoption': 'full60_adoption002',
    'full60_current_accept': 'full60_accept002',
    'full60_current_continue': 'full60_continue002',
}

GROUPS = {
    'G01': (['normal_return_current', 'normal_local_return', 'normal_local_failure', 'normal_local_failure_continue', 'local_current_start', 'local_current_continue'], ['start_identity', 'u06_starter']),
    'G02': (['core91', 'core72', 'core13', 'normal_return_current'], ['start_identity', 'u06_starter', 'game::employee::tests::', 'retreat_battle_restores']),
    'G03': (['tutorial_direct', 'normal_crash', 'core91', 'core72'], ['tutorial', 'standard_floor_three', 'fixed_standard', 'fixed_tutorial']),
    'G04': (['core91', 'normal_return_current', 'normal_crash', 'absent_copies'], ['loans_terminal', 'return_exhaustion', 'codec']),
    'G05': (['core91', 'core13', 'normal_return_current'], ['recruitment', 'public_recruitment_stops_at_ten']),
    'G06': (['core91', 'normal_return_current', 'normal_local_return'], ['return_exhaustion', 'gate', 'exhaustion']),
    'G07': (['core91', 'core72', 'core13', 'failure_direct', 'defeat_abandon_direct', 'normal_local_failure'], ['recovery', 'checkpoint', 'retreat', 'fatal', 'failure', 'defeat']),
    'G08': (['core91', 'core72', 'core13', 'server_model', 'normal_crash'], ['codec', 'start_identity', 'employee', 'checkpoint_validation', 'command_document']),
    'G09': (['normal_crash', 'normal_ack_current', 'normal_return_current', 'normal_local_failure_continue', 'core91'], ['codec', 'checkpoint', 'return_exhaustion']),
    'G10': (['standalone_current_calibration', 'standalone_current_adoption', 'server_current_calibration', 'server_current_adoption', 'local_current_start', 'local_current_continue', 'normal_return_current', 'required_missing', 'criterion_stale', 'full60_current', 'full60_current_accept', 'full60_current_continue', 'core72', 'core13'], ['content_pack', 'live_ron', 'u06_starter']),
    'G11': (['g11_capture', 'g11_boundaries', 'full60_previous', 'full60_current'], ['cost', 'owned_clone_shape']),
    'G12': (['core72', 'core13', 'normal_crash', 'required_missing', 'criterion_stale'], ['command_', 'checkpoint', 'employee']),
}

SOURCE_GROUPS = {
    'G01': ['core_enhanced/src/game/world/expedition_lifecycle.rs', 'core_enhanced/src/game/world.rs', 'game_server/src/mod_admission.rs'],
    'G02': ['core_enhanced/src/game/employee.rs', 'core_enhanced/src/game/world.rs', 'core_enhanced/src/game/managers/uuid_manager.rs', 'core_enhanced/src/game/combat_setup/player_spawns.rs'],
    'G03': ['core_enhanced/src/game/world/expedition_lifecycle.rs', 'core_enhanced/src/game/world/visual_novel_flow.rs', 'game_resources/data/map/facility_templates.ron'],
    'G04': ['core_enhanced/src/game/expedition_lifecycle.rs', 'core_enhanced/src/game/world/expedition_lifecycle.rs', 'core_enhanced/src/game/world/active_run/continuation.rs'],
    'G05': ['core_enhanced/src/game/world/headquarters.rs', 'core_enhanced/src/game/world/map_content.rs', 'core_enhanced/src/game/recruitment.rs'],
    'G06': ['core_enhanced/src/game/world/expedition_lifecycle.rs', 'core_enhanced/src/game/world/node_flow.rs', 'core_enhanced/src/game/map/progression.rs'],
    'G07': ['core_enhanced/src/game/world/expedition_recovery.rs', 'core_enhanced/src/game/world/combat.rs', 'core_enhanced/src/game/world/action_capability.rs'],
    'G08': ['core_enhanced/src/game/employee.rs', 'core_enhanced/src/game/world/active_run.rs', 'core_enhanced/src/game/world/active_run/continuation.rs', 'core_enhanced/src/game/world/active_run/v7.rs', 'core_enhanced/src/game/world/active_run/v8.rs', 'core_enhanced/src/game/behavior.rs'],
    'G09': ['game_server/src/persistence/player_profile_store/application_intent.rs', 'game_server/src/persistence/active_run_store.rs', 'game_server/src/mod_admission/persistence.rs', 'core_enhanced/src/application.rs'],
    'G10': ['tools/cmva_build.py', 'core_enhanced/build.rs', 'game_server/src/mod_admission/worker.rs', 'game_server/src/mod_admission.rs', 'core_enhanced/src/game/data/content_pack/mod.rs'],
    'G11': ['core_enhanced/src/authoring/clone_shape.rs', 'core_enhanced/src/game/expedition_lifecycle.rs', 'core_enhanced/src/game/world/state.rs', 'config/runtime_safety.ron'],
    'G12': ['core_enhanced/src/game/employee.rs', 'core_enhanced/src/game/mod.rs', 'core_enhanced/src/game/world/expedition_lifecycle.rs', 'core_enhanced/src/game/world/active_run.rs'],
}


def main():
    source_path = HERE / 'FINAL_SOURCE_MANIFEST_001.json'
    source = read(source_path)
    bound = {str(source_path): ref(source_path)}
    evidence = {}
    for key, name in RUNS.items():
        directory = P / name
        result = read(directory / 'RESULT_001.json')
        assert result['child_reaped'] and result['source_unchanged'], name
        stdout = (directory / 'stdout.log').read_text()
        summaries = re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', stdout)
        counts = dict(zip(['passed', 'failed', 'ignored'], map(int, summaries[-1]))) if summaries else None
        selectors = []
        for match in re.finditer(r'^test ([A-Za-z0-9_:]+) \.\.\. (.*?)(?=^test |^failures:|^test result:|\Z)', stdout, re.M | re.S):
            tail = match[2].strip()
            if tail == 'ok' or tail.endswith('\nok'):
                selectors.append(match[1])
        if counts:
            assert len(selectors) == counts['passed'] and counts['passed'] > 0, (name, counts, len(selectors))
        expected_code = 101 if key in {'core72', 'normal_crash'} else 1 if key in {'required_missing', 'criterion_stale'} else 0
        assert result['exit_code'] == expected_code, name
        for file in ['RESULT_001.json', 'stdout.log', 'stderr.log', 'SOURCE_BEFORE.json', 'SOURCE_AFTER.json']:
            path = directory / file
            bound[str(path)] = ref(path)
        evidence[key] = {'run': name, 'result': ref(directory / 'RESULT_001.json'), 'stdout': ref(directory / 'stdout.log'),
            'exit_code': result['exit_code'], 'counts': counts, 'passing_selectors': selectors,
            'scope': 'Only individually passing cases are reused; prior failed cases remain failed historical evidence.' if expected_code == 101 else 'Expected refusal; original state protection is reviewed in the associated negative evidence.' if expected_code == 1 else 'Successful actual execution; compilation is not functional acceptance.',
            'currentness': 'Current final identity execution.' if key.endswith('current') or '_current_' in key or key == 'absent_copies' else 'Explicit nonimpact reuse: subsequent five Core edits are confined to test bodies/helpers; every affected consumer is replaced by core13. External server test addition does not alter prior cases or production behavior. Final supported binaries separately rebind approval identity.'}
    heap = P / 'g11_heap_samples001/summary.json'
    bound[str(heap)] = ref(heap)
    groups = {}
    for group, (keys, patterns) in GROUPS.items():
        groups[group] = {'evidence_ids': keys, 'passing_selectors': sorted({selector for key in keys for selector in evidence[key]['passing_selectors'] if any(pattern in selector for pattern in patterns)}),
            'source': [ref(ROOT / path) for path in SOURCE_GROUPS[group]],
            'additional_evidence': [ref(heap)] if group == 'G11' else [],
            'scope': 'P = public command consumer; I = declared internal/codec boundary fixture; D = actual durable consumer/restart; S = reviewed source/contract. Do not promote I to ordinary public traversal.'}
    acceptance_lines = [line for line in (E / 'test_parts/TEST_PLAN_001.ko.md').read_text().splitlines() if line.startswith('| A-')]
    acceptances = []
    for line in acceptance_lines:
        fields = [field.strip() for field in line.split('|')[1:-1]]
        item, text, oracle, group_text = fields
        ids = re.findall(r'G\d{2}', group_text)
        assert ids and all(group in groups for group in ids)
        acceptances.append({'id': item, 'exact_acceptance': text, 'independent_oracle_contract': oracle, 'groups': ids,
            'evidence_ids': sorted({key for group in ids for key in groups[group]['evidence_ids']}),
            'status': 'root_reviewed_evidence_available; independent_verdict_pending',
            'limits': 'Core receipt is tested; actual VN playback/Unity UI excluded.' if item.startswith('A-12') else 'Current-copy absence is an explicit I fixture; genuine ordinary return/loan traversal is independently provided by normal_return_current.' if item == 'A-09-1' else ''})
    assert len(acceptances) == len({a['id'] for a in acceptances}) == 38
    dimensions = {
        'success': ['G01', 'G03', 'G04'], 'invalid_input': ['G02', 'G04', 'G08'],
        'failure_rollback': ['G02', 'G05', 'G09', 'G11'], 'retry_idempotency': ['G02', 'G04', 'G06', 'G09'],
        'ordering_concurrency': ['G09', 'G10'], 'boundary_empty_max': ['G02', 'G04', 'G05', 'G06', 'G07'],
        'disconnect_restart_persistence': ['G03', 'G08', 'G09'], 'live_data': ['G01', 'G03', 'G10'],
        'producer_consumer': ['G01', 'G09', 'G10'], 'legacy_fallback_absence': ['G08', 'G12'],
    }
    integrations = [{'id': 'IVA-CEXL-' + item, 'acceptance_ids': [a['id'] for a in acceptances if a['id'].startswith('A-' + item + '-')]} for item in ['PS01', 'PS02'] + [f'{n:02}' for n in range(1, 14)]]
    integrations += [{'id': 'IVA-CEXL-' + name, 'groups': ids, 'pending': ['independent_whole_goal_verdict'] if name == 'STRUCTURE' else []} for name, ids in [('PUBLIC', ['G01', 'G03', 'G10']), ('ATOMIC', ['G04', 'G09']), ('REGRESSION', ['G07', 'G08', 'G10', 'G11']), ('STRUCTURE', ['G12'])]]
    common = {'source_sha256': source['sha256'], 'source': ref(source_path)}
    write(E / 'COVERAGE.json', {**common, 'schema': 'u12.root-reviewed-coverage/v1', 'acceptances': acceptances,
        'groups': groups, 'integration_seams': integrations, 'required_dimensions': [{'id': name, 'required': True, 'groups': ids} for name, ids in dimensions.items()],
        'execution_nodes': [{'id': name, 'groups': ids} for name, ids in [('AG-CEXL-LIFECYCLE', ['G02', 'G04', 'G05', 'G06', 'G07', 'G08']), ('WP-CEXL-PUBLIC', ['G01', 'G03', 'G09', 'G10']), ('WP-CEXL-VERIFY', list(groups))]],
        'independence': 'Original test author and product author history preserved. Current corrections, new coverage oracles, execution and review are root-owned under single-executor instructions. This is not a nonparticipant audit.',
        'pending': ['isolated_restore_at_target_freeze', 'nonparticipant_whole_goal_GEX20_Complete_High_open0']})
    write(HERE / 'EVIDENCE_SELECTION.json', {**common, 'evidence': evidence, 'heap9': ref(heap), 'bound_artifacts': list(bound.values()),
        'failure_history': 'All failed/interrupted builds, strategy attempts, stale fixture assertions and rejected inputs remain in Goal records. Partial batches contribute only named passing selectors; failed selectors are never upgraded by aggregate success.'})
    write(HERE / 'PREDECESSOR_IMPACT.json', {**common, 'rows': [
        {'id': 'U03', 'affected_contract': 'exclusive prepared application draft and publication rollback', 'groups': ['G02', 'G04', 'G05', 'G09', 'G11'], 'reuse': 'Draft ownership architecture unchanged; current loan/grant collision, terminal and cost discard boundaries executed.'},
        {'id': 'U04', 'affected_contract': 'paired profile/active intent, CAS, backup and accepted recovery', 'groups': ['G08', 'G09'], 'reuse': 'Current schemas/instance/ownership were revalidated in actual fresh process crash consumers; later test-only edits do not alter persistence.'},
        {'id': 'U07M', 'affected_contract': 'clone shape, bounded work/retained slots, real heap and recovery', 'groups': ['G11'], 'reuse': 'Current expanded vault fixture12 stages x9 fresh children plus exact/short/cancel/discard/commit and final release; no production changes since measurement.'},
        {'id': 'U11', 'affected_contract': 'normal CoreBuildIdentity, issuer calibration/adoption/cache/required save', 'groups': ['G10'], 'reuse': 'Final exact worker pair and each actual host rebind supported identity; normal local/server and full60 consume current approval. Missing required and stale criterion refusals reuse unchanged producer/consumer bodies.'},
    ], 'unrelated_dirty_preserved': ['core_enhanced/docs/skills/run_relic_content_design_notes.ko.md'], 'independent_audit': 'pending_nonparticipant_required'})
    print(json.dumps({'acceptances': 38, 'integration_seams': 19, 'dimensions': 10, 'nodes': 3, 'executions': len(evidence)}))


if __name__ == '__main__':
    main()
