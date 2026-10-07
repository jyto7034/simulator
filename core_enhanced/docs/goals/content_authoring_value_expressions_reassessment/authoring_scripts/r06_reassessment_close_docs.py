from pathlib import Path
import json,hashlib,subprocess,datetime
root=Path('/mnt/f/work/simulator');p=root/'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment';rel=p.relative_to(root);old=root/'core_enhanced/docs/goals/content_authoring_value_expressions';m=json.loads((p/'baseline/intake/MANIFEST.json').read_text());by={r['path']:r for r in m['files']}
def original(path):return root/by[str(path.relative_to(root))]['original']
# Compare-before-write; all three are authorized mutable intake/status pointers.
status=root/'core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md';ops=status.parent/'OPERATING_RULES.md';intake=old/'REQUEST_INTAKE_002.ko.md'
for q in [status,ops,intake]:assert q.read_bytes()==original(q).read_bytes(),f'Concurrent update: {q}'
branch=subprocess.check_output(['git','branch','--show-current'],cwd=root).decode().strip();head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root).decode().strip()
hand='''# R06 P2G 재검토·계획 인계

**R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기**

이번 요청의 산출물은 계획이다. GEX-00 인수·activation·게임 구현·게임 시험·완료 감사는 시작하지 않았다. 이 문서는 final verification과 계획 종료 manifest를 함께 읽는 외부 인계 기록이며 새 정책 authority가 아니다.

## 하나의 current 인계 기준

| 결과 | Exact identity | 소유 파일 |
| --- | --- | --- |
| Working Policy / reviewed batch | WPD-CAVER-20260913 / WR-CAVER-P2G20-003 / RB-CAVER-P2G20-003 | [current view](WORKING_POLICY.ko.md), [complete review003](P2G_20_REVIEW_003.json) |
| Final Freeze | FPF-CAVER-P2G30-001 | [Freeze](P2G_30_FREEZE_001.json), [외부 membership](P2G_30_RESULT_001.json) |
| Phase Plan / receipt | PP-CAVER-P2G40-001 / PVR-CAVER-P2G40-001 | [Plan](P2G_40_PHASE_PLAN_001.json), [generated receipt](P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json) |
| Goal Set / member | GS-CAVER-P2G50-001 / GOAL-CAVER-01 | [구현 Goal·시험 계획](P2G_50_GOAL_SET_001.ko.md) |
| Handoff | HA-CAVER-P2G60-001 | [durable Handoff](P2G_60_HANDOFF_001.json) |
| 코드·상세 설계·비교 | CRA-CAVER-P2G20-003 | [CODE_REVIEW_003](CODE_REVIEW_003.ko.md), C01 대조 / D01–D10 설계 / V01–V21 시험 |

9개 정책 요소·27개 승인 acceptance를 유지했다. 구현 구조는 4 WP, 하나의 atomic group과 별도 통합 검증 node, **하나의 Phase·Single Goal**이다. 서로 다른 basis의 계획을 current로 혼합하지 않는다. 원래 R06 문서와 새 topic의 초기 후보는 원위치 보존 비교 이력이다.

## 기존 대비 채택한 개선

- 기존 4단계 구현 순서를 독립 Phase 완료로 사용하지 않고, public compiler/executor와 AT06 stat/profile/save의 최소 동시 전환을 명시했다.
- i128 checked 평가·정수 rounding oracle·unit table·bounded AST lowering, literal 음수/0 호환과 shared evaluator를 구체화했다.
- release capture와 live frame, target availability, finite tile/owner grouping, actual resource lease와 completed-effect prefix 결과 read를 정의했다. query_action의 active snapshot을 settled로 오인하지 않는다.
- enhancement 사본 UUID를 projection에 유지하고 definition-wide 합산을 교체한다. Attack/SpellPower·weapon/성장·기존 Percent/cadence의 책임과 순서를 고정했다.
- V7의 재직렬화 checksum과 content fingerprint를 고려한 zero omission·**이전 빌드 저장 원본** 검증, static explain의 context 부재, cache/source-copy 격리를 명시했다.
- callback 오류 전파·취소·candidate rollback·clone/retained 예산 및 21개 통합 시험과 10개 검증 차원을 구현 owner에 배정했다.

이전 AI 제안은 사용자 정책으로 승격하지 않았다. P2G_10_REASSESSMENT_001.json은 각 기존 Decision/근거, 새 사실, 유지 이유·영향·새 owner를 연결하고 CODE_REVIEW_003 C01은 기술 제안의 유지/보완/교체/제외를 연결한다. 기존 9라는 숫자가 아니라 독립 의미와 coverage 검토로 9를 유지했다. R07+ 신규 기능·R19 final weight는 제외한다.

## 실제 검증과 교정 이력

| 단계 | 실제 근거 / 결과 |
| --- | --- |
| P2G-00 | [initial/result](P2G_00_RESULT_001.json): 새 경로/identity, raw input 고정, initial element/Decision/question empty |
| P2G-10 | [15 exit 검사](P2G_10_VALIDATION_001.json): 9 decided, 27 unchanged acceptance, 11 dispositions, 0 questions |
| P2G-20 | [003 exit 검사](P2G_20_VALIDATION_003.json): 12 requirements/observations, 9 reviewed members/bindings; old review source34파일 및 선행 source270파일 현재 hash 일치 |
| P2G-30 | [Freeze/membership 결과](P2G_30_RESULT_001.json): 9 exact member·Decision·27 acceptance projection와 binding coverage |
| P2G-40 | [표준 검증 log](P2G_40_VALIDATION_001.log), [의미 검토](P2G_40_RESULT_001.json): validator1.2.0 passed, exit0, failures0; 4 WP/8 edges/1 SCC/1 AG/2 nodes/1 Phase/21 integration |
| P2G-50 | [표준 검증 log](P2G_50_VALIDATION_001.log), [의미 검토](P2G_50_RESULT_001.json): validator1.3.0 quality-contract2 passed, exit0, failures0; 9 trace rows/21 integration/10 dimensions |
| P2G-60 | [exact chain/권한 검사](P2G_60_RESULT_001.json); [최종 보존·링크·요구 검사](FINAL_VERIFICATION_001.json) |

검증은 문서·참조·구조·정적 설계에 대한 결과다. R06 게임 test·benchmark·fresh game audit는 **0건**이며 PASS를 이전하지 않는다. P2G-20 후보001의 source disposition 오기와 count19/actual270 요약 오류, 후보002 생성기의 ID 충돌을 발견해 원본을 보존했다. [003 correction](P2G_20_VALIDATION_003.json)에서 exact refs·enum·acceptance를 재검증했고003만 current다. 기존 R05 실패126·Not Complete 기록과 이전 모든 R06 산출물도 보존했다.

## 계획 종료 기준선과 실제 이전 내용

- Repository: `/mnt/f/work/simulator`
- Branch: `BRANCH_VALUE`
- HEAD: `HEAD_VALUE` — dirty/untracked 상태이며 HEAD만으로 복원할 수 없다.
- 최초 비교 원본: [intake manifest](baseline/intake/MANIFEST.json), [manifest 자체 SHA-256](baseline/intake/MANIFEST.sha256), `baseline/intake/originals/<repository-relative-path>`, [최초 Git status](baseline/intake/GIT_STATUS.txt), [tracked binary patch](baseline/intake/TRACKED.patch). 398개 파일의 실제 bytes를 보존했다.
- 추가 직접 선행 원본: [review manifest](baseline/review/MANIFEST.json), [자체 hash](baseline/review/MANIFEST.sha256), `baseline/review/originals/`의 exact audit target/report/closure 및 CA05 원문. 과거 감사 전체를 새로 인증한 것은 아니다.
- **계획 종료**: [close manifest](baseline/close/MANIFEST.json), [manifest 자체 hash](baseline/close/MANIFEST.sha256), [Git branch/HEAD/status·toolchain](baseline/close/GIT_STATE.json), [전체 status](baseline/close/GIT_STATUS.txt), [운영/인수 변경 patch](baseline/close/DOCUMENT_CHANGES.patch). manifest 각 row에 SHA-256과 읽을 수 있는 실제 보존 원본 위치가 있다. unchanged source는 intake 사본 재사용, 새 계획/최종 current 안내는 close 사본 보존.
- manifest는 자기 자신과 마지막 verifier 출력/그 detached digest를 member로 포함하지 않는다(자기참조 방지). 자체 hash는 별도 MANIFEST.sha256, 최종 검사는 FINAL_VERIFICATION_001.json이다. 이를 제외한 인계 대상은 close manifest에 exact하게 고정된다.
- 기존 REQUEST_INTAKE_002는 이전 본문을 그대로 두고 이번 링크만 덧붙였으며, 이전 전체 원문은 intake 사본으로 읽을 수 있다. 기존 Working Policy/formal artifacts는 변경하지 않았다. 새 `COMPARISON_ORIGINAL_*` 두 파일은 원본 bytes 그대로인 정책 비교 reference-resolution 사본이고 current policy map은 아니다.

다른 모델은 복원/비교 시 manifest의 original 경로를 읽고 patch와 최초 status를 함께 사용한다. working tree에 blind patch apply/전체 reset을 하지 않는다. 개인/다른 작업자의 파일을 덮어쓰는 권한은 이 인계에서 생기지 않는다. 필요하면 별도 격리 경로에 보존 원본을 복사해 비교한다.

## 남은 제약·다음 owner

[CRA D10](CODE_REVIEW_003.ko.md)과 Goal member의 prerequisites/unknown이 authoritative 내용이다. 구현 모델은 **mutation 직전** 현재 source/dirty/untracked와 close manifest를 대조하고 별도 implementation baseline을 만들어야 한다. 계획 당시 hash와 일치한다는 사실만으로 그때의 도구·선행·작업 격리를 추정하지 않는다.

정확한 parser bounded visitor hook, result-prefix/fallible callback Rust interface, old-build V7/literal fixture, zero omission의 실제 checksum/current content 검증, 현실 workload·clone 비용은 실제 구현 전 준비 및 구현 완료 전 검증이 필요하다. 초기 safety budget과 기존 allocation guard는 공식 balance/새 성능 보장이 아니다. 신규 정책·저장 손실·미정 weight가 필요해지면 해당 owner로 반환한다. 외부 Unity/server 배포와 R07+는 이 계획 범위 밖이다.

후속 구현 요청 예문(사용자가 모델을 바꾼 뒤 **별도 제출**):

> `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md`와 exact Handoff `HA-CAVER-P2G60-001`을 읽고 `(GS-CAVER-P2G50-001, GOAL-CAVER-01)`의 R06 구현을 진행해줘. mutation 전에 현재 상태를 planning-close 기준선과 대조하고 implementation baseline 및 D10 준비 점검을 기록해. 현행 GEX의 goal-run으로 인수하되, GEX-10 구현·자체 검증·Audit Target 고정 직후 **GEX-20 진입 전에 반드시 중단**하고 “구현·자체 검증 종료 / 감사 요청 대기”로 보고해. 필수 선행과 공유 계약을 건너뛰지 말고 R07 이후는 시작하지 마.

이 예문 자체는 지금의 구현 승인이나 GEX 실행이 아니다. 일반 “계속”, 자동 재개, 컨텍스트 압축, 모델 변경만으로 이 경계를 넘지 않는다.
'''.replace('BRANCH_VALUE',branch).replace('HEAD_VALUE',head)
(p/'PLAN_HANDOFF.ko.md').write_text(hand)
intake.write_text(intake.read_text()+'''\n\n## 2026-09-13 후속 요청 — 원본 보존 재검토·계획 인계\n\n위 본문은 당시 인수 상태로 그대로 보존한다. 이후 사용자가 별도 identity의 P2G-00 재검토를 명시했으며, 현재 결과는 [새 Working Policy](../content_authoring_value_expressions_reassessment/WORKING_POLICY.ko.md)와 [계획 인계](../content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md)가 소유한다. exact `FPF-CAVER-P2G30-001 / PP-CAVER-P2G40-001 / GS-CAVER-P2G50-001 / HA-CAVER-P2G60-001`; 27 acceptance 보존, P2G-00→60 재검토 완료. 원문/비교·단계별 검증·설계·branch/HEAD/status·파일별 hash·manifest 자체 hash·실제 복원 사본은 인계에 연결했다. 기존 WPD-CAVE와 formal artifact는 그대로 보존한다. **R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기**; GEX 인수·activation·구현·감사는 별도 명시적 구현/감사 요청 전 시작하지 않는다.\n''')
ops.write_text(ops.read_text()+'''\n\n### 2026-09-13 R06 재검토 요청 연결\n\n기존 요청/운영 이력은 보존한다. 후속 사용자 원문과 새 P2G-00 재검토 identity·current 인계는 [R06 재검토 PLAN_HANDOFF](../../goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md)가 소유한다. 현재 계획 종료점은 동일하게 P2G-60이며 **R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기**다. 기존 R06의 REQUEST_INTAKE_002에는 새 결과 링크만 추가했다. 자동 재개·일반 “계속”·모델 변경으로 GEX에 진입하지 않는다.\n''')
s=status.read_text()
# Replace only current summary/current card; retain all historical recent records.
s=s.replace('| 선택된 정식 Topic / Goal | R06 `WPD-CAVE-20260913` / current `WR-CAVE-P2G10-001`; 9 elements/9 Decisions/0 questions. 정식 Phase Plan·Goal Set·Goal/Handoff 및 GEX target은 아직 없음 |','| 선택된 정식 Topic / Goal | R06 재검토 `WPD-CAVER-20260913` / `WR-CAVER-P2G20-003`; current `FPF-CAVER-P2G30-001 / PP-CAVER-P2G40-001 / GS-CAVER-P2G50-001 / HA-CAVER-P2G60-001`; Single member `GOAL-CAVER-01`, Not Activated |')
rows={'현재 검증':'P2G-00→60 문서/정적 검토 완료. 27 승인 acceptance 유지, P2G-40/50 표준 validator passed(exit0/failures0), 21 integration/10 dimensions. 게임 구현·test·감사 0건. [최종 검사](../../goals/content_authoring_value_expressions_reassessment/FINAL_VERIFICATION_001.json)','진행 중 작업자·실행 기록':'계획 전용 재검토·인계 종료. 다음 owner는 별도 구현 요청을 받은 모델. GEX 실행 없음.','남은 문제':'D10 구현 전 준비·실제 runtime/old-save/비용 검증과 이후 감사는 별도 요청 범위; 계획 종료를 막지 않음. [인계](../../goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md)'}
lines=s.splitlines()
for i,line in enumerate(lines):
 for k,v in rows.items():
  if line.startswith('| '+k+' |'):lines[i]='| '+k+' | '+v+' |'
