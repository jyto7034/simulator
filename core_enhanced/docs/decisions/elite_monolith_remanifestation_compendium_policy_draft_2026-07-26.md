# Elite 모노리스·강화 재현 및 숨겨진 도감 보스 정책 초안 — 2026-07-26

- 논의·작성일: 2026-07-26
- 마지막 정책 검토: 2026-07-27
- 상태: **Policy closed / P05 구현 완료 / 숨겨진 보스는 별도 후반 콘텐츠 Goal로 분리**
- 문서 등급: P05 구현 이력·후반 콘텐츠 정책 입력 문서
- 관련 임시 정책:
  [facility_map_and_node_manifestation_policy_2026-07-25_26.md](facility_map_and_node_manifestation_policy_2026-07-25_26.md),
  [combat_power_floor_difficulty_policy_draft_2026-07-26.md](combat_power_floor_difficulty_policy_draft_2026-07-26.md),
  [local_encounter_compendium_persistence_policy_draft_2026-07-25.md](local_encounter_compendium_persistence_policy_draft_2026-07-25.md)
- 주의: 이 문서는 아직 현재 runtime, live RON, save/wire 계약의 source of truth가 아니다.
  특히 현재의 반복 encounter weight, 승리당 연구 point, 완료 node 시도 횟수와 Endless 종료 조건을
  그대로 설명하는 문서가 아니라 이를 교체할 target policy를 기록한다.
- 구현 disposition: 모노리스의 자연 Elite, 상인, node 변환, stage 2/3, 잔재 교환과 변질된
  모노리스 획득은
  `../goals/run_policy_phase_05_elite_monolith_remanifestation/COMPLETION_REVIEW.md`에서
  **Complete / High**로 검증됐다. 실제 숨겨진 보스 콘텐츠와 최종 완료 연결은 현재 master에서
  제외되어 별도 후반 콘텐츠 Goal에서 다룬다. 이 문서의 후반 target policy를 현재 runtime 설명이나
  현재 master 완료 조건으로 해석하지 않는다.

## 1. 목적과 범위

Elite 환상체의 최초 자연 조우 이후 같은 환상체를 전용 모노리스로 더 강하게 재현하고, 세 번째
누적 격파에서 스킬 파편을 얻는 run-local 성장 경로를 정의한다. 모노리스를 완벽하게 활용하고 남은
잔여분은 의문의 떠돌이 상인을 통해 숨겨진 보스 `흉탄의 사수`로 이어지는 수집가용 이스터에그
경로를 만든다.

이 문서는 다음을 결정한다.

- Elite 환상체별 최초 자연 조우와 완전 실패의 비가역적 run-local 결과
- 환상체별 전용 모노리스의 상점 등장, 구매 제한, 사용 횟수와 Floor 수명
- 일반 Combat node를 강화 재현 node로 바꾸는 명시적 map action
- 강화 재현 단계, 두 번의 전투 시도, 보상과 스킬 파편 지급 시점
- 잔여 모노리스 네 개와 떠돌이 상인의 일회성 VN 교환
- 변질된 모노리스와 `흉탄의 사수` capstone의 수집가용 위치
- 미발견 placeholder, 발견과 격파 완료를 구분하는 영구 도감 의미

시설 template topology와 Shop-tagged slot 배정은 시설 맵 정책이 소유한다. 전투력 산식과 정확한
stage multiplier는 전투력 정책이 소유하며, local profile 파일의 atomic 저장·복구는 로컬 도감
정책이 소유한다.

## 2. 모드와 콘텐츠 노출 경계

- 모노리스, 강화 재현, 잔여/변질된 모노리스와 흉탄의 사수 경로는 `Endless` 전용이다.
- `Standard`에서는 모노리스 관련 상품·action·VN·보스가 노출되지 않으며 의문의 떠돌이 상인도
  등장하지 않는다. Headquarters support Shop만 Standard의 상점 경로다.
- Endless의 의문의 떠돌이 상인은 Floor 1~3에 등장하지 않는다. 일반 무작위 배정과 조건부 보장
  배정 모두 Floor 4부터 허용한다.
- 위 Floor 경계는 Shop UI의 표시 관례가 아니라 Core assignment와 live RON validation이 보장하는
  gameplay 규칙이다.

## 3. Elite 최초 자연 조우

각 Elite 환상체는 한 run에서 자연 encounter 경로로 최대 한 번만 소비된다.

