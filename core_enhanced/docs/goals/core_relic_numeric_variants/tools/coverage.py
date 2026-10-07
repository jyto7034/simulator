"""Bind all 32 frozen sentences separately; keep freeze/audit completion pending."""
from pathlib import Path
import hashlib
import json
import re
import tarfile
from evidence_common import (ROOT, read, write, ref, load_config, result_base,
    selected, load_stage, evidence_claim, check_ref, nonempty, identity)
from coverage_contracts import contracts


def performance(selection, current):
    artifacts = selection['artifacts']
    values = {k: read(ROOT / v['path']) for k, v in artifacts.items()}
    pc = values['pc_cost']
    by_id = {r['id']: r for r in selection['receipt_bindings']}
    isolate = values['pc_isolate']
    source_rows = [[r['path'],r['bytes'],r['sha256']] for r in current['files']]
    expected_copy = {'files':source_rows,'file_count':len(source_rows),
                     'digest':hashlib.sha256(json.dumps(source_rows,separators=(',',':')).encode()).hexdigest()}
    assert isolate['current_source'] == isolate['copied_source'] == expected_copy
    assert isolate['isolate'] + '/Cargo.toml' in by_id['pc']['command']
    before_rows = {r[0]:r for r in source_rows}
    built_rows = {r[0]:r for r in isolate['built_source']['files']}
    assert before_rows.keys() == built_rows.keys()
    assert {p for p in before_rows if before_rows[p] != built_rows[p]} == set(isolate['instrumented_paths'])
    assert set(isolate['instrumented_paths']) == {'core_enhanced/src/game/battle/core/value_expr_runtime.rs',
                                                  'core_enhanced/src/game/battle/runtime_control.rs'}
    hooks = {p:row[2] for p,row in built_rows.items() if p in isolate['instrumented_paths']}
    hooks.update(isolate['supplemental'])
    assert set(isolate['supplemental']) == {'review_probes.rs','pc_probe.rs'}
    isolate_path = ROOT / artifacts['pc_isolate']['path']
    match = re.fullmatch(r'pc_isolate_([0-9]+)\.json', isolate_path.name)
    assert match
    archive_path = isolate_path.parent / ('pc_isolate_hooks_' + match[1] + '.tar.gz')
    assert ref(archive_path)['sha256'] == isolate['hooks_archive']['sha256']
    assert ref(archive_path)['bytes'] == isolate['hooks_archive']['bytes']
    assert str(archive_path.relative_to(ROOT)) in {r['path'] for r in artifacts['pc_isolate']['raw_files']}
    seen = set()
    with tarfile.open(archive_path) as archive:
        for member in archive:
            assert member.isfile() and member.name in hooks and member.name not in seen
            assert hashlib.sha256(archive.extractfile(member).read()).hexdigest() == hooks[member.name]
            seen.add(member.name)
    assert seen == set(hooks)
    assert pc['passed'] is True and pc['rows'] == 576 and pc['distinct_conditions'] == 36
    assert pc['source_sha256'] == current['sha256']
    assert pc['after']['sha256'] == artifacts['pc_raw']['sha256']
    assert pc['pc_receipt']['sha256'] == by_id['pc']['sha256']
    assert pc['layout_log_sha256'] == by_id['pc']['raw_log']['sha256']
    assert (pc['successful_conditions'], pc['typed_refusal_conditions']) == (31, 5)
    assert pc['immediate_u07m']['non_timing_exact'] is True and pc['immediate_u07m']['allowed_delta'] == 0
    controls = pc['controls']
    expected = {'new_refusal','gameplay_hp','gameplay_log','native_work','native_clone','native_retained',
                'retained_extent','allocation_count','allocation_bytes','evaluator_work','evaluator_usage',
                'completion','error_code','error_site','error_cap','error_requested','actual_historical_pc001_regression'}
    assert {r['name'] for r in controls if r.get('rejected') is True} == expected
    assert len(controls) == 18 and controls[0]['passed'] is True
    assert len(values['pc_raw']['rows']) == 576
    gameplay = values['pc_summary']
    assert gameplay['current']['sha256'] == artifacts['pc_raw']['sha256']
    assert gameplay['rows_each'] == 576 and gameplay['distinct_conditions'] == 36
    assert gameplay['successful_conditions'] == 31 and gameplay['typed_refusal_conditions'] == 5
    assert gameplay['same_gameplay_oracle_and_typed_refusal'] is True
    r05 = values['r05_summary']
    assert r05['source_digest'] == current['sha256']
    assert r05['raw_sha256'] == artifacts['r05_raw']['sha256']
    assert r05['other_hard_guard_failures'] == []
    assert r05['concurrent_sessions']['caps']['peak'] == 9030188
    assert r05['harness_proof']['workload_input_exact_prior'] is True
    assert r05['harness_proof']['guards_oracles_caps_unchanged'] is True
    assert r05['raw_exit_code'] == by_id['r05']['exit_code']
    excess = r05['concurrent_peak_excess_bytes']
    assert excess == max(0, r05['concurrent_sessions']['max_peak_bytes'] - 9030188)
    assert r05['classification'] == ('warning_concurrent_peak_only' if excess else 'pass_original_guards')
    assert r05['raw_exit_code'] == (101 if excess else 0)
    for key in ('event_heavy','concurrent_sessions'):
        row = r05[key]
        assert len(row['operations']) == len(row['retained_samples']) == len(row['peak_samples']) == 5
        assert len(values['r05_raw'][key]['allocation_raw_samples']) == 5
        assert len(values['r05_raw'][key]['latency_raw_samples']) == 21
        assert row['repeatable'] is True and row['parity'] is True
        assert max(row['operations']) <= row['caps']['operations']
        assert max(row['retained_samples']) <= row['caps']['retained']
        if key == 'event_heavy':
            assert max(row['peak_samples']) <= row['caps']['peak']
    deps = values['r05_dependencies']
    assert deps['passed'] is True and deps['target_was_absent'] is True
    assert deps['source_sha256'] == current['sha256']
    assert deps['raw_sha256'] == artifacts['r05_raw']['sha256']
    assert deps['summary_sha256'] == artifacts['r05_summary']['sha256']
    assert deps['receipt_sha256'] == by_id['r05']['sha256']
    for key, count in [('playback_actual',26), ('lifecycle_actual',10), ('host_actual',4)]:
        value = values[key]
        assert value['passed'] is True and value['source_sha256'] == current['sha256']
        assert len(value['rows']) == count
        binary_name = 'game_server' if key == 'host_actual' else 'measure_runtime'
        binaries = selection['binary_bindings']['debug']['binaries']
        digest = next(v for p, v in binaries.items() if Path(p).name == binary_name)
        assert value['binary_sha256'] == digest
    assert all(len(row['phases']) == 2 for row in values['host_actual']['rows'])
    assert values['playback_actual']['mismatch']['exit_code'] != 0
    assert values['playback_actual']['mismatch']['output_absent'] is True
    return {'pc':artifacts['pc_cost'], 'r05':artifacts['r05_summary'],
            'r05_classification':r05['classification'], 'r05_concurrent_peak_excess_bytes':excess,
            'other_hard_guard_failures':[], 'sample_counts':{'playback':26,'lifecycle':10,'host':4,'host_phases':8},
            'limitations':['Shared-host timing observations only.', 'Inherited R05 concurrent-peak-only warning; cap9030188 unchanged.',
                          '20-minute stress excluded under the inherited policy.',
                          'Native admission, retained owner extent, allocator requested bytes and process RSS are distinct.']}