s='\n'.join(lines)+'\n'
start=s.index('## 3. 다음 작업');end=s.index('## 4. 연결된 원본과 근거')
s=s[:start]+'''## 3. 다음 작업

**작업 표지: R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기**

| 항목 | 수행할 내용 |
| --- | --- |
| 현재 대상 | `GS-CAVER-P2G50-001 / GOAL-CAVER-01`, `HA-CAVER-P2G60-001`; Authored / Not Activated / Implementation Authorized: No |
| 근거 | [재검토 인계·exact 계획·검증·baseline](../../goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md); [새 Working Policy](../../goals/content_authoring_value_expressions_reassessment/WORKING_POLICY.ko.md). 이전 WPD-CAVE 기록은 원위치 보존 비교 자료 |
| 완료 | P2G-00→60 재검토, 기존 27 acceptance coverage, 1 Phase·4 WP·21 integration, 코드 상세 설계, Git/status/hash/manifest/실제 원본 인계 |
| 다음 owner | 구현을 명시하는 별도 사용자 요청을 받은 모델. mutation 전 planning-close 대비 current source·dirty·선행 확인, implementation baseline·D10 준비 점검 필수 |
| 제한 | R06 게임 구현·시험·감사 미실행. R07+ 착수 없음. 일반 계속/자동 재개/모델 변경으로 GEX 인수·activation·구현·감사를 시작하지 않음 |
| 기존 종료 | R04/R05의 과거 target·실패·Not Complete 및 successor Complete / High는 당시 원문으로 보존. 이번은 새 게임 감사가 아님 |

'''+s[end:]
s=s.replace('| CA05 | 계산식·집계·성장 | R06 P2G-00/10 완료; 9 policy elements decided. 현재 요청은 single-r-v4-plan-only, P2G-60 계획 인계 후 종료. 구현·감사 별도 요청 | [R06 Working Policy](../../goals/content_authoring_value_expressions/WORKING_POLICY.ko.md) |','| CA05 | 계산식·집계·성장 | R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기. 게임 구현·감사 미실행 | [새 계획 인계](../../goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md) |')
s=s.replace('## 6. 최근 기록\n','''## 6. 최근 기록

### 2026-09-13 · R06 원본 보존 P2G-00 재검토→P2G-60 인계 완료

- 새 `WPD-CAVER-20260913`에서 원문 intake·empty initial로 시작해 각 stage를 재검토했다. 9 정책/27 acceptance를 유지하고 source/result/snapshot/live/copy/stat/save/cache 경계를 구체화했다.
- [인계](../../goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md)에 exact Freeze/Plan/Goal/Handoff, 표준 검증 결과, 비교·개선, 실패 후보 보존, planning-close branch/HEAD/status·관련 SHA·manifest hash·실제 원본을 연결했다.
- 기존 R06/과거 실행 기록을 보존하며 구현 요청 대기에서 정상 종료한다. R06 게임 완료나 GEX 감사 PASS를 주장하지 않는다.
''')
status.write_text(s)
q=p/'WORKING_POLICY.ko.md';s=q.read_text().replace('P2G-00→60 passed; planning-close preservation verification pending','P2G-00→60 passed; R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기');s+='\n[FINAL_VERIFICATION](FINAL_VERIFICATION_001.json)과 [planning-close manifest](baseline/close/MANIFEST.json)를 함께 대조한다. GEX/다른 R 진행은 별도 명시적 사용자 요청 전 금지.\n';q.write_text(s)
# Retain authoring scripts as planning tooling, not edits to repository validators.
q=p/'authoring_scripts';q.mkdir()
for src in Path('/tmp').glob('r06_reassessment_*.py'):(q/src.name).write_bytes(src.read_bytes())
print('Current intake/status/operating links updated; old intake/operating bytes preserved as exact prefix. Closing baseline next.')
