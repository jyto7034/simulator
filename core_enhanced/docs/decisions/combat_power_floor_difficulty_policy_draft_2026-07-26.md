# 플레이어 전투력·Floor 난이도 분포 정책 초안 — 2026-07-26

- 논의·작성일: 2026-07-26
- 최종 검토일: 2026-07-27
- 상태: **Policy closed / P04 구현 완료 (2026-07-27)**
- 문서 등급: 구현 이력·정책 근거 문서
- 구현 disposition: certified power, 비되감기 run peak, Floor anchor, node/role difficulty와 Unity
  projection은 `../goals/run_policy_phase_04_combat_power_floor_difficulty/COMPLETION_REVIEW.md`에서
  **Complete / High**로 검증됐다. 현재 계약의 source of truth는 durable canonical과 runtime/live
  data이며 이 문서는 승인 정책의 역사적 입력으로 유지한다.
- 관련 정책:
  [facility_map_and_node_manifestation_policy_2026-07-25_26.md](facility_map_and_node_manifestation_policy_2026-07-25_26.md)

## 1. 목적과 범위

이 정책은 반복 성장으로 막힌 stage를 충분히 강화한 뒤 재도전하는 일반적인 tower-defense 구조와 달리,
한 run 안에서 제한된 성장 기회와 손실을 관리하는 게임에 Floor별 난이도 기준과 node별 편차를 만든다.

- 새 Floor 생성 시점의 run 전투력을 기준으로 고정 `floor_power_anchor`를 확정한다.
- 같은 Floor의 Combat node를 Easy/Standard/Hard 등 여러 난이도 band로 사전 배정한다.
- 장비 해제 등으로 다음 Floor 난이도를 의도적으로 낮추는 조작을 막는다.
- 같은 Floor 진행 중에는 성장·손실·재접속으로 이미 생성된 node를 재스케일하지 않는다.

시설 topology, slot/category assignment와 template pool은 관련 시설 맵 정책이 소유한다. 실제 직원·장비·
스킬 전투력 산식, encounter/wave budget 변환과 난이도 projection은 이 독립 정책과 구현 goal이 소유한다.

## 2. 전투력 authority

### 2.1 Certified combat power

- `certified_combat_power`는 Core가 authoritative roster/loadout 상태에서 계산한 derived value다.
- Floor 1 생성 전에는 시작 직원 선발과 initial loadout commit 결과를 최초 certified snapshot으로 사용한다.
- 이후에는 실제 전투 진입이 확정될 때 사용한 합법적인 roster/loadout의 certified power를 기록한다.
- 직원의 영구 stat·level, 장착 장비와 강화도, 장착 skill fragment처럼 전투 진입 전에 확정되는
  지속 요소를 입력으로 사용한다.
- 전투 진입 시 실제 출전 가능한 직원과 그 전투에서 사용할 현재 장착 장비·skill fragment만 계산한다.
  미장착 inventory와 아직 실제 전투에서 한 번도 사용하지 않은 신규 장비는 포함하지 않는다.
- 현재 HP·Trauma, 일회성 consumable, 전투 중 temporary buff와 플레이어 조작 숙련도는 raw combat
  power 산식에 직접 포함하지 않는다. HP·Trauma, Breakdown과 출전 가능 인원은 별도 readiness/risk
  projection의 입력으로만 사용한다.

### 2.2 임시 산식과 장기 authority

- 실질적인 대표 직원·장비·skill·enemy 콘텐츠가 완성되기 전에는 실제 runtime 정량값을 사용하는
  `provisional` 분석 산식으로 certified combat power를 계산한다.
- 임시 산식은 HP, 공격력, 공격 주기, 방어·저항, block, range와 같이 현재 runtime에서 결정적으로
  읽을 수 있는 정량 요소를 합성한다. 구체 계수와 band 경계는 live `RunPolicy`가 소유하고 strict
  validation한다.
- 임시 수치는 시스템 진행, 결정론 검증과 내부 밸런스 작업을 위한 값이다. 최종 밸런스가 검증됐다는
  의미로 취급하지 않는다.
- 대표 콘텐츠가 준비되면 Core headless 전투를 반복 실행해 승률, 생존 인원, 잔여 HP, Trauma,
  Breakdown과 전투 시간을 측정한다. 공격적·보수적 등 복수의 고정 전술 profile을 사용해 특정 한
  조작 방식에만 맞는 산식이 되지 않도록 한다.
- 복잡한 damage/control/sustain/utility 효과는 가능한 한 제한된 typed effect로 표현한다. 임시 분석
  산식으로 설명할 수 없는 예외만 범위와 사유가 검증되는 capped calibration offset을 허용하며,
  대표 콘텐츠 시뮬레이션 결과로 재검토한다.
