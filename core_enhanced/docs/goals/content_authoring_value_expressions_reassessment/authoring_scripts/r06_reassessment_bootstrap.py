from pathlib import Path
import json,hashlib,subprocess,datetime
root=Path('/mnt/f/work/simulator'); rel=Path('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment'); dest=root/rel
identity='WPD-CAVER-20260913'
assert not dest.exists(), 'Resume existing document; do not recreate'
for p in (root/'core_enhanced/docs/goals').glob('*/WORKING_POLICY*.md'):
 assert identity not in p.read_text(), str(p)
old=root/'core_enhanced/docs/goals/content_authoring_value_expressions'
paths=set(p for p in old.rglob('*') if p.is_file())
for folder in ['core_enhanced/src','core_enhanced/tests','game_resources/data','core_enhanced/docs/architecture','core_enhanced/docs/tools','core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2','game_server/src']:
 paths.update(p for p in (root/folder).rglob('*') if p.is_file())
for name in ['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md','core_enhanced/docs/goal_execution_pipeline(1).md','core_enhanced/docs/roadmaps/content_authoring/AUTORUN_REQUEST.ko.md','core_enhanced/docs/roadmaps/content_authoring/OPERATING_RULES.md','core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md']:
 paths.add(root/name)
paths.update(p for p in (root/'core_enhanced/docs').glob('*.md') if p.is_file())
def git(*args):return subprocess.check_output(['git',*args],cwd=root).decode()
head=git('rev-parse','HEAD').strip(); branch=git('branch','--show-current').strip(); status=git('status','--porcelain=v1','--untracked-files=all')
captured={p:p.read_bytes() for p in sorted(paths)}
assert all(p.read_bytes()==b for p,b in captured.items()), 'Concurrent change in observation'
assert git('rev-parse','HEAD').strip()==head
# All evidence retained as actual bytes; bootstrap capture is not semantic source review.
dest.mkdir(); before=dest/'baseline/intake/originals'
rows=[]
for p,b in captured.items():
 q=before/p.relative_to(root);q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b)
 rows.append({'path':str(p.relative_to(root)),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'original':str(q.relative_to(root))})
request=Path('/home/acr0tic/.codex/attachments/316cab10-6cd5-4e82-85ea-05989dfa8abe/pasted-text-1.txt').read_bytes()
(dest/'REQUEST_ORIGINAL.txt').write_bytes(request)
def write(p,data):
 q=dest/p;q.parent.mkdir(parents=True,exist_ok=True);q.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
