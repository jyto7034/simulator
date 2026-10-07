# M0–M8 / CA00–CA23 전체 대응

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

**원래 M·CA는 누락 방지 추적 ID로 보존하고, 새 R은 구현·검증 작업의 권고 분해다.** CA01 전체 로드맵 범위와 지금 작성된 최소 GOAL-CAMEC-01의 범위는 같지 않다. CA01의 미래 확장을 다른 R에 배정해도 기존 Goal의 범위를 확대하는 것은 아니다.

| 기존 milestone | 기존 CA | 새 대응 단위 |
| --- | --- | --- |
| M0 | CA00, CA01 | [R00](../roadmap_v2.2-review/goals/R00.ko.md), [R01](../roadmap_v2.2-review/goals/R01.ko.md), [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md) |
| M1 | CA02, CA03 | [R01](../roadmap_v2.2-review/goals/R01.ko.md), [R02](../roadmap_v2.2-review/goals/R02.ko.md), [R03](../roadmap_v2.2-review/goals/R03.ko.md) |
| M2 | CA04, CA05, CA06, CA07 | [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R05](../roadmap_v2.2-review/goals/R05.ko.md), [R06](../roadmap_v2.2-review/goals/R06.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md) |
| M3 | CA08, CA09, CA10 | [R06](../roadmap_v2.2-review/goals/R06.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R11](../roadmap_v2.2-review/goals/R11.ko.md), [R12](../roadmap_v2.2-review/goals/R12.ko.md) |
| M4 | CA11, CA12, CA13 | [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R15](../roadmap_v2.2-review/goals/R15.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md), [R17](../roadmap_v2.2-review/goals/R17.ko.md) |
| M5 | CA14, CA15, CA16 | [R08](../roadmap_v2.2-review/goals/R08.ko.md), [R13](../roadmap_v2.2-review/goals/R13.ko.md), [R17](../roadmap_v2.2-review/goals/R17.ko.md), [R18](../roadmap_v2.2-review/goals/R18.ko.md) |
| M6 | CA17, CA18, CA19 | [R02](../roadmap_v2.2-review/goals/R02.ko.md), [R05](../roadmap_v2.2-review/goals/R05.ko.md), [R06](../roadmap_v2.2-review/goals/R06.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R09](../roadmap_v2.2-review/goals/R09.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R19](../roadmap_v2.2-review/goals/R19.ko.md), [R20](../roadmap_v2.2-review/goals/R20.ko.md) |
| M7 | CA20, CA21 | [R21](../roadmap_v2.2-review/goals/R21.ko.md), [R22](../roadmap_v2.2-review/goals/R22.ko.md) |
| M8 | CA22, CA23 | [R02](../roadmap_v2.2-review/goals/R02.ko.md), [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R05](../roadmap_v2.2-review/goals/R05.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md), [R23](../roadmap_v2.2-review/goals/R23.ko.md), [R24](../roadmap_v2.2-review/goals/R24.ko.md) |

## CA00 — 기준 상태·전체 요구사항·정책 추적표

**직접 대응:** [R00](../roadmap_v2.2-review/goals/R00.ko.md). **동반·확장 owner:** 없음.

CA00 요구/COV 조사 성과를 인수한다. 새 정책 결정·전수 게임 실행 증거로 승격하지 않는다.

