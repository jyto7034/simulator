from pathlib import Path
import json,hashlib,re,subprocess,sys
import run
H=Path(__file__).resolve().parent;E=H/'evidence';R=run.ROOT
write=lambda p,x:p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
current=run.identity();proof=[]
mandatory=['integrated_authoring_003','integrated_value_expr_003','integrated_list_003','integrated_all_003','integrated_feature_003','integrated_check_003','integrated_event_check_003','integrated_release_003','integrated_clippy_003','integrated_fmt_003','pc_integrated_003','cli_aggregate_003']
mandatory += [p.stem for p in E.glob('cli_*_003.json') if p.stem not in mandatory]
for name in mandatory:
 d=json.loads((E/(name+'.json')).read_text());expected=1 if name in ['cli_signed_below_min_003','cli_signed_above_max_003'] else 0
 assert d['exit_code']==expected,(name,d['exit_code']);assert d['source_before']==d['source_after']==current,name
 proof.append({'receipt':name+'.json','exit':expected,'passed':d['test_passed'],'source_delta':[]})
r05=json.loads((E/'R05_SUMMARY_003.json').read_text());assert not r05['other_hard_guard_failures']
d=json.loads((E/'r05_integrated_003.json').read_text());assert d['source_before']==d['source_after']==current
pc=json.loads((E/'PC_SUMMARY_003.json').read_text());assert pc['same_gameplay_oracle_and_typed_refusal']
isolate=json.loads((E/'pc_isolate_integrated_003.json').read_text());assert isolate['current_source']==isolate['copied_source']==current
write(E/'VALIDATION_CURRENTNESS.json',{'source_digest':current['digest'],'receipts':proof,'R05':r05['classification'],'PC':pc,'isolated_measurement_copy_matches_current':True,'independent_audit':'not performed'})
log=(E/'integrated_all_003.log').read_text();passed=set(re.findall(r'^test (\S+) \.\.\. ok$',log,re.M));assert passed
listed=(E/'integrated_list_003.log').read_text()
assert all((t+': test') in listed for t in passed if t.startswith(('game::','authoring::','owned_collections::','application::')))
r06dir=R/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001'
boundary=r06dir/'target/r06_boundary_001/MANIFEST.json'
assert hashlib.sha256(boundary.read_bytes()).hexdigest()=='0b6938d77cc6cded598b2b01e51d41bf7e651dd66e8acf2f695a1cf3b5fd44f1'
for artifact in json.loads(boundary.read_text())['archives_and_patches']:
 assert hashlib.sha256((boundary.parent/artifact['path']).read_bytes()).hexdigest()==artifact['sha256'],artifact['path']
raw=R/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'
assert hashlib.sha256(raw.read_bytes()).hexdigest()=='47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9'
write(E/'R06_BOUNDARY_PRESERVATION.json',{'manifest_sha256':hashlib.sha256(boundary.read_bytes()).hexdigest(),'all_frozen_archive_and_patch_bytes_match':True,'raw_V7_sha256':hashlib.sha256(raw.read_bytes()).hexdigest()})
r06=json.loads((r06dir/'COVERAGE.json').read_text())
for row in r06['integration']:
 row['actual_selectors']=[t.replace('r06_original_preimplementation_v7_bytes_restore_without_checksum_rewrite','r07_original_preimplementation_v7_bytes_convert_explicitly_without_overwrite') for t in row['actual_selectors']]
 assert set(row['actual_selectors'])<=passed,(row['id'],set(row['actual_selectors'])-passed)
 row.update(receipt='evidence/integrated_all_003.json',raw='evidence/integrated_all_003.log',source_digest=current['digest'],reuse='actual final integrated source execution; V7 oracle explicitly converted under R07 policy')
 if 'additional_evidence' in row:row['additional_evidence']=['evidence/PC_SUMMARY_003.json','evidence/R05_SUMMARY_003.json','evidence/CLI_SUMMARY_003.json','evidence/CLI_AGGREGATE_SUMMARY_003.json']