- 콘텐츠 검증 단계에서는 분석 산식이 시뮬레이션 결과를 근사하도록 계수와 band 경계를 다시 정한다.
  실제 게임 실행 중 매번 가상 전투를 돌려 power를 계산하지 않는다.
- 산식과 계수는 versioned `power_model_version`을 가진다. 이미 생성된 Floor와 node는 생성 당시 모델과
  확정값을 보존하며, 모델 갱신으로 진행 중인 run을 소급 재스케일하지 않는다.

### 2.3 Run peak

- `run_peak_certified_power`는 현재 run에서 지금까지 확정된 certified combat power의 최댓값이다.
- 장비를 약하게 바꾸거나 직원을 편성에서 제외해도 이 값은 내려가지 않는다.
- 직원 사망·부상이나 inventory 변화로 현재 가용 전력이 낮아져도 run peak를 낮추지 않는다. 손실이
  다음 Floor 난이도를 되돌리는 자동 보정으로 작동하지 않아야 한다.
- run peak는 active-run 저장의 비되감기 metadata다. Retreat, checkpoint load와 게임 재실행으로
  낮아지지 않으며, 과거 checkpoint의 낮은 값이 현재 최댓값을 덮어쓰지 않는다.
- v1은 소유 중인 모든 직원·장비 조합의 이론상 최적 전투력을 탐색하지 않는다. 새 장비를 획득하고
  실제 전투에서 한 번도 사용하지 않은 채 다음 Floor를 먼저 생성해 얻는 제한적 이점은 허용한다.

## 3. Floor power anchor

새 Floor를 생성할 때 Core는 다음 값을 한 번 계산해 run state에 저장한다.

```text
floor_power_anchor = max(
    Floor profile의 authored 최소값,
    이전 Floor anchor + authored 최소 상승분,
    run_peak_certified_power
)
```

- Floor 1에는 이전 anchor가 없으므로 Floor 1 authored 최소값과 initial certified power의 최댓값을 쓴다.
- authored 최소값과 최소 상승분은 mode/Floor difficulty profile의 live RON 값이다.
- anchor는 같은 run에서 단조 증가하며 overflow, 음수·0 값과 profile schedule 누락을 strict validation한다.
- 플레이어 전투력이 Floor 요구치보다 낮아져도 이미 확정된 anchor를 낮추지 않는다.

## 4. 생성·스냅샷 경계

- 일반 Floor의 power anchor와 모든 node difficulty는 새 run의 Floor 1 생성 및 Gate 확정에 따른 다음
  Floor 전환 transaction에서만 생성한다.
- 클라이언트의 map/snapshot 재요청, 재접속, 같은 Floor의 node 선택·완료와 Safezone 왕복은 기존
  Floor 데이터를 반환하며 anchor나 node difficulty를 다시 계산하지 않는다.
- BossOmen capstone은 같은 Floor의 특수 map 전환이므로 현재 Floor anchor를 그대로 사용한다.
- Floor 생성 transaction은 선택한 facility template, assignment profile, node category/kind/content,
  difficulty band와 encounter seed를 함께 결정론적으로 확정한다.
- checkpoint/save는 현재 Floor anchor, node별 확정 difficulty, run peak와 certified snapshot에 필요한
  결정 상태를 보존한다. Load나 reconnect가 더 낮은 난이도로 재생성되는 경로를 허용하지 않는다.

## 5. Combat node 난이도 분포

- 일반 Combat node는 Floor assignment profile이 명시한 Easy/Standard/Hard quota 또는 비율에 따라
  결정론적으로 band를 배정한다.
- 예를 들어 일반 Combat이 10개이고 profile이 `3/3/4`를 명시하면 Easy 3, Standard 3, Hard 4를
  정확히 배정한다. 이 수치는 예시이며 모든 Floor의 하드코딩 기본값이 아니다.
- Combat 수가 비율과 정확히 나누어지지 않으면 profile이 승인한 결정론적 rounding 규칙을 사용하고
  최종 band 수의 합은 실제 대상 Combat node 수와 정확히 같아야 한다.
- node target power는 `floor_power_anchor`, difficulty-band modifier와 encounter-specific modifier로
  계산한다. 같은 band라도 authored encounter 구성에 따라 전술적 성격은 달라질 수 있다.
- 이미 생성된 node target power, enemy/wave budget과 reward contract는 같은 Floor 안에서 플레이어
  전투력 변화에 맞춰 재스케일하지 않는다.
- Easy/Standard/Hard quota에는 ordinary Combat node만 포함한다.
- Elite, NormalBoss, FinalBoss, BossOmen과 모노리스 강화 재현은 일반 Combat quota에서 제외하고 각각의
  role difficulty profile로 target power, encounter/wave budget과 reward를 산정한다.
