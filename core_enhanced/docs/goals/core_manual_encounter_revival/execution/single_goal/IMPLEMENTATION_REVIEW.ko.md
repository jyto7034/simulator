# U17 구현 자체 검토

GS-CMER-001 / GOAL-CMER-01 / PHASE-CMER-01. 구현자가 실제 코드와 원시 결과를 검토한 기록이며
GEX-20 감사나 Goal Complete 판정이 아니다. 현재 결과는 [CURRENT_RESULTS](CURRENT_RESULTS.json),
22수용·10차원·6통합·3실행노드 연결은 [COVERAGE](COVERAGE.json)를 따른다.

최초 보상은 `EncounterProgressionState`에서 source가 가리키는 환상체 종류를 기준으로 판정한다.
인벤토리 파편 보유량을 조회하지 않으며 결과 transaction에서만 원장을 갱신한다. 새 원정은 새 원장이다.
엘리트 자연 배정은 숙달/격파횟수를 제거하고 기존 배정·예약을 유지한다. 수동 반복은 작성 가루·XP·파츠를
사용한다. 일반 보스 최초 성공만 Stage2 알을 만들고 Stage2 성공만 Stage3 알을 만든다. Stage3·패배·
엘리트·최종보스에는 다음 알이 없다. 마탄의 무보상 예외는 source 식별/보스 cycle 소비와 보상 생산을 분리했다.

`revival.rs`는 알·부화된 노드·출처별 지불 영수증을 소유하고 `world/revival.rs`는 현재 NodeConfirm,
실제 예약/전조/일반 활동 자격과 작성 정책을 검증한다. Gate 우회로 존재를 검사하지 않는다. U15의
`pay_materials`로 소지/창고/혼합 출처를 후보에 지불한 뒤 progression/map/profile debit을 같은
application candidate에 반영한다. 잘못된 stage/알/노드·부족·preview 생성 오류는 후보 전체를 버린다.
부화는 선택 중인 node의 조우만 바꾸며 battle·entry charge·retreat count를 시작하지 않는다.

전투 결과는 기존 staged result와 entitlement owner를 사용한다. 부활 고정 수량 보상은 기존 무억제
배율 경로에 들어가고 장비 선택 개수는 늘지 않는다. 보상 후보 생성과 Continue reader는
`RevivalStagePolicy::equipment_reward_options`를 공유한다. 후보0은 고정 보상만 처리하고 선택을 생략한다.
부활 node에는 원래 봉인/전조 보상의 황금가지·억제력 회복을 합성하지 않는다. 선택 확정/claim/결과 확인은
기존 저장된 권리를 소비하므로 재접속·재시도에서 중복 지급하지 않는다.

후퇴는 기존 입장 snapshot과 횟수 owner를 재사용한다. 두 번 후퇴 뒤 마지막 도전이 있고 부화 비용은
다시 내지 않는다. 패배는 즉시 node를 소비하며 성공 보상과 환급이 없다. 같은 층 이동은 부화 owner를
유지하고 하강은 해당 층 node 목록만 버리며 원정 알은 유지한다. 완료/패배 node의 preview를 제거하는
두 지점은 결과/선택권 저장 owner를 건드리지 않는다. 실제 default-budget Continue 실패를 먼저 재현하고
관련223검사·native35검사·새 release host/fresh reader로 교정을 확인했다.

Continue는 최초 미확정 입장 전 root를 보관한다. 입장 전에 한 부화는 유지되고 미확정 node 안 부화는
알/조우/촉매/창고 debit과 함께 되돌아간다. 수동 후퇴의 소비 횟수는 재시작에도 유지하지만 입장 비용과
그에 따른 억제력 조회는 입장 전 값으로 돌아간다. 이 때문에 후퇴 직후 live map과 fresh map의
`entry_preview`가 다르며, fresh oracle은 실제 Stage2_hatched 입장 전 map 전체와 대조한다.
명시 SavePoint는 U15의 해당 원정 debit 차이만 복원하고 무관한 창고 자산을 덮어쓰지 않는다.

구 모노리스/소탕 command는 typed retired rejection이고 capability/query/shop의 활성 제안에서 빠졌다.
RunState와 저장 codec에는 구 MonolithRunState가 없다. 기존 wire DTO와 standalone monolith 모델·
clone provenance 회귀는 남아 있으나 live Core에서 인스턴스를 소유하거나 실행하지 않는다. 이를 신규
부화의 fallback이나 두 번째 progression owner로 사용하지 않는다. V17 이전 개발 저장은 원본을
보존하며 거절한다. 자동 migration은 이번 정책의 선택이 아니다.

정상 콘텐츠는 공식 정의를 지원 RON으로 내보내고 fixture에 경로·starter 사거리·가격·촉매2개·stage 보상을
명시 작성했다. debug 지급/결과 주입 없이 실제 일반전→엘리트→장비 분쇄→자연 반복→첫 보스→상점→
SavePoint→2/3단계 수동 전투·선택을 실행했다. 공식 balance의 적정성은 주장하지 않는다.
확장은 동일 경로에서 단계 장비 후보3개를64개로 늘렸으며 전투와 한도를 축소하지 않았다.

검사 실패를 제품/입력/진행기/측정기/환경으로 구분해 [EXPERIMENTS](EXPERIMENTS.md)에 기록했다.
전체 workspace 원시 실행은 exit101이며38실패를 영향 재실행으로 닫았다. 단일 full-green 실행으로
표현하지 않는다. 이후 제품 변경은 preview 수명 두 지점뿐이고 해당 영향은 다시 검사했다.
기본16MiB와 작업 한도는 유지한다. 성능 수치는 dev 진단값이며 release host 승인 결과와 구분한다.

남은 범위는 새 요청에 따른 독립 감사다. Unity·U18 이후·개별 최종보스·U16 극단 XP 최적화는 수행하지
않았다. 알려진 제약은 [COVERAGE](COVERAGE.json)의 known_limits와 [COST_REVIEW](COST_REVIEW.json)에 있다.
