# U16-1 독립 완료 감사 — Complete / High

2026-10-15, GEX-20 stage-run. GS-CSNA-001 / GOAL-CSNA-01 / PHASE-CSNA-01.

**판정: Complete / High. 확인된 제품 결함 0건, 열린 완료 차단 0건.** 수용18개·검증차원10개·통합조건5개·실행node4개를 검토했다. 이 판정은 상점·HQ 유료채용·Event·소비품의 U16-1에 한정한다. 전체 U16 완료가 아니며 U16-2/3/4를 시작하지 않았다.

## 권한·독립성·고정 대상

사용자의 「CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md를 지침 삼아 U16-1 독립 감사에 착수」 요청을 직접 GEX-20 실행 권한으로 인수했다. 이번 대화의 감사자는 U16-1 구현·교정에 참여하지 않았다. 서브에이전트나 위임 없이 수행했으며 실제 모델 전환을 주장하지 않는다. 제품·기존 시험·정책·동결 자료의 수정은 없다.

적용 기준은 현행 상시 지시서, GEX `2026-10-04-gex-r8` Reader Contract/GEX-20, 로드맵 §1.9, FPF-CSNA-001, PP-CSNA-002 및 [Goal Set](../../P2G_50_GOAL_SET.ko.md)이다. 구현자의 자체검토를 독립 판정으로 인수하지 않고 실제 변경·호출·assertion·원시 결과를 대조했다.