write(H/'R06_IMPACT_COVERAGE.json',r06)
r07=[]
for i in range(1,13):
 marker=f'f{i:02d}'
 selectors=sorted(t for t in passed if 'r07_' in t and marker in t)
 if i==9:selectors += sorted(t for t in passed if 'checkpoint_validation::current_v8' in t and any(w in t for w in ['graph_rejects','corrupt_strict','content_authority','exact_envelope']))
 if i==10:selectors += sorted(t for t in passed if 'r07_original' in t)
 assert selectors,i
 r07.append({'id':f'F{i:02d}','actual_selectors':selectors,'receipt':'evidence/integrated_all_003.json','raw':'evidence/integrated_all_003.log','result':'self-validation passed; independent audit not performed'})
dimensions={'success':[1,2,7,9],'invalid_input':[2,3,4,9,10,12],'failure_rollback':[1,3,4,5,9,12],'retry_idempotency':[1,4,6,12],'ordering_concurrency':[1,4,8,10],'boundary_empty_max':[4,10,12],'disconnect_restart_persistence':[9,10],'live_data':[7,12],'producer_consumer':list(range(1,12)),'legacy_fallback_absence':[10,12]}
write(H/'COVERAGE.json',{'owner':'GOAL-CAFCP-01','source_digest':current['digest'],'F01_F12':r07,'roadmap_acceptance':{'T11':[1,2,3,4,5,6,9],'T18':[1,7,8,11],'T20':[9,10],'T27':[7,8]},'dimensions':dimensions,'R06':'R06_IMPACT_COVERAGE.json','scope_note':'Only introduced R06/R07 slices of multi-owner T18/T20/T27, no claim on R08+','independent_audit':'not performed'})
counts={n:json.loads((E/(n+'.json')).read_text())['test_passed'] for n in ['integrated_all_003','integrated_feature_003']}
with (H/'EXPERIMENTS.md').open('a') as f:f.write(f"\n최종 동일 source 검증 {counts}. Current digest `{current['digest']}`; check/event/release/clippy/fmt 및 공개CLI 성공. PC36 실제576rows/31success/5typed refusals, gameplay/completion/error-code 차이0. R05 {r05['classification']}, concurrent peak {r05['concurrent_sessions']['max_peak_bytes']} bytes, 단독 초과 {r05['concurrent_peak_excess_bytes']} bytes; 다른hard실패0. 각 명령 before/after/current source 완전 일치. independent audit 미수행. COVERAGE/R06_IMPACT_COVERAGE/VALIDATION_CURRENTNESS에 실제 selector를 검증해 연결했다.\n")
with (H/'PLAN.md').open('a') as f:f.write('\n## 최종 통합 종료\n\nGEX-10 구현·자체 검증 종료. COVERAGE.json 및 R06_IMPACT_COVERAGE.json이 모든 수용표의 실제 검증을 연결한다. target/r06_r07_integrated_001/MANIFEST.json과 REVIEW_HANDOFF.md가 최종 locator. R06 경계 보존. 독립 감사 요청 대기이며 GEX20/30 또는 R08을 수행하지 않았다.\n')
(H/'REVIEW_HANDOFF.md').write_text(f'''# R06·R07 최종 통합 구현 인계

R06·R07 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기.

독립 감사 GEX-20/30은 수행하지 않았다. 최종 Complete/High 또는 독립 PASS를 선언하지 않는다. 이전 C07의 독립 미검토도 최종 감사 범위다. 기존 dirty/untracked 작업을 reset/stash/clean하지 않았고 전체 변경을 이번 작업으로 귀속하지 않는다.

## Target과 검증

최종 source digest `{current['digest']}`. [최종 manifest](target/r06_r07_integrated_001/MANIFEST.json), [수용표](COVERAGE.json), [R06 31 acceptance/21 IVA 영향](R06_IMPACT_COVERAGE.json), [명령별 currentness](evidence/VALIDATION_CURRENTNESS.json). Actual source/data/tests/config/tools/raw V7 membership, before/after archives, forward/reverse patch 및 격리 복원 근거는 target에 있다. 동결 후 manifest SHA는 FINAL_TARGET.json에 기록한다.

R06 경계 manifest SHA256 `0b6938d77cc6cded598b2b01e51d41bf7e651dd66e8acf2f695a1cf3b5fd44f1`: [별도 경계](../../../content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001/target/r06_boundary_001/MANIFEST.json). 이 상대 locator의 실제 root 경로는 `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/continuations/r06_completion_001/target/r06_boundary_001/MANIFEST.json`이다. 경계 원자료는 덮어쓰지 않았다.

기본 전체 {counts['integrated_all_003']} / test-fixtures 전체 {counts['integrated_feature_003']} passed, failed0, ignored0, 기본 병렬. check/event_log_checks/release bins/clippy/fmt 성공. Clippy의 기존 never_loop/eq_op allow는 R06 동일 명령을 유지했다. Release CLI에서 R06 정확 i128 수치, signed MIN/범위 밖 거부,4종 resolved-owner 집계 및 R07 Snapshot/Live 정상 지급·Maintenance·개화/선택·전투의 trace off/on4문서가 일치했다. 공개 API 새 process 저장 복원은 inventory와 effective-input fingerprint를 비교했다.

PC36:576rows,36conditions,31success/5typed refusals, gameplay/completion/error code 차이0. R05 concurrent peak `{r05['concurrent_sessions']['max_peak_bytes']}` bytes(cap9030188), 초과 `{r05['concurrent_peak_excess_bytes']}`만 합의된 경고다. 나머지 allocation operations/retained/event peak/repeatability/parity hard guard 실패0. latency는 non-comparable 환경의 관찰값으로 보존한다. 실패 원자료와 교정은 EXPERIMENTS.md에 연결했고 덮어쓰지 않았다.

## 구현과 R06→R07 변경 분류

R06: C07-OBS01 delivery 만료 settlement, C03-OBS01 i32::MIN RON decode, 공개 지원 설명 교정, typed expression Snapshot/Live·유한 집계·stat/fingerprint·읽기 전용 amount/trace·정확 숫자·한도/취소/rollback의 구현을 보존하고 최종 소스에서 다시 실행했다.

R07: 단일 OwnedFragment map과 checked UUID grant; 양방향 copy binding; exact equip/unequip/transfer; recipe 개화 해금과 별도 형태 선택; 확인 분쇄의 해제/제거/산출/Run owner 정리; 사망 회수; 공개 DTO·form eligibility와 command 일치. 실제 profile→actor→cast/delivery→amount/trace에 FragmentSourceSnapshot을 연결하고 R05 FragmentCopy/SourceFragment를 기존6 lifetime에 통합했다. Runtime retained/CloneShape accounting, bounded map/set decode, exact/short/overflow 및 중간 취소 재시도도 검증했다.

저장 전환: live schema4 / canonical strict V8. V7 자동 로드는 거부하고 explicit converter가 원본 checksum·caller 원래 identity를 검사한 뒤 새 사본들을 결정적으로 만든다. old awakened는 해당 모든 사본으로 승계, rank/progress/0보유 history와 dust는 보존 정보다. 원본 raw V7 SHA256 `47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9` 유지. V8 전체 graph/compatibility/content mismatch를 commit 전에 거부한다.

## 위임 정책과 경계

사용자 과거 확정 정책으로 위장하지 않은 결정은 [P2G 검토](../../P2G_POLICY_AND_REVIEW.json), [현재 기술 판단](EXPERIMENT_NOTES.md)에 기록했다. 주요 결정: copy cap1024/forms16/recipe64; 기존 UUID stream을 보존하는 FRAGCOPY 전용 checked stream; 개화는 unlock-only/선택 별도/occupied 이전 거부; definition+form compatibility 교집합; recipe 없는 정의는 unavailable; 기존 dust reward/잔액 보존과 새 개화 소비 분리; shared rank gameplay 제거와 explicit V7 history; FragmentCopy owner 및 기존 lifetime 재사용; 원본을 쓰지 않는 deterministic V8 변환.

공식 active 정의의 미작성 recipe와 BasicAttackModifier placeholder는 명시 unavailable다. 검증용 외부 공개 콘텐츠에 통화+환상체 부위 recipe를 작성해 전체 경로를 증명했으며 공식 밸런스를 임의로 발명하지 않았다. Host/GUI의 V8 저장 파일 게시 UX는 이번 Core 범위가 아니다. 구현 hard blocker 없음. 남은 것은 별도 요청의 독립 감사이며 R08 이후/20분 soak/스킬 자동 취소는 수행하지 않았다.
''')
with (H/'EXPERIMENT_NOTES.md').open('a') as f:f.write('\n최종 판단: 공개 describe의 authored_expression_publication을 실제 검증된 DamageExpr/HealExpr Snapshot/Live 지원 상태로 교정했다. 이전 staged 문구를 유지하는 대안은 실제 기능·공개 계약과 모순되어 폐기했다. 이 마지막 제품 변경도 003 전체 검증에 포함했고, 최종 동일 source의 비용/공개 CLI/수용표 및 격리 정역 복원으로 종료한다. 구현 hard blocker는 없고, R05 concurrent peak 단독 초과 경고와 공식 미작성 recipe의 unavailable 상태를 명시한다. 독립 감사는 별도 요청 대기다.\n')
road=R/'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md';s=road.read_text().replace('v1.119','v1.120').replace('| 개발 주작업 | R06·R07 최종 통합 자체 검증·target 동결 |','| 개발 주작업 | R06·R07 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기 |').replace('**R06·R07 최종 통합 자체 검증**','**R06·R07 독립 감사 요청 대기**').replace('현재 [실행 기록]','[실행 기록]').replace('에 따라 최종 통합 검증 중이다.','에 따라 최종 통합 자체 검증과 동결을 마쳤다. [최종 인계](../../goals/content_authoring_fragment_copies/execution/single_goal/REVIEW_HANDOFF.md)를 따른다.').replace('집중14개 PASS 후 전체 검증 진행',f'기본/feature 전체 각각{counts["integrated_all_003"]} PASS, 최종 통합 동결').replace('기본/feature 전체·공개 release CLI·PC36/R05와 currentness/격리 정역 복원 후 최종 통합 인계','별도 요청에 따른 R06·R07 독립 감사. GEX20/30 미수행').replace('**진행 중 · WP02 부분 구현**','**구현·자체 검증 종료 · 독립 감사 대기**',1).replace('**이번 연속 요청 포함 · 정식 계획/구현 미착수**','**구현·자체 검증 종료 · 독립 감사 대기**',1);s=s.replace('R06 상세 설계는 아래 기존 지시서를 재사용하고, R06 target 보존 후 R07 계획 준비·구현으로 계속한다.','R06 상세 설계는 아래 기존 지시서를 재사용했고, R06 target 보존 후 R07 정식 계획·구현과 최종 통합 검증을 수행했다.').replace('현재 canonical execution directory는 기존 `execution/single_goal/`이며, 재개 시 그 안의\n`continuations/r06_completion_001/`에 세 기록 역할의 current locator를 둔다. 새 Goal·별도 전역 진입점을 만들지 않는다.','R06 기록은 기존 `execution/single_goal/continuations/r06_completion_001/`에 보존한다. R07 및 최종 통합 current locator는 [실행 디렉터리](../../goals/content_authoring_fragment_copies/execution/single_goal/PLAN.md)다. 두 Goal의 범위와 수용표를 분리했다.');road.write_text(s)
subprocess.run([sys.executable,str(H/'freeze.py'),'r06_r07_integrated_001'],check=True)
t=H/'target/r06_r07_integrated_001/MANIFEST.json';write(H/'FINAL_TARGET.json',{'manifest':str(t.relative_to(R)),'sha256':hashlib.sha256(t.read_bytes()).hexdigest(),'source_digest':current['digest'],'R06_manifest_sha256':'0b6938d77cc6cded598b2b01e51d41bf7e651dd66e8acf2f695a1cf3b5fd44f1','status':'R06·R07 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기'})
