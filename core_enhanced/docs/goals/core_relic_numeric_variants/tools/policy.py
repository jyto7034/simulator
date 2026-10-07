"""U08 P2G-10 publisher. No product code execution or modification."""
import copy
import json
from initialize import HERE, artifact, write, sha

POLICIES = json.loads((HERE / 'tools/policy_rows.json').read_text())
EXCLUSIONS = [
    ('全상점/유물노드 배치·공급량·가격·공식 개별 penalty·Unity UI', 'ROADMAP U08 exclusions/D03,D04;U16/U21/U26-B'),
    ('새 생성효과 closure/구조 recipe와 인과 root/fork/join 구현', 'ROADMAP U09/U10;H04 R15/R16은 후속 owner'),
    ('새 슬롯/중복/강화 정책·최종보스 고유 구현·전투중 disk codec·역사소급수정', 'ROADMAP U08;current request;U04 supported boundary'),
]


def main():
    intake, before = json.loads((HERE / 'P2G_00_INITIAL_001.json').read_text())['artifacts']
    assert json.loads((HERE / 'P2G_00_RESULT_001.json').read_text())['status'] == 'complete'
    initial_hash = sha(HERE / 'P2G_00_INITIAL_001.json')
    basis = {'Input Working Revision Ref': before['ref'], 'Exact User-Approved Target Scope/Change Request': None,
             'Policy Intake Snapshot Ref': intake['ref']}
    arts, current, decisions, dispositions = [], {}, [], []
    assert len(POLICIES) == 16
    for n, (name, span, statement, acceptance, reason) in enumerate(POLICIES, 1):
        eid = f'PE-CRNV-{n:02}'
        opened = artifact('Policy Element Revision', f'PER-CRNV-{n:02}-OPEN-001', {
            'Stable Element ID': eid, 'Predecessor Policy Element Revision Ref': None,
            'Policy Statement': statement, 'Decision Revision Ref': None,
            'Observable Policy Acceptance': [], 'Intrinsic status': 'open'})
        decision = artifact('Decision Revision', f'DR-CRNV-{n:02}-001', {
            'Predecessor Decision Revision Ref': None, 'Affected Element IDs': [eid],
            'Decision Name': name, 'Exact Decision': statement,
            'Decision Authority': 'AI application/detail of adopted P08/H04 R14 policy and current continuous scope. Existing user agreements remain primary; no new user answer fabricated.',
            'Exact User Approval Reference': {'intake_ref': intake['ref'], 'items': ['IN-001', 'IN-002', 'IN-003', 'IN-005', 'IN-009'], 'specific_span': span},
            'Source Reference': {'intake_ref': intake['ref'], 'items': ['IN-002', 'IN-005', 'IN-009'], 'span': span},
            'Reason': reason, 'Impact': acceptance,
            'Alternatives not selected': 'Shared mutable relic definition;type-wide corrosion;unconditional closure fallback;new slot/duplicate policy;silent unsupported authoring;rawsource replacement;free Event bypass;cost/result partial commit;new official balance numbers.',
            'Delegation Criteria': {
                'existing_agreements_preserved': 'P08 latest rulebook and ROADMAP C02/C03/C04 preserve3slots/copy independence/flat+percent/one-time corruption and current prior contracts.',
                'derived_from_ownership': reason,
                'no_unapproved_global_rule': 'No new official individual penalty,drop frequency,price,total gameplay possession cap,common amplification cap or whole-damage formula.',
                'local_impact': 'U08 normal relic copy/numeric/variant execution. U09/U10/U15/U16/U21/U26-B retain their later assigned scopes.',
                'user_data_preserved': 'Existing dirty work,legacy assets,saves,failed evidence,frozen targets and audits preserved.'},
            'Revisability': 'Implementation means may vary within preserved public meaning; changed meaning/ownership/acceptance returns to earliest P2G owner using immutable successor.'})
        decided = artifact('Policy Element Revision', f'PER-CRNV-{n:02}-DECIDED-001', {
            'Stable Element ID': eid, 'Predecessor Policy Element Revision Ref': opened['ref'],
            'Policy Statement': statement, 'Decision Revision Ref': decision['ref'],
            'Observable Policy Acceptance': acceptance, 'Intrinsic status': 'decided'})
        disposition = artifact('Candidate Policy Element Disposition Record', f'CD-CRNV-{n:02}-001', {
            'Candidate ID': f'C-CRNV-{n:02}', 'P2G-10 Evaluation Basis': basis,
            'Source Reference': decision['payload']['Source Reference'], 'Candidate Statement': statement,
            'Disposition': 'promoted', 'Stable Element ID': eid,
            'Initial Open Policy Element Revision Ref': opened['ref'], 'Decision Revision Ref': decision['ref']})
        arts.extend([opened, decision, decided, disposition])
        current[eid] = decided['ref']; decisions.append(decision['ref']); dispositions.append(disposition['ref'])
    for n, (statement, locator) in enumerate(EXCLUSIONS, 1):
        item = artifact('Candidate Policy Element Disposition Record', f'CD-CRNV-OUT-{n:02}-001', {
            'Candidate ID': f'C-CRNV-OUT-{n:02}', 'P2G-10 Evaluation Basis': basis,
            'Source Reference': {'intake_ref': intake['ref'], 'span': locator},
            'Candidate Statement': statement, 'Disposition': 'out_of_scope',
            'Evidence/Decision Reference': locator, 'Stable Element ID': None,
            'Initial Open Policy Element Revision Ref': None})
        arts.append(item); dispositions.append(item['ref'])
    payload = copy.deepcopy(before['payload'])
    payload.update({'Predecessor Working Revision Ref': before['ref'], 'Current Policy Element Revision Map': current,
                    'Decision Revision Ref Set': decisions})
    result = artifact('Working Revision', 'WR-CRNV-002', payload); arts.append(result)
    assert len(current) == len(decisions) == 16 and len(dispositions) == 19
    assert len(set(x['Artifact ID'] for x in dispositions)) == len(dispositions)
    for a in arts:
        assert artifact(a['ref']['Artifact Type'], a['ref']['Artifact ID'], a['payload']) == a
    assert sha(HERE / 'P2G_00_INITIAL_001.json') == initial_hash
    write('P2G_10_POLICY_001.json', {'artifacts': arts})
    write('P2G_10_RESULT_001.json', {'stage': 'P2G-10', 'status': 'complete',
        'Input Working Revision Ref': before['ref'], 'Result Working Revision Ref': result['ref'],
        'Produced Candidate Disposition Ref Set': dispositions,
        'Result Current Candidate Policy Element Disposition Effective View': dispositions,
        'is_p2g10_complete': True,
        'checks': {'all19_candidates_dispositioned': True, 'all16_elements_decided': True,
            'observable_acceptances_consistent': True, 'authority_per_decision': True,
            'open_questions_empty': True, 'raw_intake_unchanged': True, 'digest_refs_checked': True},
        'Finding Triage': [
            {'classification': 'material_policy', 'disposition': '16 adopted/application leaf concerns cover C02–C04 and adopted R14 minimal handling;unagreed official content/frequency values excluded.'},
            {'classification': 'implementation_finding', 'disposition': 'Read-only orientation finds missing relic owners,legacy dust distinct from current target equipment dust,catalog exact-plan authority,copy transition and normal staged grant/save reuse. P2G20 owns formal currentness/design.'},
            {'classification': 'out_of_scope', 'disposition': '3 candidates retain named future/exclusion owners;whole store/economy/closure/causal machinery is not silently included.'}],
        'Open Policy Question Log': [], 'Evidence Gap Log': [], 'production_mutation': False,
        'new_user_request_required': False, 'next': 'P2G-20'})
    view = (HERE / 'WORKING_POLICY.ko.md').read_text().replace('`WR-CRNV-001`', '`WR-CRNV-002`')
    view = view.replace('P2G-00 완료;초기 정책 요소·결정·질문은 비어 있으며 다음 P2G-10이다. 초기화는 정책 검토·구현 완료가 아니다.\nP2G-10 Completion Predicate: N/A.',
                        'P2G-10 완료;16정책·19candidate(승격16/범위외3),열린질문0. 다음 P2G-20 정식 코드 검토다.\nP2G-10 Completion Predicate: true. [정책·결정](P2G_10_POLICY_001.json)과 [결과](P2G_10_RESULT_001.json)는 제품 구현 완료가 아니다.')
    for n, (name, span, statement, acceptance, reason) in enumerate(POLICIES, 1):
        view += f'\n## PE-CRNV-{n:02} — {name}\n\n{statement}\n\n근거: {span}. 채택/위임 적용 이유: {reason}\n\n'
        view += '\n'.join('- ' + value for value in acceptance) + '\n'
    (HERE / 'WORKING_POLICY.ko.md').write_text(view)
    print('U08 P2G10 complete:16 decided elements/19 dispositions;no product mutation')


if __name__ == '__main__':
    main()
