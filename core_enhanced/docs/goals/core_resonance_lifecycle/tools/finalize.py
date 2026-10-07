"""Record GEX-10 self-verification without assigning an independent verdict."""
import datetime,json,os,subprocess,sys
from pathlib import Path
import run
H=run.HERE;S=H/'execution/single_goal';E=H/'evidence'
selection=json.loads((S/'EVIDENCE_SELECTION.json').read_text())
c=json.loads((S/'COVERAGE.json').read_text());i=json.loads((S/'PREDECESSOR_IMPACT.json').read_text())
assert c['source_sha256']==run.identity()['sha256']
r05=json.loads((S/selection['r05_summary']).read_text())
f=json.loads((E/(selection['fault_prefix']+'SUMMARY.json')).read_text());assert f['passed']
public={key:json.loads((E/selection[key+'_summary']).read_text()) for key in ('continuation','rewind','schema','independent_area')}
assert all(row['passed'] for row in public.values())
result={
 'stage':'GEX-10','goal':'GOAL-CRSL-01','goal_set':'GS-CRSL-001','phase':'PHASE-CRSL-01',
 'status':'implementation and self-verification complete; independent GEX-20 pending',
 'recorded_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'source_sha256':c['source_sha256'],'frozen_policy':'../../P2G_30_FREEZE_001.json',
 'phase_plan':'../../P2G_40_PHASE_PLAN_001.json','handoff':'../../P2G_60_HANDOFF_001.json',
 'coverage':'COVERAGE.json','predecessor_impact':'PREDECESSOR_IMPACT.json','state_inventory':'STATE_INVENTORY.md',
 'canonical_contract':os.path.relpath(H.parents[1]/'core_runtime_contract.md',S),
 'canonical_rulebook':os.path.relpath(H.parents[1]/'game_rulebook_next.md',S),
 'canonical_gameplay_detail':os.path.relpath(H.parents[1]/'game_rules_next/combat.md',S),
 'document_authority_intake':'DOCUMENT_AUTHORITY_INTAKE.json',
 'suite_counts':c['suite_counts'],'frozen_acceptance_count':len(c['acceptances']),
 'validation_seams':len(c['validations']),'dimensions':len(c['required_dimensions']),
 'inherited_conditions':len(i['conditions']),'r06_acceptances':len(i['r06_acceptances']),
 'raw_r05_exit':r05['raw_exit_code'],'r05_classification':r05['classification'],
 'r05_concurrent_peak_excess_bytes':r05['concurrent_peak_excess_bytes'],'r05_other_hard_guard_failures':r05['other_hard_guard_failures'],
 'public_fault_modes':f['mode_result_exact_equality'],
 'public_continuation':os.path.relpath(E/selection['continuation_summary'],S),'public_rewind':os.path.relpath(E/selection['rewind_summary'],S),
 'next':'Freeze target/u05_001 exact bytes and reversible restoration; fresh implementation-nonparticipant GPT-6 Astra / High GEX-20. U06 remains ineligible until Complete / High.',
 'public_schema':os.path.relpath(E/selection['schema_summary'],S),
 'public_independent_area':os.path.relpath(E/selection['independent_area_summary'],S),
 'compatibility_adaptation':selection['compatibility_adaptation'],
 'self_awarded_independent_verdict':False,
}
assert (result['frozen_acceptance_count'],result['validation_seams'],result['dimensions'],result['inherited_conditions'],result['r06_acceptances'])==(14,14,10,127,31)
assert (S/selection['compatibility_adaptation']).is_file()
selection['status']='Successful current-source receipt bindings verified by COVERAGE; independent audit pending'
selection['source_sha256']=c['source_sha256']
(S/'EVIDENCE_SELECTION.json').write_text(json.dumps(selection,ensure_ascii=False,indent=2)+'\n')
with (S/'GEX_10_RESULT.json').open('x') as stream:json.dump(result,stream,ensure_ascii=False,indent=2);stream.write('\n')
with (S/'PLAN.md').open('a') as stream:stream.write('\nGEX-10 자체 검증 종료: GEX_10_RESULT.json, COVERAGE.json, PREDECESSOR_IMPACT.json에 정확한 source와 실행 증거를 결속했다. 독립 감사 판정은 아직 없다. target/u05_001의 실제 before/after와 정역 복원을 고정해 별도 GPT-6 Astra / High GEX-20에 인계한다. U06는 독립 Complete/High 이후에만 실행한다.\n')
with (S/'EXPERIMENTS.md').open('a') as stream:stream.write('\n최종 자체 검증: GEX_10_RESULT.json. 기본/fixture/관측 Core·host와 전체 검사 및16 public 소비자, 실제 다섯 fault 관측 모드·stock WebSocket 영속/재접속·선택권 동시성·SavePoint 재전투를 현재 source에서 검증했다. R05 raw exit '+str(r05['raw_exit_code'])+' / '+r05['classification']+'는 승인된 해석과 원자료를 분리 보존한다. 새 독립 완료 판정은 포함하지 않는다.\n')
subprocess.run([sys.executable,str(H/'tools/refresh_status.py'),'self_verified'],check=True)
print(json.dumps(result,ensure_ascii=False))
