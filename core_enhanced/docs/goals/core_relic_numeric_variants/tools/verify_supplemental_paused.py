"""Verify closed F01 evidence separately from the historical performance matrix.

This reads an actual execution; it never runs Cargo, the product, or a finalizer.
The original receipt and composite validators are reused without exceptions.
"""
import argparse
import datetime
import difflib
import json
from pathlib import Path
import tomllib

from evidence_common import (ROOT, HERE, read, ref, check_ref, receipt, identity,
                             source_valid, write, nonempty)
from execution_binding import validate_execution_binding, instant, end

ROLE = 'supplemental_paused_manual_003'
SCHEMA = 'U08_SUPPLEMENTAL_PAUSED_SPEC_V1'
SKILL = 'u08_trial_damage'
FRAGMENT = 'u08_trial_fragment'
RELIC = 'u08_trial_blend'
DAMAGE_FIELDS = ('damage_mutations', 'damage_strikes', 'damaging_strikes',
                 'damage_hp_lost', 'latest_damage', 'latest_nonzero_damage')
PERFORMANCE = {'pc_prepare', 'pc', 'pc_gameplay', 'pc_cost', 'r05_prepare',
               'r05', 'r05_summary', 'r05_dependencies'}
SOURCE_ASSERTIONS = {
    'fail_closed_check': 'assert!(condition, "assertion failed: {name}");',
    'paused_full_facts': 'p.check("paused host tick full facts frozen",p.facts()==accepted_facts);',
    'retained_employee0_arc': 'p.check("first actual fact retained unchanged after second and later public tick",serde_json::to_value(hits[0].as_ref().unwrap()).unwrap()==first_retained);',
    'typed_cast_parent': 'p.check("typed cast cause is accepted ManualCastStart",hit.source.cast.as_ref().unwrap().cause.parent_seq()==Some(start_sequences[i]));',
}


def require(condition, reason):
    assert condition, reason


def only(rows, reason):
    require(len(rows) == 1, reason)
    return rows[0]


def parse_raw(text):
    documents, summaries = [], []
    for line in text.splitlines():
        if line.startswith('{'):
            documents.append(json.loads(line))
        elif line.startswith('U08_PAUSED_PUBLIC_PASS '):
            summaries.append(json.loads(line.split(' ', 1)[1]))
    require(len(summaries) == 1, 'Exactly one completed raw summary required')
    return documents, summaries[0]


def hp(checkpoint):
    values = [[u['unit_instance_id'], u['stats']['current_health']]
              for u in checkpoint['units']]
    require(len({x[0] for x in values}) == len(values), 'Duplicate unit HP row')
    return sorted(values)


