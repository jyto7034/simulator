# CA01 최소 작성·실행 지원 계약 — P2G-40 계획 후보

이 문서는 NEXT-0005에서 작성한 **non-current candidate**다. 필수 표준 Receipt의 생성·검증이 통과하기 전에는 complete/current Phase Plan이 아니다. 구현·게임 테스트·Goal 작성은 실행하지 않았다. 검증 시도와 최종 선택 여부는 외부 [단계 결과](P2G_40_RESULT_001.json)가 소유한다.

## Phase Plan

- Phase Plan Ref: `PP-CAMEC-P2G40-001`
- Phase Plan Validation Receipt Ref: `PVR-CAMEC-P2G40-001`
- Bound Final Policy Freeze Ref: `FPF-CAMEC-P2G30-001`
- Exact Prerequisite Composite Basis: Working Revision `WR-CAMEC-P2G20-001`; Final Policy Freeze `FPF-CAMEC-P2G30-001`; Phase Plan/Goal Set null; Authoritative Change Obligation Declaration Ref Set explicit empty.
- Repair Basis: no applicable Change; Applicable P2G-40 Obligation Key Set, P2G-40 Resolution Record Ref Set, Applicable Obligation Supersession Record Ref Set all typed sets explicit empty.
- Goal Structure Fact: `single_phase`
- Parallelization: `none approved`
- Validator/Method Version: `p2g40-receipt-validator/1.1.0`
- Known Unknown/Limitations: Bound Freeze의 KU30-01–05와 TR30-01–05를 인수한다. 정확한 타입·문법·주소 encoding은 구현 owner가 정하고, 미정 gameplay·사본/세대의 새 영구 상태·외부 host/배포는 이번 구현 범위로 승격하지 않는다.

Ref 표기는 field의 typed alias와 표시된 Artifact ID를 가진 Exact Artifact Ref의 축약이며 Content Digest는 null이다. Plan의 자신의 Ref는 identity envelope다. Receipt Ref는 생성 명령용 예약 identity로, 실제 Receipt의 존재·통과를 뜻하지 않는다. lifecycle/current selection은 외부 단계 결과가 소유한다.

정책·member·Decision·acceptance의 원본은 [Bound Freeze](P2G_30_FREEZE_001.json)이고, selector는 그 Freeze의 member에서 실제 `Stable Element ID`를 읽어 해석한다. `CAMEC-E01` 같은 기존 ID를 검증기 편의를 위해 개명하지 않는다. acceptance 본문은 이 계획에 복사하지 않는다.

## Entry Evidence and Design Basis

현재 [코드 검토](CODE_REVIEW_001.ko.md) §2–5, `RB-CAMEC-P2G20-001`의 6개 binding과 `RSB-CAMEC-P2G20-001`, [P2G-30 결과](P2G_30_RESULT_001.json)를 인수했다. NEXT-0004 종료 대비 1,683개 경로와 31개 review source가 동일하고 기존 CRAR/CWBS 기록을 재초기화하지 않았다. 실제 지침은 P2G `2026-09-09-r2`; 배치된 GEX `2026-09-09-gex-r1`은 실행하지 않는다.

기존 `ValidatedGameData` 공개·SkillId 조회·step/command·checkpoint 경계는 이미 있는 선행이다. 새 descriptor/효과 주소/SourceMap은 아직 없다. 기존 CRAR Master의 Complete / High는 기존 경계의 이력이며 새 작성 계약의 시험 성공이 아니다. CA10 사본화·CA12 유물 활성 전체·CA16 보스 호출 전체를 최소 작성 metadata의 필수 선행으로 추가하지 않는다.

작성 schema와 공통 effect wrapper가 바뀌면 최신 parser, embedded catalog, generated preview와 직접 runtime 소비자가 한 전환으로 맞아야 한다. 이 상호 제약을 아래 Atomic Group으로 묶는다. 지속적인 구형 별칭·이중 실행은 만들지 않는다. 기존 SkillId·checkpoint 상태는 유지하고 V4/V5 reader 제거·자동 변환·개인 저장 삭제는 범위 밖이다.

## Work Package Set