- [원본 target](target/u16_1_current_001/AUDIT_TARGET_RECORD.json): `u16_1_current_001`, HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`.
- SOURCE archive SHA256: `9f052914849f6b6a6b0f75689c390e62fb4f6734811ffee7143d32ea475c4acc`.
- [입구](audit/independent001/ENTRY_CURRENTNESS.json)·[출구](audit/independent001/EXIT_CURRENTNESS.json): source1006/evidence963 모두 일치, `current / unchanged`. 세 production root의 누락 파일0.
- [독립 검산](audit/independent001/EVIDENCE_RECHECK.json): SOURCE archive의 전체 member/bytes와 manifest 일치, BEFORE585→SOURCE1006의 delta443개 preimage/postimage 일치. 이 근거로 구현자의 격리 정역복원 결과를 재사용했다. 작업트리 patch 적용0.
- [감사 대상 결속](audit/independent001/AUDIT_TARGET_RECORD.json)과 [기계 결과](audit/independent001/GEX_20_RESULT.json). 감사 evidence와 완료 문서는 implementation target 밖에 둔다. 동결 PLAN/COVERAGE/GEX10의 ‘감사 대기’는 당시 인계 상태로 보존한다.

## 조건별 판단

[독립 조건표](audit/independent001/CONDITION_REVIEW.json)에 A01–A18 각각의 원문·실제 source·판단·재사용 증거, 10차원·5IVA·4node를 연결했다.

| 조건 | 직접 검토한 결과 |
| --- | --- |
| A01–A04 / 상품·거래 | typed 상품 identity는 가격과 독립이다. 공통 EconomyGrantState와 enclosing application candidate가 비용·사본·UUID·재고를 함께 확정한다. 정상 사본 지급, 반복 offer 거절, 전체 페이지 갱신, 부족·소진 무변경, 매입올림/매각내림·차익방지, 유물/귀속/장착/재료 판매금지를 확인했다. guaranteed 부활촉매와 별도 침식촉매의 자산 owner를 유지한다. |
| A05 / HQ | 가격100·세션2·깊이4·보유10의 기존 자격을 공개 terms와 명령이 공유한다. 호스트의 두 채용은200 차감·직원2·기본세트4사본 증가이며 대여·queue·반복거절·입장전 복원/완료유지를 확인했다. |
| A06–A09 / Event | node seed의 단일 d20, raw1/20·허용 DC·적성3, 직원이 장착한 실제 소유 장비 최고보정 하나·UUID 동률순서·열쇠 비소비를 확인했다. committed_check와 무료침식 receipt가 재지급을 막는다. 분기·전투 승패/후퇴·무료침식·재시작은 해당 현재 회귀와 공개 trace로 확인했다. |
| A10–A12 / 소비품·보유 | 즉시치료도 이전 modifier를 제거하고 수량1을 소비한다. 결과확정만 기간을 감소하며 비전투·후퇴·중복결과확인은 추가 감소하지 않는다. 실제 HQ 저장으로 승리18tick/패배25tick 모두 확인했다. 장비71/occupied73 및 유물65번째 구매에 UI 크기 제한을 적용하지 않는다. |
| A13–A15 / 저장·실패 | 미완료 Continue는 입장전 거래·채용·사본·원장을 함께 되돌리고 완료 결과는 새 프로세스에서 유지한다. SavePoint와 무관한 영구기록/창고의 경계를 U15 현재 회귀와 대조했다. 실제 OS21 네 건은 모두 memory/disk 불변과 동일 유효요청 재시도 성공이다. |
| A16–A18 / 작성·통합·비용 | 실제 pack loader 정상1/작성오류12, U13 배정·U14 비용을 사용하는 지원활동, 현재 승인host/worker·codec·server consumer를 확인했다. Value 예약 수정은 typed decode·독립3단계 누적·checked overflow·기본16MiB를 유지한다. allocator 상계와 회수·동일 workload 비용 근거를 별도로 검산했다. |

`world/shop`, `grant_state`, `reward`, `headquarters`, `maintenance`, `event_node`, `combat`, `application`, `active_run/continuation`, `relic/allocation`, loader/validation 및 behavior/snapshot을 함께 검토했다. 생성·조회·명령·저장 소비자가 새 상품 세 종류를 동일하게 처리한다. 별도 자산권한, 시험명 전용 제품 분기, 보유칸 상한, 무조건 fallback, 임의 가격 재해석이나 자동 한도상향은 발견하지 않았다.

U15 independent001의 Complete/High는 그 과거 target의 판정으로 보존한다. 이번 typed 상품·공개결과·복원 계량 delta와 실제 U15/U08 소비 경로를 직접 검토했다. 그 밖의 변경 없는 자산/원장/paired store 구현은 현재 회귀와 비영향 근거로 재사용하며 선행 감사를 새 제품의 자동 승인으로 표기하지 않는다.

## 실행·재사용과 검토 질문

[원시 증거 검산](audit/independent001/EVIDENCE_RECHECK.json)은 각 coverage selector에 실제 성공 시험명이 있는지, 최초 실패의 정확 closure, 공개 요청/결과와 비용 raw를 확인했다.

- Core 고유2188은 library2108+교정6+integration72+새 확장회귀1+동일 source socket1의 결속 coverage다. 최종 집중28/복원44/공개15를 중복 합산하지 않았다. 기존 ignore6, 별도 native4/server31을 구분한다. 단일 all-targets 전체녹색 호출이 아니다.
- 정상host15프로세스84명령과 IO4건은 기존 실행을 재사용했다. 상점완료·HQ상점완료·SavePoint·Event완료에서 모두 실제 OS21→상태불변→재시도 성공이 확인된다. 미완료 구매의 writer 동일성 fast path를 저장실패 시험으로 세지 않는다.
- 정상/확장 공유비용6process72stage, 실제 확장Continue3process, Value30형태는 전부 원시값을 검산했다. 실제 요청 heap 회수와 native 예약/RSS를 구별한다. checkpoint171980bytes, 장비71개의 기존16MiB 복원이 성공하고 typed decode+Value+Value 누적 경계 회귀도 유효하다.
- 감사 단계만을 이유로 Rust 전체 suite·비용·host를 반복하지 않았다. 한 추가 공개 Core 진단만 새로 실행했다: 준비12명령→유물구매1→상태조회1, exit0. [결과](audit/independent001/FIRST_PURCHASE.json)·[빌드/실행 receipt](audit/independent001/PROBE_RECEIPT.json).

Q-U16-001은 첫 유물 구매의 native scope 진입 누락 가능성을 검토한 질문이며 제품 finding이 아니다. `GrantExecutor::grant_effects_with_run_state`가 relic-bearing effect를 감지해 `run_with_profile`을 호출하므로 누락 가설을 배제했다. 추가 진단에서는530→470의 정확 지불과 native scope1/누적27530bytes/16MiB 한도를 확인했다. 해당 pilot은 equipment_dust9를 이미 보유하므로 ‘모든 relic owner가 빈 상태’의 동적 시험이라고 주장하지 않는다. 빈 owner 분기의 판단은 실제 공유 executor의 조건식에 근거한다. [질문·판정](audit/independent001/FINDINGS.json).

감사 probe의 초기1024bytes profile은 loader의 generated combat preview에서 거절돼 구매까지 도달하지 못했다. 이를 구매 거절이나 제품 결함으로 세지 않았다. lock 누락 및 후속 빌드 완료 전 구 executable 실행도 감사 도구 준비 실패로 보존했다. [실패·교정 기록](audit/independent001/AUDIT_EXECUTION_NOTES.json). 표준 profile의 최종 실행만 성공 증거다.

## 한계·재현·다음 경계

공식 relic catalog는 비어 있어 유물 구매는 지원 외부 제작 pack으로 검증했다. 초기 가격60/150/30은 기록된 위임값이며 밸런스 완성 주장이 아니다. B-tree 노드 상계는 현재 Rust/serde_json backend에 의존한다. 비용은 유한 dev/승인 진단이며 배포기기 SLO가 아니다. 외부 임의 저장/checksum/메모리 변조 전용 방어는 추가 finding·필수시험에서 제외했으며 지원 작성오류·명령오용·실제 IO는 검토했다.

저장소 루트에서 검산을 재현한다. 출력은 존재하지 않는 새 파일이어야 한다.

```bash
python3 core_enhanced/docs/goals/core_support_node_activities/execution/single_goal/audit/independent001/recheck_evidence.py /tmp/u16-audit-recheck-new.json
python3 core_enhanced/docs/goals/core_support_node_activities/execution/single_goal/audit/independent001/currentness.py /tmp/u16-currentness-new.json
```

추가 공개 진단의 빌드와 argv는 PROBE_RECEIPT에 있으며 재현 시 결과 출력 경로를 새 경로로 바꾼다. 제품과 기존 증거는 읽기전용이다.

이 single-phase Goal은 별도 GEX-30 없이 이번 감사로 완료한다. 사용자 요청의 종료점은 U16-1 감사이며 U16-2 기본보상·U16-3 거래상인/미니게임·U16-4 특수전투·U17 부화·Unity는 이번 실행 범위 밖이다.