manifest={'kind':'planning intake preservation, not implementation baseline','at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'repository':str(root),'branch':branch,'head':head,'files':rows,'capture':'read bytes then reread every file; exact byte equality before publication','limitations':['Scoped source/data/tests/server-read-only and documentation snapshot; not entire repository backup. Unrelated dirty files are recorded in status only.','No runtime/code behavior claim made at P2G-00.']}
write('baseline/intake/MANIFEST.json',manifest)
(dest/'baseline/intake/MANIFEST.sha256').write_text(hashlib.sha256((dest/'baseline/intake/MANIFEST.json').read_bytes()).hexdigest()+'  MANIFEST.json\n')
(dest/'baseline/intake/GIT_STATUS.txt').write_text(status)
(dest/'baseline/intake/TRACKED.patch').write_text(git('diff','--binary','HEAD'))
ref=lambda typ,id:{'Artifact Type':typ,'Artifact ID':id,'Content Digest':None}
intakeref=ref('Policy Intake Snapshot','PIS-CAVER-P2G00-001');wrref=ref('Working Revision','WR-CAVER-P2G00-001')
sourcepaths=['core_enhanced/docs/roadmaps/content_authoring/AUTORUN_REQUEST.ko.md','core_enhanced/docs/roadmaps/content_authoring/OPERATING_RULES.md','core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md','core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md']+[str(p.relative_to(root)) for p in sorted(old.iterdir()) if p.is_file()]
intakerows=[{'Intake Item ID':'IN-001','Intake Kind':'user request','Exact Content or Content Reference':{'path':str(rel/'REQUEST_ORIGINAL.txt'),'sha256':hashlib.sha256(request).hexdigest()},'User-Provided Locator':'/home/acr0tic/.codex/attachments/316cab10-6cd5-4e82-85ea-05989dfa8abe/pasted-text-1.txt','User-Stated Constraint':'R06 reassessment P2G-00 through P2G-60; preserve originals; plan-only; stop before GEX','Capture Limitation':'Exact raw bytes; no semantic assessment at bootstrap'}]
for i,name in enumerate(sourcepaths,2):
 row=next(r for r in rows if r['path']==name)
 intakerows.append({'Intake Item ID':f'IN-{i:03}','Intake Kind':'exact repository reference','Exact Content or Content Reference':row,'User-Provided Locator':name,'User-Stated Constraint':'Compare existing material at its owner stage; latest user request overrides historical execution delegation','Capture Limitation':'Bootstrap access and bytes only; no policy/current-runtime assessment'})
scope=[{'ID':'S01','Disposition':'in','Scope':'기존 R06 승인 기능·수용 기준·제외 범위를 계획 대상으로 보존하고 별도 Working Policy에서 P2G-00→60 재검토·비교·취합·상세 설계·검증·기준선 인계를 완료','Approval Reference':'REQUEST_ORIGINAL.txt §§1–6'}, {'ID':'S02','Disposition':'out','Scope':'GEX 인수·activation·게임 구현·완료 감사·R07 이후 신규 작업','Approval Reference':'REQUEST_ORIGINAL.txt §§1,6'}, {'ID':'S03','Disposition':'out','Scope':'기존 원본 삭제·이동·덮어쓰기, P2G/GEX 본문·검증기·AGENTS·전역 설정 수정, 사용자 정책 임의 변경/축소','Approval Reference':'REQUEST_ORIGINAL.txt §§2,3,6'}]
wr={'Predecessor Working Revision Ref':None,'Predecessor Final Policy Freeze Ref':None,'Approved Metadata and Scope':{'Working Policy Document ID':identity,'Policy Topic':'R06 ValueExpr·집계·Attack/SpellPower와 무기 성장 재검토·최종 취합','Requested Result':'bounded_pipeline P2G-00→P2G-60; R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기','Target Project/Repository Identity':{'root':str(root),'git_head_at_intake':head},'User-Approved Initial Scope':scope},'Policy Intake Snapshot Ref':intakeref,'Current Policy Element Revision Map':{},'Decision Revision Ref Set':[],'Open Policy Question Ref Set':[],'Post-Freeze Change/Closure Basis':None,'Authoritative Change Obligation Declaration Ref Set':[]}
write('P2G_00_INITIAL_001.json',{'artifacts':[{'ref':intakeref,'payload':{'Working Policy Document ID':identity,'Snapshot Kind':'initial','Predecessor Policy Intake Snapshot Ref':None,'Intake Rows':intakerows}},{'ref':wrref,'payload':wr}]})
checks=['new path and identity verified vacant','conforming policy workspace path','topic/result/repository explicit','all scope items exact user approval refs','no proposed scope','raw exact bytes preserved','snapshot identity/required members','immutable initial WR and head selection','exact intake reference match','empty elements/decisions/questions','no later-stage artifacts created','bounded continuation and next owner explicit']
write('P2G_00_RESULT_001.json',{'stage':'P2G-00','result':'passed','checks':[{'check':x,'result':'passed'} for x in checks],'current':wrref,'next_owner':'P2G-10','new_user_request_required':False,'comparison':'Old WPD-CAVE-20260913 and all artifacts retained byte-for-byte at original paths; new distinct user-authorized reassessment lineage. No old decided/code_reviewed payload injected.','preservation_manifest':'baseline/intake/MANIFEST.json'})
(dest/'WORKING_POLICY.ko.md').write_text('# R06 재검토·최종 취합 — Working Policy\n\n- Working Policy Document ID: `'+identity+'`\n- Physical Path: `'+str(rel/'WORKING_POLICY.ko.md')+'`\n- Current Working Revision: `WR-CAVER-P2G00-001`\n- Current Stage: P2G-00 passed; P2G-10 next\n- Current elements / Decisions / questions: 0 / 0 / 0; P2G-10 complete predicate false (not evaluated)\n- Requested Result: bounded_pipeline P2G-00→60; 계획 전용, 완료 후 구현 요청 대기\n\n[요청 원문](REQUEST_ORIGINAL.txt), [initial immutable artifacts](P2G_00_INITIAL_001.json), [P2G-00 검사](P2G_00_RESULT_001.json), [원본 manifest](baseline/intake/MANIFEST.json). 기존 `../content_authoring_value_expressions/`는 원위치 보존 비교 대상이며, 새 stage 검증 후 이 문서에서 하나의 current만 선택한다. 기존 단계 완료를 승계하지 않는다. GEX 인수·activation·구현·감사는 별도 명시적 사용자 요청 없이는 시작하지 않는다.\n')
print(json.dumps({'result':'P2G-00 passed','directory':str(rel),'preserved_files':len(rows),'preserved_bytes':sum(r['bytes'] for r in rows),'head':head,'branch':branch},ensure_ascii=False))