- 최초 자연 Elite node는 기존 일반 Elite 규칙에 따라 최대 3회의 전투 기회를 가진다.
- 승리하면 현현한 육체를 격파하고 해당 환상체의 run-local 격파 단계가 `1`이 된다. 환상체의
  근원인 추상적 개념 자체가 소멸한 것은 아니다.
- 이 최초 승리가 commit된 뒤에만 해당 환상체의 전용 모노리스가 떠돌이 상인의 상품 후보가 된다.
- 세 번 모두 패배하면 node는 보상 없는 `Completed` transit이 된다. 실제로 조우한 환상체의 발견은
  영구 도감에 남지만 전용 모노리스는 해금하지 않는다.
- 완전 실패한 환상체는 같은 run의 자연 Elite pool에서 영구 제외하고, 다른 자연 node나 fallback으로
  다시 배정하지 않는다. 해당 환상체의 파편·잔여 모노리스 경로는 그 run에서 닫힌다.
- 최초 승리를 완료한 환상체도 자연 Elite pool에서 제외한다. 이후 동일 환상체의 등장은 해당 전용
  모노리스로만 만들 수 있다.
- Floor에 배정됐지만 방문·소비하지 않은 자연 Elite node는 해당 환상체를 소비한 것이 아니다. Gate로
  Floor를 떠난 뒤에도 그 환상체는 이후 Floor의 자연 Elite 후보로 다시 등장할 수 있다.
- 확정 Elite 로스터가 모두 최초 승리 또는 완전 실패로 소비되면 이미 소비한 환상체를 재사용하지
  않는다. 이후 Endless Combat 배정은 authored 일반 Combat 후보를 사용한다. Endless assignment
  profile은 Elite pool 소진 뒤에도 유효한 일반 Combat 후보로 Floor를 생성할 수 있어야 하며, 이는
  구 generator fallback이 아니라 명시적인 `elite_pool_exhausted` content-selection 상태다.

따라서 현재 canonical의 일반 반복 조우 weight 감소나 최근 Floor soft exclusion은 이 Elite
진행에는 적용하지 않는다. 일반 Boss·FinalBoss·BossOmen의 반복/선정 정책은 별도다.

## 4. 환상체별 전용 모노리스

### 4.1 Identity와 상점 후보

- 모노리스는 범용 재도전권이 아니라 정확히 하나의 `abnormality_id`를 가리키는 환상체별 전용
  run-local item이다.
- 일반 inventory stack이나 장비 instance가 아니라 전용 run-local progression ledger가 보유량,
  구매 이력과 `uses_remaining`을 소유한다. Ordinary inventory slot을 차지하지 않고 consumable UX,
  판매, 분쇄와 장착 대상이 아니다. Shop 구매 대금은 일반 Shop과 같은 Enkephalin을 사용한다.
- 서로 다른 환상체의 모노리스는 같은 run에서 각각 구매·보유할 수 있다.
- 동일 환상체의 모노리스는 한 run에서 정확히 한 번만 구매할 수 있다. 구매 commit 뒤에는 같은
  Shop의 refresh와 이후 모든 떠돌이 상인의 후보에서 제거한다.
- 구매하지 않고 상점을 떠난 모노리스는 이후 다른 eligible 떠돌이 상인에게 다시 등장할 수 있다.
- 모노리스 상품은 대응 환상체의 최초 자연 격파가 commit됐고, 아직 구매하지 않았으며, 해당
  환상체의 세 번째 격파가 끝나지 않은 경우에만 eligible하다.
- Headquarters Shop은 모노리스를 판매하지 않는다. 의문의 떠돌이 상인만 판매한다.
- 떠돌이 상인은 한 page에 정확히 3개의 offer를 진열한다. 최초 page 구성은 eligible 모노리스 수에
  따라 다음과 같다.
  - eligible 0개: 고가치 특수 아이템 3개
  - eligible 1개: 서로 다른 종류를 중복하지 않는 모노리스 1개와 고가치 특수 아이템 2개
  - eligible 2개 이상: 서로 다른 모노리스 2개와 고가치 특수 아이템 1개
- 모노리스 후보가 둘보다 많으면 run/Floor/node/Shop session seed와 authored weight로 두 개를
  결정론적으로 비복원 추출한다. Unity가 후보를 다시 고르지 않는다.
- 진열된 서로 다른 모노리스는 같은 Shop session에서 둘 다 구매할 수 있다. 구매한 각
  abnormality-specific ID는 해당 run의 이후 모든 Shop 후보에서 제외한다.
- 떠돌이 상인의 일반 item slot은 하이리스크·하이리턴 buff 또는 그에 준하는 고가치 효과를 제공하는
  authored item pool을 사용한다. 스킬 파편과 장비 완제품은 판매하지 않는다.
