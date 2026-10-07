# R06 WP02 D04 유닛 관계 구간 — 독립 단계검토 001

2026-09-27 · 판정: **수정 필요** · 발견 사항: **D04-OWNER-F01 1건**.

대상은 `wp02_d04_owner_relation_001`의 동결 구현이다. 부분 범위 검토이며 WP02/R06 완료 감사가 아니다.
제품 코드와 동결 산출물은 변경하지 않았다. 재현용 계측과 시험은 `/tmp` 격리 복사본에만 추가했다.

## 1. 검토 대상과 신원

- [구현 인계](../../segments/wp02_d04_owner_relation_001/REVIEW_HANDOFF.ko.md)
- [동결 manifest](../../segments/wp02_d04_owner_relation_001/target/d04_owner_relation_001/MANIFEST.json)
- manifest SHA-256: `c2e703c2a08e1fc38141d3d070e052d8207cd7aaf9790f650c685429728a2a9f`
- 확장 337개 source digest: `eb9bd8fb3927f8a9f3c5b35908012300e4fad9c4e8d7a03a9e612e44b0dc804a`
- 제품 333개 source digest: `8bde59151f95ad7301c7451f91af7b69c0a5e61c2a0a46239c1da8dd13f2b7b2`
- [정책·O01–O15](../../../../successors/r06_scope_20260927_001/IMPLEMENTATION_REQUEST.ko.md), [구현 방향](../../handoffs/WP02_D04_OWNER_IMPLEMENTATION_GUIDE_20260927.ko.md)

변경 32개 경로를 대조하고 관계/필터 의미, 실제 장착 생산 경로, node별 view 선택,
Snapshot/Live 연결, parser/compiler/fingerprint 및 비용 계량을 확인했다.
진입 currentness와 양방향 복원 결과는 [CURRENTNESS_AND_RESTORATION.json](CURRENTNESS_AND_RESTORATION.json),
종료 재확인은 [EXIT_CURRENTNESS_AND_RESTORATION.json](EXIT_CURRENTNESS_AND_RESTORATION.json)에 있다.
복원은 격리 디렉터리에서 수행하며 workspace에 patch를 적용하지 않았다.

## 2. D04-OWNER-F01 — 장비 관계 조회의 전역 탐색 비용 누락

우선순위 **P2**. O12와 구현 방향 §7의 관계 조회 작업량 계량 요구를 충족하지 못한다.

위치: `src/game/battle/core/value_expr_runtime.rs`의 `resolved_slot_binding()` 1545–1554행,
`prepare_resolved_target_members_with_base()` 1152–1161행 및 1286–1433행.

실제 사본이 있는 슬롯을 검사할 때마다 `self.items.values().filter(...).count()`로
**해당 전투의 모든 RuntimeItem**을 방문한다. 같은 슬롯을 `HasEquippedSlot` 조건과
`EquippedSlot` 그룹 기준으로 쓰면, member 수 산정과 view 생성의 두 순회에서 총 네 번 실행한다.
준비 작업량 공식에는 targets/aggregate/predicate/relation/ref 수만 있고 items 수는 없다.
전역 탐색 자체에도 charge/poll이 없다.

이 때문에 집계에 참여하지 않는 유닛의 장비를 늘려도 안전장치가 관측하는 작업량은 증가하지 않는다.
장비를 가진 대상이 N명이고 전투 장비가 I개라면 이 경로의 실제 탐색은 O(N×I)로 늘어난다.
여러 aggregate가 있으면 추가 반복된다. 일반적인 작은 전투가 즉시 느려진다거나 실제 중단이 발생했다고
판정한 것은 아니다. **입력 크기에 따라 증가하는 작업이 기존 제한에서 빠져 있는 결함**이다.

### 독립 재현

기존 `owner_fixture`를 복제한 시험 helper에 정상 scenario spawn으로 직원과 각각의 실제 무기 사본을 추가했다.
이미 배치된 전투에서 A 한 명만 집계하고, A의 Weapon 조건과 Weapon 사본별 그룹을 사용했다.
기존 filter closure 내부에 atomic 방문 카운터만 추가했으며, 조회 결과나 제한 계산은 변경하지 않았다.