| Work Package ID | Covered Stable Element ID Selector Set and coverage role | Execution node disposition | Implementation boundary | Expected change | Prerequisites | Validation/completion | Applicable unknown |
| --- | --- | --- | --- | --- | --- | --- | --- |
| WP-CAMEC-01 | implementation: CAMEC-E02, CAMEC-E03, CAMEC-E04, CAMEC-E06; supporting: CAMEC-E01, CAMEC-E05 | member of AG-CAMEC-01 | 작성·compile·검증 owner. `skill_data` raw→validated와 `live_ron` publication에서 효과 identity/tag/typed property, action owner, SourceMap과 읽기 전용 지원 descriptor/index를 일관되게 구성한다. | L01–03의 작성 주소/효과별 query와 진단, 기존 executor에 대응한 literal Damage/Heal/Instant 지원 정보. 태그는 op 권한이 아니며 action-owned 속성은 effect scalar와 구분한다. | 현재 opaque publication; WP-CAMEC-02의 실제 consumer·catalog와 공동 전환 | 유효 입력은 metadata/실행 정의가 한 token에 연결, invalid/unsupported·중복·carrier 오용은 원문 주소로 거부. 기존 다른 op는 실제 지원을 보존하고 미검증 조합을 지원 목록으로 광고하지 않음. | 정확한 schema version/타입/field locator는 구현 선택. 공개 지원 조합별 완전성은 WP-CAMEC-03과 검증. |
| WP-CAMEC-02 | implementation: CAMEC-E01, CAMEC-E04, CAMEC-E05; supporting: CAMEC-E02, CAMEC-E03, CAMEC-E06 | member of AG-CAMEC-01 | 실행·직접 consumer와 catalog 전환 owner. 기존 `ability`/`skill`/`battle` command 경로, fragment SkillId, equipment proc, range preview와 projectile/area helper를 새 작성 metadata publication에 연결한다. | 최신 embedded manifest/RON 및 공통 타입을 직접 생성하는 기존 fixture를 함께 맞춘다. cast/step 및 effect별 작성 출처를 연결하되 기존 source·순서·대상·타격 수·취소 의미를 보존한다. | WP-CAMEC-01의 새 compiler/주소 출력; 기존 executor·SkillId·checkpoint 경계 | 현재 다른 op·spatial consumer 회귀와 compile 확인, Damage→별도 Self Heal step의 기존 결과 보존. signed Heal/HP-loss gate/ModifyDamage/signal 의미를 바꾸지 않음. definition index를 영구 사본 ID로 저장하지 않음. | 모든 실제 직접 call site와 fixture의 exact 수정 위치는 구현 시 확인. wrapper/cutover의 기존 시험 보수까지 이 node가 소유. |
| WP-CAMEC-03 | validation_only: CAMEC-E01, CAMEC-E02, CAMEC-E03, CAMEC-E04, CAMEC-E05, CAMEC-E06 | standalone | 새 계약의 durable 통합 fixture·지원 조합/관찰 oracle owner. 게임 의미를 새로 정하지 않고 AG-CAMEC-01이 공개한 계약을 core/headless 테스트 입구에서 검증한다. | V01–06을 재현 가능한 정상·거부·경계 fixture와 완료 evidence로 만든다. descriptor→compiler→executor 및 diagnostic/source 관찰을 연결한다. | AG-CAMEC-01의 compile 가능한 단일 publication과 기존 게임 fixture. 후보 API를 이미 존재하는 것처럼 호출하지 않음. | 아래 IVA 전부 실행·관찰하고 누락된 지원 조합/빈 성공·출처 단절·상태 변경을 실패 처리. 실패의 production 교정은 AG-CAMEC-01 소유자가 같은 Phase 안에서 수행 후 관련 재검증. | 신규 fixture/테스트 함수 이름과 실행 filter는 구현 뒤 실제 이름으로 기록. 현재는 새 시험 작성·실행 0. |

WP-CAMEC-03은 새 계약의 통합 증거를 소유하는 독립 검증 node다. 기존 source/fixture가 새 schema에서 컴파일·로드되지 않는 상태를 이 node에 미루지 않는다. 그 보수는 AG-CAMEC-01의 cutover에 포함한다. Phase의 완료는 모든 node와 IVA를 요구하며 AG만 끝낸 상태를 정책 완료로 표시하지 않는다.

## Dependency Graph