def main():
    cfg, current, names = load_config()
    contract = contracts()
    selection = selected(cfg, current, names)
    impact = load_stage('impact', cfg, current, names)
    adaptation = load_stage('adaptation', cfg, current, names)
    assert impact['selection'] == adaptation['selection'] == ref(names['selection'])
    assert adaptation['predecessor_impact'] == ref(names['impact'])
    assert len(impact['conditions']) == 191 and len(impact['r06_acceptances']) == 31
    assert adaptation['immediate_input_count'] == 448
    all_sentences = {s['id']:s for g in contract['groups'] for s in g['sentences']}
    assert len(all_sentences) == 32 and set(cfg['sentences']) == set(all_sentences)
    sentences = []
    for key, original in all_sentences.items():
        row = cfg['sentences'][key]
        assert row['text'] == original['text'], key
        assert isinstance(row['limitations'], list)
        if key == 'PE-CRNV-16.2':
            assert row['status'] == 'pending_freeze_and_independent_audit'
            assert row['claims'] == [], 'No independent verdict can be self-awarded'
            assert row['followup']['target'] == cfg['target']
            assert row['followup']['required_verdict'] == 'Complete / High / open0'
            nonempty(row['followup']['closure_locator_contract'])
            claims = []
        else:
            assert row['status'] == 'self_verified_current_scope', key
            claims = [evidence_claim(c, selection) for c in row['claims']]
            assert claims, key
        if key == 'PE-CRNV-11.2':
            scope = row['scope_boundary']
            assert scope['actual_revival_catalyst_owner_exists'] is False
            assert scope['actual_two_catalyst_cross_test_executed'] is False
            assert scope['future_owner'] == 'U17'
            assert scope['frozen_policy_preserved'] is True
            assert row['limitations'], 'Do not invent an existing revival-catalyst owner'
            check_ref(scope['decision_record'])
        sentences.append(dict(row, id=key, claims=claims))
    dimensions, nodes, seams = [], [], []
    for family, output in [('dimensions', dimensions), ('nodes', nodes), ('seams', seams)]:
        originals = {r['id']: r for r in contract[family]}
        assert set(cfg[family]) == set(originals)
        for key, original in originals.items():
            row = cfg[family][key]
            nonempty(row['meaning'])
            assert row['sentence_refs'] and set(row['sentence_refs']) <= set(all_sentences)
            claims = [evidence_claim(c, selection) for c in row['claims']]
            if family == 'seams' and key == 'IVA-CRNV-V16':
                assert row['pending_independent_completion'] is True
            assert claims, (family, key)
            output.append(dict(row, id=key, contract=original, claims=claims))
    assert {s for row in seams for s in row['sentence_refs']} == set(all_sentences)
    performance_result = performance(selection, current)
    documents = cfg['canonical_documents']
    assert documents
    for row in documents:
        check_ref(row)
    groups = []
    for original in contract['groups']:
        ids = {s['id'] for s in original['sentences']}
        groups.append(dict(original, sentence_evidence=[s for s in sentences if s['id'] in ids],
                           status='self-verification mapped; independent completion pending'))
    assert identity() == current
    write(names['coverage'], dict(result_base(cfg, current),
        goal_set='GS-CRNV-001', status='GEX10 self-verification; independent GEX20 pending',
        acceptances=groups, sentences=sentences, required_dimensions=dimensions,
        execution_nodes=nodes, integration_seams=seams,
        counts={'predecessor_conditions':191,'u08_groups':16,'main_conditions':207,
                'u08_acceptance_sentences':32,'r06_separate':31,'dimensions':10,'nodes':3,'integration_seams':16},
        suite_counts={r['id']:r['passed_count'] for r in selection['receipt_bindings'] if r['kind'] in ('suite','focused')},
        selection=ref(names['selection']), predecessor_impact=ref(names['impact']),
        compatibility_adaptation=ref(names['adaptation']), performance=performance_result,
        canonical_documentation=documents, pending_completion_sentences=['PE-CRNV-16.2'],
        frozen_policy=contract['freeze'], phase_plan=contract['phase_plan']))
    print('Mapped32 sentences/16 groups/10 dimensions/191+31; freeze and independent completion remain pending')


if __name__ == '__main__':
    main()