| 집계 밖 추가 직원 | 전투 장비 수 | 집계 대상 | 실제 장비 방문 | authoring 기록 work | max_work | 결과 |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 3 | 1 | 12 | 149 | 256 | 허용 |
| 100 | 103 | 1 | 412 | 149 | 256 | 허용 |
| 1,000 | 1,003 | 1 | 4,012 | 149 | 256 | 허용 |

work 단위를 CPU 명령 수나 시간과 동일시하지 않는다. 이 결과는 실제 순회량 증가가 계량에 전혀 반영되지
않음을 보여준다. 초기 max_work=64는 기존 retained admission에서 requested=149로 거부되어 보존했고,
고정 준비 비용을 허용하는 256으로 재시험했다. 준비를 제외한 spawn 비용은 이 측정 범위 밖이다.

증거: [REPRO_TEST.log](REPRO_TEST.log), [시험 코드](REPRO_TEST.rs), [계측 신원](REPRO_SETUP.json),
[초기 한도 시험](REPRO_TEST_INITIAL_BUDGET.log). 재현 시험의 `1 passed`는 **결함 관측 assertion 통과**이며
교정 완료를 뜻하지 않는다.

재현하려면 `python3 PREPARE_REPRO.py`가 출력한 디렉터리의 `core_enhanced`에서 다음을 실행한다.
이 도구는 동결 archive/manifest/source SHA를 검사한 뒤 임시 디렉터리만 작성한다.

```bash
CARGO_HOME=/tmp/wp02-cargo-home CARGO_TARGET_DIR=/home/blast/.cache/d04-owner-probe-review-target /home/blast/.cargo/bin/cargo test -p game_core --lib review_equipment_scan_work_is_not_charged --offline --locked -- --nocapture --test-threads=1
```

### 교정 방향과 재검토 조건

1. 매 조회의 전투 전체 장비 스캔을 제거하는 방향을 우선한다. 두 spawn 경로의 기존
   `make_item_instance_id(base_uuid, side, unit_id, equipped_items_index)`로 만든 **정확한 runtime item ID**를
   runtime 슬롯 projection에 연결하면 `items.get(id)`로 조회할 수 있다.
   `equipped_items`의 순번과 Weapon/Armor/Accessory 슬롯 주소를 혼동하면 안 된다.
2. 직접 조회 후에도 runtime ID, 소유 유닛, definition UUID, 실제 copy UUID의 일치 검사는 유지한다.
   중복 사본/슬롯 검증은 생성 경계에 유지한다. stale/nil/누락을 성공이나 자동 제외로 바꾸지 않는다.
3. 직접 조회 비용을 기존 준비/authoring 계량에 포함한다. 전역 탐색을 유지한다면 실제 방문 범위의
   checked 비용과 취소 확인을 탐색 전에 보장해야 하며, 다중 node·두 pass·필터 경로를 모두 계산한다.
   단순히 max_work 상한을 올려 통과시키는 것은 교정이 아니다.
4. projection이 커지면 retained extent/clone/scratch 재사용을 함께 반영한다.
   새 조회 구조를 추가한다면 생성·복제·메모리·갱신 비용도 계량한다.
5. 집계 밖 장비를 늘렸을 때 직접 조회 횟수가 일정하거나, 실제 증가분만큼 계량되어 한도에서 거부되는
   회귀 시험을 추가한다. 여러 member/node, 필터와 관계의 동시 사용, stale identity 및 낮은 한도도 확인한다.
6. O01–O15·D04-F01 회귀와 기존 검증을 유지하고 successor target을 새로 동결하여 독립 재검토한다.

기존 gameplay 의미를 바꾸는 새 정책 결정은 필요하지 않다. 제품 교정 구현은 별도 구현 대화의 작업이다.

## 3. 그 밖의 검토와 검증

- Side/실제 Employee/명시 슬롯의 copy 관계를 구별하며 소환물의 owned UUID를 직원으로 오인하지 않는다.
- 조건은 aggregate 입력만 바꾼다. node별 view 조회에서 다른 node/legacy 입력으로 fallback하지 않는다.
- inventory의 두 projection과 두 spawn 경로가 layout/address/copy를 전달하며,
  clone/scratch/retained 경로에 새 projection을 반영했다.