| Prerequisite Work Package | Dependent Work Package | Dependency Type | Evidence/reason | Broken prerequisite, valid state or completion claim if ignored |
| --- | --- | --- | --- | --- |
| WP-CAMEC-01 | WP-CAMEC-02 | schema_data / producer_consumer | Review B01–04/L01/L04: 새 raw 문법·metadata/publication을 runtime와 최신 RON이 함께 소비 | parser 없는 새 RON 또는 주소/실행 정의 불일치 |
| WP-CAMEC-02 | WP-CAMEC-01 | producer_consumer / migration_compatibility | Review §3 direct consumer/L04: 최신 parser는 actual catalog·generated preview·공유 wrapper 호출자가 맞아야 publish 가능 | 구형 catalog 로드 실패, 무시된 metadata 또는 새 descriptor와 실제 executor의 불일치 |
| WP-CAMEC-01 | WP-CAMEC-03 | validation | V02–04의 지원 descriptor·typed 오류·효과 query가 통합 oracle의 실제 입력 | 미구현 API를 시험했다고 주장하거나 파싱만으로 실행 지원을 판정 |
| WP-CAMEC-02 | WP-CAMEC-03 | validation | V01/V05–06의 runtime·직접 consumer·취소·저장 비변경 관찰 | 신규 fixture가 출처·실행 결과·공유 consumer 회귀를 검증하지 못함 |

## Cyclic SCC Disposition Set

| SCC member Work Packages | Disposition | Boundary/edge reconstruction | Evidence | Recomputed cycle result |
| --- | --- | --- | --- | --- |
| WP-CAMEC-01, WP-CAMEC-02 | coimplement_atomic | AG-CAMEC-01로 결합. edge를 삭제하거나 미완성 fallback으로 우회하지 않음. | Review §3/L04의 schema·catalog·runtime publication 동시 전환 | condensation 뒤 AG-CAMEC-01 → WP-CAMEC-03, acyclic; 기계 검사의 실제 결과는 단계 결과에 기록 |

## Atomic Group Set

| Atomic Group ID | Member Work Packages | Indivisible boundary and evidence |
| --- | --- | --- |
| AG-CAMEC-01 | WP-CAMEC-01, WP-CAMEC-02 | 최신 작성 schema·embedded RON·실행 정의·직접 소비자·기존 fixture가 하나의 유효한 publication을 만드는 최소 전환. 분리 완료하면 reader/input mismatch 또는 지원 metadata와 실제 실행 불일치가 생긴다. 저장 포맷 전환이나 외부팩/전체 VM은 이 group에 포함하지 않음. |

Standalone Final Execution Nodes: WP-CAMEC-03

## Ordered Phase Set

| Phase ID | Primary Stable Element ID Selectors | Supporting policy relations | Assigned final execution nodes | Dependencies | Exit acceptance |
| --- | --- | --- | --- | --- | --- |
| PHASE-CAMEC-01 | CAMEC-E01, CAMEC-E02, CAMEC-E03, CAMEC-E04, CAMEC-E05, CAMEC-E06 | 별도 supporting Phase 없음. WP별 supporting/validation relation은 위 표 소유. | AG-CAMEC-01, WP-CAMEC-03 | 현재 Freeze/Review와 기존 publication·executor; 내부 AG 완료 결과를 검증 node가 소비 | Freeze의 모든 acceptance를 아래 IVA에 따라 실제 검증. 새 문법·catalog·direct consumer 단일 전환, 구현/API·실행 출처·오류 위치·지원 조합의 일치와 기존 의미 보존. |

독립 완료 Phase는 하나다. 먼저 문서/타입만 만드는 foundation Phase에는 현재 계약을 독립적으로 만족하는 consumer-visible 완료 경계가 없고, AG 뒤 검증을 별도 Phase로 미루면 acceptance가 닫히지 않은 상태를 완료로 넘기게 된다. `single_phase`는 구조 사실이며 Single Goal의 실제 작성은 P2G-50 소유다.

## Intermediate-State Audit

