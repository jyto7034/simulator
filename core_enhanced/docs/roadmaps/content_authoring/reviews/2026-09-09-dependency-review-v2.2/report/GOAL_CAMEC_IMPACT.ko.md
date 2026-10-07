# 기존 M0·GOAL-CAMEC-01 영향 판정

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

## 1. 권고: A — 현재 Plan과 최소 Goal을 첫 구현 단위로 유지

현재 조사에서 이 최소 Goal의 **전제·범위·필수 선행을 수정해야 할 오류는 발견하지 않았다.** 외부 팩 전체, 파편 사본화, 공유 보호막, 완전한 Flow, 보스 확장, 유물 chain, 침식을 이 Goal 앞에 구현해야 할 기능 의존도 확인되지 않았다.

이것은 이번 조사 범위의 권고이며 **정식 current/stale 판정, GEX 실행 자격, Goal activation, 구현 승인**을 대신하지 않는다. 사용자가 후속 구현을 요청할 때 기존 GEX 인수에서 그 시점의 current basis를 확인해야 한다. 이번에는 NEXT-0009를 실행하지 않았다.

## 2. 범위를 고정한 실제 근거

| 판정 질문 | 확인 결과 | 근거 |
| --- | --- | --- |
| embedded 최소 계약이 독립 유효한가? | 현재 loader/compiler/실행 kernel과 검증 후 게시가 존재한다. metadata를 현재 경로에 연결하는 최소 slice는 외부 팩 게시와 독립이다. | [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S017](../evidence/SOURCES.ko.md#s017) |
| 기존 executor를 재사용하면 곧 폐기할 구조를 고정하는가? | literal Damage/Heal/Instant와 현 타격/취소 의미를 보존하는 adapter는 재사용 가능하다. 전체 후속 Flow를 현 step 구조로 강제한다는 승인 내용은 아니다. | [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S086](../evidence/SOURCES.ko.md#s086) |
| 정의 주소를 사본/시전 ID와 혼동하는가? | Freeze/Goal은 작성 EffectSite/ActionSite/SourceMap과 runtime caster/cast/proc를 구별한다. 새 영구 사본과 generation/save는 제외한다. | [S009](../evidence/SOURCES.ko.md#s009) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S092](../evidence/SOURCES.ko.md#s092) |
| 계산/타격 수/공유 delivery/취소 의미가 보존되는가? | 현재 ModifyDamage folding·signed Heal·HP-loss gate·released projectile/fixed area 존속을 직접 확인했다. 최소 Goal의 보존 조건과 맞는다. | [S018](../evidence/SOURCES.ko.md#s018) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S065](../evidence/SOURCES.ko.md#s065) · [S097](../evidence/SOURCES.ko.md#s097) |
| Atomic Group 범위가 충분한가? | WP01 compiler/descriptor와 WP02 latest RON/catalog/direct consumers가 기존 AG01로 묶이고 WP03의 durable oracle은 같은 Single Goal이 소유한다. | [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S009](../evidence/SOURCES.ko.md#s009) |
| 저장을 먼저 다시 써야 하는가? | 새 영구 state/format을 만들지 않는 최소 Goal이다. 현 V5 writer/V4·V5 reader와 metadata index 비영구성 확인을 보존한다. | [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S065](../evidence/SOURCES.ko.md#s065) · [S092](../evidence/SOURCES.ko.md#s092) |

## 3. 그대로 재사용할 exact artifact

공통 Topic 경로는 `core_enhanced/docs/goals/content_authoring_minimal_effect_contract/`다. 아래 모든 원본은 수정하지 않았다.

| 원본 파일 | identity | 재사용/영향 | 최소 수정 owner |
| --- | --- | --- | --- |
| WORKING_POLICY.ko.md | WPD-CAMEC-20260909 / WR-CAMEC-P2G20-001 | 정책 의미·범위/미정 분리 유지 | 없음. 새 정책 의미가 필요할 때만 P2G-10 |
| CODE_REVIEW_001.ko.md | 기존 code-informed Review | 기존 source/재사용 근거 인수; 이번 조사 F01–F21은 별도 보고서 | 기존 Review 덮어쓰기 없음. 실제 새로운 코드 사실이 전제를 깨면 P2G-20 |
| P2G_30_FREEZE_001.json | FPF-CAMEC-P2G30-001 | literal slice/계산·취소 보존/사본·저장 제외 유지 | 이번에는 불필요. 승인 의미 수정이 생기면 가장 이른 정책 owner부터 |
| P2G_40_PHASE_PLAN_002.json | PP-CAMEC-P2G40-002 | AG-CAMEC-01(WP01+WP02) + WP03 단일 Phase 유지 | 이번에는 불필요. 실제 atomic/dependency/scope 수정 필요 시 P2G-40 |
| P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_002.json | PVR-CAMEC-P2G40-002 | 기존 receipt를 당시 증거로 인수 | 새 검사라고 세지 않음. 실제 Plan successor가 있어야 대응 검증 |
| P2G_50_GOAL_SET_001.ko.md | GS-CAMEC-P2G50-001 / GOAL-CAMEC-01 | 현재 최소 Single Goal·검증 owner/원문 acceptance 유지 | 이번에는 불필요. 상위 Plan/Goal 내용 변경 때 P2G-50 |
| P2G_60_HANDOFF_001.json | HOF-CAMEC-P2G60-001 | 현재 선택된 basis와 한계/비범위 인수 | 이번에는 불필요. 변경된 upstream 인계가 필요할 때 P2G-60 |
| P2G_50_AUTHORING_REVIEW_001.ko.md / P2G_60_RESULT_001.json | 기존 작성 검토/complete result | 당시 표준 검증과 선택 상태를 인수 | 이 연구가 당시 해시·PASS를 새 값으로 재발행하지 않음 |

그 밖의 CA00 intake/requirements/COV, 기존 Decision·감사 원본과 단계별 history는 그대로 보존한다. 이전 전체 의존성 검토가 미완료였다는 사실은 기존 M0 성과가 없다는 뜻이 아니다.

## 4. 지금 닫은 질문과 후속 구현의 상세 선택

이번 조사에서는 실제 loader/publication, shared step semantics, direct cast/queued/impact consumers, 취소 뒤 payload 수명, durable 저장에서 정의 index의 비영구성이라는 핵심 경계를 조사했다. 따라서 “나중에 구현자가 필수 소비자를 알아서 찾아야 한다”는 막연한 B 권고로 돌리지 않았다.

정확한 Rust 타입·source locator 소유 방식·ID 표현과 fixture 함수 이름은 기존 AG의 상세 구현 책임이다. 이 선택은 지금 최소 계약을 정확히 구현하기 위한 기술 상세이며 외부 팩·사본·Flow 전체의 선구현 승인이 아니다.

현재 최소 Goal이 처음으로 해야 할 전환은 **metadata/compiler + 최신 RON/catalog + 실제 consumer**의 결속이다. source locator는 외부 팩에서 owned source로 확장될 수 있고, 개인 합성은 미래 R14에서 explicit effective-plan reference를 필요로 한다. 이 비용을 기록하되 지금 미래 상태를 phantom ID/빈 executor로 미리 실행하지 않는다.

## 5. 인접 이슈와 별도 owner

**저장 문구:** `data_loading_contract.md`의 durable 단락 V4와 actual V5 writer/V4·V5 reader의 차이는 기존 작성 검토에도 기록돼 있다. 최소 Goal의 저장 비변경 범위를 확장하지 않는다. 해당 canonical 저장 owner가 정합성을 닫되, 이번에는 원문을 교정하거나 old reader를 제거하지 않았다. [S039](../evidence/SOURCES.ko.md#s039) · [S092](../evidence/SOURCES.ko.md#s092) · [S094](../evidence/SOURCES.ko.md#s094).

**침식 시점/Breakdown:** 이 보고서는 실제 미구현 boundary와 구체 정책 질문을 R09로 분리했다. 그것을 minimum metadata의 선행으로 끌어올리지 않는다. [S018](../evidence/SOURCES.ko.md#s018) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S077](../evidence/SOURCES.ko.md#s077).

**외부 wire 소비:** 코어 projection의 영향을 기록했지만 서버/Unity 계약 불일치를 이 Goal의 전체 재작성 선행으로 사용하지 않는다. [S090](../evidence/SOURCES.ko.md#s090) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

## 6. A 권고가 바뀌는 조건

현재 허용한 최소 metadata 전환만으로 새 persistent-state·구형/신형 이중 실행·계산/취소 의미 변경이 불가피하다는 **새 실제 근거**가 발견되면, 그 최초 문제 지점의 owner에 반환해야 한다. 정책 의미면 P2G-10, 코드 사실/전제면 P2G-20, Work Package·Atomic Group·Phase 구조면 P2G-40이 가장 이른 수정 owner다. 그때 exact artifact와 필요한 최소 successor를 지정하고 필요한 후속만 다시 검증한다. 지금은 그 조건이 확인되지 않았으므로 C 권고/정식 successor/재실행을 발행하지 않았다.

최종 답: **GOAL-CAMEC-01을 첫 구현 단위로 유지해도 된다는 A 권고다. 별도 구현 요청 전에는 여전히 미활성·미승인이다.**
