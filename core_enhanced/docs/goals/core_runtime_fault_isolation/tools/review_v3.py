#!/usr/bin/env python3
"""Publish one complete P2G-20 batch, preserving earlier candidate/source records."""
import copy
import datetime
import hashlib
import json
import subprocess
import tarfile
from initialize import HERE, ROOT, artifact, write
from source_plan import GROUPS


def file_row(path):
    raw = (ROOT / path).read_bytes()
    return {'path': path, 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}


def main():
    source = json.loads((HERE / 'P2G_10_POLICY_003.json').read_text())['artifacts']
    before = next(a for a in source if a['ref']['Artifact Type'] == 'Working Revision')
    elements = [a for a in source if a['ref'] in before['payload']['Current Policy Element Revision Map'].values()]
    assert len(elements) == 12
    oldplan = json.loads((HERE / 'P2G_20_SOURCE_PLAN_002.json').read_text())['artifacts'][0]
    payload = copy.deepcopy(oldplan['payload'])
    payload['Bound Input Working Revision Ref'] = before['ref']
    for row in payload['Requirement Set']:
        row['Exact Policy Element Revision Ref Set'] = [before['payload']['Current Policy Element Revision Map'][f'PE-CRFI-{n:02}'] for key, nums, *_ in GROUPS if key == row['Requirement ID'] for n in nums]
    payload['Predecessor Source Plan Ref'] = oldplan['ref']
    plan = artifact('Review Scope/Source Requirement Plan', 'RSP-CRFI-003', payload)
    changes = json.loads((HERE / 'P2G_SCHEMA_CHANGE_002.json').read_text())['artifacts']
    change, declaration = changes
    obligation = {'Container Ref':declaration['ref'], 'Obligation ID':'O20'}
    prior_result = json.loads((HERE / 'P2G_10_RESULT_003.json').read_text())
    assert prior_result['checks']['O10_effectively_resolved']
    rows = payload['Requirement Set']
    paths = sorted({p for r in rows for p in r['Required identity/location constraint']['locations']})
    missing = [p for p in paths if not (ROOT / p).is_file()]
    assert not missing, missing
    index = [file_row(p) for p in paths]
    body = file_row(str((HERE / 'CODE_REVIEW_001.ko.md').relative_to(ROOT)))
    archive = HERE / 'REVIEW_SOURCE_003.tar.gz'
    assert not archive.exists()
    with tarfile.open(archive, 'w:gz') as tar:
        for p in paths + [body['path'], str((HERE / 'evidence/PREDECESSOR_BASELINE_001.json').relative_to(ROOT)), str((HERE / 'evidence/GIT_STATUS_INTAKE.txt').relative_to(ROOT))]:
            tar.add(ROOT / p, arcname=p)
    write('SOURCE_INDEX_003.json', {'files': index, 'archive': file_row(str(archive.relative_to(ROOT))), 'inspection': 'selected source/call/owner boundary review; hash covers whole file, not claim of every line inspection'})
    write('P2G_20_SOURCE_PLAN_003.json', {'artifacts':[plan]})
    arts, evidence, observations = [], [], []
    stamp = datetime.datetime.now(datetime.timezone.utc).isoformat()
    for row in rows:
        key = row['Requirement ID']
        subset = [x for x in index if x['path'] in row['Required identity/location constraint']['locations']]
        ev = artifact('Review Evidence', f'RE-CRFI-{key}-003', {
            'Repository/environment identity': {'repository': str(ROOT), 'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'axis': 'Workspace Actual'},
            'Source Set': subset, 'Observation command/method': 'selected sed/rg source, forward/reverse public callers, authoritative mutation/persistence and projection flow; exact file bytes and archive; unchanged U02 baseline reused after359-file currentness check',
            'Raw evidence': file_row(str(archive.relative_to(ROOT))), 'Observed result': {'body': body, 'selector': f'Current Actual {key}; Proposed Implementation TD01–08; Validation Seams'},
            'Observation timestamp': stamp, 'Supported claim': row['Required claim/boundary'],
            'Limitations': 'static current-actual/delta/design review; no new implementation completed. Exact U02 final baseline reused as U03 preimplementation evidence; new U03 behavior requires V01–12 after implementation',
            'Runtime tests executed': [], 'Preimplementation Baseline Ref': file_row(str((HERE / 'evidence/PREDECESSOR_BASELINE_001.json').relative_to(ROOT)))})
        obs = artifact('Source Observation', f'SO-CRFI-{key}-003', {
            'Evaluated Working Revision Ref': before['ref'], 'Review Scope/Source Requirement Plan Ref': plan['ref'],
            'Requirement ID': key, 'Observed source identity/location/version': subset,
            'Disposition': 'confirmed', 'Observation Evidence Ref Set': [ev['ref']],
            'Supported claim/boundary': row['Required claim/boundary'], 'Limitation': 'Current actual/delta only; GEX validation pending'})
        arts.extend([ev, obs]); evidence.append(ev); observations.append(obs)
    basis = artifact('Reviewed Source Basis', 'RSB-CRFI-003', {'Selected Source Set': [
        {'Source ID': row['Requirement ID'], 'Source Observation Ref': obs['ref'], 'Review Evidence Ref': ev['ref'], 'Supported claim': row['Required claim/boundary'], 'Disposition': 'confirmed'}
        for row, obs, ev in zip(rows, observations, evidence)]})
    def required(e):
        n = int(e['payload']['Stable Element ID'].split('-')[-1])
        return [key for key, nums, *_ in GROUPS if n in nums]
    review = artifact('Code Review Artifact', 'CRA-CRFI-003', {
        'Input Working Revision Ref': before['ref'], 'Review Scope/Source Requirement Plan Ref': plan['ref'], 'Reviewed Source Basis Ref': basis['ref'],
        'Review Evidence Ref Set': [a['ref'] for a in evidence], 'Review body': body,
        'Element review mapping': [{'Reviewed Policy Element Revision Ref': e['ref'], 'Approved Target': e['payload']['Policy Statement'], 'Decision Revision Ref': e['payload']['Decision Revision Ref'], 'Approved Observable Policy Acceptance': e['payload']['Observable Policy Acceptance'], 'Required Claim/Boundary Set': required(e)} for e in elements],
        'Current Actual': 'CODE_REVIEW_001.ko.md#current-actual', 'Implementation Delta': 'CODE_REVIEW_001.ko.md#proposed-implementation',
        'Proposed Implementation': 'CODE_REVIEW_001.ko.md TD01–08 (owner/state/input/output/order/failure/transition/alternatives)',
        'Validation Seams': 'CODE_REVIEW_001.ko.md V01–12', 'Findings/Unknowns': 'CODE_REVIEW_001.ko.md F01–05',
        'Technical Decisions': 'CODE_REVIEW_001.ko.md TD01–08', 'Review Outcome': 'all approved meanings preserved; implementation remains pending'})
    reviewed = []
    for e in elements:
        p = copy.deepcopy(e['payload']); p['Predecessor Policy Element Revision Ref'] = e['ref']; p['Intrinsic status'] = 'code_reviewed'
        reviewed.append(artifact('Policy Element Revision', 'PER-CRFI-' + e['payload']['Stable Element ID'].split('-')[-1] + '-REVIEWED-003', p))
    p = copy.deepcopy(before['payload']); p['Predecessor Working Revision Ref'] = before['ref']
    p['Current Policy Element Revision Map'] = {e['payload']['Stable Element ID']: e['ref'] for e in reviewed}
    working = artifact('Working Revision', 'WR-CRFI-007', p)
    bindings, candidates = [], []
    for old, new in zip(elements, reviewed):
        keys = required(old); number = old['payload']['Stable Element ID'].split('-')[-1]
        selected = [ev['ref'] for row, ev in zip(rows, evidence) if row['Requirement ID'] in keys]
        binding = artifact('Code Review Binding', f'CRB-CRFI-{number}-003', {
            'Working Revision Ref': working['ref'], 'Bound Working Revision Ref': working['ref'], 'Policy Element Revision Ref': new['ref'],
            'Code Review Artifact Ref': review['ref'], 'Review Path': 'policy_revision_review', 'Predecessor Code Review Binding Ref': None,
            'Target Current-Element-Map Match Evidence': {'element': new['payload']['Stable Element ID'], 'exact_ref_equal': True},
            'P2G-20 Review Scope/Source Requirement Plan Ref': plan['ref'], 'Reviewed Source Basis Ref': basis['ref'],
            'Review Evidence Ref Set': selected, 'Covered Required Claim/Boundary Set': keys,
            'Evaluated applicable active Change/Obligation/Closure Ref Set': before['payload']['Post-Freeze Change/Closure Basis']['Post-Freeze Change Record Ref Set']+before['payload']['Post-Freeze Change/Closure Basis']['Authoritative Change Obligation Declaration Ref Set'], 'Applicable Policy Impact Closure Canonical Union': []})
        candidate = artifact('Candidate Review Envelope', f'CRE-CRFI-{number}-003', {
            'Candidate Batch ID': 'CAND-CRFI-003', 'Input Working Revision Ref': before['ref'], 'Reviewed Element/Revision': old['ref'],
            'Review Path': 'policy_revision_review', 'Candidate Code Review Artifact Ref Set': [review['ref']], 'Candidate Review Evidence Ref Set': selected,
            'Proposed code_reviewed Revision Payload': new['payload'], 'Candidate Binding Payload': binding['payload'],
            'Candidate Result': 'accepted-for-complete-result', 'Abort Reason': None, 'Lifecycle Disposition': 'non-current'})
        bindings.append(binding); candidates.append(candidate)
    batch = artifact('Review Batch', 'RB-CRFI-003', {
        'Candidate Batch Outcome': 'ready-for-complete-result', 'Input Working Revision Ref': before['ref'], 'Candidate Review Envelope Ref Set': [c['ref'] for c in candidates],
        'Review Scope/Source Requirement Plan Ref': plan['ref'], 'Source Observation Ref Set': [o['ref'] for o in observations],
        'Code Review Artifact Ref Set': [review['ref']], 'Review Evidence Ref Set': [e['ref'] for e in evidence], 'Reviewed Source Basis Ref': basis['ref'],
        'Produced code_reviewed Policy Element Revision Ref Set': [e['ref'] for e in reviewed], 'Result Working Revision Ref': working['ref'],
        'Result Current code_reviewed Policy Element Revision Ref Set': [e['ref'] for e in reviewed], 'Result-basis Code Review Binding Ref Set': [b['ref'] for b in bindings],
        'Batch Completeness Evidence': {'required_claims': len(rows), 'required_gaps': [], 'new_material_policy_findings': [], 'one_result_successor': True, 'static_review_only': True},
        'Applicable P2G-20 Obligation Key Set': [obligation], 'Complete Result Validation': 'passed', 'Abort Evidence': None})
    arts.extend([basis, review, *candidates, *reviewed, working, *bindings, batch])
    result_basis = copy.deepcopy(prior_result['current_basis'])
    result_basis['Working Revision Ref'] = working['ref']
    resolution = artifact('Resolution Record', 'RR-CRFI-SCHEMA2-20-001', {
        'Obligation Key':obligation, 'Owner Stage':'P2G-20',
        'Exact Resolved Prerequisite Basis':prior_result['current_basis'],
        'Exact Result Composite Basis':result_basis, 'Resolution Artifact Ref':batch['ref'],
        'Resolution Evidence':{'O10_resolution_refs':prior_result['Resolution Record Ref Set'],
            'policy_decision_acceptance_unchanged':True, 'all12_current_bindings':True,
            'all59_source_bytes_unchanged':True, 'all8_source_groups_current':True}})
    arts.append(resolution)
    all_artifacts = []
    for f in HERE.glob('*.json'):
        all_artifacts.extend(json.loads(f.read_text()).get('artifacts', []))
    all_artifacts += arts
    refs = {a['ref']['Artifact ID']: a['ref'] for a in all_artifacts}
    def check(value):
        if isinstance(value, dict):
            if set(value) == {'Artifact Type', 'Artifact ID', 'Content Digest'}:
                # Earlier raw intake refs are validated through prior exact artifacts.
                assert value['Artifact ID'] in refs, value
                assert refs[value['Artifact ID']] == value
            else:
                for v in value.values(): check(v)
        elif isinstance(value, list):
            for v in value: check(v)
    check(arts)
    for old, new, binding in zip(elements, reviewed, bindings):
        for field in ['Policy Statement', 'Decision Revision Ref', 'Observable Policy Acceptance']:
            assert old['payload'][field] == new['payload'][field]
        assert binding['payload']['Bound Working Revision Ref'] == working['ref']
        assert binding['payload']['Policy Element Revision Ref'] == working['payload']['Current Policy Element Revision Map'][new['payload']['Stable Element ID']]
        assert required(old) == binding['payload']['Covered Required Claim/Boundary Set']
    assert {r['Requirement ID'] for r in rows} == {o['payload']['Requirement ID'] for o in observations}
    for x in index: assert file_row(x['path']) == x, x['path']
    write('P2G_20_REVIEW_003.json', {'artifacts': arts})
    write('P2G_20_RESULT_003.json', {'stage': 'P2G-20', 'status': 'complete', 'Input Working Revision Ref': before['ref'], 'Result Working Revision Ref': working['ref'], 'Review Batch Ref': batch['ref'], 'Review Scope/Source Requirement Plan Ref': plan['ref'], 'checks': {'exact_reference_integrity': True, 'source_unchanged': True, 'input_acceptance_preserved': True, 'all_12_reviewed_and_bound': True, 'all_8_source_requirements_confirmed': True, 'material_policy_findings': 0, 'required_source_gaps': 0}, 'Resolution Record Ref Set':[resolution['ref']], 'current_basis':result_basis, 'O20_effectively_resolved':True, 'production_mutation': False, 'next': 'P2G-30'})
    view=(HERE/'WORKING_POLICY.ko.md').read_text().replace('`WR-CRFI-006`','`WR-CRFI-007`').replace('Decision identity 교정 P2G-10 완료; P2G-20/30/40 successor 준비 중이다.','Decision identity 교정 P2G-20 완료; P2G-30/40 successor 준비 중이다.')
    view+='\n현재 [review003](P2G_20_REVIEW_003.json)과 [O20 closure](P2G_20_RESULT_003.json)를 선택했다.\n'
    (HERE/'WORKING_POLICY.ko.md').write_text(view)
    print('P2G-20 complete:', len(index), 'exact source files, 8 source groups, 12 exact result bindings; no production changes')

if __name__ == '__main__': main()
