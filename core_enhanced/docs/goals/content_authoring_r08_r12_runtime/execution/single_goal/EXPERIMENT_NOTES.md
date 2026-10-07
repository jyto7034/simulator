# 현재 기술 판단 — R08~R12 통합 인계

최신 상태: WP08~12 구현·자체 검증 자료를 동결해 독립 감사 요청 대기다. 54조건 자체 검증/X09 환경 제한1이며 무조건 전체 PASS는 아니다. 아래 WP09/WP10 메모는 당시 기록이다. 최종 source별 판정과 환경 한계는 COVERAGE/REVIEW_HANDOFF를 따른다.

선행 R06/R07 독립 감사 Complete/High 후 종료했고 정식 P2G40/P2G50 계획 검증도 통과했다. 현재 R08~R12 구현자다. R08 focused8개 중간 통과이며 최종 통합 판정은 아니다. WP09는 runtime/World/durable 경로 구현 및 검증 중, WP10~12/INT 미착수.

R09 공통 gate는 manual 접수 시 event를 예약하고 실제 unpaused 처리에서 자격/target을 재확인한다. auto도 같은 침식 함수로 수렴한다. 선택된 exact form/skill과 authored 제외 class를 사용한다. live 최종 보스3종+reference fragment는 명시 제외. trauma80/1000ms/2node는 위임 잠정 RON 값이지 사용자 확정 밸런스가 아니다.

처음 release(0)→cost/death는 action receipt가 조기 settle되는 결함을 실제 시험으로 검출했다. cost/death 기록→release(0)로 교정하여 전체/action 대가 모두100이 확인됐다. stage별 실제 token cancellation4회는 rollback/retry 중간 통과. host freeze는 raw elapsed만 소비하고 expiry 초과분을 버린다. 중단된 timestamp를 계속 처리할 CorrosionBoundary와 잔여 event 원형 requeue로 wave/boss bookkeeping을 이어간다.

BreakdownCounter는 직원 durable enum이며 remaining_nodes와 entered_node를 가진다. 진입 node는 차감하지 않고 회복 먼저/이후 완료만 차감한다. 만료는 combat incapacitation과 별개 사망 후 기존 R07 exact copy/equipment 회수. 실제 CompleteNode에서 DeathPrevent를 우회하며 asset 회수/중복 거부 중간 통과. V9 counter는 필수 입력; V7/V8은 original checksum/identity 확인 후 새 bytes 반환하는 명시 변환. V8 legacy-shape 시험은 생성 fixture이며 역사적 원본 bytes라고 주장하지 않는다. 원래 source와 V7원본은 별도 보존했다.

실패 이력과 각 source/log는 EXPERIMENTS와 evidence receipts 참조. World 첫 두 시험은 fixture의 self selection/global reach/self-hostile damage 부적격으로 거부되었으며 정상 self-heal 입력으로 교정 중이다. readiness/강제 pause/최종 result/save/Fresh-process/회복 우선/미출현 wave/선택 death trigger 억제의 전체 수용 증거는 아직 미완료. 55조건 PASS·동결·독립 감사 PASS를 발행하지 않는다.

## WP10 진행 메모

r10_tests_001에서 legacy도 compiled flow로 내려간 source의 R08 8개 시험이 통과했다. 아직 structured 공개 입력의 수용 증거는 없다. r10_tests_002는 fixture가 FactSummary의 실제 healing_delta 대신 없는 필드를 참조하여 compile 실패했으며 수용 통과로 세지 않는다. 새 scheduler의 native retained/clone과 작성 entity admission, checked schedule horizon, structured 취소와 released child 보존, authored 주소와 occurrence 출처를 함께 연결 중이다. raw enum만 추가한 상태를 완료로 보고하지 않는다. R09 남은 World fixture라는 이전 메모는 r09_tests_010의13개 통과로 대체되며, 최종 source 재실행과 환경 EPERM 한 건은 여전히 남는다.


## 통합 단계의 확정 계약

R10 structured flow는 action마다 결과 occurrence 주소를 분리하고 authored 순서의 유한 continuation으로 실행한다. R08 task는 opaque action handle을 보유하고 Released/EffectsSettled를 구별한다. state exit와 timeout의 CancelUnreleased/Detach는 이미 방출한 payload source를 취소하지 않는다. 공개 보스 팩의 실제 projectile→area 뒤 후속 marker action 시각으로 milestone 전이를 확인했다.

