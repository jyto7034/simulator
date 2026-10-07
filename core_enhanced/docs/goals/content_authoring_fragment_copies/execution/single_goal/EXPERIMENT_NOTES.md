# R07 현재 기술 판단과 위임 결정

사본 ID를 definition ID에 임의 대응시키는 편의 fallback은 허용하지 않는다. runtime actor UUID도 fragment identity를 대체하지 않는다. 장착 reverse binding이 실제 validation과 사망 회수에 참여해야 독점 소유가 보장된다.

신규 UUID namespace의 checked counter는 모든 admission/정적 참조 검증 뒤 후보에서 증가한다. 같은 사본의 form unlock은 지속 상태이며 장착/이전/사망에서 초기화하지 않는다. 개화와 선택을 분리하면 기본 형태로 되돌아갈 수 있고 자동 선택으로 compatibility를 우회하지 않는다.

V7 converter는 typed V8 decode로 checksum을 검사하지 않는다. old raw canonical bytes의 checksum과 old identity를 먼저 검사하고 새 shape로 명시 변환한 뒤 target generation 전체 validation을 적용한다. old rank는 새 gameplay 수치로 몰래 환산하지 않고 출처 history에 보존한다. 자동 원본 저장 덮어쓰기는 없다.

R05 자원 owner는 현재 Employee/EquipmentCopy만 존재한다. fragment source를 equipment로 위장하면 잘못된 lease 권한이 생기므로 최소 FragmentCopy/SourceFragment 확장이 적절하다. shared lifetime state machine을 복제하지 않고 기존 validation/binding/cleanup 경로를 확장한다.

기존 repeat reward에는 GrantFragmentDust가 본편 RON과 PVE validation에 명시되어 있다. 일반 강화 제거를 이유로 보상 identity·수치를 임의 변경하지 않는다. 신규 recipe 재료와 구분한 보존 처리를 구현 전에 확정하고 테스트한다. 이 점은 mutable shared rank ledger 보존을 허용하지 않는다.

## D07-IMPL01 — 기존 dust 보상 보존

위임 구현 결정: 기존 PVE repeat reward의 GrantFragmentDust와 수량은 유지하고, 잔액은 신형 inventory의 별도 보존 자산으로 남긴다. 일반 강화/개화가 이 잔액을 소비하거나 rank/해금으로 환산하는 경로는 제거한다. 기존 확정 보상5/10/20을 임의로 enkephalin/파츠로 바꾸는 대안은 밸런스 및 R19 보상 계약을 넓히므로 채택하지 않았다. DTO는 이 잔액의 신규 개화 사용 불가를 명시한다. legacy progress는 별도 compatibility 모듈에 읽기 전용 저장되며 gameplay resolver가 참조하지 않는다. 이후 별도 정책에서 보상 자산 전환을 승인하면 명시 migration으로 바꿀 수 있다. dust를 recipe의 기본 재료로 취급하지 않는다.

형태 compatibility는 definition 공통 요구에 추가되는 제약으로 결정했다. selected form은 base를 포함한 stable map key이며 base에 개화 recipe를 작성하면 거부한다. 명시 recipe의 재료를 정렬된 checked 합산으로 처리하고 최대64개 authored 항목/16형태/1024사본을 유한 host 상한으로 둔다. 이는 공식 밸런스가 아니다.

## 구현 교정 — DTO 게시 시점과 결정적 저장

Maintenance 진입 DTO는 후보 state에 session을 설치한 뒤 생성한다. 기존 장비 preview는 context 검사가 없어 드러나지 않았지만, 새 form preview와 command가 같은 eligibility를 사용하려면 이 순서가 필요하다. 외부 state commit 경계는 기존 application transaction 그대로다.

V8 checkpoint writer는 checksum뿐 아니라 전체 envelope도 canonical JSON으로 출력한다. explicit V7 converter의 반복 결과에서 내부 HashMap seed에 따른 byte 순서 차이가 드러나 채택했다. 의미상 동등 비교만 하는 대안보다 deterministic conversion과 재현에 유리하며 원본 V7 파일을 다시 쓰지 않는다.

기존 sweep의 synthetic BasicAttackModifier placeholder는 active 파편/recipe가 아니므로 분쇄 unavailable이 맞다. 해당 회귀는 이 거부와 exact copy 제거 후 정상 reacquisition selector를 검증하도록 전환했다. 실제 confirmed dismantle·산출 rollback은 새 active 사본 F05 통합시험이 담당한다. 후속으로 미루거나 placeholder에 가상 dust recipe를 넣지 않는다.

## 최종 관측·검증 해석

terminal battle은 live resource view가 아니라 pending result의 bounded fact 요약을 제공한다. 공개 R07 시험은 하나의 atomic Resources 효과에 Run/Attempt/Deployment/Activation/Delivery/Chain6개 Consume를 작성하고, 실제6 typed cost+SP cost의 mutation7과 exact Chain owner/source를 확인한다. Attempt/Deployment lease 및 위조 source 거부·withdrawal Snapshot 보존은 별도 실제 BattleCore 시험이며 Run owner 이전/해체는 World 시험이다. latest-only fact에6개 개별 이력을 보존하도록 제품을 바꾸는 대안은 기존 R06 관측 계약과 달라 채택하지 않았다.

새 source capsule은 Arc 생성 전 retained payload+owner overhead를 계량하고 RuntimeFactSource/RuntimeUnit/observation witness의 extent·CloneShape에 반영했다. 일반 다른 skill의 auto/manual accept가 장착 fragment로 잘못 귀속되지 않도록 실제 skill ID 일치를 확인한다. 공개 default loader를 사용한 RON 시험·새 subprocess·release CLI와 native boundary 시험을 함께 사용하며 private fixture 하나로 전체 공개 지원을 주장하지 않는다.

PC36의192개 final_facts 차이는 새 optional source의 None을 null로 노출한 직렬화 변화였다. 기존무파편 결과의 byte/shape 호환을 위해 RuntimeFactSource와 ApplicationWitness에서 None을 생략한다. 실제 사본의 Some 출처는 모두 직렬화된다. 비교기에 예외를 추가하는 대안보다 실제 원래 출력을 유지하는 교정을 채택했고, 교정 후 PC36 gameplay/state/error oracle 차이0을 확인했다. source가 바뀌었으므로 전체·feature·빌드 검사도002로 다시 실행했다.

최종 판단: 공개 describe의 authored_expression_publication을 실제 검증된 DamageExpr/HealExpr Snapshot/Live 지원 상태로 교정했다. 이전 staged 문구를 유지하는 대안은 실제 기능·공개 계약과 모순되어 폐기했다. 이 마지막 제품 변경도 003 전체 검증에 포함했고, 최종 동일 source의 비용/공개 CLI/수용표 및 격리 정역 복원으로 종료한다. 구현 hard blocker는 없고, R05 concurrent peak 단독 초과 경고와 공식 미작성 recipe의 unavailable 상태를 명시한다. 독립 감사는 별도 요청 대기다.