- 모노리스 강화 재현의 typed stage는 role profile에 추가 합성한다. Stage modifier는 target power와
  적 최대 HP/공격/방어 및 reward에 적용하지만 generated wave budget/extra waves에는 적용하지 않는다.
  Role+stage는 적용 전에 한 번 합성하고 실제 stat/reward 수량은 한 번만 반올림한다.

## 6. 위험·보상과 플레이어 projection

- Hard node가 선택 가능한 위험만 늘리고 보상 기대값이 같아지는 구조는 피한다. Difficulty profile은
  band별 reward budget 또는 희귀 후보 weight를 명시해야 한다. Hard band와 boss/특수 role profile은
  대응하는 일반 조우보다 높은 reward budget 또는 희귀도 weight를 가져야 한다.
- Core는 node의 확정 difficulty와 플레이어의 현재 certified/readiness 값을 이용해 Unity-facing 위협
  projection을 만든다. Unity에는 정확한 power 값이나 추천 수치를 공개하지 않고 다음만 표시한다.
  - ordinary Combat의 확정 band: Easy/Standard/Hard
  - 현재 certified power 대비 상대 위협: 낮음/대등/높음/매우 높음
  - 부상·Trauma·Breakdown과 출전 가능 인원에 따른 readiness 경고
- 전투력 수치는 승리를 보장하지 않는다. 배치 위치, 상성, wave 구성, 스킬 사용과 retreat 판단은
  같은 power에서도 결과를 바꿀 수 있다.

## 7. 난이도 곡선 검증

- 기대 보상·성장 곡선과 Floor authored 최소 power 상승분을 함께 시뮬레이션·감사한다.
- 평균 선택뿐 아니라 불리한 보상 결과와 직원 손실 뒤에도 의도한 hard-run 정책 안에서 유효한 진행
  경로가 있는지 검증한다.
- Floor anchor가 player power를 참조한다는 이유로 reward 성장과 enemy scaling이 무의미한 treadmill이
  되지 않도록, 같은 Floor의 고정 난이도와 Easy/Standard/Hard 편차를 보존한다.
- 수치 formula와 modifier는 live RON이 소유하되 누락·겹침·0 divisor·overflow와 비단조 Floor schedule은
  fail-fast validation한다.
- Standard campaign의 유한 Floor schedule은 모든 target Floor를 명시한다. Endless는 마지막 finite
  stage를 clamp해 영구 반복하지 않고 open-ended profile과 Floor별 최소 상승분으로 계속 확장한다.
- 임시 산식 단계의 검증은 결정론, 단조 증가, quota, 저장·복원과 유효 범위에 집중한다. 대표 콘텐츠가
  준비된 뒤의 focused simulation과 실제 playtest가 최종 계수·band 경계 승인 조건이다.

## 8. 비목표와 후속 독립 범위

- 완료 node 재도전권 아이템의 획득처, 사용 대상, replay session과 반복 보상
- 시설 map topology, conditional door/connection과 puzzle unlock
- 전투력만으로 승패를 예측하거나 자동 편성하는 기능
- v1에서 소유 inventory의 이론상 최적 loadout을 전수 탐색하는 기능

완료 node 재도전권은 의도적인 추가 성장 기회를 제공하는 별도 gameplay system 후보다. 일반 Combat,
Elite/Boss, Event/Safe Node 중 어디까지 재도전할지와 반복 보상·연구·BossOmen effect를 먼저 결정한 뒤
독립 정책과 goal로 분리한다.

## 9. 정책 종료와 구현·밸런스 후속 범위

정책 결정은 종료됐다. 독립 구현 goal은 다음을 live data와 runtime 계약으로 구체화한다.

1. provisional certified-power feature, 계수와 `power_model_version` RON shape
2. ordinary band와 role profile별 target-power/reward modifier RON shape
3. current certified power, 비되감기 run peak와 Floor/node 확정값의 저장·복원 계약
4. Unity-facing band, 상대 위협과 readiness projection schema
5. Standard schedule과 Endless open-ended profile의 시작값·최소 상승분
6. 대표 콘텐츠 준비 뒤 사용할 headless benchmark encounter, 전술 profile과 승인 지표

구체적인 임시 수치와 최종 계수·경계값은 content/balance authoring이며 정책 미결정이 아니다. 임시 산식은
대표 콘텐츠가 준비되기 전까지만 사용하고, 이후 focused simulation과 실제 playtest 근거로 재승인한다.
현재 구현 여부와 exact schema는 위 completion review, durable canonical과 runtime/live data로
확인하며 이 문서만을 runtime 증거로 사용하지 않는다.