- 떠돌이 상인은 Shop session당 유료 refresh를 최대 1회 허용한다. Refresh는 현재 page 전체와 구매로
  생긴 빈자리를 버리고 고가치 특수 아이템 정확히 3개로 교체하며, refreshed page에는 모노리스를
  절대 넣지 않는다.
- 따라서 refresh는 다른 환상체의 모노리스를 찾는 기능이 아니라 현재 남아 있는 모노리스 offer를
  포기하고 item-only page를 선택하는 일방향 경제 행동이다.
- Refresh 전에 구매하지 않고 버린 모노리스는 구매 이력에 기록하지 않으므로 이후 다른 떠돌이
  상인의 eligible 후보로 다시 등장할 수 있다. 이미 구매한 모노리스는 그대로 run-global Shop
  후보에서 제외한다.
- 정확한 모노리스 가격, item/monolith weight와 단 한 번의 refresh 비용은 live RON 밸런스로
  저작한다.

### 4.2 사용 횟수와 수명

- 구매한 전용 모노리스 하나는 `uses_remaining = 3`으로 시작한다.
- 모노리스와 구매 이력, 사용 횟수, 환상체별 격파 단계는 run-local이며 새 run에서 초기화한다.
- 일반 모노리스는 Floor를 넘어 보존한다. 현재 Floor에 적합한 일반 Combat node가 없으면 이후
  Floor에서 사용할 수 있다.
- run 종료 시 사용하지 않은 일반 모노리스와 남은 횟수는 소멸한다.

## 5. 일반 Combat node 변환

### 5.1 Player flow

- 플레이어가 맵에서 `Available` 상태의 일반 Combat node를 선택하면 기존 전투 preview와 진입
  action을 그대로 제공한다.
- 사용할 수 있는 모노리스가 하나 이상이면 진입 버튼 옆에 `모노리스 사용 (N)` action을 하나만
  추가한다. `N`은 보유 수가 아니라 현재 node에 실제 사용할 수 있는 모노리스 수다. 모노리스별
  버튼을 NodeConfirm에 나열하지 않는다.
- Action을 누르면 Core가 투영한 선택 panel을 연다. Eligible card는 환상체 identity, 다음 재현 단계,
  `uses_remaining` before/after, 전투 기회 2회, 확정 위험도·보상 preview를 표시한다. 최종 강화 재현
  가능 항목을 먼저, 같은 단계는 표시 이름 오름차순으로 정렬한다.
- Card를 고르는 것만으로 상태를 바꾸지 않는다. Detail은 원래 일반 encounter/reward가 사라지고
  사용 횟수를 돌려주지 않는다는 직접 결과를 표시하며, `이 모노리스로 재현` 하나가 최종 commit
  action이다. 이 action 자체를 확인으로 사용하고 같은 내용을 묻는 2차 modal은 만들지 않는다.
- 취소는 node와 모노리스를 변경하지 않는다. 성공한 변환 뒤에는 변환된 NodeConfirm preview로
  돌아가며 전투에 자동 진입하지 않는다. 플레이어가 새 preview를 확인한 뒤 기존 전투 진입 action을
  별도로 실행한다.
- 같은 모노리스가 만든 미해결 재현 node가 있으면 그 모노리스로 다른 node를 동시에 변환할 수 없다.
  해당 항목은 panel 아래 disabled card로 유지하고 `진행 중: <room 표시 이름>`을 보여준다. 기존
  재현 node가 승리 또는 2회 완전 실패로 해결된 뒤에만 다음 node에 사용할 수 있다.
- 서로 다른 환상체의 모노리스는 각각 하나의 미해결 재현 node를 가질 수 있다. 여러 전용 모노리스로
  서로 다른 일반 Combat node를 변환하는 것은 허용한다.
- Unity는 local inventory나 encounter catalog로 목록, 진행 중 상태와 eligibility를 재계산하지 않는다.
  선택 command는 node와 abnormality-specific monolith identity를 보내며 Core가 현재 state에서 다시
  검증한다.
- Normal이 아닌 Elite, Boss, FinalBoss, Event, Shop, Reward, Gate와 이미 시작·완료된 node는 변환할
  수 없다.
- Category가 Combat이더라도 BossOmen source나 다른 시스템의 content overlay/reservation이 이미
  적용된 node는 변환할 수 없다. 모노리스 action은 다른 override가 없는 순수 Normal Combat에만
  노출하며 override 우선순위로 한쪽을 조용히 덮지 않는다.