def validate_behavior(summary, transcript, documents, raw_summary, oracle):
    """Recompute available observations; never turn a missing field into null."""
    require(oracle == {
        'scope': 'F01 paused manual consumer only; no Auto positive claim',
        'base': 100,
        'first': {'flat': 20, 'percent': 20, 'expected': 144, 'variant': 'Normal'},
        'second': {'flat': 40, 'percent': 50, 'expected': 210, 'variant': 'Corrupt'},
        'frozen_host_ticks': 5, 'frozen_host_tick_ms': 100,
        'resume_host_tick_ms': 1, 'readiness_limit_ticks': 600,
        'readiness_tick_ms': 100}, 'Changed independent oracle or probe scope')
    expected = [(oracle['base'] + oracle[k]['flat']) *
                (100 + oracle[k]['percent']) // 100 for k in ('first', 'second')]
    require(expected == [144, 210] == summary['expected'], 'Independent amount oracle')
    require(summary == raw_summary, 'Raw completion and summary differ')
    require(summary['passed'] is True and summary['scope'] == 'F01 paused manual casts only',
            'Missing focused completion')
    require(summary['checks'] == 84, 'Exact executed assertion count')
    raw_transcript = [x for x in documents if 'operation' in x]
    require(raw_transcript == transcript, 'Raw transcript order/membership differs')
    require(all(x['operation'] in ('command', 'admin', 'host_tick') for x in transcript),
            'Unknown public operation')
    commands = [x for x in transcript if x['operation'] == 'command']
    ticks = [x for x in transcript if x['operation'] == 'host_tick']
    require(len(commands) == summary['commands'] == 20 and
            len(ticks) == summary['host_ticks'] == 57, 'Actual command/tick counts')
    require([x['source_command_id'] for x in commands] ==
            [f'paused-probe-command-{i}' for i in range(1, 21)], 'Command order/identity')
    require(commands[0]['input'] == {'type': 'start_new_game', 'game_mode': 'Standard',
                                    'replace_active_run': False}, 'Normal public start')
    def command(kind):
        return only([x for x in commands if x['input']['type'] == kind], kind)

    for key in ('actors', 'employees', 'relics', 'fragments'):
        require(len(summary[key]) == len(set(summary[key])) == 2, 'Distinct ' + key)
        require(all(isinstance(x, str) and x for x in summary[key]), 'Missing ' + key)
    actors, employees = summary['actors'], summary['employees']
    relics, fragments = summary['relics'], summary['fragments']
    starter = command('select_starter_employees')['payload']
    require(set(employees) <= set(starter['employee_uuids']), 'Normally selected employees')
    selected = command('select_map_node')
    node = only([n for n in starter['map']['nodes']
                 if n['id'] == selected['input']['node_id']], 'Selected original map node')
    require(node['state'] == 'Available' and node['category'] == 'Combat' and
            node['payload']['Encounter']['encounter_id'] == 'normal_corroded_patrol',
            'Normal available combat node')
    require(selected['payload']['combat_preview']['node_type'] == 'Defense', 'Defense preview')
    require(command('confirm_enter_node')['payload']['encounter_id'] ==
            'normal_corroded_patrol', 'Entered previewed encounter')
    admins = [x for x in transcript if x['operation'] == 'admin']
    require([x['input'] for x in admins] == [
        {'type': 'admin_grant_relic', 'definition_id': RELIC, 'count': 2},
        {'type': 'admin_grant_skill_fragment', 'fragment_id': FRAGMENT, 'count': 2},
        {'type': 'admin_grant_relic_corruption_catalyst', 'amount': 1}],
        'Only public grant administration; no direct battle injection')
    converted = command('corrupt_relic')
    require(converted['input']['copy_uuid'] == relics[1], 'Converted second copy')
    require(converted['payload']['resources']['relic_corruption_catalysts'] == 0,
            'Actual normal catalyst consumption')
    copies = {x['copy_uuid']: x for x in converted['payload']['copies']}
    for i, variant in enumerate(('Normal', 'Corrupt')):
        require(copies[relics[i]]['definition_id'] == RELIC and
                copies[relics[i]]['variant'] == variant, 'Independent copy variants')
        equip = only([x for x in commands if x['input']['type'] == 'equip_relic' and
                      x['input']['employee_uuid'] == employees[i]], 'Relic equipped once')
        require(equip['input']['copy_uuid'] == relics[i] and equip['input']['slot'] == 2,
                'Exact employee/relic slot')
        loadout = only([x for x in equip['payload']['loadouts']
                        if x['employee_uuid'] == employees[i]], 'Equipped loadout')
        require(loadout['slots'] == [None, None, relics[i]], 'Actual third-slot copy')
        fragment = only([x for x in commands if x['input']['type'] == 'equip_skill_fragment'
                         and x['input']['employee_uuid'] == employees[i]], 'Fragment equipped once')
        require(fragment['input']['copy_uuid'] == fragments[i] ==
                fragment['payload']['active_copy_uuid'], 'Exact independent fragment copy')
        deployed = only([x for x in commands if x['input']['type'] == 'deploy_unit' and
                         x['input']['employee_uuid'] == employees[i]], 'Normal employee deploy')
        require(deployed['payload']['unit_instance_id'] == actors[i], 'Actual employee actor')

    pause, resume = command('pause_battle'), command('resume_battle')
    pause_index, resume_index = transcript.index(pause), transcript.index(resume)
    require(pause_index < resume_index, 'Pause/resume order')
    frozen = pause['payload']['battle_update']['checkpoint']
    require(frozen['battle_time_ms'] == summary['frozen_time_ms'] == 5000,
            'Actual paused simulation time')
    require(hp(frozen) == summary['frozen_hp'], 'Frozen HP snapshot')
    require(frozen['playback']['paused'] is True, 'Pause accepted')
    accepted = [x for x in commands if x['input']['type'] == 'activate_skill']
    targeting = [x for x in commands if x['input']['type'] == 'begin_skill_targeting']
    require(len(accepted) == len(targeting) == 2, 'Two manual acceptances')
    target = accepted[0]['input']['target']['unit_instance_id']
    require(dict(summary['frozen_hp'])[target] == 1000000, 'Nonclamped target at pause')
    for i, row in enumerate(accepted):
        require(pause_index < transcript.index(targeting[i]) < transcript.index(row) < resume_index,
                'Target and accept while paused')
        require(row['input'] == {'type': 'activate_skill', 'employee_uuid': employees[i],
                                'skill_id': SKILL, 'target': {'type': 'Unit', 'unit_instance_id': target}},
                'Exact same-skill manual command')
        require(targeting[i]['input'] == {'type': 'begin_skill_targeting',
                'employee_uuid': employees[i], 'skill_id': SKILL}, 'Exact targeting command')
        require(row['payload']['unit_instance_id'] == actors[i], 'Accepted employee actor')
    for row in [x for x in transcript[pause_index:resume_index + 1]
                if x['payload'] is not None]:
        cp = row['payload']['battle_update']['checkpoint']
        require(cp['battle_time_ms'] == 5000 and hp(cp) == summary['frozen_hp'],
                'Acceptance/resume time and all unit HP frozen')
        require(cp['playback']['paused'] is (row is not resume), 'Actual pause state')
    frozen_ticks = [x for x in transcript[pause_index:resume_index] if x['operation'] == 'host_tick']
    require(len(frozen_ticks) == summary['frozen_host_ticks'] == 5 and
            all(x['milliseconds'] == 100 and x['payload'] is None for x in frozen_ticks),
            'Five paused host ticks return None')
    require([x for x in ticks if x['payload'] is None] == frozen_ticks, 'No other null ticks')
    require(all(transcript.index(x) > transcript.index(accepted[-1]) for x in frozen_ticks),
            'Paused ticks follow both accepted commands')
    resumed = transcript[resume_index + 1:]
    require(len(resumed) == 2 and all(x['operation'] == 'host_tick' and
            x['milliseconds'] == 1 for x in resumed), 'Release tick and later retained-fact tick')
    require([x['payload']['battle_update']['checkpoint']['battle_time_ms'] for x in resumed] ==
            [5001, 5002], 'Actual post-resume times')

    facts = only([x for x in documents if x.get('stage') == 'paused_facts'], 'Paused facts output')
    require(facts['before_acceptance'] == summary['facts_before_acceptance'] and
            facts['after_acceptance'] == summary['facts_after_acceptance'], 'Raw acceptance facts')
    before, after = facts['before_acceptance']['summary'], facts['after_acceptance']['summary']
    for key in DAMAGE_FIELDS:
        require(key in before and key in after, 'Required damage field absent: ' + key)
        require(before[key] == after[key], 'Paused acceptance changed damage: ' + key)
    require([before[k] for k in DAMAGE_FIELDS] == [0, 0, 0, 0, None, None],
            'No damage before release')
    require((before['resource_spent'], after['resource_spent']) == (20, 22),
            'Normal acceptance consumes two resonance; full acceptance facts need not equal')

    events = {}
    for row in transcript:
        payload = row['payload']
        if payload is None or 'battle_update' not in payload:
            continue
        for event in payload['battle_update']['events_delta']['events']:
            seq = event['seq']
            require(seq not in events or events[seq] == event, 'Conflicting event replay')
            events[seq] = event
    require([events[k] for k in sorted(events)] == summary['events'], 'Exact event stream/summary')
    require(set(events) == set(range(1, 27)), 'Complete observed event sequence')
    require(all(e['time_ms'] > 5000 for e in events.values()
                if e['event']['type'] in ('AbilityCast', 'HpChanged')),
            'No accepted cast/damage released before resume')
    require(summary['start_sequences'] == [16, 18] and
            summary['ability_event_sequences'] == [21, 19], 'Actual acceptance/release sequence')
    hit_rows = [x for x in documents if x.get('stage') == 'actual_skill_hit']
    require(len(hit_rows) == len(summary['observed']) == 2, 'Two actual damage facts')
    authored = {'field_path': 'skills[0].steps[0].effects[0]', 'origin': 'Original',
                'origin_definition_id': SKILL, 'source_file': 'official/data/skills/base.ron',
                'site': {'action': {'action_id': 'hit', 'definition_id': SKILL}, 'effect_id': 'amount'}}
    observed = []
    for i, amount in enumerate(expected):
        hit_row = only([x for x in hit_rows if x['employee_index'] == i], 'Exact employee hit')
        hit = summary['observed'][i]
        require(hit_row['fact'] == hit, 'Raw hit and summary differ')
        require(hit['requested_damage'] == amount and hit['hp_before'] > amount and
                hit['hp_before'] - hit['hp_after'] == amount, 'Nonclamped actual HP arithmetic')
        require(hit['time_ms'] == 5001 and hit['target'] == target, 'Actual release time/target')
        source = hit['source']; binding = source['relic_numeric']
        fragment = {'copy_uuid': fragments[i], 'definition_id': FRAGMENT,
                    'form_id': 'base', 'skill_id': SKILL}
        contribution = {'copy_uuid': relics[i], 'definition_id': RELIC, 'effect_id': 'damage',
                        'variant': ('Normal', 'Corrupt')[i], 'flat': (20, 40)[i], 'percent': (20, 50)[i]}
        require(source['actor'] == binding['actor'] == actors[i] and
                binding['employee_uuid'] == employees[i], 'Exact relic employee/actor source')
        require(binding['generation'] == summary['generation'], 'Catalog generation binding')
        require(binding['contributions'] == [contribution], 'Exact relic contribution')
        require(source['authored'] == authored and source['fragment'] == binding['fragment'] == fragment,
                'Exact original authored/fragment source')
        require(binding['target'] == {'kind': 'skill_damage', 'site': authored['site']} and
                binding['damage_type'] == 'True', 'Exact effect site/type')
        require(source['action_ordinal'] == binding['action_ordinal'] == i and
                source['equipment'] is None and binding['equipment'] is None, 'Exact action identity')
        require(source['cast'] == {'cast_seq': (1, 0)[i], 'caster_instance_id': actors[i],
                'proc_origin': None, 'skill_id': SKILL}, 'Exact serialized cast identity')
        require(hit['segments'] == [{'calculated_damage': amount, 'damage_type': 'True',
                'ordinal': 0, 'raw_damage': amount, 'source': source}], 'One exact bound damage segment')
        require(hit['modifier_sources'] == [] and all(v == 0 for v in hit['unattributed_modifiers'].values()),
                'No hidden damage modifiers')
        start = events[summary['start_sequences'][i]]
        ability = events[summary['ability_event_sequences'][i]]
        require(start['event'] == {'type': 'ManualCastStart', 'caster_instance_id': actors[i],
                'skill_id': SKILL, 'target': {'type': 'Unit', 'unit_instance_id': target}} and
                start['time_ms'] == 5000 and start['source_command_id'] == accepted[i]['source_command_id'],
                'ManualCastStart caused by exact accepted command')
        require(ability['event'] == {'type': 'AbilityCast', 'caster_instance_id': actors[i],
                'skill_id': SKILL, 'target_instance_id': target} and ability['time_ms'] == 5001 and
                ability['cause'] == {'cause_type': 'parent', 'seq': start['seq']} and
                hit_row['ability_event_seq'] == ability['seq'], 'Typed/raw cast linkage')
        ending = only([e for e in events.values() if e['event']['type'] == 'ManualCastEnd' and
                       e['event']['caster_instance_id'] == actors[i]], 'ManualCastEnd')
        require(ending['time_ms'] == 5001 and ending['cause'] == ability['cause'], 'End accepted-start parent')
        step = only([e for e in events.values() if e['event']['type'] == 'AbilityStepTriggered' and
                     e['event']['caster_instance_id'] == actors[i]], 'Exact ability step')
        require(step['cause'] == {'cause_type': 'parent', 'seq': ability['seq']} and
                step['event']['skill_id'] == SKILL and step['event']['step_id'] == 'hit' and
                step['time_ms'] == 5001, 'Step caused by exact cast')
        damage = only([e for e in events.values() if e['event']['type'] == 'HpChanged' and
                       e['event'].get('source_instance_id') == actors[i]], 'Exact actor HP event')
        e = damage['event']
        require(damage['cause'] == {'cause_type': 'parent', 'seq': step['seq']} and
                damage['time_ms'] == 5001 and e['damage_source'] == 'Ability' and
                e['source_instance_id'] == source['actor'] and e['target_instance_id'] == hit['target'] and
                e['hp_before'] == hit['hp_before'] and e['hp_after'] == hit['hp_after'] and
                e['delta'] == -amount and e['final_damage'] == e['raw_damage'] == amount,
                'HP event caused by exact bound effect')
        observed.append({'employee_index': i, 'amount': amount, 'hp_before': hit['hp_before'],
                         'hp_after': hit['hp_after'], 'source': source, 'time_ms': hit['time_ms'],
                         'event_chain': [start['seq'], ability['seq'], step['seq'], damage['seq']]})
    damage_events = [e for e in events.values() if e['event']['type'] == 'HpChanged']
    require([e['event']['final_damage'] for e in damage_events] == [210, 144], 'Actual chronological damage order')
    require(summary['observed'][1]['hp_after'] == summary['observed'][0]['hp_before'], 'Shared-target HP chain')
    require(dict(hp(resumed[0]['payload']['battle_update']['checkpoint']))[target] == 999646 and
            hp(resumed[0]['payload']['battle_update']['checkpoint']) ==
            hp(resumed[1]['payload']['battle_update']['checkpoint']), 'Actual final HP and later-tick stability')
    require(summary['first_fact_preserved'] is True, 'Executed retained-fact assertion did not complete')
    return {'commands': len(commands), 'host_ticks': len(ticks), 'accepted_manual_casts': 2,
            'paused_time_ms': 5000, 'paused_null_ticks': 5, 'resumed_times_ms': [5001, 5002],
            'damage_fields_present_and_unchanged': list(DAMAGE_FIELDS),
            'acceptance_resource_spent': [20, 22], 'observed': observed,
            'chronological_damage': [210, 144],
            'raw_direct_checks': 'Transcript/checkpoints, six acceptance damage fields, HP arithmetic, actor/copy/fragment/catalog/action source and event causality.',
            'source_asserted_checks': 'Each paused tick full facts and retained Normal144 Arc after both hits and later tick: exact executed fail-closed source assertions; per-tick full facts and final Arc value were not separately serialized.',
            'retained_fact_scope': 'Employee index0 Normal144 is chronologically second in the same5001ms bucket. No claim of retaining before the other chronological release.'}


def validate_build(documents, raw, handoff, inputs, current):
    root_lock = check_ref(handoff['lock_basis']['root_lock'])
    probe_lock = check_ref(handoff['lock_basis']['probe_lock'])
    require(handoff['lock_basis']['root_lock'] in current['files'], 'Root lock is an exact source input')
    def key(p):
        return p['name'], p['version'], p.get('source')
    root_packages = tomllib.loads(root_lock.read_text())['package']
    roots = {key(p): p for p in root_packages}
    require(len(roots) == len(root_packages), 'Duplicate root lock package')
    packages = tomllib.loads(probe_lock.read_text())['package']
    require(len({key(p) for p in packages}) == len(packages) == 140, 'Exact unique probe lock packages')
    nonprobe = [p for p in packages if p['name'] != 'u08_paused_probe']
    require(len(nonprobe) == 139, 'Exact nonprobe dependency count')
    for package in nonprobe:
        require(key(package) in roots and package.get('checksum') == roots[key(package)].get('checksum'),
                'Dependency version/source/checksum differs from root: ' + package['name'])
    manifest = inputs / 'crate/Cargo.toml'
    cargo = tomllib.loads(manifest.read_text())
    require(cargo['package']['name'] == 'u08_paused_probe' and cargo['workspace'] == {} and
            cargo['dependencies']['game_core'] == {'path': str(ROOT / 'core_enhanced')},
            'Actual independent crate consumes current product library')
    build = only([d for d in documents if d.get('stage') == 'build'], 'Actual build stage')
    binary = only([d for d in documents if d.get('stage') == 'binary'], 'Actual binary stage')
    probe = only([d for d in documents if d.get('stage') == 'probe'], 'Actual probe stage')
    require(build['command'] == ['cargo', 'build', '--offline', '--locked', '--manifest-path', str(manifest)],
            'Exact offline locked debug crate build')
    require(documents.index(build) < documents.index(binary) < documents.index(probe), 'Build before actual probe')
    require('Finished `dev` profile [unoptimized + debuginfo]' in raw and
            'Compiling game_core v0.1.0 (' + str(ROOT / 'core_enhanced') + ')' in raw,
            'Actual current library debug build output')
    binary_path = check_ref(handoff['binary'])
    require(binary_path.parts[-2:] == ('debug', 'u08_paused_probe') and
            binary == {'stage': 'binary', 'path': str(binary_path),
                       'bytes': handoff['binary']['bytes'], 'sha256': handoff['binary']['sha256']},
            'Actual built binary hash')
    require(probe['command'] == [str(binary_path), str(inputs), str(inputs.parent / 'runtime')],
            'Actual consumer binary/inputs/output')
    return {'profile': 'dev/debug', 'binary': handoff['binary'], 'build_command': build['command'],
            'root_lock': handoff['lock_basis']['root_lock'], 'probe_lock': handoff['lock_basis']['probe_lock'],
            'nonprobe_exact_name_version_source_checksum': len(nonprobe),
            'old_release_executable_consumed': False}


def verify(spec):
    require(__debug__ and spec['schema'] == SCHEMA, 'Supplemental schema/debug validation')
    current = identity(); source_valid(current)
    require(spec['source'] == current and len(current['files']) == 438, 'Wrong/foreign source')
    for row in spec['verifier_inputs']:
        check_ref(row)
    required_tools = {ref(HERE / 'tools' / name)['path'] for name in
                      ('verify_supplemental_paused.py', 'evidence_common.py', 'execution_binding.py',
                       'run.py', 'finalization_paths.py')}
    require(required_tools <= {r['path'] for r in spec['verifier_inputs']}, 'Unbound verifier dependencies')
    base = read(check_ref(spec['base_config']))
    selection = read(check_ref(spec['base_selection']))
    require(base['source'] == current and selection['config'] == spec['base_config'] and
            selection['source_sha256'] == current['sha256'], 'Exact original matrix source/config')
    require(len(base['receipts']) == len(selection['receipt_bindings']) == 58 and
            set(base['performance_receipt_ids']) == PERFORMANCE, 'Original58/performance8')
    require(len([r for r in selection['receipt_bindings'] if r['id'] not in PERFORMANCE]) == 50,
            'Original functional50')
    check_ref(base['execution_harness_binding'])
    require(selection['execution_harness_binding'] == base['execution_harness_binding'], 'Original composite binding')
    for row in list(base['functional_quiescence'].values()) + list(base['performance_environments'].values()):
        check_ref(row)
    handoff = read(check_ref(spec['handoff']))
    require(handoff['schema'] == 'U08_F01_PAUSED_PUBLIC_HANDOFF_V1' and handoff['selected_attempt'] == '003' and
            handoff['source'] == current and handoff['source_unchanged'] is True and
            handoff['active_runners'] == handoff['product_changes'] == handoff['old_tool_fixture_target_changes'] == 0,
            'Closed exact Main handoff')
    require(handoff['authorization'] == spec['authorization'], 'Actual Root035 authorization')
    check_ref(spec['authorization'])
    for row in handoff['closed_artifacts']:
        check_ref(row)
    check_ref(handoff['readback_helper'])
    actual_spec = spec['receipt']
    require(actual_spec['id'] == ROLE and actual_spec['kind'] == 'execution' and
            actual_spec['receipt'] == handoff['receipt'] and actual_spec['raw_log'] == handoff['raw'],
            'Actual supplemental producer identity')
    require(ROLE not in {r['id'] for r in selection['receipt_bindings']} and
            handoff['receipt']['path'] not in {r['path'] for r in selection['receipt_bindings']},
            'Supplemental execution cannot replace an old selected receipt')
    bound = receipt(actual_spec, current)
    require(bound['runner']['sha256'] == handoff['runner']['sha256'] == handoff['runner_archive']['sha256'],
            'Actual runner collection hash')
    for row in (handoff['runner'], handoff['runner_archive']):
        check_ref(row)
    require(check_ref(handoff['runner']).read_bytes() == check_ref(handoff['runner_archive']).read_bytes() ==
            check_ref(bound['runner']).read_bytes(), 'Exact runner copy')
    binding = read(check_ref(spec['execution_binding']))
    validate_execution_binding(binding, current, [bound], {ROLE}, ROOT)
    union = read(check_ref(binding['prepared_manifest']))
    require(len(union['files']) == handoff['dependency_count'] == 1356 and
            {r['path']: r for r in union['files']} == {r['path']: r for r in handoff['dependencies']},
            'Separate actual1356 union')
    snapshots = binding['intervals']
    require(len(snapshots) == 1 and snapshots[0]['roles'] == [ROLE] and binding['gaps'] == [],
            'Single actual supplemental interval')
    actual_sets = [snapshots[0]['snapshot']] + snapshots[0]['input_sets']
    require(actual_sets == [{k: r[k] for k in ('id', 'adapter', 'before', 'after')}
                           for r in handoff['snapshot_sets']], 'Exact actual snapshot refs')
    before_sets = [read(check_ref(s['before'])) for s in actual_sets]
    require([len(s['files']) for s in before_sets] == [35, 1322], 'Primary35/base1322')
    require(all(s['handoff_review'] == spec['authorization'] for s in before_sets), 'Timely Root035 bindings')
    old_manifest = read(check_ref(spec['original_1322_manifest']))
    require(before_sets[1]['files'] == old_manifest['files'], 'Old1322 inputs unchanged')
    require(before_sets[0]['external_inputs'] == handoff['primary_external_archives'] and
            len(handoff['primary_external_archives']) == 2 and
            sum(x['member_count'] for x in handoff['primary_external_archives']) == 8,
            'Preserved external2/eight reviewed members')
    dependency = read(check_ref(binding['dependency_review']))
    require(dependency['roles'][0]['inputs'] == union['files'], 'Full conservative dependency set')
    adaptations = read(check_ref(spec['adaptations']))
    rows = {r['current']['path']: r for r in adaptations['harnesses']}
    required_inputs = {str(Path(i['path']).relative_to(ROOT)) for i in bound['inputs']
                       if i['path'].endswith('.py')} | {handoff['runner']['path']}
    require(required_inputs <= rows.keys(), 'Every executed Python source reviewed')
    union_by_path = {r['path']: r for r in union['files']}
    for row in adaptations['harnesses']:
        path = check_ref(row['current']); nonempty(row['reason'])
        require(union_by_path.get(row['current']['path']) == row['current'], 'Adaptation absent from timely inputs')
        for evidence in row['evidence']:
            check_ref(evidence)
        if row['predecessor'] is None:
            require(row['classification'] == 'new_supplemental_tool', 'New tool classification')
        else:
            prior = check_ref(row['predecessor'])
            patch = ''.join(difflib.unified_diff(prior.read_text().splitlines(True), path.read_text().splitlines(True),
                fromfile=row['predecessor']['path'], tofile=row['current']['path']))
            require(check_ref(row['diff']).read_text() == patch, 'Exact supplemental adaptation diff')
    review = read(check_ref(spec['source_review']))
    main_path = check_ref(review['main_source'])
    require(union_by_path.get(review['main_source']['path']) == review['main_source'], 'Actual consumed Rust source')
    require({x['id']: x['text'] for x in review['source_assertions']} == SOURCE_ASSERTIONS,
            'Required reviewed fail-closed assertions')
    for item in review['source_assertions']:
        require(item['text'] in main_path.read_text(), 'Reviewed fail-closed source assertion changed')
    require(len(review['authored_oracle_inputs']) == 3, 'Exact authored skill/relic/fragment oracle inputs')
    for item in review['authored_oracle_inputs']:
        path = check_ref(item['ref'])
        require(union_by_path.get(item['ref']['path']) == item['ref'], 'Authored oracle absent from timely inputs')
        require(item['reviewed_text'] and all(text in path.read_text() for text in item['reviewed_text']),
                'Reviewed authored oracle/source changed')
    driver = check_ref(review['driver'])
    require(bound['command'] == ['python3', '-B', str(driver), str(main_path.parents[3]),
            str(main_path.parents[3] / 'PREPARED_MANIFEST.json'), before_sets[0]['prepared_manifest']['sha256']],
            'Actual reviewed driver/attempt/prepared-manifest command')
    raw = check_ref(handoff['raw']).read_text()
    documents, raw_summary = parse_raw(raw)
    summary = read(check_ref(handoff['summary']))
    transcript = read(check_ref(spec['transcript']))
    inputs = main_path.parents[2]
    oracle_ref = spec['oracle']
    require(union_by_path.get(oracle_ref['path']) == oracle_ref, 'Oracle was an actual timely input')
    behavior = validate_behavior(summary, transcript, documents, raw_summary, read(check_ref(oracle_ref)))
    build = validate_build(documents, raw, handoff, inputs, current)
    require(end(bound) <= instant(handoff['recorded_at_utc']), 'Handoff precedes completed execution')
    require(identity() == current, 'Source changed during supplemental verification')
    return {'schema': 'U08_SUPPLEMENTAL_PAUSED_VERIFICATION_V1', 'passed': True,
            'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'source': current, 'handoff': spec['handoff'], 'producer': bound,
            'execution_binding': spec['execution_binding'], 'union': binding['prepared_manifest'],
            'dependency_count': 1356, 'snapshot_counts': [35, 1322],
            'adaptations': spec['adaptations'], 'source_review': spec['source_review'],
            'raw': handoff['raw'], 'summary': handoff['summary'], 'transcript': spec['transcript'],
            'build': build, 'behavior': behavior, 'verifier_inputs': spec['verifier_inputs'],
            'original_matrix': {'config': spec['base_config'], 'selection': spec['base_selection'],
                'execution_binding': base['execution_harness_binding'], 'selected_roles': 58,
                'functional_roles_before_measurements': 50, 'performance_roles': 8,
                'quiescence': base['functional_quiescence'], 'performance_environments': base['performance_environments'],
                'scope': 'Original measurements remain after their actual50 functional roles. This later evidence-only execution is outside that matrix; no historical timestamp or role set is changed.'},
            'limitations': [behavior['source_asserted_checks'], behavior['retained_fact_scope'],
                'Focused paused manual consumer only; no normal-employee active AutoCastStart claim.',
                'No product/performance execution and no independent completion verdict by this verifier.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--spec', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    spec_path = Path(args.spec).resolve()
    output = Path(args.output).resolve()
    require(spec_path.is_relative_to(ROOT) and output.is_relative_to(ROOT), 'Repository metadata paths required')
    result = verify(read(spec_path))
    result['spec'] = ref(spec_path)
    write(output, result)
    print(json.dumps({'passed': True, 'report': ref(output), 'supplemental_roles': 1,
                      'unchanged_selected_roles': 58, 'product_executions': 0}))


if __name__ == '__main__':
    main()
