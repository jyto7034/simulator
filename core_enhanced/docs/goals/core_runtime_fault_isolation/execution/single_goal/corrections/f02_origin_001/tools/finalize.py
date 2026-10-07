import sys
sys.path.insert(0, '/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_fault_isolation/tools')
import datetime,json,os
from pathlib import Path
import run
H=run.HERE;S=Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_fault_isolation/execution/single_goal/corrections/f02_origin_001');E=H/'evidence'
c=json.loads((S/'COVERAGE.json').read_text());i=json.loads((S/'PREDECESSOR_IMPACT.json').read_text())
assert c['source_sha256']==run.identity()['sha256']
r05=json.loads((S/'compatibility/current_001/evidence/R05_SUMMARY_CURRENT_002.json').read_text())
f=json.loads((E/'final_fault_f02_001_SUMMARY.json').read_text());assert f['passed']
result={'stage':'GEX-10','goal':'GOAL-CRFI-01','goal_set':'GS-CRFI-001','phase':'PHASE-CRFI-01','status':'implementation and self-verification complete; independent GEX-20 pending','recorded_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_sha256':c['source_sha256'],'frozen_policy':'../../P2G_30_FREEZE_003.json','phase_plan':'../../P2G_40_PHASE_PLAN_002.json','handoff':'../../P2G_60_HANDOFF_001.json','coverage':'COVERAGE.json','predecessor_impact':'PREDECESSOR_IMPACT.json','canonical_contract':'../../../../core_runtime_contract.md','host_contract':'../../../../../../game_server/config/observability/README.md','suite_counts':c['suite_counts'],'frozen_acceptance_count':len(c['acceptances']),'validation_seams':len(c['validations']),'dimensions':len(c['required_dimensions']),'inherited_conditions':len(i['conditions']),'r06_acceptances':len(i['r06_acceptances']),'raw_r05_exit':r05['raw_exit_code'],'r05_classification':r05['classification'],'public_fault_modes':f['mode_result_exact_equality'],'next':'Stable target/u03_003 freeze and fresh independent GEX-20 by implementation-nonparticipant GPT-6 Astra / High. No U04 implementation until Complete / High.','self_awarded_independent_verdict':False}
result.update({
    'frozen_policy': os.path.relpath(H/'P2G_30_FREEZE_003.json', S),
    'phase_plan': os.path.relpath(H/'P2G_40_PHASE_PLAN_002.json', S),
    'handoff': os.path.relpath(H/'P2G_60_HANDOFF_001.json', S),
    'canonical_contract': os.path.relpath(H.parents[1]/'core_runtime_contract.md', S),
    'host_contract': os.path.relpath(run.ROOT/'game_server/config/observability/README.md', S),
    'predecessor_finding': os.path.relpath(H/'post_audit/u03_20261004_002/FINDINGS.json', S),
    'producer_review': 'PRODUCER_REVIEW.md',
    'correction_entry': 'ENTRY.json',
})
with (S/'GEX_10_RESULT.json').open('x') as stream:json.dump(result,stream,ensure_ascii=False,indent=2)
with (S/'PLAN.md').open('a') as stream:stream.write('\nGEX-10 현재 종료: GEX_10_RESULT.json과 COVERAGE.json의 source-bound 자체 검증 완료.\n독립 판정은 아직 없다. target/u03_003의 exact bytes/archive/restore를 고정한 뒤\n별도 GEX-20으로 인계한다. U04는 독립 감사 긍정 종료 후에만 eligible하다.\n')
with (S/'EXPERIMENTS.md').open('a') as stream:stream.write('\n최종 결과: GEX_10_RESULT.json. 모든 final_* matrix 및 compat_* receipt의 raw log/source identity를 검증했다.\nfinal_fault_f02_001_*은 최종 release source에서 native failure와 다섯 관측 모드 exact parity, 실제 stock websocket\n예정 tick 오류→reconnect/replacement/save 보호→Quit/new owner/valid Continue를 검증했다.\nR05 raw exit '+str(r05['raw_exit_code'])+' / '+r05['classification']+'는 원본과 별도 분류를 유지한다.\n새 독립 Complete/High 판정은 이 구현 결과에 포함하지 않는다.\n')
print(json.dumps(result,ensure_ascii=False))