### 5.2 변환 authority와 atomicity

- 모노리스 사용은 선택한 node의 `node_id`, `slot_id`, `Combat` category와 template connection을
  유지한 채 일반 encounter/content를 지정 환상체의 다음 강화 재현 encounter로 교체한다.
- 원래 일반 encounter와 그 보상은 보존하거나 나중에 복원하지 않는다. 강화 재현 encounter와
  stage 보상이 node의 유일한 전투 authority가 된다.
- 변환과 모노리스 사용 횟수 1 차감은 하나의 원자적 commit이다. 실패한 command는 node와 item을
  모두 변경하지 않는다.
- 성공적으로 변환한 뒤 preview를 닫거나 다른 node를 선택해도 사용 횟수는 반환하지 않으며 node는
  변환된 상태로 남는다.
- 변환한 node에 진입하지 않고 Floor를 떠나거나 `ForwardOnly` 진행으로 접근을 잃어도 사용 횟수를
  반환하지 않는다. 별도 환불·경고·추천 projection을 만들지 않는다.
- 변환된 node marker/preview는 환상체 identity, 재현 단계, 최대 두 번의 전투 기회와 더 높은 위험을
  Core projection으로 표시한다.
- 원래 일반 Combat node의 Easy/Standard/Hard band를 골라 재현 encounter를 약하게 만드는 것은
  허용하지 않는다. 재현 난이도는 현재 Floor의 고정 anchor와 환상체·stage 전용 profile로 새로
  계산하며 원래 node band/reward를 상속하지 않는다.
- 변환된 map difficulty는 stage 2/3을 typed authority로 저장하고 run-local monolith ledger와 exact
  일치해야 한다. Stage는 target power와 적 최대 HP/공격/방어를 강화하지만 wave budget은 늘리지 않는다.
  Role+stage reward multiplier는 authored 원수량에 한 번만 적용·반올림하며 public UI는 exact 수치 대신
  `enhanced`/`final_enhanced` qualitative tier만 표시한다.

이 변환은 topology나 room slot을 추가·삭제하는 동적 map 생성이 아니다. 고정 Combat slot의
content source만 명시적으로 교체하는 모노리스 전용 overlay/override다.

## 6. 강화 재현 단계·시도·보상

환상체별 누적 격파 단계는 `0..3`이며 승리 commit만 증가시킨다.

1. `1회차 / 자연 현현`
   - 자연 Elite encounter다.
   - 승리 시 현현체 격파, identity 파악과 전용 모노리스 상품 eligibility를 연다.
2. `2회차 / 강화 재현`
   - 모노리스가 일반 Combat node를 바꿔 생성한다.
   - 1회차보다 높은 stat profile과 더 많은 authored 보상을 사용한다.
3. `3회차 / 최종 강화 재현`
   - 1·2회차보다 높은 stat profile과 더 많은 authored 보상을 사용한다.
   - 승리하면 일반 stage 보상과 함께 해당 환상체의 고유 스킬 파편을 정확히 한 번 지급한다.

각 모노리스 사용으로 생성된 강화 재현 node는 최대 2회의 전투 기회를 가진다.

- 첫 패배 뒤에는 추가 모노리스 사용 없이 같은 변환 node에서 한 번 더 도전할 수 있다.
- 전투 결과가 확정되기 전 `RetreatBattle`은 일반 전투와 같이 진입 상태를 복원하며 두 번의 기회를
  소비하지 않는다.
- 두 번 모두 패배하면 node를 보상 없는 `Completed` transit으로 만들고 격파 단계는 올리지 않는다.
- 여기서 보상 없음은 authored 재현 stage reward가 없다는 뜻이다. 패배 결과의 생존 직원 XP처럼
  모든 CombatResult에 공통인 기존 post-battle 성장은 그대로 적용한다.
- 남은 모노리스 횟수가 있으면 다른 `Available` 일반 Combat node를 같은 재현 단계로 다시 변환할 수
  있다.
- 한 재현 node의 두 시도 중 한 번이라도 승리하면 node를 완료하고 다음 단계로 진행한다.
- 세 번째 격파가 완료되면 사용 횟수가 0이더라도 스킬 파편은 지급한다. 잔여 모노리스만 만들지
  못한다.
- 세 번의 모노리스 사용을 모두 소진하고도 세 번째 격파를 완료하지 못하면 해당 run에서는 같은
  전용 모노리스를 다시 구매할 수 없으므로 그 환상체의 연구 경로가 종료된다.