**현재 확인 상태:** 기존 조사/정식 설계 성과 인수, 게임 미구현.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S001](../evidence/SOURCES.ko.md#s001) · [S002](../evidence/SOURCES.ko.md#s002) · [S005](../evidence/SOURCES.ko.md#s005) · [S006](../evidence/SOURCES.ko.md#s006) · [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S078](../evidence/SOURCES.ko.md#s078) · [S088](../evidence/SOURCES.ko.md#s088) · [S089](../evidence/SOURCES.ko.md#s089) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093).

## CA01 — 작성 단위·타입·실행 지원 계약

**직접 대응:** [R00](../roadmap_v2.2-review/goals/R00.ko.md), [R01](../roadmap_v2.2-review/goals/R01.ko.md), [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md). **동반·확장 owner:** 없음.

최소 formal Goal은 유지. 확장된 handle/Flow/state 의미는 후속 R로 분리한다.

**현재 확인 상태:** 기존 조사/정식 설계 성과 인수, 게임 미구현.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S001](../evidence/SOURCES.ko.md#s001) · [S002](../evidence/SOURCES.ko.md#s002) · [S005](../evidence/SOURCES.ko.md#s005) · [S006](../evidence/SOURCES.ko.md#s006) · [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S064](../evidence/SOURCES.ko.md#s064) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S088](../evidence/SOURCES.ko.md#s088) · [S089](../evidence/SOURCES.ko.md#s089) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

## CA02 — 최신 schema 전환·외부 다중 팩 게시

**직접 대응:** [R01](../roadmap_v2.2-review/goals/R01.ko.md), [R02](../roadmap_v2.2-review/goals/R02.ko.md). **동반·확장 owner:** 없음.

최소 schema와 catalog의 embedded 전환은 R01, 외부 입력/세대/저장은 R02가 소유한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S015](../evidence/SOURCES.ko.md#s015) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S024](../evidence/SOURCES.ko.md#s024) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S079](../evidence/SOURCES.ko.md#s079) · [S085](../evidence/SOURCES.ko.md#s085) · [S091](../evidence/SOURCES.ko.md#s091) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095) · [S097](../evidence/SOURCES.ko.md#s097).

## CA03 — 공통 검증·설명·headless 시험 실행 포트

**직접 대응:** [R03](../roadmap_v2.2-review/goals/R03.ko.md). **동반·확장 owner:** 없음.

공개 제작 포트는 먼저 권장하되 kernel 기능 선행으로 과장하지 않는다. 각 새 opcode의 descriptor/negative test는 해당 owner가 동반한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S017](../evidence/SOURCES.ko.md#s017) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S085](../evidence/SOURCES.ko.md#s085).

## CA04 — 자원·상태 소유와 수명 kernel

**직접 대응:** [R05](../roadmap_v2.2-review/goals/R05.ko.md). **동반·확장 owner:** [R04](../roadmap_v2.2-review/goals/R04.ko.md).

현재 owner/source 일부를 R04에서 정하고 자원/store/lifetime은 R05로 분리한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S082](../evidence/SOURCES.ko.md#s082) · [S091](../evidence/SOURCES.ko.md#s091).

## CA05 — 계산식·집계·스탯 성장

**직접 대응:** [R06](../roadmap_v2.2-review/goals/R06.ko.md). **동반·확장 owner:** 없음.

R06에서 계산·집계·무기 성장. 파편 일반 rank 제거는 R07. 전투력 최종 모델은 R19.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S034](../evidence/SOURCES.ko.md#s034) · [S043](../evidence/SOURCES.ko.md#s043) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095).

## CA06 — 의미 있는 전투 사건·피해 해결 결과

**직접 대응:** [R04](../roadmap_v2.2-review/goals/R04.ko.md). **동반·확장 owner:** [R09](../roadmap_v2.2-review/goals/R09.ko.md), [R12](../roadmap_v2.2-review/goals/R12.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md).

R04에서 사건/결과/출처를 앞으로 이동하고 기존 basic trigger 의미·죽음 진행 처리를 보존한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082).

## CA07 — 대상·전달·단계·분기·반복 실행

**직접 대응:** [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md). **동반·확장 owner:** [R08](../roadmap_v2.2-review/goals/R08.ko.md).

