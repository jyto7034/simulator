# R06 WP02 D04-OWNER-F01 — 독립 재검토 001

2026-09-27 · **부분 범위 PASS** · **D04-OWNER-F01 해소** · 추가 수정 요구 없음.

교정 target에서 전역 장비 탐색이 제거되었고, 고정된 직접 조회 비용이 기존 준비 작업량 상한 안에 들어감을 확인했다.
이 판정은 앞선 owner 관계 단계검토의 보류 사유를 해소한다. 이번 D04 유닛 관계 범위의 단계검토를 통과로 기록한다.
WP02/R06 전체 완료, production publication 해제 또는 독립 완료 감사 판정은 아니다.
제품 코드와 동결 산출물은 변경하지 않았다.

## 1. 대상과 신원

- [교정 지시서](../../handoffs/WP02_D04_OWNER_F01_CORRECTION_REQUEST_20260927.ko.md)
- [선행 독립 검토](../wp02_d04_owner_relation_001_review_001/REVIEW.md): D04-OWNER-F01, 당시 수정 필요 판정은 보존한다.
- [교정 구현 인계](../../corrections/wp02_d04_owner_f01_001/REVIEW_HANDOFF.ko.md)
- [successor manifest](../../corrections/wp02_d04_owner_f01_001/target/d04_owner_f01_correction_001/MANIFEST.json)
- manifest SHA-256: `58505bbcd34089b9c1ec3d70f4d9df4ca32a2397e1115c68386364e44514e549`
- before 337개 digest: `eb9bd8fb3927f8a9f3c5b35908012300e4fad9c4e8d7a03a9e612e44b0dc804a`
- after 337개 digest: `8aa03d6bd04f8d524fa61b465a9bd7a4171062a1190f615b794e20f915190f04`
- after 제품 333개 digest: `8f6a58059254d4db4dc726bc8aeb2a6c6ec348e9aef685052837df903fd94bcd`

before가 선행 owner target의 after와 일치한다. 변경은 `core/build.rs`, `core/ids.rs`,
`core/types.rs`, `core/value_expr_runtime.rs`, `world/tests/combat.rs`의 5개 경로다.
337개 source의 currentness, 증거70개와 archive 신원, 정·역 patch의 격리 byte 복원을 확인했다.
진입/종료 결과는 [ENTRY_CURRENTNESS_AND_RESTORATION.json](ENTRY_CURRENTNESS_AND_RESTORATION.json)과
[EXIT_CURRENTNESS_AND_RESTORATION.json](EXIT_CURRENTNESS_AND_RESTORATION.json)에 기록한다.

## 2. F01 해소 판단

### 전역 탐색 제거와 비용 대응

`value_expr_runtime.rs::resolved_slot_binding`은 선택된 entry의 runtime item ID로
`items.get(&entry.runtime_item_instance_id)`를 최대 한 번 수행한다. 기존 `items.values()` 순회와
대체 검색 fallback은 없다. [CODE_RECHECK.json](CODE_RECHECK.json)은 해당 함수의 직접 조회1곳,
전체 순회0곳, identity 비교4종을 코드 대조한 보조 증거다.

`prepare_resolved_target_members_with_base`는 모든 aggregate spec의 predicate와 relation 수를 합산한다.
count pass와 view construction pass 각각 대상당 최대 한 번 호출하므로,
canonical 대상 N명·슬롯 predicate P개·장비 relation R개의 조회 횟수는 `2*N*(P+R)` 이하이다.
raw target 수로 계산한 기존 `2*predicates + 2*relation_views` 항은 이를 보수적으로 포함한다.
준비 전 checked 계산, ExprLimits 확인, authoring→native charge 및 evaluator 별도 meter도 유지되어 있다.