2·3회차를 각각 첫 번째로 생성한 재현 node의 두 시도 안에서 완료하면 모노리스는 정확히 1회를
남긴다. 개별 node의 첫 전투에서 패배하고 두 번째 전투에서 승리해도 추가 모노리스 횟수는 소비하지
않으므로 잔여분을 만들 수 있다.

## 7. 잔여 모노리스와 떠돌이 상인 교환

### 7.1 잔여분

- 세 번째 누적 격파가 commit됐을 때 해당 전용 모노리스의 사용 횟수가 정확히 1이면 더 이상 Elite
  재현에 사용할 수 없는 `잔여 모노리스`로 전환한다.
- 잔여 모노리스는 출처 환상체 ID를 기록하되 변질 교환에서는 서로 교환 가능한 재료다.
- 동일 환상체 모노리스의 run당 구매 제한 때문에 한 run의 잔여분은 자연스럽게 서로 다른 환상체에서
  온다.
- 잔여분과 교환 진행은 run-local이며 run 종료 시 소멸한다.
- 변질 교환 초기 요구량은 `4`이며 live RON의 양수 정책값으로 둔다.
- 잔여분이 요구량보다 많으면 획득 sequence가 오래된 것부터 정확히 네 개를 소비한다. 교환은 run당
  한 번뿐이며 남은 잔여분은 다른 용도·판매 없이 run 종료 때까지 전용 ledger에 남는다.

### 7.2 다음 Floor Shop-tag 보장

- 잔여 모노리스 수가 처음으로 요구량 이상이 되면 `monolith_exchange_pending`을 기록한다.
- 현재 Floor의 template나 이미 배정된 node를 수정하지 않는다.
- 다음 Floor 입장 시 facility template 후보는 도달 가능한 `Shop` tag slot을 가진 Endless template로
  제한하고, assignment solver가 그중 정확히 하나에 의문의 떠돌이 상인을 강제 배정한다.
- 후보 slot이 여러 개면 Floor seed로 결정론적으로 하나를 선택한다. 조건을 만족하는 template가
  하나도 없으면 일반 template나 구 generator로 fallback하지 않고 validation/content error다.
- 보장 배정은 run당 한 번만 소비한다. 플레이어가 해당 Shop node를 방문하지 않고 Gate를 선택하면
  보장 배정을 다시 만들지 않는다.
- 아직 교환 제안을 실제로 보지 않았다면 이후 무작위로 만난 eligible 떠돌이 상인도 같은 제안을
  할 수 있다.

### 7.3 일회성 VN 교환

- 잔여 모노리스 4개 이상을 가진 채 떠돌이 상인에게 진입하면 일반 Shop UI보다 먼저 전용 VN을
  시작한다.
- 상인은 특별한 물건이 있다며 잔여 모노리스 정확히 4개와 교환할지 묻는다.
- 선택지는 최소 `4개를 건넨다`와 `거절한다`를 제공한다.
- 수락은 잔여분 4개 소비와 `변질된 모노리스` 1개 지급을 원자적으로 commit한다. 추가 Enkephalin은
  요구하지 않는 비밀 물물교환이다.
- 거절은 아무 자원도 소비하지 않지만 되돌릴 수 없다.
- VN 제안을 실제로 시작한 시점부터 수락 여부와 무관하게 같은 run에서는 다시 제안하지 않는다.
  reconnect/load로 같은 활성 VN을 복구하는 것은 두 번째 제안이 아니다.
- VN 해결 뒤에는 같은 떠돌이 상인의 일반 Shop session을 계속 이용할 수 있다.

## 8. 변질된 모노리스와 흉탄의 사수

- 변질된 모노리스는 run-local 이스터에그 key item이며 일반 Elite 재현에는 사용할 수 없다.
- 숨겨진 보스는 `흉탄의 사수`로 확정한다. 현재 live의 일반 보스 `마탄의 사수/Der Freischütz`와
  다른 신규 canonical actor/encounter이며 마탄의 사수 ID나 전투를 재사용하지 않는다.
- 일반 Endless 완료에 필요한 최종 보스 3종은 `백야`, `고요한 오케스트라`, `종말새`다. 현재 live
  RON에서 FinalBoss로 잘못 분류된 `증오의 여왕`과 `웃는 시체의 산`은 target data에서 NormalBoss로
  정정한다.
- 같은 run에서 변질된 모노리스를 보유한 채 위 일반 최종 보스 3종을 모두 격파하면 일반 run 완료를
  즉시 확정하지 않고 관리자 자신이 화자인 전용 VN을 강제로 시작한다.