| Phase ID | State immediately after completion | Applicable validity checks | Completion-claim evidence | Result/limitation |
| --- | --- | --- | --- | --- |
| PHASE-CAMEC-01 | 최신 schema에서 검증한 metadata+실행 정의가 한 token으로 공개되고 기존 SkillId/cast/command/저장 수명은 유지. 뒤 Phase 없이 사용 가능. | 단일 publication authority; 실패 시 token 없음; 모든 direct consumer/기존 fixture 동시 전환; old/new grammar 이중 경로 없음; effect source와 cast source 구분; 기존 상태·취소/늦은 step 보존. 새 저장 state/포맷·live reload·배포 rollback은 N/A. | AG-CAMEC-01의 compile/로드·기존 회귀와 WP-CAMEC-03의 IVA 전부. 개인 저장 파일 대신 격리 fixture 사용. | 계획상 경계는 Review와 일치. 실제 구현·시험 전이므로 runtime 유효성 PASS 아님. 필수 표준 Receipt 통과 전 current Plan으로 선택하지 않음. |

## Shared Contract Handoff

| Local boundary / source | Meaning owner and producer/consumer | Implementing / validating node | Availability and remaining technical choice |
| --- | --- | --- | --- |
| SC01 작성 주소/SourceMap — Review L01/L02, Freeze E02/E04 | 정책은 해당 Decision. compiler가 definition/action/effect 원문 주소를 만들고 diagnostics/runtime가 소비. runtime caster/cast/proc 발생 source를 정의 ID와 합치지 않음. | AG-CAMEC-01 / WP-CAMEC-03 | 아직 설계안. AG 내 producer/consumer 합치고 통합 oracle 확보 후 사용 가능. 타입·container·field encoding은 경계 불변인 구현 선택. |
| SC02 effect predicate/property와 carrier — Review L02/L03, Freeze E01/E03/E06 | compiler/지원 정보가 typed 지원과 owner를 선언. query/consumer는 같은 effect AND와 다른 effect 존재를 구분하며 태그로 실행 권한을 얻지 않음. | AG-CAMEC-01 / WP-CAMEC-03 | 기존 전체 selector API가 있다고 가정하지 않음. 최소 주소/판별·진단 계약만 구현. 미지원 carrier 변형은 관련 주소로 거부; 새 유물 activation/closure는 후속 owner. |
| SC03 공개·실행·수명 — Review L04/§3, Freeze E01/E04/E05 | loader의 opaque token authority와 기존 step/command executor 재사용. 하나의 schema/cutover, 효과 순서·cast 취소·checkpoint 수명 보존. | AG-CAMEC-01 / WP-CAMEC-03 | 기존 token/SkillId는 현재 존재. 새 metadata 연결은 AG 완료 뒤 사용 가능. 기존 V4/V5 reader를 유지하며 새 generation/사본 key는 추가하지 않음. |

## Integration Validation Ownership Map

`E..[n]`은 Bound Freeze에서 selector로 해석한 member의 acceptance 0-based index다. 이 참조가 요구 원본이며 아래 fixture는 이를 관찰할 기술적 recipe다. 모든 새 durable seam에 실행 node/Phase를 배정했고 미래 GEX 감사자가 시험 코드를 대신 만들도록 미루지 않는다. 단일 Phase이므로 cross-Phase 경계는 없지만 Freeze의 통합 불변식은 모두 아래에서 검증한다.