새 회귀 시험을 포함한 독립 전체 실행에서, 집계 밖 장비0/100/1,000개에 대해 조회4/4/4회 assertion이 통과했다.
이전 target의 전체 방문12/412/4,012회와 달리 **장비 identity 조회가 전투 전체 장비 수에 따라 늘어나지 않는다.**
이는 해당 조회 경로의 결론이며, retained 합산을 포함한 준비 함수 전체가 상수 시간이라는 주장은 아니다.
work를 CPU 명령 수나 실행 시간과 동일시하지 않는다.

### 생성 경로와 identity

`build.rs`의 두 spawn 경로는 unit ID 계산 후 projection을 만들고,
`draft.equipped_items.enumerate()`의 원 순번으로 장비 ID를 생성한다.
`prepared_item_instance_id`는 known-layout entry의 동일 ID를 provider 등록과 RuntimeItem 삽입에 전달한다.
legacy layout=None 경로는 기존 ID 생성 방식을 유지하며, 장비 loop/등록 순서는 바뀌지 않았다.

Armor-only의 슬롯 칸1과 원 목록 순번0, Any3 역주소 fixture를 구별한다.
`ids.rs::make_item_instance_id`의 stack buffer는 기존37바이트의 순서와 seed/namespace를 그대로 유지한다.
그룹 키는 계속 persistent copy UUID이며 runtime item ID로 바뀌지 않았다.

직접 조회 뒤 내부 instance ID, owner unit, base definition, persistent copy를 모두 확인한다.
nil ID·누락 item·다른 유닛 장비·각 identity 손상은 InvalidOwner다.
unknown layout/정상 빈 슬롯/copy 미제공의 기존 구별과 생성 경계의 중복 검증을 유지한다.

### retained 및 공유

`RuntimeEquipmentSlot`은 binding과 runtime ID를 함께 저장한다.
`BattleEquipmentSlots::retained_extent()`의 `size_of::<Self>() + Arc header`와 기존 생성 전 admission,
runtime fact extent 합산이 새 payload 크기를 반영한다. 해당 x86_64 인계 측정은122→170 bytes, +48 bytes다.
RuntimeUnit의 Arc handle, CloneShape의 공유 계약, timestamp scratch 재사용은 유지된다.
독립 실행에서 새 projection extent보다1byte 부족한 거부/정확한 한도 허용과 Arc 공유 시험이 통과했다.

## 3. 요구 대응과 직접 실행

독립 재검토는 source/차이/시험 assertion을 읽고, 현재 source를 **별도 새 product target**에서 빌드하여
`cargo test -p game_core --all-targets --offline --locked`를 실행했다.
결과는 **1,526 passed / 0 failed, exit0**이다. 별도 probe나 외부 절대경로 시험 hook을 추가하지 않았다.

| 범위 | 확인한 근거 |
| --- | --- |
| C01–C02 | 실제 get 직전 thread-local 계측. 배경 장비 증가 시4회 고정, predicate/relation2명 각각4/4/8회, 다중 슬롯 node12회(상한16) |
| C03 | 기존 filtered input/낮은 work/native retained/취소 거부 회귀 재실행과 두-pass 비용식 대조. 이번 교정은 한도를 올리거나 별도 무제한 예산을 만들지 않음 |
| C04 | 잘못된 슬롯 ID, nil ID, map 누락, internal instance/owner/base/copy 개별 손상의 typed 거부 |
| C05–C06 | ByType 앞 빈 슬롯/Any3 원 순번, 실제 inventory의 AtBattleStart·Wave에서 copy별 projection/provider runtime ID 연결 |
| C07 | projection retained 경계, binding+ID 및 Arc clone/scratch 공유 유지 |
| C08 | 기존 Snapshot/Live·ordered effect rollback·실제 capture identity·D04-F01 무대상 projectile 회귀 유지 |
| C09 | 예전 Vec 구성과 동일한 UUID 결과를 Player/Opponent·owner/definition/salt 변화로 비교.37바이트 slice 구성도 직접 대조 |