- 세 보스를 모두 격파했을 때 변질된 모노리스가 없으면 일반 `RunComplete`를 즉시 확정한다. 이
  3종 격파가 target Endless의 일반 완료 조건이며 현재의 `all_response_complete` 종료 조건을
  교체한다. Elite 모노리스·파편 수집과 다른 일반 Boss 연구는 선택형 run progression이다.
- VN은 변질된 모노리스가 이상하게 반응한다는 관리자 내면 독백과 이를 건드려 보는 흐름을 authored
  content로 표현한다. Core는 trigger와 선택 결과만 소유하고 실제 대사·연출 resource는 Unity-local
  VN content가 소유한다.
- 마지막 gameplay 선택지는 `변질된 모노리스를 건드린다` 하나만 제공한다. 일반 완료로 돌아가거나
  모노리스를 사용하지 않는 거절 선택지는 두지 않는다.
- VN이 활성화된 동안 일반 map action이나 run 완료를 진행하지 않는다. reconnect/snapshot 재표시는
  같은 활성 VN과 단일 선택지를 복구하며 이미 본 것으로 간주해 건너뛰지 않는다.
- 단일 선택 확정은 변질된 모노리스 소비, VN 완료와 흉탄의 사수 capstone 전환을 하나의 원자적
  transition으로 commit한다. 어느 단계든 실패하면 item을 소비하거나 VN만 완료하지 않는다.
- 실제 도전은 일반 map에 node/connection을 추가하지 않는다. 선택을 확정하면 현재 일반 map을
  종료하고 authored 흉탄의 사수 capstone template로 원자 전환한다.
- capstone은 시설 맵 문서의 BossOmen 전환 원칙과 같은 고정 template authority를 사용한다.
- 흉탄의 사수 전투는 한 번만 도전할 수 있다. 패배하면 일반 FinalBoss 패배와 같이 즉시
  `RunFailed`이며 모노리스 전용 재도전은 없다. 공통 SavePoint/RunFailed 정책이 별도로 허용하는
  선택만 적용한다.
- 한 번 도전은 현재 timeline의 capstone node attempt 수다. 공통 실패 전 checkpoint 선택이나 로컬
  active-run 이어하기로 과거 SavePoint를 복원하면 capstone 진입 전 상태와 변질된 모노리스가 함께
  복원되므로 다시 도전할 수 있다.
- 흉탄의 사수 승리는 전투·도감 격파와 특별 completion record를 하나의 결과 transaction으로
  commit한 뒤 즉시 `RunComplete`로 종료한다. Exit Gate를 공개하거나 다음 Floor로 진행하지 않는다.
- 특별 completion reason은 일반 최종 보스 3종만 소진한 완료와 구분하되 별도의 대규모 스토리
  엔딩을 요구하지 않는다. 현재 최소 보상은 흉탄의 사수 영구 발견/격파 표식과 특별 완료 기록이다.
- 흉탄의 사수 고유 item/reward와 승리 뒤 후속 Floor는 미래 콘텐츠 추가 범위다. 추가되기 전에는
  보상이나 진행을 빈 placeholder로 노출하지 않는다.
- 이 콘텐츠의 우선 목적은 큰 스토리 분기가 아니라 영구 도감의 마지막 숨겨진 보스 항목을 찾는
  수집가용 목표다.

`노말엔딩`/`진엔딩`이라는 명칭과 별도 대규모 스토리는 현재 중요한 요구사항이 아니다. Runtime은
일반 완료와 숨겨진 보스 완료의 typed reason만 구분한다.

## 9. Checkpoint load와 rollback 경계

- 전용 모노리스 구매 이력과 보유 상태, `uses_remaining`, 환상체별 격파 단계, 변환된 일반 Combat
  node와 남은 전투 시도, 잔여/변질된 모노리스, 조건부 Shop 보장과 교환 VN 상태는 모두 일반
  run-local checkpoint payload에 포함한다.
- 유효한 SavePoint load는 위 상태를 checkpoint 시점의 값으로 함께 복원한다. 모노리스 실패·소비나
  교환 수락/거절만 현재 값으로 남기는 부분 복원은 허용하지 않는다.
- Load 뒤에는 복원된 map/node, inventory, Shop purchase history와 모노리스 상태가 서로 일치해야
  한다. checkpoint 뒤에 변환했던 node는 원래 checkpoint content로 돌아가고, 그때 소비했던
  모노리스 횟수도 함께 복원된다.
