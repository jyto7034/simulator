from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'roadmap_v2.2-review/GOALS.json';m=json.loads(P.read_text());u={x['id']:x for x in m['units']}
for e in m['new_relationships']:
 fs=[f for f in m['findings'] if e['to'] in f['owners']]
 e['finding_ids']=[f['id'] for f in fs]
 if e['type']=='검증선행':
  e['broken_if_ignored']='후행 완료 주장에 필요한 '+e['producer_output']+' 증거가 없어진다. 코드가 존재하는 것만으로 후행 인수를 통과할 수 없다.'
 elif e['type']=='진행상권장':e['broken_if_ignored']='기능이 반드시 깨진다는 뜻은 아니다. '+e['reason']
 else:e['broken_if_ignored']=('조건이 성립하는 경우에만: ' if e['type']=='조건부' else '')+'후행 소비 지점 ['+e['consumer']+']에서 필요한 산출물 ['+e['producer_output']+']을 공급하지 못해 다음 완료 상태를 보장할 수 없다: '+u[e['to']]['valid_after']
 if e['id']=='REL-074': e['broken_if_ignored']='공유 보호막을 사용하는 보스가 원천 pool과 membership을 제공받지 못하면 잔량·흡수·종료의 authority가 없어 그 기믹이 성립하지 않는다. 공유 보호막을 사용하지 않는 보스의 요청/방향/상태 전환은 이 관계로 차단하지 않는다.'
 if e['id']=='REL-075': e['broken_if_ignored']='이미 구현된 침식에 새 chain subscriber가 억제 계약을 적용하지 않으면 침식 kill에서 금지된 유물/장비/파편 연쇄가 새로 발생한다. R09가 뒤에 도입되면 R09가 이 결합 검증을 소유한다.'
 if e.get('conditional_revalidation'): e['broken_if_ignored']='독립 제작 시험 뒤 의미·지원 범위·API를 바꾼 최적화가 들어가면 이전 동결 revision의 PASS는 변경된 엔진을 증명하지 못한다. 해당 변경 영향이 없으면 이 재검증 관계는 적용하지 않는다.'
 e['already_satisfied_or_reusable']=u[e['from']]['reuse']
 e['remaining_policy_gates']=u[e['to']]['policy_gates']
 e['evidence_classification']={'observation':'finding_ids의 현재 source 본문과 source_ledger 범위','judgment':'필요한 producer→consumer의 목표 전환/검증 관계','execution':'이번 게임 실행 없음','assumption':'condition 및 remaining_policy_gates의 해당 family만 조건부'}
m['validation_contract']={'old_edge_count':49,'unit_count':25,'ca_count':24,'milestone_count':9,'contract_count':10,'gate_count':10,'test_family_count':30,'representative_case_count':7,'game_tests_executed':False,'independent_audit_performed':False,'repo_changes_applied':False,'next_0009_executed':False,'source_hash_limit':'전체 dirty/전체 source 해시를 확보하지 않았다.'}
P.write_text(json.dumps(m,ensure_ascii=False,indent=2)+'\n')
# Source content IDs available through read-only git-diff headers.
q=ROOT/'evidence/source_ledger.json';d=json.loads(q.read_text())
for id,old,new in [('S018','57ee97b','6e1acaf'),('S013','c18caa1','441faf5')]:
 s=next(x for x in d['sources'] if x['id']==id)
 s.update(git_head_blob_abbrev=old,git_worktree_blob_abbrev=new,git_identifier_evidence='마지막 git_diff(mode=head) index header. 짧은 Git object ID이며 전체 diff 본문 검토/전체 dirty snapshot/SHA256이 아니다.')
q.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
end={'phase':'closing_currentness_observation','workspace_id':'56af2c31db48','workspace_name':'simulator','branch':'cleanup/simulator-dirty-worktree-20260803','head_abbrev':'f318414','dirty':True,'status_path':'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md','status_range':[1,137],'status_version':'v1.9','record':'0010','next_card':'NEXT-0009','goal':'GOAL-CAMEC-01','goal_status':'Authored / Not Activated / Implementation Authorized: No','observation':'마지막 workspace_info와 ROADMAP_STATUS 전체 본문에서 시작 관찰과 같은 선택/진행 상태를 재확인. 기록/권한 변경 없음.','limits':'최종 git_status 전체를 새로 대조하거나 모든 dirty source를 원자적으로 해시/동결하지 않았다. Git blob ID는 해당 파일만 식별한다.','repo_write_available':False,'command_execution_available':False,'game_tests_executed':False,'repository_changes_made_by_this_review':False}
(ROOT/'evidence/workspace_closing_observation.json').write_text(json.dumps(end,ensure_ascii=False,indent=2)+'\n')
print('Enriched relation failure/reuse/evidence fields; recorded final observed selection and 3 total Git content IDs')