R11의 modifier/channel/overlay는 battle-local source owner에 붙는다. source 제거 시 기준값과 남은 source를 다시 합성하며 영구 base를 나눗셈으로 되돌리지 않는다. channel은 실제 주기 공격·새 시전·태세·이동/교대·작업 칸 진입을 막고 명시 중지를 허용한다. 피격/Silence와 hard CC/interrupt/철수/사망을 구별한다. 이미 발사된 강화 탄환은 aura 만료 뒤에도 저장한 피해를 유지한다. Hello 재입력은 공개 activation 검증 후 committed remaining time을 별도 Goodbye source에 넘긴다. 새로운 공명 지불이나 기존 공명 부족 검사로 대체하지 않는다.

R12 shared pool은 activation당 잔량 하나이고 membership은 참조다. 감쇄 후 개인→공유, priority/creation 순서의 소비와 실제 HP loss를 분리한다. source scale은 생성 때 한 번, 재시전은 새 activation, 과거 expiry는 옛 owner에만 적용한다. Heal/Rest는 pool을 건드리지 않고 recharge만 현존 capacity까지 바꾼다. HP 비용은 별도 CostFact이며 leave-one 기본/명시 lethal opt-in을 사용한다. FloorOne과 일회 death prevention은 ordinary damage 중재이고 침식의 mandatory death를 막지 않는다.

Parry는 첫 직접 적 strike와 명시 결속된 attachment/basic OnHit를 억제한다. 성공 후 무적은 추가 counter/반사가 없으며 miss는 자기 경감만 half-open 구간에서 해제한다. counter는 원래 공격자를 대상으로 한 별도 즉시 damage leaf이고 실패 시 enclosing transaction을 복원한다. Black-swan passive는 실제 장착 사본에서 파생한 좁은 ActiveSkill metadata 예외이며 별도 passive slot/영속 record/유물 축적을 만들지 않는다. stable seed+target+strike namespace의 한번 추첨과 actual HP loss 기반 반사는 typed policy/outcome/provider를 보유한다.

BasicLifesteal은 실제 weapon OnAttack의 release provider를 보존한다. area/관통의 actual HP loss를 strike 단위로 합산한 뒤 내림과 cap을 각각 한번 적용한다. absorbed/overkill/Goodbye/반사는 포함하지 않는다. Physical/Magic/True의 다른 장비 UUID들에 같은 경로를 적용했다.

이 기능 primitive의 구현은 공식 콘텐츠 전체 저작·밸런스 확정을 뜻하지 않는다. trauma80/freeze1000ms/Breakdown2node와 검증용 pool·counter·흡혈 수치는 위임된 실행 정책 또는 시험값이며 사용자의 과거 확정값으로 소급하지 않는다. 현재 계약은 전투 attempt와 node 완료를 기준으로 하며 미래 원정/임대/귀환 설계는 이번에 구현하지 않았다.

## 최종 증거의 해석

source/data/test를 바꾸는 동안 validation을 병행하지 않는다. 전체 원명령의 Unix socket 생성 EPERM 실패를 숨기지 않으며 명시 제외 재실행을 별도 집계한다. 기본/feature의 실제 selector 결과, public CLI의 trace/amount/public result 동등성, 원본 PC36/R05 workload, currentness와 byte 단위 격리 정역 복원을 각각 인계한다. R05 peak 단독 경고 외 예산/retained/allocation/결정론/rollback guard를 완화하지 않는다. 독립 감사는 별도 GEX-20 요청 대기이며 구현자가 PASS를 발행하지 않는다.

## 최종 비용·호환 판단

source `bd89f9c42191217a333d648d35f495e802d13e6fb1420caa9c08da3deb731690`에서 일반 피해의 빈 보호 상세와 flow/reinput 출처를 희소 저장한다. 보호 상세의 Box와 출처 Arc를 만들기 전에 native admission을 수행하고 실제 retained/CloneShape 계산에도 포함한다. 일반 피해 wire shape는 기존대로이고 실제 보호·reaction에는 명시 receipt를 제공한다.

EventLog32는 신규 event variant를 구분하는 공개 형식이다. 엔진 버전을 되돌리거나 workload를 줄이지 않고, 동결 R07의 실제65로그를 byte 대조한 뒤 envelope31→32만 이행한 성능 baseline을 별도로 만든다. 모든 원 cap·sample·입력과 official R05 runner source는 유지했다. PC의 raw v32 hash와 v31 envelope hash 둘 다 보존한다. 비용 교정 이전의 실패는 성능 통과로 재분류하지 않는다.

현재 한계는 환경 socket1건과 concurrent peak 단독 경고(+342658 bytes)이며, latency는 관찰이다. 독립 GEX-20과 R06/R07 공유 경계 영향 감사는 후속 요청 대상이다. 최종 근거는 REVIEW_HANDOFF/COVERAGE/VALIDATION_CURRENTNESS다.