- 이 복구는 기존 run당 최대 3회의 checkpoint load 중 한 번을 소비한다. `loads_used`는 기존
  SavePoint 정책대로 checkpoint가 되돌리지 않는 run-scoped monotonic metadata다.
- 영구 도감의 `discovered`와 `defeated` set은 run checkpoint 밖의 local profile이므로 load로
  되돌리지 않는다. 패배 결과에서 흉탄의 사수를 발견한 뒤 과거 checkpoint로 돌아가도 발견은
  유지되며, 승리하지 않았다면 격파 표식은 생기지 않는다.

## 10. 영구 도감 placeholder와 격파 표식

- 도감 snapshot은 발견된 ID만 나열하지 않고 authored 도감 순서의 현재 지원 항목 전체를 투영한다.
- 미발견 항목은 이름, 이미지, 설명, 위험 등급과 기타 semantic metadata를 숨긴 placeholder로 표시한다.
- 보스 도감은 일반 최종 보스 세 항목과 흉탄의 사수 placeholder 한 항목, 총 네 자리를 가진다.
  플레이어는 일반 최종 보스를 모두 발견한 뒤에도 남은 한 칸으로 숨겨진 콘텐츠의 존재를 추론할
  수 있다.
- 영구 profile은 `discovered`와 `defeated`를 구분한다. `defeated`는 `discovered`의 부분집합이어야
  한다.
- 실제 적군 spawn을 포함한 encounter가 결과로 소비될 때 기존 발견 규칙에 따라 `discovered`를
  commit한다. 최종 패배로 RunFailed가 되어도 실제 조우 발견은 남는다.
- 승리 결과가 commit될 때 primary abnormality ID를 `defeated`에 추가한다. 죽이지 않고 격리·제압하는
  authored victory도 해당 encounter의 primary는 격파 완료로 본다.
- 비primary 환상체와 침식 직원은 승리한 attempt의 `UnitDied`에 실제 source identity가 기록된
  경우에만 `defeated`에 추가한다. Survive timer 승리 때 살아서 남은 적이나 패배 attempt에서만
  죽인 적은 격파 표식을 얻지 않는다.
- 발견되면 placeholder를 현재 catalog metadata로 교체하고, 격파되면 별도의 완료 표식을 표시한다.
- UI는 발견률과 격파 완료율을 구분한다. 흉탄의 사수에게 도달했으나 패배한 플레이어는 항목을
  발견하지만 격파 표식은 얻지 못한다.
- 두 set은 run/checkpoint와 분리된 local persistent profile에 누적되며 새 run, RunFailed와
  checkpoint load로 제거하지 않는다.

## 11. 현재 정책·구현과의 알려진 충돌

이 초안은 최소 다음 current-state 의미를 교체하거나 확장한다.

- 현재 Endless 연구의 requirement 100/승리 20 point와 보너스 기반 완료는 Elite의 고정 3단계
  스킬 파편 지급 정책과 일치하지 않는다.
- 현재 일반 encounter repeat weighting은 완료/실패한 동일 Elite를 다시 자연 선정할 수 있지만 새
  정책은 최초 자연 encounter 소비 뒤 hard exclusion한다.
- 현재 전투 node 기본 최대 3회 규칙과 달리 모노리스 강화 재현 node는 최대 2회다.
- 현재 map/navigation에는 일반 Combat content를 item action으로 원자 교체하는 계약이 없다.
- 현재 Shop에는 환상체별 모노리스 typed merchandise, run-local 구매 exclusion과 Floor 4 최소 조건이
  없다.
- 현재 도감은 발견 ID set만 저장·투영하며 미발견 placeholder, authored 전체 순서와 격파 set이 없다.
- 현재 Endless 완료 조건과 최종 보스 pool 3종 기반 완료/숨겨진 보스 gate는 아직 서로 동기화되지
  않았다.
- 현재 live encounter는 콘텐츠 카탈로그와 달리 증오의 여왕과 웃는 시체의 산을 FinalBoss로
  분류한다. Target migration은 두 encounter를 NormalBoss로 내리고 세 일반 최종 보스와 숨겨진
  흉탄의 사수를 서로 다른 canonical identity로 검증해야 한다.

이 차이는 초안을 무효화하는 근거가 아니라 독립 goal에서 함께 이관해야 할 migration 대상이다.

## 12. 구현 전 콘텐츠·밸런스 Authoring

다음 항목은 초기 live RON 콘텐츠·밸런스 authoring에서 값을 정한다.