로그: [ALL_TARGETS.log](ALL_TARGETS.log), 명령·env·결과·SHA: [REVIEW_VALIDATION.json](REVIEW_VALIDATION.json).
기존 O01–O15 구현과 시험을 삭제하거나 완화한 변경은 없다. C03/C08은 선행 회귀와 코드 경계 대조를 함께 사용했다.
새 정책이나 다음 기능의 시험을 이 교정에 추가하지 않았다.

## 4. 인계 증거와 비용 판정

선택된14개 영수증의 로그 SHA, exit, 최종337개 source 전후 동일성을 확인했다:
[HANDOFF_RECEIPTS_CHECK.json](HANDOFF_RECEIPTS_CHECK.json).
인계의 check/clippy·88개 value_expr·43개 장비·9개 D04-F01 결과는 해당 source에 귀속된다.
이번 재검토에서 check/clippy/각 focused selector를 별도로 반복하지 않았으며 전체 제품 시험을 직접 재실행했다.

PC는 원자료576행을 직전 owner target과 다시 비교했다.36조건 중 성공31/typed refusal5,
gameplay·완료 상태·오류 코드·requested 상세 차이 모두0이다:
[PC_RECOMPARISON.json](PC_RECOMPARISON.json). PC workload 자체는 재측정하지 않았다.

R05는 동결 원자료·요약·panic log를 대조했다. concurrent peak 최대9,041,800 bytes,
원 cap9,030,188 대비11,612 bytes 초과다. event-heavy guard와 다른 raw 판정은 유지되어 있으며
exit101의 assertion은 concurrent allocation/retained/peak guard다. 개별 원자료에서 retained/operations는
선행 값이 유지되고 peak만 초과한 것을 확인했다: [R05_RAW_CHECK.json](R05_RAW_CHECK.json).
합의한 비용 정책에 따라 **peak 단일 경고**로 유지한다. workload를 이번에 재실행했거나 raw exit0이라고 주장하지 않는다.
이번 PASS는 peak를 낮춰서 얻은 판정이 아니라 F01의 전역 탐색을 제거한 판정이다.

### 증거의 한계

[실패 이력](../../corrections/wp02_d04_owner_f01_001/FAILURE_HISTORY.md)에 따르면 개발 초기
Armor-only fixture의 WeaponRequired 실패는 receipt 도입 전이라 stdout 원파일이 없다.
당시 실행을 직접 검증한 것으로 취급하지 않는다. 최종 fixture가 TestFixture로 변경된 이유를 코드로 확인했고,
실제 Employee 생산 경로는 별도 C06에서 검증된다. 최종 target의 같은 시험과 전체 suite를 직접 재실행했으므로
이 누락을 현재 교정의 기능 판정 근거로 사용하지 않았다.
0건 선택된 C09 실행도 통과 증거에서 제외하며, 수정 selector의1건과 독립 전체 실행을 근거로 한다.

## 5. 종료와 다음 단계

**D04-OWNER-F01: 해소 / 이번 D04 유닛 관계 단계검토: 부분 범위 PASS.**
이전 target의 수정 필요 기록과 동결 증거는 그대로 보존하며, 이 successor 재검토가 이후 판단의 근거다.
WP02·R06 전체 완료나 production publication 해제는 선언하지 않는다.

다음 작업은 [successor 계획 §5](../../../../successors/r06_scope_20260927_001/REVIEW_AND_PLAN.ko.md)에 따라
**D07 정적 explain/실제 frame preview의 구현 범위·지시서를 정리하는 것**이다.
WP03의 최종 stat/effective input·copy/save와 연결되는 D07 완료 조건도 함께 구별해야 한다.
이번 재검토에서 D07·WP03/04 제품 구현은 시작하지 않았다.
20분 시험과 안전 초과 자동 스킬 취소는 합의대로 제외하며 R05 기능 개발이나 별도 실험실 로드맵으로 전환하지 않는다.
