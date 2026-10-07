"""Author the R08-R12 policy/plan once; validation is performed separately."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = Path('/mnt/f/work/simulator')
TEMPLATE = HERE.parent / 'content_authoring_fragment_copies'

def write(name, value):
    path = HERE / name
    assert not path.exists(), path
    path.write_text(value if isinstance(value, str) else json.dumps(value, ensure_ascii=False, indent=2)+'\n')

def ref(kind, id):
    return {'Artifact Type': kind, 'Artifact ID': id, 'Content Digest': None}

def rid(kind, prefix):
    return ref(kind, prefix+'-CARX-001')

rsb = rid('Reviewed Source Basis','RSB')
cra = rid('Code Review Artifact','CRA')
wr = rid('Working Revision','WR')
rb = rid('Review Batch','RB')
freeze = rid('Final Policy Freeze','FPF')
plan = rid('Phase Plan','PP')
receipt = rid('Phase Plan Validation Receipt','PVR')
coverage = json.loads((HERE/'preparation/20260930_001/COVERAGE.json').read_text())
audit = json.loads((HERE/'reviews/r06_r07_20260930_001/AUDIT_TARGET.json').read_text())
for row in audit['files']:
    assert hashlib.sha256((ROOT/row['path']).read_bytes()).hexdigest() == row['sha256'], row['path']

# Each work package owns its schema/compile/runtime/consumer atomic transition.
# These are development dependencies, not a single runtime transaction.
specs = [
 ('08','명시 보스 action','boss_runtime_data.rs; core/boss_runtime.rs; core/sim.rs; core/runtime_facts.rs',
  'StartAction의 skill/target/facing/milestone/timeout/cancel 정책을 검증한다. 권한별 player/boss adapter와 공통 action 실행을 분리한다. state activation이 retained handle을 소유하며 Released/EffectsSettled/거부/소실/timeout을 명시 route에 연결한다. 미방출만 취소하고 방출 payload는 보존한다.',
  '현재 task에 skill 요청이 없고 공개 request_skill_action은 Player/Manual/공명만충을 요구한다. 기존 transition candidate와 ActionSnapshot의 서로 다른 milestone을 재사용한다.',
  'P06: timeout은 명시 CancelUnreleased 또는 Detach; state exit는 CancelUnreleased. 유한 양수 timeout 필수. action ordinal 재생성이나 tick별 전체 fact 스캔을 거부한다.'),
 ('09','침식·강제정지·Breakdown','core/sim.rs; core/death_runtime.rs; world/application.rs; world/combat.rs; world/node_flow.rs; employee.rs; world/active_run.rs',
  '일반 시전 자격을 지나는 첫 unpaused start gate에서 침식을 한 번 판정한다. exact R07 source/trauma/공명과 최종 보스 제외를 사용한다. 직원 Withdrawn/trauma100·현재 적 특수 death·공명·연출을 원자 commit한다. 필수 death/wave는 유지하고 선택 반응은 억제한다. host elapsed 정지는 사용자 Pause와 독립이다. Breakdown은 identity별 완료 node 잔여 수를 저장한다.',
  'Manual 접수와 cast end가 분리되어 있고 공명은 release 후 reset한다. 특수 dispatch 정책/host freeze/Breakdown count는 없다. incapacitation은 신체부상을 추가하므로 침식 대가에 재사용할 수 없다.',
  'P01~05: 접수 아닌 첫 unpaused 실행을 start로 채택; 준비 중 취소 비용은 기존 그대로. trauma 임계80/연출1000ms는 위임 잠정 live 값이며 RON 명시 양수 검증, 사용자 직접 확정값 아님. 회복 먼저 적용하고 기존 trauma100만 완료 node 차감, 새 진입은2부터. 만료 elapsed의 초과분은 버리고 다음 호출부터 battle 진행. P14: durable 필드 추가는 새 schema와 명시 변환으로만 인수하며 원본을 덮어쓰지 않는다.'),
 ('10','유한 구조화 Flow','ability.rs; data/skill_data.rs; skill validation/compiler; core/skill_runtime; core/sim.rs; value_expr; authoring descriptors',
  'Sequence/Branch/Parallel(All)/Wait/유한 Repeat를 공통 compiled flow로 실행한다. 기존 steps는 같은 계획으로 lowering한다. authored 주소와 occurrence/typed result를 분리하고 선행 result ref만 허용한다. self/unit/tile/facing/anchor/retarget을 Instant/Projectile/Area로 전달한다. R08 waiter와 R09 start를 유지한다.',
  '현재 flat steps와 repeat/when, deferred delivery settlement가 존재한다. hit count가 아닌 actual HP-loss 조건과 R06 Snapshot/Live provenance를 보존해야 한다.',
  'P07: 같은 시각 authored 순서, Parallel All join, 실패시 미방출 sibling 중단/방출분 보존. no-target도 Repeat 기회 소비. 반복 수는 interval과 독립. 시간+1 충돌회피/무한 반복/forward typed ref 금지.'),
 ('11','source 효과·channel·overlay','battle stats/buffs; core/types.rs ActionLocks; basic_attack.rs; movement; R05 retained; R07 fragment resolver',
  'base와 source-owned modifier/lock을 분리하여 종료 시 자기 source만 제거한다. aura membership은 실제 직원 배치/거리/자격을 반영한다. 태아 channel과 Hello/Goodbye 공격 overlay를 정상 fragment 명령에 연결한다. 이미 release된 공격 profile은 보존한다.',
  '현재 lock은 until 최대값이고 일부 buff만 존재한다. 가역 source ledger 없이 직접 stat을 변경하면 중첩 제거 drift가 생긴다. R07 selected form provenance를 유지해야 한다.',
  'P08: 태아 기존 저지 유지, self/Guest/summon/object 제외, 같은 family 비중첩이며 priority 높은 값→동률 creation ordinal 먼저 선택. 기본 공격 피해20%/속도15 points 계약 보존. P09: Hello [start,end), 재입력은 공명 재청구 없이 남은 시간 commit, 만료는 복귀만. 다음 공격 overlay는 release에 소비. P13: channel/overlay/pool은 attempt 밖 저장하지 않는다.'),
 ('12','보호막·생존·제어·HP 비용','core/damage_runtime.rs; death_runtime.rs; buffs; movement; runtime_facts; ability/skill data',
  '개인/공유 pool의 실제 잔량을 단일 source owner에 보관하고 membership 변화로 복제하지 않는다. ordinary damage 감쇄 뒤 shield/HP1/death-prevent를 중재해 HP-loss와 absorbed를 구분한다. parry 첫 strike의 피해/부착 효과만 차단하고 counter 한 번. HP cost는 별도 fact, slow/root/taunt/pull은 명시 capability별로 실행한다.',
  '현재 final_damage 이후 lethal death를 미리 준비한다. 중재를 그 전으로 옮겨야 한다. 기존 Stun/Freeze/Silence와 movement/fieldwork 정리는 보존한다.',
  'P10: 개인→공유, 분류 내 authored priority 내림차순→creation ordinal 오름차순. P11: ordinary만 HP1, cost는 기본 HP≥1/명시 lethal opt-in이며 shield/parry 우회, 특수 kill은 모두 우회. P12: 첫 직접 enemy strike만 parry, 반사/self/cost 제외, 공격자 소실은 no-target으로 counter 종료. Black Swan standing passive는 해당 fragment만의 명시 예외이며 전체 passive 활성화 아님.'),
 ('INT','통합 검증·동결','public RON/CLI/World/save; authoring clone/admission; docs/contracts; archive restoration',
  'A08~A12와 X01~10을 공개 입력/정상 명령/실전/결과/저장까지 실행한다. R06/R07 변경 영향과 PC36/R05를 최종 source에서 검증한다. 실제 before/after membership와 정역 복원/currentness를 확인한 최종 target을 독립 감사 대기로 인계한다.',
  'R06/R07 source367은 선행 감사 Complete/High의 exact target과 일치한다. 현재 새 기능 수용55개는 미실행이다. 옛 시험을 새 구현의 통과로 사용하지 않는다.',
  'P13~14: durable identity/counter와 ephemeral effect 소유 분리. 기존 V7→V8 명시 변환을 보존하고 새 schema 변환도 명시적. R05 peak cap9030188 단독 초과 WARNING, 나머지 hard guard 유지. 20분/stress/metrics/차기 원정/GUI/R13 제외.')
]

artifacts=[]
def add(r,p): artifacts.append({'ref':r,'payload':p})
add(rsb,{'scope':'R08~R12 current code review, prerequisite audited target','source_digest':audit['source_digest'],'files':audit['files']})
add(cra,{'source':rsb,'Current behavior':[s[4] for s in specs], 'Proposed Implementation':[s[3] for s in specs],
         'Known limitations':'Code-based policy review, not implementation or completion audit. Per-WP exact consumer inventory remains a GEX preparation obligation before its mutations.',
         'Rollback':'actual dirty/untracked before archive; no git reset; final isolated forward/reverse restoration',
         'evidence':['preparation/20260930_001/CODE_EXCERPTS.log','reviews/r06_r07_20260930_001/AUDIT_CLOSURE.json']})
pers=[];drs=[];bindings=[];wps=[];integrations=[]
for i,s in enumerate(specs,1):
    n,title,boundary,contract,current,decision=s
    stable=f'CARX-E{i:02}'
    ids=[c['id'] for c in coverage['conditions'] if c['id'].startswith('X' if n=='INT' else 'A'+n)]
    pr=[ref('Policy Element Revision',f'PER-{stable}-{state}-001') for state in ['OPEN','DECIDED','REVIEWED']]
    dr=ref('Decision Revision',f'DR-{stable}-001');binding=ref('Code Review Binding',f'CRB-{stable}-001')
    add(pr[0],{'Stable Element ID':stable,'Intrinsic status':'open','Question':title,'Observable Policy Acceptance':ids})
    add(dr,{'authority':'2026-09-30 explicit user scope and delegated detail choices; existing contracts prevail', 'decision':decision,
            'rationale':contract,'alternatives_rejected':'Silent success, new budget auto-cancel, player impersonation, shared mutable identity, implicit save defaults violate existing owner/rollback/fact semantics.',
            'impact':boundary,'reversal':'Change requires successor policy review and affected acceptance rerun; archive source restoration is not save downgrade.'})
    for j,status in [(1,'decided'),(2,'code_reviewed')]:
        p={'Stable Element ID':stable,'Intrinsic status':status,'Predecessor':pr[j-1],'Decision Revision Ref':dr,'Observable Policy Acceptance':ids}
        if j==2:p.update(contract=contract,review=cra)
        add(pr[j],p)
    add(binding,{'element':pr[2],'review':cra,'source':rsb,'completeness':current+'; '+boundary+'; acceptance in COVERAGE required, no implementation PASS'})
    pers.append(pr[2]);drs.append(dr);bindings.append(binding)
    wp=f'WP-CARX-{n}'
    wps.append({'id':wp,'selectors':{'implementation':[stable]},'boundary':boundary,'expected_change':contract,'prerequisite':'R06/R07 audited source; earlier WP shared-port preservation; AT'+n+' parser/compiler/runtime/consumer tested together','validation':','.join(ids),'unknown':'exact API/resource-cost measurements closed in GEX before completion'})
    for id in ids:
        integrations.append({'id':'IVA-CARX-'+id,'source':'mandatory design acceptance '+id,'disposition':'assigned_execution_node','nodes':[wp],'phases':['PHASE-CARX-01'],'locator':'execution/single_goal/COVERAGE.json '+id,'evidence':'public input + actual selector/count + raw receipt + source/lock/toolchain identity','prerequisite':wp+' integrated with all later WPs on final source'})
add(rb,{'bindings':bindings,'source':rsb,'disposition':'complete policy-to-code binding; execution pending'})
add(wr,{'Current Policy Element Revision Map':{f'CARX-E{i:02}':r for i,r in enumerate(pers,1)},'User-Approved Scope':'R08~R12 continuous implementation after now-authorized R06/R07 audits; final self-validation/freeze; independent R08~R12 audit excluded','remaining_policy_questions':[]})
write('P2G_POLICY_AND_REVIEW.json',{'artifacts':artifacts})
write('P2G_30_FREEZE.json',{'artifacts':[{'ref':freeze,'payload':{'Working Revision Ref':wr,'Review Batch Ref':rb,'Ordered code_reviewed Policy Element Revision Ref Set':pers,'Decision Revision Ref Set':drs,'Observable Policy Acceptance Set':[{'Policy Element Revision Ref':p,'Observable Policy Acceptance':[c['id'] for c in coverage['conditions'] if c['id'].startswith('X' if s[0]=='INT' else 'A'+s[0])]} for p,s in zip(pers,specs)],'Current Code Review Binding Ref Set':bindings,'Reviewed Source Basis Ref':rsb,'Binding Completeness Evidence':cra,'Lifecycle Disposition':'current','Predecessor Freeze':None,'Applicable Change/Obligation Basis':[],'Known Unknowns':'Implementation and actual costs not yet validated; no prerequisite audit gap remains','Compatibility/Migration Disposition':'flat steps lower into same flow; explicit durable schema conversion only; old source preserved'}}]})
nodes=[w['id'] for w in wps]
deps=[{'from':nodes[i],'to':nodes[i+1],'type':'validation_input','evidence':'Later WP must preserve earlier action/start/delivery/source contracts with focused regressions','broken':'Public R08/R09 paths could diverge under Flow, channel or damage changes'} for i in range(4)]
deps += [{'from':n,'to':nodes[-1],'type':'validation_input','evidence':'Final cross acceptance needs actual integrated public runtime/save','broken':'Older per-WP source cannot prove final currentness'} for n in nodes[:-1]]
write('P2G_40_PHASE_PLAN.json',{'artifacts':[{'ref':plan,'payload':{'Bound Final Policy Freeze Ref':freeze,'Exact Prerequisite Composite Basis':{'Working Revision Ref':wr,'Final Policy Freeze Ref':freeze},'Applicable P2G-40 Obligation Key Set':[],'P2G-40 Resolution Record Ref Set':[],'Applicable Obligation Supersession Record Ref Set':[],'Goal Structure Fact':'single_phase','Parallelization':'none; one implementing agent','work_packages':wps,'dependencies':deps,'atomic_groups':[],'standalone':nodes,'cyclic_sccs':[],'phases':[{'id':'PHASE-CARX-01','primary':[f'CARX-E{i:02}' for i in range(1,7)],'nodes':nodes,'exit':'All55 acceptance plus final regressions and reversible freeze; independent audit pending'}],'audits':[{'phase':'PHASE-CARX-01','state':'public boss/fragment/flow/channel/shield paths share action/fact owners','checks':'source-local cleanup, admission rollback, actual HP loss, durable identity and schema','evidence':'final COVERAGE, receipts and target manifest required','limitation':'plan review only; no implementation or independent audit verdict'}],'integration':integrations,'Shared Contract Handoff':'R04 retained ActionHandle and released/settled distinctions; R05 typed faults and pre-allocation admission; R06 amount facts; R07 exact copy provenance remain authoritative. WP internal AT is a code cutover, not one monolithic runtime transaction.','Known Unknown/Limitations':'Exact consumer-level implementation and measured resource costs pending GEX; no external UI/server implementation.'}}],'validation_receipt_ref':receipt,'validator_method_version':'p2g40-receipt-validator/1.2.0'})
write('WORKING_POLICY.ko.md','# R08~R12 승인 범위와 위임 결정\n\n선행 감사는 reviews/r06_r07_20260930_001/AUDIT_CLOSURE.json에서 종료했다. 이 문서는 R08~R12 구현 계획이며 독립 감사가 아니다.\n\n'+'\n\n'.join('## WP'+s[0]+' — '+s[1]+'\n\n'+s[3]+'\n\n'+s[5] for s in specs)+'\n\n55개 조건의 정확한 oracle은 preparation/20260930_001/COVERAGE.json을 인수한다. 수치80/1000은 위임 잠정값이며 공식 사용자 확정 수치라고 주장하지 않는다.\n')
print('authored policy and phase plan; not yet validated')