1. 모노리스 가격, item/monolith weight와 떠돌이 상인 refresh 비용의 초기 RON 밸런스 값
2. 재현 stage별 stat·reward multiplier와 흉탄의 사수 전용 reward profile

위 항목은 현재 합의한 gameplay 흐름을 막는 필수 정책 gate가 아니다. 플레이테스트 가능한 보수적
초기값을 저작하고 focused simulation/test로 조정한다. 현재 지원 도감 항목은 모두 authored 순서의
placeholder 자리를 표시하며, 자리 자체를 숨길 future entry가 생길 때만 별도 reveal 정책을 추가한다.

정책 검토 결과 이 문서가 소유하는 gameplay 흐름에는 추가 사용자 결정이 없다. 흉탄의 사수 전용
스킬·encounter content와 위 수치는 후속 콘텐츠 제작 범위이며, 구현 감사에서 문서 의미를 바꿔야만
해결할 수 있는 충돌이 확인될 때만 사용자 결정을 다시 요청한다.

## 13. 독립 goal 경계와 동기화 대상

이 정책은 시설 맵 구현 goal에 흡수하지 않고 별도 영향 감사와 구현 goal로 진행한다. 최소 다음을
함께 감사·갱신한다.

- `game_rulebook.md`: Elite 반복, 연구/파편, 전투 시도, Shop, 도감과 Endless 완료 조건
- `core_runtime_contract.md`: node conversion transaction, battle result, run-local/persistent state와
  checkpoint 경계
- `data_loading_contract.md`: 모노리스 item/Shop entry, 재현 stage profile, compendium order와 capstone
  template validation
- facility template/profile RON: `Shop` slot tag, Floor 4+ Unknown Shop와 조건부 다음 Floor 보장
- PVE/abnormality/run policy RON: 최초 자연 hard exclusion, stage encounter/stat/reward와 hidden boss
- Core map preview/action capability, Shop, inventory, combat result, research, run completion과 snapshot
- local compendium profile의 discovered/defeated set, placeholder projection과 host persistence
- game server mapping과 외부 Unity map/node-confirm/Shop/VN/compendium 계약
- Core strict loading/test, host persistence test, server probe와 Unity EditMode/PlayMode test

## 14. 다른 작업자에게 넘길 때의 기준

- 이 문서는 current runtime 설명이 아니라 사용자와 합의한 target policy다.
- Elite는 자연 encounter로 한 번만 소비되고 완전 실패도 해당 run의 영구 exclusion이다.
- 전용 모노리스는 3회 사용 가능하며 일반 Combat node를 다음 강화 재현 단계로 바꾼다.
- 여러 모노리스를 보유해도 NodeConfirm에는 `모노리스 사용 (N)` action 하나만 표시하고 Core가
  투영한 card panel에서 하나를 골라 detail 확인 뒤 commit한다.
- 강화 재현 node는 두 번 도전하고, 완전 실패 시 다른 일반 Combat node와 추가 사용 횟수가 필요하다.
- 세 번째 누적 격파는 스킬 파편을 지급하고, 모노리스 1회가 남았을 때만 잔여분을 만든다.
- 서로 다른 잔여분 네 개와 Floor 4+ 떠돌이 상인의 일회성 VN으로 변질된 모노리스를 얻는다.
- Standard에는 떠돌이 상인과 모노리스가 없으며 Endless 떠돌이 상인은 Floor 4부터 등장한다.
- 떠돌이 상인 최초 page는 eligible 수에 따라 모노리스 최대 2개를 보장하고, 단 한 번의 유료
  refresh는 모노리스를 모두 포기하는 item-only 3-slot page로 전환한다.
- 변질된 모노리스를 가진 채 일반 최종 보스 3종을 모두 격파하면 관리자 독백 VN을 강제 시작하고,
  거절 없는 단일 선택으로 모노리스를 소비해 흉탄의 사수 capstone에 진입한다.
- 흉탄의 사수 승리는 도감 격파와 특별 완료 기록을 commit하고 Exit Gate 없이 즉시 run을 완료한다.
  고유 보상과 후속 Floor는 미래 콘텐츠다.
- 숨겨진 보스의 핵심 가치는 스토리 엔딩보다 authored 네 번째 보스 도감 항목과 격파 표식이다.
- 모노리스 run-local 상태는 checkpoint와 함께 복원하지만 `loads_used`와 영구 도감은 되돌리지 않는다.
- 12절의 초기 수치는 live RON에 보수적인 초깃값을 저작하고 focused simulation과 플레이테스트로
  조정하되, 이 문서의 gameplay 흐름을 바꾸지 않는다.
