# 공통 계약과 구현 owner

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

계약명은 목표 구조의 논리 식별자다. 동일한 Rust module/type 이름이 이미 존재한다는 뜻이 아니다. 최소 선언과 실제 capability를 구별하며 새 기능을 빈 descriptor/executor로 선등록하지 않는다.

## K01 — 정의/효과/사본/시전/타격/segment/전달 identity·SourceBinding

초기 owner: [R01](goals/R01.ko.md). 확장 owner: [R04](goals/R04.ko.md), [R07](goals/R07.ko.md), [R14](goals/R14.ko.md).

정의 주소만 먼저 고정한다. 영구 사본, 전투 actor, cast, 직접 strike, damage segment는 별도 key다.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S027](../evidence/SOURCES.ko.md#s027) · [S031](../evidence/SOURCES.ko.md#s031) · [S064](../evidence/SOURCES.ko.md#s064).

## K02 — typed authoring·descriptor·SourceMap·compiled reference

초기 owner: [R01](goals/R01.ko.md). 확장 owner: [R02](goals/R02.ko.md), [R03](goals/R03.ko.md), [R06](goals/R06.ko.md), [R10](goals/R10.ko.md), [R14](goals/R14.ko.md), [R15](goals/R15.ko.md).

지원 opcode만 공개한다. 외부 pack 원문 소유권/위치는 R02에서 확장하고 AST 내부 경로를 public property로 쓰지 않는다.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S081](../evidence/SOURCES.ko.md#s081).

## K03 — ContentGeneration pin·fingerprint·저장 일치

초기 owner: [R02](goals/R02.ko.md). 확장 owner: [R07](goals/R07.ko.md), [R14](goals/R14.ko.md), [R19](goals/R19.ko.md), [R20](goals/R20.ko.md).

새 런 briefing부터 고정, 기존 런 세대 보존. checksum과 content fingerprint는 구별한다.

근거: [S015](../evidence/SOURCES.ko.md#s015) · [S037](../evidence/SOURCES.ko.md#s037) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S085](../evidence/SOURCES.ko.md#s085).

## K04 — ActionHandle·milestone·terminal result·cancel

초기 owner: [R04](goals/R04.ko.md). 확장 owner: [R08](goals/R08.ko.md), [R10](goals/R10.ko.md), [R11](goals/R11.ko.md), [R18](goals/R18.ko.md).

Accepted는 완료가 아니다. 현재 carrier부터 port를 만들고 미방출 취소/방출 후 존속을 구별한다.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S048](../evidence/SOURCES.ko.md#s048).

## K05 — StateOwner×Lifetime·ResourceTxn·lease

초기 owner: [R05](goals/R05.ko.md). 확장 owner: [R07](goals/R07.ko.md), [R09](goals/R09.ko.md), [R11](goals/R11.ko.md), [R12](goals/R12.ko.md), [R13](goals/R13.ko.md), [R14](goals/R14.ko.md), [R16](goals/R16.ko.md), [R17](goals/R17.ko.md), [R20](goals/R20.ko.md).

owner와 수명은 직교. 새 Run 상태는 도입 단계에서 save/entry/result까지 소유한다.

근거: [S027](../evidence/SOURCES.ko.md#s027) · [S034](../evidence/SOURCES.ko.md#s034) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S044](../evidence/SOURCES.ko.md#s044) · [S082](../evidence/SOURCES.ko.md#s082).

## K06 — EventEnvelope·Damage/Heal/Death/CostReceipt·원자성

초기 owner: [R04](goals/R04.ko.md). 확장 owner: [R09](goals/R09.ko.md), [R12](goals/R12.ko.md), [R16](goals/R16.ko.md), [R18](goals/R18.ko.md), [R20](goals/R20.ko.md).

요청과 확정 사실, 기본공격 트리거와 일반 damage, optional proc와 필수 death/wave를 구별한다.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S028](../evidence/SOURCES.ko.md#s028) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072).

## K07 — EffectSelector·PatchPlan·활성/휴면·write conflict

초기 owner: [R14](goals/R14.ko.md). 확장 owner: [R07](goals/R07.ko.md), [R15](goals/R15.ko.md), [R17](goals/R17.ko.md), [R19](goals/R19.ko.md).

최소 numeric 실행을 먼저, 유한 grant/구조 recipe를 다음에 연결한다. 전용 identity AND 같은 effect의 태그 검사.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S045](../evidence/SOURCES.ko.md#s045) · [S076](../evidence/SOURCES.ko.md#s076) · [S083](../evidence/SOURCES.ko.md#s083).

## K08 — ChainContext·fork/join·root 생산자·budget

초기 owner: [R16](goals/R16.ko.md). 확장 owner: [R04](goals/R04.ko.md), [R10](goals/R10.ko.md), [R13](goals/R13.ko.md), [R15](goals/R15.ko.md), [R17](goals/R17.ko.md), [R23](goals/R23.ko.md).

R04는 인과를 잃지 않을 최소 source/occurrence 포트, R16은 실제 branch-local 재방문/지연 전파를 소유한다.

근거: [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082).

## K09 — ScenarioSignal·actor binding·생성/진행 권한

초기 owner: [R08](goals/R08.ko.md). 확장 owner: [R13](goals/R13.ko.md), [R18](goals/R18.ko.md), [R19](goals/R19.ko.md).

기본 보스 요청은 R08, 객체 adapter는 R13, field-work 신호·다중 actor는 R18. Wave/보상 authority는 보스 내부가 아니다.

근거: [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S084](../evidence/SOURCES.ko.md#s084).

## K10 — 진단·TraceReceipt·실제 실행 증거

초기 owner: [R01](goals/R01.ko.md). 확장 owner: [R03](goals/R03.ko.md), [R04](goals/R04.ko.md), [R08](goals/R08.ko.md), [R14](goals/R14.ko.md), [R18](goals/R18.ko.md), [R21](goals/R21.ko.md), [R22](goals/R22.ko.md), [R23](goals/R23.ko.md), [R24](goals/R24.ko.md).

초기 최소 진단→공개 도구→각 기능의 source trace. 진단 on/off와 gameplay authority는 별개다.

근거: [S013](../evidence/SOURCES.ko.md#s013) · [S017](../evidence/SOURCES.ko.md#s017) · [S068](../evidence/SOURCES.ko.md#s068) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S085](../evidence/SOURCES.ko.md#s085).

## 새 상태의 저장 책임

Run 귀속 state는 도입 owner가 최신 payload/validation/entry/result를 동반한다. Attempt/Deployment/Activation/Delivery/Chain state는 해당 종료·철수·retry/rollback·stale event·live reconnect를 소유한다. compiled index·Arc 주소를 durable identity로 쓰지 않는다. 소유자와 lifetime은 직교한다.

## 공개 작성 지원의 일치 기준

`runtime-supported`라고 공개한 op/property마다 실제 parser·validator·compiler·executor·descriptor·positive/negative 시험이 있어야 한다. 후속 계획 vocabulary와 현재 지원 목록은 별도로 표시한다. production 사용자가 문자열 내부 필드 path나 실행 가능한 코드를 제출하는 API를 만들지 않는다.
