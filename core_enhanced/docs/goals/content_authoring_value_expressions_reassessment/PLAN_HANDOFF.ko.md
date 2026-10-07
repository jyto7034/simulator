# R06 P2G 재검토·계획 인계

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
| P2G-60 | [exact chain/권한 검사](P2G_60_RESULT_001.json); [최종 보존·링크·요구 검사](FINAL_VERIFICATION_002.json) |

검증은 문서·참조·구조·정적 설계에 대한 결과다. R06 게임 test·benchmark·fresh game audit는 **0건**이며 PASS를 이전하지 않는다. P2G-20 후보001의 source disposition 오기와 count19/actual270 요약 오류, 후보002 생성기의 ID 충돌을 발견해 원본을 보존했다. [003 correction](P2G_20_VALIDATION_003.json)에서 exact refs·enum·acceptance를 재검증했고003만 current다. 기존 R05 실패126·Not Complete 기록과 이전 모든 R06 산출물도 보존했다.

## 계획 종료 기준선과 실제 이전 내용

- Repository: `/mnt/f/work/simulator`
- Branch: `cleanup/simulator-dirty-worktree-20260803`
- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09` — dirty/untracked 상태이며 HEAD만으로 복원할 수 없다.
- 최초 비교 원본: [intake manifest](baseline/intake/MANIFEST.json), [manifest 자체 SHA-256](baseline/intake/MANIFEST.sha256), `baseline/intake/originals/<repository-relative-path>`, [최초 Git status](baseline/intake/GIT_STATUS.txt), [tracked binary patch](baseline/intake/TRACKED.patch). 398개 파일의 실제 bytes를 보존했다.
- 추가 직접 선행 원본: [review manifest](baseline/review/MANIFEST.json), [자체 hash](baseline/review/MANIFEST.sha256), `baseline/review/originals/`의 exact audit target/report/closure 및 CA05 원문. 과거 감사 전체를 새로 인증한 것은 아니다.
- **계획 종료**: [close manifest](baseline/close_002/MANIFEST.json), [manifest 자체 hash](baseline/close_002/MANIFEST.sha256), [Git branch/HEAD/status·toolchain](baseline/close_002/GIT_STATE.json), [전체 status](baseline/close_002/GIT_STATUS.txt), [운영/인수 변경 patch](baseline/close_002/DOCUMENT_CHANGES.patch). manifest 각 row에 SHA-256과 읽을 수 있는 실제 보존 원본 위치가 있다. unchanged source는 intake 사본 재사용, 새 계획/최종 current 안내는 close 사본 보존.
- manifest는 자기 자신과 마지막 verifier 출력/그 detached digest를 member로 포함하지 않는다(자기참조 방지). 자체 hash는 별도 MANIFEST.sha256, 최종 검사는 FINAL_VERIFICATION_002.json이다. 이를 제외한 인계 대상은 close manifest에 exact하게 고정된다.
- 기존 REQUEST_INTAKE_002는 이전 본문을 그대로 두고 이번 링크만 덧붙였으며, 이전 전체 원문은 intake 사본으로 읽을 수 있다. 기존 Working Policy/formal artifacts는 변경하지 않았다. 새 `COMPARISON_ORIGINAL_*` 두 파일은 원본 bytes 그대로인 정책 비교 reference-resolution 사본이고 current policy map은 아니다.

다른 모델은 복원/비교 시 manifest의 original 경로를 읽고 patch와 최초 status를 함께 사용한다. working tree에 blind patch apply/전체 reset을 하지 않는다. 개인/다른 작업자의 파일을 덮어쓰는 권한은 이 인계에서 생기지 않는다. 필요하면 별도 격리 경로에 보존 원본을 복사해 비교한다.

## 남은 제약·다음 owner

[CRA D10](CODE_REVIEW_003.ko.md)과 Goal member의 prerequisites/unknown이 authoritative 내용이다. 구현 모델은 **mutation 직전** 현재 source/dirty/untracked와 close manifest를 대조하고 별도 implementation baseline을 만들어야 한다. 계획 당시 hash와 일치한다는 사실만으로 그때의 도구·선행·작업 격리를 추정하지 않는다.

정확한 parser bounded visitor hook, result-prefix/fallible callback Rust interface, old-build V7/literal fixture, zero omission의 실제 checksum/current content 검증, 현실 workload·clone 비용은 실제 구현 전 준비 및 구현 완료 전 검증이 필요하다. 초기 safety budget과 기존 allocation guard는 공식 balance/새 성능 보장이 아니다. 신규 정책·저장 손실·미정 weight가 필요해지면 해당 owner로 반환한다. 외부 Unity/server 배포와 R07+는 이 계획 범위 밖이다.

후속 구현 요청 예문(사용자가 모델을 바꾼 뒤 **별도 제출**):

> `core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md`와 exact Handoff `HA-CAVER-P2G60-001`을 읽고 `(GS-CAVER-P2G50-001, GOAL-CAVER-01)`의 R06 구현을 진행해줘. mutation 전에 현재 상태를 planning-close 기준선과 대조하고 implementation baseline 및 D10 준비 점검을 기록해. 현행 GEX의 goal-run으로 인수하되, GEX-10 구현·자체 검증·Audit Target 고정 직후 **GEX-20 진입 전에 반드시 중단**하고 “구현·자체 검증 종료 / 감사 요청 대기”로 보고해. 필수 선행과 공유 계약을 건너뛰지 말고 R07 이후는 시작하지 마.

이 예문 자체는 지금의 구현 승인이나 GEX 실행이 아니다. 일반 “계속”, 자동 재개, 컨텍스트 압축, 모델 변경만으로 이 경계를 넘지 않는다.

2026-09-13 재개 시 기존 최종 검증001의 정상 종료를 확인했다. 현황 상단의 이전 상태 문장을 current 인계와 맞추는 문서 수정만 추가했다. `baseline/close/`와 FINAL_VERIFICATION_001은 첫 종료 관찰로 보존하며, 최종 current 기준선은 `baseline/close_002/`와 FINAL_VERIFICATION_002다. Freeze·Plan·Goal·Handoff 및 production source는 그대로다.
