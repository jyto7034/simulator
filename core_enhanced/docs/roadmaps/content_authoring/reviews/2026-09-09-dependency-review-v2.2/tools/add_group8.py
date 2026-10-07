from record_sources import add, ROOT
import json
add('core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md',[(1,135)],['Document Revision: `2026-09-09-r2`'],'pipeline')
add('core_enhanced/docs/goal_execution_pipeline(1).md',[(1,220)],['Document Revision: `2026-09-09-gex-r1`'],'pipeline')
add('core_enhanced/docs/project_map.ko.md',[(1,150)],['## 3. 상태 소유와 수명'],'map')
add('core_enhanced/src/game/world/snapshot.rs',[(1,205),(577,751)],['fn get_skill_catalog_snapshot_dto','pub fn get_inventory_snapshot_dto'])
add('core_enhanced/tests/skill_runtime_contract.rs',[(2200,2470)],['fn hit_gated_self_heal_and_buff_stack_repeat_attack_work_together() {'])
add('core_enhanced/docs/goals/content_authoring_minimal_effect_contract/P2G_50_AUTHORING_REVIEW_001.ko.md',[(1,60)],['Authored / Not Activated / Implementation Authorized: No'],'inherited-review')
add('core_enhanced/docs/goals/content_authoring_minimal_effect_contract/P2G_60_RESULT_001.json',[(1,210)],['"owner_stage": "P2G-60"','"outcome": "completed"'],'inherited-result')
add('core_enhanced/docs/data_loading_contract.md',[(1,180)],['### Durable runtime documents are not live RON'],'contract')
add('core_enhanced/docs/core_runtime_contract.md',[(1104,1162)],['## Unity-Facing Projection Boundaries'],'contract')
add('core_enhanced/docs/skills/abnormality_design_notes.ko.md',[(1,77)],['문서 성숙도: 미래 콘텐츠 concept.'],'content')
add('core_enhanced/docs/game_rulebook.md',[(1120,1155)],['`breakdown_collapse_node_limit`'],'contract')
add('core_enhanced/docs/skill_target_contract.md',[(418,448)],['`ModifyDamage`는 같은 step의 `Damage`에만 적용되는 step-local modifier다.'],'contract')
p=ROOT/'evidence/source_ledger.json'
d=json.loads(p.read_text())
r=next(x for x in d['sources'] if x['id']=='S020')
r['git_worktree_blob_abbrev']='97b9603'
r['git_head_blob_abbrev']='ce5988d'
r['git_identifier_evidence']='git_diff(mode=head,path=core_enhanced/src/game/battle/core/skill_runtime/projectile.rs) index header; abbreviated Git object ID, not SHA256 or a complete dirty snapshot'
p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
with (ROOT/'evidence/RESEARCH_LOG.ko.md').open('a') as f:
 f.write('''\n## 조사 묶음 8 — canonical 경계·실제 assertion·현재 내용 식별자\n\nP2G r2와 GEX gex-r1의 Reader/owner 규칙을 읽었고 inspect/document-edit 범위만 적용했다. formal P2G-50 작성 검토 전체와 P2G-60 결과의 basis·선택 기록을 추가 인수했다. 여기에 적힌 validator PASS·source hash는 과거 기록이며 이번 재실행/재해시가 아니다.\n\nsnapshot의 skill catalog는 전역 정의 요약, fragment inventory는 definition별 count/progress다. 전역 catalog를 지우거나 개인 plan으로 혼합하는 대신 정의 설명을 유지하고 개인 loadout/preview에 effective context를 별도로 연결해야 한다.\n\n실제 skill_runtime_contract의 hit_gated_self_heal_and_buff_stack_repeat_attack_work_together 본문은 4step 실행, 자기 회복 1회, buff 3stack에 따른 barrage 3회를 assert한다. 이 unit/integration seam은 새 전체 CLI가 없어도 kernel 회귀 시험을 작성할 수 있다는 근거이며 이번 test PASS는 아니다.\n\n룰북 1120–1155를 통해 Breakdown countdown·회복시 폐기·node 완료당 1·retreat 미증가·checkpoint 보존·일반 사망 authority·snapshot 투영이 기존 승인 정책임을 확인했다. 실제 상태 전환 도입은 R09 소유다. recovery와 만료가 같은 node에 만날 때 세부 order의 판정은 명시적 gate로 남긴다.\n\n현재 data canonical의 durable payload V4 서술과 실제 V5 writer/V4·V5 reader의 인접 불일치는 기존 formal 검토도 기록했다. 최소 Goal의 저장 비변경 범위를 확대하지 않는다. targeting canonical은 ModifyDamage step-local과 cast/footprint 범위 권한을 확인했다.\n\nprojectile.rs의 Git diff header에서 HEAD blob ce5988d, 현재 worktree blob 97b9603을 확보했다. 전체 body diff를 읽었다는 뜻은 아니며, 짧은 Git object ID를 SHA256 또는 dirty 저장소 전체 식별자로 부르지 않는다.\n''')
print([(x['id'],x['path']) for x in d['sources'] if int(x['id'][1:])>=88])