| Integration acceptance / source | Seam disposition | Read-only locator, invocation and evidence contract | Owning Final Execution Node / Phase | Availability prerequisite | Evidence / limitation |
| --- | --- | --- | --- | --- | --- |
| IVA-CAMEC-01 / CAMEC-E01[0,1], E02[1]; Review V01/V04 | assigned_execution_node | `skill_runtime_contract`의 `run_battle`와 기존 `hit_gated_self_heal_and_buff_stack_repeat_attack_work_together`를 바탕으로 다른 step의 상수 Damage→Self Heal fixture 작성. metadata를 제외한 기존 실행 결과·순서와 재조합/unsupported 분기를 대조. | WP-CAMEC-03 / PHASE-CAMEC-01 | AG-CAMEC-01의 단일 publication | 공식 HP 식/accepted-hit 정책을 새로 결정하지 않음. 테스트 이름·명령은 구현 후 실제 filter로 기록. |
| IVA-CAMEC-02 / CAMEC-E02[0], E03[2], E06[0]; Review V02/V04 | assigned_execution_node | `skill_data` contract fixture와 `live_ron::bundle_validation_returns_typed_skill_contract_errors` 확장. 중복 주소·음수 Damage·dangling 참조·unsupported op/property·carrier 오용에 source/object/action/effect/field 원인을 assert. 공개 descriptor의 실제 지원 조합마다 compiler/executor 결과 대조. | WP-CAMEC-03 / PHASE-CAMEC-01 | AG-CAMEC-01의 descriptor·typed validator·원문 위치 | 태그가 op를 허용하지 않음. 동적 스크립트·임의 property eval·OS/network/file 실행 입구 부재를 코드 경계와 거부 fixture로 확인. |
| IVA-CAMEC-03 / CAMEC-E03[0,1], E04[1]; Review V03 | assigned_execution_node | 같은 effect AND와 두 effect 존재를 서로 다른 fixture로 판별. 특정 원형/형태 ID 선택과 일반 tag predicate 분리. Damage scalar 검사·변경이 다른 Self Heal payload/대상을 바꾸지 않고 공유 carrier 요청은 영향 주소가 있는 진단인지 확인. | WP-CAMEC-03 / PHASE-CAMEC-01 | AG-CAMEC-01의 effect/action 주소와 최소 query/진단 seam | 지원되지 않은 고급 carrier 변형이나 유물 활성 closure를 통과 fixture로 만들지 않음. |
| IVA-CAMEC-04 / CAMEC-E04[0,1]; Review V01/V05 | assigned_execution_node | 작성 원형/추가 효과의 source와 effect 태그 주소를 fixture에서 구분. runtime caster/cast/proc origin 연결을 대조하고 fragment/equipment SkillId·range preview·projectile/area shared command 경로의 metadata 연결 회귀를 기록. | WP-CAMEC-03 / PHASE-CAMEC-01 | AG-CAMEC-01의 source 전달 | 최소 metadata 표현/연결 검증. 유물 영구 사본·실제 부여/연쇄 runtime 구현 완료 claim 없음. |
| IVA-CAMEC-05 / CAMEC-E05[0,1], E02[0]; Review V02/V04/V06 | assigned_execution_node | 유효 embedded 최신 bundle 로드, invalid bundle의 token 미반환, old/new 문법 병렬 성공 부재와 direct fixture 동시 전환을 검사. checkpoint capture/restore 기존 fixture로 정의 index가 새 영구 key가 되지 않았음을 확인. | WP-CAMEC-03 / PHASE-CAMEC-01 | AG-CAMEC-01 cutover 완료 | 개인 저장 접근/삭제/자동 변환 없음. 기존 V4/V5 reader의 제거 또는 미래 호환 인증을 요구하지 않음. |
| IVA-CAMEC-06 / Freeze TR30-03/04; Review V01/V05/V06 | assigned_execution_node | cast 완료·철수 뒤 늦은 step, HP-loss gate Pending/terminal, signed Heal, step ModifyDamage와 tactical resource signal, effect 순서·타격 수를 기존 fixture 관찰과 대조. invalid publication이 gameplay state를 부분 변경하지 않는 경계 확인. | WP-CAMEC-03 / PHASE-CAMEC-01 | AG-CAMEC-01과 보존한 기존 executor/상태 fixture | 기존 actual 보존 검사. 새 accepted-hit·비동기 handle·저장/배포 전환 의미는 별도 owner. |

## Known Unknown Ownership

KU30-01의 정확한 작성 문법·Rust 타입·source locator와 consumer 호출 형태는 AG-CAMEC-01 구현 owner가 상세 설계하고, 신규 fixture/지원 조합별 실제 증거는 WP-CAMEC-03이 소유한다. 내부 선택은 위 책임·동시 전환·완료 경계를 바꾸지 않아야 한다. 공개 의미/수명/취소/상태 소유가 달라지면 해당 P2G owner로 반환한다.

KU30-02–05는 기존 Freeze의 owner/범위를 유지한다. R/F 콘텐츠 초안·공식 수치/태그·외부팩/사본·보스 graph 전체를 현재 정책 또는 실제 지원으로 확정하지 않는다. 이 후보의 구조 검사와 의미 검토는 게임 실행·새 GEX 감사·배포 검증을 대신하지 않는다.
