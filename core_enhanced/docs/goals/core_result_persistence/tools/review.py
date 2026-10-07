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
    source = json.loads((HERE / 'P2G_10_POLICY_001.json').read_text())['artifacts']
    before = next(a for a in source if a['ref']['Artifact Type'] == 'Working Revision')
    elements = [a for a in source if a['ref'] in before['payload']['Current Policy Element Revision Map'].values()]
    assert len(elements) == 15
    plan = json.loads((HERE / 'P2G_20_SOURCE_PLAN_002.json').read_text())['artifacts'][0]
    payload = plan['payload']
    rows = payload['Requirement Set']
    paths = sorted({p for r in rows for p in r['Required identity/location constraint']['locations']})
    missing = [p for p in paths if not (ROOT / p).is_file()]
    assert not missing, missing
    index = [file_row(p) for p in paths]
    body = file_row(str((HERE / 'CODE_REVIEW_001.ko.md').relative_to(ROOT)))
    archive = HERE / 'REVIEW_SOURCE_001.tar.gz'
    assert not archive.exists()
    with tarfile.open(archive, 'w:gz') as tar:
        for p in paths + [body['path'], str((HERE / 'evidence/PREDECESSOR_BASELINE_001.json').relative_to(ROOT)), str((HERE / 'evidence/GIT_STATUS_INTAKE.txt').relative_to(ROOT))]:
            tar.add(ROOT / p, arcname=p)
    write('SOURCE_INDEX_001.json', {'files': index, 'archive': file_row(str(archive.relative_to(ROOT))), 'inspection': 'selected source/call/owner boundary review; hash covers whole file, not claim of every line inspection'})
    arts, evidence, observations = [], [], []
    stamp = datetime.datetime.now(datetime.timezone.utc).isoformat()
    for row in rows:
        key = row['Requirement ID']
        subset = [x for x in index if x['path'] in row['Required identity/location constraint']['locations']]
        ev = artifact('Review Evidence', f'RE-CRRP-{key}-001', {
            'Repository/environment identity': {'repository': str(ROOT), 'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'axis': 'Workspace Actual'},
            'Source Set': subset, 'Observation command/method': 'selected sed/rg source, forward/reverse public callers, authoritative mutation/persistence and projection flow; exact file bytes and archive; unchanged U03 baseline reused after363-product/367-source currentness check with explicit status-only ROADMAP successor',
            'Raw evidence': file_row(str(archive.relative_to(ROOT))), 'Observed result': {'body': body, 'selector': f'Current Actual {key}; Proposed Implementation TD01–08; Validation Seams'},
            'Observation timestamp': stamp, 'Supported claim': row['Required claim/boundary'],
            'Limitations': 'static current-actual/delta/design review; no new implementation completed. Exact U03 final baseline reused as U03 preimplementation evidence; new U03 behavior requires V01–15 after implementation',
            'Runtime tests executed': [], 'Preimplementation Baseline Ref': file_row(str((HERE / 'evidence/PREDECESSOR_BASELINE_001.json').relative_to(ROOT)))})
        obs = artifact('Source Observation', f'SO-CRRP-{key}-001', {
            'Evaluated Working Revision Ref': before['ref'], 'Review Scope/Source Requirement Plan Ref': plan['ref'],
            'Requirement ID': key, 'Observed source identity/location/version': subset,
            'Disposition': 'confirmed', 'Observation Evidence Ref Set': [ev['ref']],
            'Supported claim/boundary': row['Required claim/boundary'], 'Limitation': 'Current actual/delta only; GEX validation pending'})
        arts.extend([ev, obs]); evidence.append(ev); observations.append(obs)
    basis = artifact('Reviewed Source Basis', 'RSB-CRRP-001', {'Selected Source Set': [
        {'Source ID': row['Requirement ID'], 'Source Observation Ref': obs['ref'], 'Review Evidence Ref': ev['ref'], 'Supported claim': row['Required claim/boundary'], 'Disposition': 'confirmed'}
        for row, obs, ev in zip(rows, observations, evidence)]})
    def required(e):
        n = int(e['payload']['Stable Element ID'].split('-')[-1])
        return [key for key, nums, *_ in GROUPS if n in nums]
    review = artifact('Code Review Artifact', 'CRA-CRRP-001', {
        'Input Working Revision Ref': before['ref'], 'Review Scope/Source Requirement Plan Ref': plan['ref'], 'Reviewed Source Basis Ref': basis['ref'],
        'Review Evidence Ref Set': [a['ref'] for a in evidence], 'Review body': body,
        'Element review mapping': [{'Reviewed Policy Element Revision Ref': e['ref'], 'Approved Target': e['payload']['Policy Statement'], 'Decision Revision Ref': e['payload']['Decision Revision Ref'], 'Approved Observable Policy Acceptance': e['payload']['Observable Policy Acceptance'], 'Required Claim/Boundary Set': required(e)} for e in elements],
        'Current Actual': 'CODE_REVIEW_001.ko.md#current-actual', 'Implementation Delta': 'CODE_REVIEW_001.ko.md#proposed-implementation',
        'Proposed Implementation': 'CODE_REVIEW_001.ko.md TD01–08 (owner/state/input/output/order/failure/transition/alternatives)',
        'Validation Seams': 'CODE_REVIEW_001.ko.md V01–15', 'Findings/Unknowns': 'CODE_REVIEW_001.ko.md F01–05',
        'Technical Decisions': 'CODE_REVIEW_001.ko.md TD01–08', 'Review Outcome': 'all 15 approved meanings preserved; implementation remains pending'})
    reviewed = []
    for e in elements:
        p = copy.deepcopy(e['payload']); p['Predecessor Policy Element Revision Ref'] = e['ref']; p['Intrinsic status'] = 'code_reviewed'
        reviewed.append(artifact('Policy Element Revision', e['payload']['Stable Element ID'] + '-r3', p))
    p = copy.deepcopy(before['payload']); p['Predecessor Working Revision Ref'] = before['ref']
    p['Current Policy Element Revision Map'] = {e['payload']['Stable Element ID']: e['ref'] for e in reviewed}
    working = artifact('Working Revision', 'WR-CRRP-003', p)
    bindings, candidates = [], []
    for old, new in zip(elements, reviewed):
        keys = required(old); number = old['payload']['Stable Element ID'].split('-')[-1]
        selected = [ev['ref'] for row, ev in zip(rows, evidence) if row['Requirement ID'] in keys]
        binding = artifact('Code Review Binding', f'CRB-CRRP-{number}-001', {
            'Working Revision Ref': working['ref'], 'Bound Working Revision Ref': working['ref'], 'Policy Element Revision Ref': new['ref'],
            'Code Review Artifact Ref': review['ref'], 'Review Path': 'policy_revision_review', 'Predecessor Code Review Binding Ref': None,
            'Target Current-Element-Map Match Evidence': {'element': new['payload']['Stable Element ID'], 'exact_ref_equal': True},
            'P2G-20 Review Scope/Source Requirement Plan Ref': plan['ref'], 'Reviewed Source Basis Ref': basis['ref'],
            'Review Evidence Ref Set': selected, 'Covered Required Claim/Boundary Set': keys,
            'Evaluated applicable active Change/Obligation/Closure Ref Set': [], 'Applicable Policy Impact Closure Canonical Union': []})
        candidate = artifact('Candidate Review Envelope', f'CRE-CRRP-{number}-001', {
            'Candidate Batch ID': 'CAND-CRRP-001', 'Input Working Revision Ref': before['ref'], 'Reviewed Element/Revision': old['ref'],
            'Review Path': 'policy_revision_review', 'Candidate Code Review Artifact Ref Set': [review['ref']], 'Candidate Review Evidence Ref Set': selected,
            'Proposed code_reviewed Revision Payload': new['payload'], 'Candidate Binding Payload': binding['payload'],
            'Candidate Result': 'accepted-for-complete-result', 'Abort Reason': None, 'Lifecycle Disposition': 'non-current'})
        bindings.append(binding); candidates.append(candidate)
    batch = artifact('Review Batch', 'RB-CRRP-001', {
        'Candidate Batch Outcome': 'ready-for-complete-result', 'Input Working Revision Ref': before['ref'], 'Candidate Review Envelope Ref Set': [c['ref'] for c in candidates],
        'Review Scope/Source Requirement Plan Ref': plan['ref'], 'Source Observation Ref Set': [o['ref'] for o in observations],
        'Code Review Artifact Ref Set': [review['ref']], 'Review Evidence Ref Set': [e['ref'] for e in evidence], 'Reviewed Source Basis Ref': basis['ref'],
        'Produced code_reviewed Policy Element Revision Ref Set': [e['ref'] for e in reviewed], 'Result Working Revision Ref': working['ref'],
        'Result Current code_reviewed Policy Element Revision Ref Set': [e['ref'] for e in reviewed], 'Result-basis Code Review Binding Ref Set': [b['ref'] for b in bindings],
        'Batch Completeness Evidence': {'required_claims': len(rows), 'required_gaps': [], 'new_material_policy_findings': [], 'one_result_successor': True, 'static_review_only': True},
        'Applicable P2G-20 Obligation Key Set': [], 'Complete Result Validation': 'passed', 'Abort Evidence': None})
    arts.extend([basis, review, *candidates, *reviewed, working, *bindings, batch])
    all_artifacts = json.loads((HERE / 'P2G_00_INITIAL_001.json').read_text())['artifacts'] + source + [plan] + arts
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
    write('P2G_20_REVIEW_001.json', {'artifacts': arts})
    write('P2G_20_RESULT_001.json', {'stage': 'P2G-20', 'status': 'complete', 'Input Working Revision Ref': before['ref'], 'Result Working Revision Ref': working['ref'], 'Review Batch Ref': batch['ref'], 'Review Scope/Source Requirement Plan Ref': plan['ref'], 'checks': {'exact_reference_integrity': True, 'source_unchanged': True, 'input_acceptance_preserved': True, 'all_15_reviewed_and_bound': True, 'all_8_source_requirements_confirmed': True, 'material_policy_findings': 0, 'required_source_gaps': 0}, 'production_mutation': False, 'next': 'P2G-30'})
    view = (HERE / 'WORKING_POLICY.ko.md').read_text().replace('`WR-CRRP-002`', '`WR-CRRP-003`')
    view = view.replace('현재 P2G-10 완료, 다음은 P2G-20 정식 코드 검토다.', '현재 P2G-20 완료, 다음은 P2G-30 정책 동결이다.')
    view = view.replace('코드 검토·제품 구현 완료를 뜻하지 않는다.', '제품 구현 완료를 뜻하지 않는다.')
    view += '\n[코드 검토](CODE_REVIEW_001.ko.md) · [완전한 Review Batch](P2G_20_REVIEW_001.json) · [검증 결과](P2G_20_RESULT_001.json).\n'
    (HERE / 'WORKING_POLICY.ko.md').write_text(view)
    print('P2G-20 complete:', len(index), 'exact source files, 8 source groups, 15 exact result bindings; no production changes')

if __name__ == '__main__': main()
