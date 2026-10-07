"""U08 P2G-40 publication and quality2 complete Goal candidate."""
import json
from initialize import HERE,write

def main():
    pa=json.loads((HERE/'P2G_40_PHASE_PLAN_001.json').read_text())['artifacts'][0];p=pa['payload']
    v=json.loads((HERE/'P2G_40_VALIDATION_001.json').read_text());assert v['result']=='passed' and not v['failures']
    basis=json.loads((HERE/'P2G_30_RESULT_001.json').read_text())['current_basis'];basis['Phase Plan Ref']=pa['ref']
    write('P2G_40_RESULT_001.json',{'stage':'P2G-40','status':'complete','Phase Plan Ref':pa['ref'],'Phase Plan Validation Receipt Ref':'PVR-CRNV-001',
      'P2G-40 Resolution Record Ref Set':[],'Applicable Obligation Supersession Record Ref Set':[],'current_basis':basis,'All Applicable Change Obligations Resolved':True,
      'raw_exit':0,'validation':v,'semantic_design':'TD01–06/P2G_40_DESIGN_001:4WP runtimeatomiccutover+PUBLIC+VERIFY;3finalnodes/1phase','next':'P2G-50'})
    arts=json.loads((HERE/'P2G_20_REVIEW_001.json').read_text())['artifacts']
    es=sorted([a for a in arts if a['ref']['Artifact Type']=='Policy Element Revision'],key=lambda a:a['payload']['Stable Element ID'])
    nodes=', '.join(p['phases'][0]['nodes'])
    s=f'''# U08 유물 사본·수치 계획·침식 변형 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CRNV-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CRNV-003; FPF-CRNV-001; PP-CRNV-001; PVR-CRNV-001; changes/obligations explicit empty
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CRNV-001
- Final Policy Freeze Ref: FPF-CRNV-001
- Complete Goal Member Set: GOAL-CRNV-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

Goal 작성 자체는 구현 권한이 아니다. 기존 연속 사용자 요청은 P2G 인계 뒤 GEX 구현·자체검증·독립감사/교정·후속을 별도로 승인했다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CRNV-01 | GOAL-CRNV-01 | {nodes} | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CRNV-001 현 연속 요청/효율화 지침/ROADMAP U08;유효한exact이전증거 재사용과 필수current통합·독립감사.
- Baseline Evidence Key: source/data/schema/test/config/input/toolchain/lock/dependency·실제command/selector/rawreceipt/ownerGoal/claim을 함께 결속한다.
- Reuse Rule: PREDECESSOR_INTAKE_001의 U07-M target001/audit001와 published상태문서2개delta를 인수한다. exactcurrent 또는 해당claim에 영향없다는명시근거가 있어야 재사용한다.
- Production Input Invalidation / Focused Revalidation Map: schema/copy/slots→V01–05/09/14;numeric/capture/basic/source→V05–08/09/13/15/R06/U05/U07/PC;grant/cost/Event→V04/09–12;save/entry/death/clone→V03/10/12/13/15/U04;layout/metrics→V15/U07M;전체→V16.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 원input/oracle/raw/실패를보존한다. 신규fixture와실행은새label. defaultPC31성공5거절과hardguard를comparator로완화하지 않는다. temporarydevprofile결과로default회귀를대체하지않는다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: affectedproducer/consumer를함께현재검증한다. test파일불변은commonruntime/layout비영향의증거가아니다. source/clone/amount/save확장과host직렬화영향을별도분류한다.
- Phase-Start Rule: 새단계시작만으로전체시험을반복하지않는다. 변경·실패·미해결위험에비례한focused검증과최종current필수set을실행한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CRNV-01이제품mutation전U07-M CompleteHigh product f2144e9a4c5ddeb41245799294a75b41de9d843018cb2166f711b7f07821cda8의선정54receipts를exact인수한다. Core1875/1875/1892,host93/103/122+heap,normalplayback26/lifecycle10/host4x2,PC002576/36/31성공5거절,R05002원실패101/peak9669160cap9030188의단독경고와finalcheck/fmt/clippy/public/release를현재impact에연결한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CRNV-01/WP-CRNV-VERIFY가현재 Coredefault/alltargets/test-fixtures/runtime-observation,check/event_log_checks/releasebins/clippy/fmt,영향hostdefault/observation/measurement/heap와publicCLI/native/fault/save/numeric/PC/R05,U07M관측을닫는다. 기능workers완료뒤성능측정한다.
- Normal Full-Validation Count: 초기exact선행baseline1회재사용,최종current성공set1회. 실패·교정·입력변경은필요재실행사유이며원본보존한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: 실패/skip/0선택/실행뒤입력변경은PASS아님. 원receipt→교정→새label·현재결속. R05 concurrentpeak-only기존경고와20분stress제외만상속한다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
'''
    for r in p['integration']:
        s+=f"| {r['id']} / {r['source']} | {r['disposition']} | GOAL-CRNV-01 / WP-CRNV-VERIFY | {r['locator']}; {r['evidence']} |\n"
    s+='\n## Single Goal Member `GOAL-CRNV-01`\n\n'
    fields={'Goal ID':'GOAL-CRNV-01','Goal Kind':'single','Phase ID':'PHASE-CRNV-01','Assigned Final Execution Node Set':nodes,
      'Covered Policy Element Revision Ref Set':', '.join(e['ref']['Artifact ID'] for e in es),
      'Covered Decision Revision Ref Set':', '.join(e['payload']['Decision Revision Ref']['Artifact ID'] for e in es),
      'Derived Observable Policy Acceptance Coverage':'FPF-CRNV-001 exact16member acceptance전체;CODE_REVIEW ElementCoverage01–16과integrationmapping이실행경계를소유한다.'}
    fields.update(json.loads((HERE/'tools/goal_fields.json').read_text()))
    for key,val in fields.items():s+=f'- {key}: {val}\n'
    s+='\n### Validation Dimension Matrix\n\n| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |\n| --- | --- | --- | --- |\n'
    dims=[
      ('success','V01–14normalgrant/equip/numeric/variant/save','두직원개인실제결과·공개RON·cost/query일치'),
      ('invalid_input','V02/05/09/10/11/14badcopy/selector/variant/authority','위치명확한거절·무부분commit'),
      ('failure_rollback','V04/10/12/15lategrant/plan/cap/savefault','copy/catalyst/dust/slot/RNG/UUID/Event/저장원본보존'),
      ('retry_idempotency','V03/10/11/13duplicatecorrupt/Event/death','once변환·한번Event·무중복회수/소비'),
      ('ordering_concurrency','V02/06/07/08permutation/twoactor/queued','동일집합동일계획·개인격리·SnapshotLive/방출권한보존'),
      ('boundary_empty_max','V02/06/09/15empty/3slot/overflow/cap','empty빠른경로·4th거절·유한수치/실행한도·no게임play증폭cap'),
      ('disconnect_restart_persistence','V12/13freshCore/reconnect/Continue/retreat','소비변형동시시점·variant비정화·구저장원본보호'),
      ('live_data','V09/14externalnewID/recombine/normaltrial','실제등록/명령/효과/패널티·schema만성공금지'),
      ('producer_consumer','V05–08/11/12allsource/plan/Event/save','baseauthority→개인plan→carrier/actual/query와atomicrunroots일치'),
      ('legacy_fallback_absence','V08/14/15/16noobserveronly/rawsourceoverwrite','기존PC/default/source/legacydust보존·no무조건closure/freeflag우회')]
    for d,seam,condition in dims:s+=f'| {d} | required | {seam} | {condition} |\n'
    s+='\n## Policy–Phase–Goal Traceability\n\n| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |\n| --- | --- | --- | --- | --- |\n'
    for e in es:s+=f"| {e['payload']['Stable Element ID']} | {e['ref']['Artifact ID']} | {e['payload']['Decision Revision Ref']['Artifact ID']} | PHASE-CRNV-01 | GOAL-CRNV-01 |\n"
    assert len(es)==16 and len(dims)==10
    write('P2G_50_GOAL_SET.ko.md',s)
    print('U08 P2G40 complete;P2G50 quality2 candidate complete,validation pending')

if __name__=='__main__':main()