- 기존 source/group fingerprint 태그를 유지하고 새 variant의 payload를 명시적으로 encoding했다.
- Snapshot release에서 view를 준비하고 Live effect의 mutation 전에 view를 준비하는 연결을 확인했다.
  O09의 새 시험은 준비된 view 불변성을 직접 확인한다. 모든 새 관계를 실제 projectile로 교차 시험한 것은 아니며,
  기존 delivery 회귀와 실제 호출 경로 대조를 함께 근거로 삼았다.
- 집중 시험은 독립 재실행 **83 passed / 0 failed**: [FOCUSED_TESTS_RETRY.log](FOCUSED_TESTS_RETRY.log).
  첫 실행은 offline Cargo cache 설정 누락으로 실패했으며 [FOCUSED_TESTS.log](FOCUSED_TESTS.log)에 보존했다.
- 전체 제품 시험은 원본 전용 새 빌드에서 **1,520 passed / 0 failed**, exit0:
  [ALL_TARGETS.log](ALL_TARGETS.log), [검증 영수증](REVIEW_VALIDATION.json). 재현 probe가 포함되지 않았음도 확인했다.
  공유 target을 쓴 첫 전체 실행은 격리 probe의 lib test artifact를 재사용해 1,521개를 실행했다.
  [ALL_TARGETS_SHARED_CACHE.log](ALL_TARGETS_SHARED_CACHE.log)를 보존하되 원본 검증 근거에서 제외하고,
  원본 전용 빈 target으로 다시 빌드·실행했다. `/tmp`의 16GiB tmpfs가 부족해 첫 전용 빌드가 실패했다:
  [ALL_TARGETS_TMP_SPACE_FAILURE.log](ALL_TARGETS_TMP_SPACE_FAILURE.log). 이 검토에서 생성한 실패 빌드
  디렉터리만 제거하고, 여유 공간이 있는 `/home/blast/.cache/d04-owner-original-review-target`으로 옮겼다.
  실패를 제품 시험 실패로 분류하거나 통과로 숨기지 않는다.
  이후 probe 재현 명령도 원본 검증과 별도 target을 사용하도록 위와 같이 구분했다.
- 인계의 build/clippy/전체 시험/PC/R05 영수증과 로그 SHA 및 source 전후 일치를 확인했다:
  [HANDOFF_RECEIPTS_CHECK.json](HANDOFF_RECEIPTS_CHECK.json). 인계 전체 시험 결과는 1,520/0이다.
- PC는 동결 원자료 576행을 선행 target과 재비교했다. 36조건 중 성공31/typed refusal5,
  gameplay·완료·오류 코드 차이0. 비용의 requested 상세 차이80행은 구별해 보존했다:
  [PC_RECOMPARISON.json](PC_RECOMPARISON.json). 이번 검토에서 PC workload 자체를 재측정하지 않았다.
- R05도 workload를 재실행하지 않고 원자료와 요약을 대조했다:
  [R05_RAW_CHECK.json](R05_RAW_CHECK.json). concurrent peak 최대9,049,432 bytes,
  기존 cap 대비19,244 bytes 초과는 **합의된 경고**다. 당시 exit101을 성공으로 바꾸지 않는다.
  이 peak 경고가 이번 수정 필요 판정의 원인은 아니다.

인계 O12의 “limit 6 경계의 5 성공” 문구는 실제 시험의 `limit=6 성공 / limit=5 거부`와 구별해 읽는다.
인계 말미에 “남아 있다”고 함께 기재된 자동 스킬 취소·20분 시험은 승인된 잔여 작업이 아니다.
합의에 따라 제외되어 있으며, 이를 새 완료 조건으로 추가하지 않는다.

## 4. 인계 상태

**D04 owner 관계: 구현·자체 검증 완료 → 독립 검토 수정 필요 → F01 교정 후 재검토.**
O12 미충족으로 이번 부분 범위 PASS는 보류한다. D07, WP03, WP04로 자동 진행하지 않는다.
production publication 차단을 유지한다. 기존 target과 실패/경고 증거는 수정하거나 덮어쓰지 않는다.
현재 작업은 계속 R06이며 별도 실험실 로드맵이나 R05 재개로 전환하지 않는다.