현재 전달체의 handle은 R04, 일반 target/carrier/Flow는 R10. 보스 최소 adapter는 R08.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S097](../evidence/SOURCES.ko.md#s097).

## CA08 — 가역 modifier·오라·집중·기본 공격 교체

**직접 대응:** [R11](../roadmap_v2.2-review/goals/R11.ko.md). **동반·확장 owner:** [R18](../roadmap_v2.2-review/goals/R18.ko.md).

R11: source별 ledger/lease/membership·기본 공격 override·재입력/종료. 현재 직접 modifier 변경을 가역이라고 부르지 않는다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027) · [S029](../evidence/SOURCES.ko.md#s029) · [S030](../evidence/SOURCES.ko.md#s030) · [S043](../evidence/SOURCES.ko.md#s043) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

## CA09 — 공유 보호막·생존·방어 중재·제어

**직접 대응:** [R12](../roadmap_v2.2-review/goals/R12.ko.md). **동반·확장 owner:** [R13](../roadmap_v2.2-review/goals/R13.ko.md).

R12: pool/HP/제어/생존 중재. R08 보스 호출과 R09 침식의 전역 선행으로 두지 않는다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S028](../evidence/SOURCES.ko.md#s028) · [S029](../evidence/SOURCES.ko.md#s029) · [S043](../evidence/SOURCES.ko.md#s043) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

## CA10 — 파편 사본·개화·형태와 무기 성장 연결

**직접 대응:** [R06](../roadmap_v2.2-review/goals/R06.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md). **동반·확장 owner:** 없음.

R07 사본화와 모든 장착/지급/저장; 무기 성장 입력은 R06. 유물 aggregate와 필수 합병하지 않는다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S043](../evidence/SOURCES.ko.md#s043) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S075](../evidence/SOURCES.ko.md#s075) · [S091](../evidence/SOURCES.ko.md#s091) · [S095](../evidence/SOURCES.ko.md#s095).

## CA11 — 유물 소유·장착·회수·분쇄

**직접 대응:** [R14](../roadmap_v2.2-review/goals/R14.ko.md). **동반·확장 owner:** 없음.

관리 도메인은 독립성을 유지한다. 기본안은 R14에서 최소 수치 유물 실행까지 같은 작업 단위로 인수한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S045](../evidence/SOURCES.ko.md#s045) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091).

## CA12 — 효과 선택·변형·유효 효과 구성

**직접 대응:** [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R15](../roadmap_v2.2-review/goals/R15.ko.md), [R17](../roadmap_v2.2-review/goals/R17.ko.md). **동반·확장 owner:** [R17](../roadmap_v2.2-review/goals/R17.ko.md).

R14 numeric 실행 연결→R15 grant/구조 합성→R17 관계 특화 recipe. 선택 효과와 carrier를 구별한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S029](../evidence/SOURCES.ko.md#s029) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S045](../evidence/SOURCES.ko.md#s045) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S076](../evidence/SOURCES.ko.md#s076) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S087](../evidence/SOURCES.ko.md#s087) · [S091](../evidence/SOURCES.ko.md#s091).

## CA13 — 연쇄 실행·경로별 재방문 차단

**직접 대응:** [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md). **동반·확장 owner:** [R04](../roadmap_v2.2-review/goals/R04.ko.md).

R16: 즉시·지연 모두 branch-local chain. R09 suppression은 R16의 선행이 아니라 도입 순서에 따른 회귀 인수.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S045](../evidence/SOURCES.ko.md#s045) · [S064](../evidence/SOURCES.ko.md#s064) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083).

## CA14 — 소환·설치물·전장 객체

**직접 대응:** [R13](../roadmap_v2.2-review/goals/R13.ko.md). **동반·확장 owner:** [R18](../roadmap_v2.2-review/goals/R18.ko.md).

R13: 기존 SpawnKernel 재사용, boss/opponent 상위 결속 해제. actor/trap/fieldobject의 capability를 분리한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S022](../evidence/SOURCES.ko.md#s022) · [S047](../evidence/SOURCES.ko.md#s047) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

## CA15 — 관계·감염·공간 집계

**직접 대응:** [R17](../roadmap_v2.2-review/goals/R17.ko.md). **동반·확장 owner:** 없음.

R17: source별 관계/감염/타일 집계. 정밀 line-crossing 알고리즘은 이번 source 상세 미검토로 남긴다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S021](../evidence/SOURCES.ko.md#s021) · [S025](../evidence/SOURCES.ko.md#s025) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S054](../evidence/SOURCES.ko.md#s054) · [S058](../evidence/SOURCES.ko.md#s058) · [S076](../evidence/SOURCES.ko.md#s076) · [S082](../evidence/SOURCES.ko.md#s082) · [S087](../evidence/SOURCES.ko.md#s087).

## CA16 — 보스 제작·실행 확장

**직접 대응:** [R08](../roadmap_v2.2-review/goals/R08.ko.md), [R18](../roadmap_v2.2-review/goals/R18.ko.md). **동반·확장 owner:** [R19](../roadmap_v2.2-review/goals/R19.ko.md).

R08에서 최소 호출을 조기 분리하고 R18에서 형태·signal·selector·다중 actor를 확장한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S058](../evidence/SOURCES.ko.md#s058) · [S084](../evidence/SOURCES.ko.md#s084) · [S094](../evidence/SOURCES.ko.md#s094) · [S096](../evidence/SOURCES.ko.md#s096).

## CA17 — 등록·획득·평가 연결

**직접 대응:** [R06](../roadmap_v2.2-review/goals/R06.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R19](../roadmap_v2.2-review/goals/R19.ko.md). **동반·확장 owner:** [R03](../roadmap_v2.2-review/goals/R03.ko.md).

R07/R14가 최초 정상 획득/장착, R06이 평가 입력, R19가 복수 선택/등록/평가 모델 통합을 닫는다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S028](../evidence/SOURCES.ko.md#s028) · [S030](../evidence/SOURCES.ko.md#s030) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S043](../evidence/SOURCES.ko.md#s043) · [S045](../evidence/SOURCES.ko.md#s045) · [S054](../evidence/SOURCES.ko.md#s054) · [S055](../evidence/SOURCES.ko.md#s055) · [S056](../evidence/SOURCES.ko.md#s056) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S083](../evidence/SOURCES.ko.md#s083) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094) · [S095](../evidence/SOURCES.ko.md#s095).

## CA18 — 침식

**직접 대응:** [R09](../roadmap_v2.2-review/goals/R09.ko.md). **동반·확장 owner:** 없음.

R09으로 조기 이동. 현재 적 kill·optional 억제·직원 대가·강제 clock·Breakdown을 함께 소유한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S075](../evidence/SOURCES.ko.md#s075) · [S077](../evidence/SOURCES.ko.md#s077) · [S091](../evidence/SOURCES.ko.md#s091).

## CA19 — 저장·결과·수명 통합

**직접 대응:** [R02](../roadmap_v2.2-review/goals/R02.ko.md), [R05](../roadmap_v2.2-review/goals/R05.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R09](../roadmap_v2.2-review/goals/R09.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R20](../roadmap_v2.2-review/goals/R20.ko.md). **동반·확장 owner:** [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R06](../roadmap_v2.2-review/goals/R06.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md), [R11](../roadmap_v2.2-review/goals/R11.ko.md), [R12](../roadmap_v2.2-review/goals/R12.ko.md), [R13](../roadmap_v2.2-review/goals/R13.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md), [R17](../roadmap_v2.2-review/goals/R17.ko.md), [R18](../roadmap_v2.2-review/goals/R18.ko.md), [R19](../roadmap_v2.2-review/goals/R19.ko.md).

R02/R05/R07/R09/R14 및 새 persistent-state owner에 저장을 분산하고 R20은 복합 검증이다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S015](../evidence/SOURCES.ko.md#s015) · [S018](../evidence/SOURCES.ko.md#s018) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S027](../evidence/SOURCES.ko.md#s027) · [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S044](../evidence/SOURCES.ko.md#s044) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S071](../evidence/SOURCES.ko.md#s071) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S077](../evidence/SOURCES.ko.md#s077) · [S082](../evidence/SOURCES.ko.md#s082) · [S083](../evidence/SOURCES.ko.md#s083) · [S085](../evidence/SOURCES.ko.md#s085) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094).

## CA20 — 대표 공식 RON 및 표현력 검증

**직접 대응:** [R21](../roadmap_v2.2-review/goals/R21.ko.md). **동반·확장 owner:** 없음.

R21에서 C01–C07 실제 최신 RON과 전체 COV 표현력 disposition을 별도 인수한다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S045](../evidence/SOURCES.ko.md#s045) · [S046](../evidence/SOURCES.ko.md#s046) · [S054](../evidence/SOURCES.ko.md#s054) · [S056](../evidence/SOURCES.ko.md#s056) · [S058](../evidence/SOURCES.ko.md#s058) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S078](../evidence/SOURCES.ko.md#s078) · [S079](../evidence/SOURCES.ko.md#s079) · [S087](../evidence/SOURCES.ko.md#s087) · [S096](../evidence/SOURCES.ko.md#s096).

## CA21 — 독립 제작 인수

**직접 대응:** [R22](../roadmap_v2.2-review/goals/R22.ko.md). **동반·확장 owner:** 없음.

R22는 독립 작성자와 frozen-engine data-only 증거가 필요하다. 이번 연구에서 수행하지 않았다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058) · [S084](../evidence/SOURCES.ko.md#s084) · [S085](../evidence/SOURCES.ko.md#s085).

## CA22 — 안전·결정론·성능

**직접 대응:** [R02](../roadmap_v2.2-review/goals/R02.ko.md), [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R05](../roadmap_v2.2-review/goals/R05.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md), [R23](../roadmap_v2.2-review/goals/R23.ko.md). **동반·확장 owner:** [R03](../roadmap_v2.2-review/goals/R03.ko.md), [R06](../roadmap_v2.2-review/goals/R06.ko.md), [R08](../roadmap_v2.2-review/goals/R08.ko.md), [R09](../roadmap_v2.2-review/goals/R09.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md), [R11](../roadmap_v2.2-review/goals/R11.ko.md), [R12](../roadmap_v2.2-review/goals/R12.ko.md), [R13](../roadmap_v2.2-review/goals/R13.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R15](../roadmap_v2.2-review/goals/R15.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md), [R17](../roadmap_v2.2-review/goals/R17.ko.md), [R18](../roadmap_v2.2-review/goals/R18.ko.md).

각 기능 도입부터 예산/fault/결정론 관찰, R23에서 복합 측정/회귀를 닫는다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S015](../evidence/SOURCES.ko.md#s015) · [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S022](../evidence/SOURCES.ko.md#s022) · [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S027](../evidence/SOURCES.ko.md#s027) · [S028](../evidence/SOURCES.ko.md#s028) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S048](../evidence/SOURCES.ko.md#s048) · [S049](../evidence/SOURCES.ko.md#s049) · [S057](../evidence/SOURCES.ko.md#s057) · [S064](../evidence/SOURCES.ko.md#s064) · [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S079](../evidence/SOURCES.ko.md#s079) · [S081](../evidence/SOURCES.ko.md#s081) · [S082](../evidence/SOURCES.ko.md#s082) · [S085](../evidence/SOURCES.ko.md#s085) · [S091](../evidence/SOURCES.ko.md#s091) · [S094](../evidence/SOURCES.ko.md#s094) · [S097](../evidence/SOURCES.ko.md#s097).

## CA23 — 최종 감사·인계

**직접 대응:** [R24](../roadmap_v2.2-review/goals/R24.ko.md). **동반·확장 owner:** 없음.

R24의 독립 감사는 미래 작업이다. 이번 문서 정합성 검사는 대체물이 아니다.

**현재 확인 상태:** 변경 경계 정적 조사; 향후 구현·게임 검증 미실행.

**제한:** 소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조

근거: [S006](../evidence/SOURCES.ko.md#s006) · [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058) · [S088](../evidence/SOURCES.ko.md#s088) · [S089](../evidence/SOURCES.ko.md#s089).

## COV 기준선 보존 규칙

기존 `records/CA00_NEXT-0002_COVERAGE.json`의 COV/BOSS-COV ID·source·line·status·owner_ca·policy_gates·actual_validation_receipt는 원본으로 보존한다. 본 패키지는 그 행들을 재작성하거나 완료 상태를 덧씌우지 않는다. 각 행의 owner_ca는 위 대응표를 통해 하나 이상의 새 R로 이어진다. 하나의 행이 여러 R을 요구하면 마지막 R 하나에만 몰지 않고 primitive 구현 owner와 통합 검증 owner를 구별한다.

기존 기록의 354 COV/127 BOSS-COV를 이번에 새로 모두 재계수·재검증한 것은 아니다. 원문 공백·도구형 별도 범위·미정 수치가 있는 행은 그대로 남는다. 원문 해시가 달라지면 바뀐 행을 다시 읽고 영향 대응만 갱신해야 한다. 관련 source: [S005](../evidence/SOURCES.ko.md#s005) · [S006](../evidence/SOURCES.ko.md#s006) · [S078](../evidence/SOURCES.ko.md#s078) · [S075](../evidence/SOURCES.ko.md#s075) · [S076](../evidence/SOURCES.ko.md#s076) · [S096](../evidence/SOURCES.ko.md#s096).
