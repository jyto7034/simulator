# A03 — 실행 identity·상태·사건·원자성 권장안

**권장 설계**다. 현재 BattleCore의 queue·source snapshot·rollback 경계는 구현 AI가 원문으로 추적한 뒤 재사용한다. 본 문서는 전체 이벤트 순서를 현재 코드와 대조 없이 강제로 바꾸는 지시가 아니다.

## 1. identity와 출처

다음은 서로 대체하지 않는 개념이다.

| identity | 의미 |
| --- | --- |
| ContentRef / EffectSiteId | namespaced definition과 형태 안의 stable 효과 위치 |
| OwnedCopyId / EmployeeId | 런 자산 사본과 직원 identity |
| AttemptId / UnitInstanceId | 전투 도전과 그 안의 배치 인스턴스 |
| ActivationId / ActionId | 유지 상태와 직접 행동 한 번 |
| StrikeId / SegmentId | 한 행동의 직접 타격과 그 타격을 구성하는 피해 항목 |
| DeliveryId / EffectInstanceId | 독립 전달과 생성된 지속 효과 |
| OccurrenceId / ChainPathId | 실제 사건과 그 인과 경로 |

제안 `SourceContext`에는 `origin_effect`, `source_copy`, `executor`, `provider_chain`, `resource_owner`, `beneficiary`, `credit_owner`, `damage_kind`, `cause`를 필요한 타입으로 보존한다. 전부 같은 유닛이라고 default하지 않는다. source kind를 단순 `RelicDamage` 하나로 뭉개 원래 기본 공격 의미를 잃게 하지 않는다.

게임play identity와 RNG는 디버그 로그가 찍힌 횟수에 의존하지 않는다. 같은 seed/ID/입력/환경에서는 진단 on/off와 컨테이너 순회 순서에 독립적이어야 한다. ID 변경은 RNG 입력을 바꿀 수 있으므로 이름 바꾸기 시험은 비확률 의미와 hardcode 여부를 보고 bit 동일성을 강제하지 않는다.

## 2. 정의·binding·state의 분리

```text
ImmutablePlan
   ├─ EffectSite / compiled expression / action flow
   └─ SourceSlot (논리 참조)

BindingTable
   └─ SourceSlot → 실제 사본·직원·형태·도메인 owner

RuntimeState
   └─ (owner, lifetime instance, effect site, local key) → 값/handle
```

`StateOwner`는 사본·직원·공유 원천을 식별한다. `Lifetime`은 Run/Attempt/Deployment/Activation/Delivery 등을 식별한다. **owner와 lifetime은 직교**한다. 사본 소유 상태도 deployment 종료에 초기화할 수 있고, 직원이 재배치되어도 attempt 사용 기록은 남길 수 있다.

유물 전투당 기록의 권장 key는 `(AttemptId, RelicCopyId, RelicEffectSite)`이고, 배치 임시 스택에는 추가로 deployment epoch를 붙인다. instance generation을 포함한 handle을 사용해 이전 종료 이벤트가 재사용된 슬롯에 적용되는 것을 차단한다.

## 3. ActionHandle과 진행 상태

제안 계약:

```text
StartAction(request, source, owner_scope)
    → Rejected(reason) | Accepted(ActionHandle)

ActionHandle의 진행:
    Admitted → Released → FlowFinished → EffectsSettled
    어느 단계에서든 관련 contract에 따라 Failed / Canceled 가능
```

Accepted는 접수 응답이며 효과 해결 완료가 아니다. 단계가 모두 항상 존재하는 것은 아니므로 operation descriptor가 가능한 milestone과 finite completion을 선언한다. waiter는 무엇을 기다리는지 명시한다. 상태형 aura나 자립 소환처럼 종료가 외부에 달렸다면 EffectsSettled를 무조건 기다리지 않는다. 종료 보장이 없는 wait는 timeout·취소·수명 경계를 요구한다.

하나의 handle은 여러 milestone을 한 번씩 발표할 수 있지만 각 milestone과 terminal result는 중복 delivery로 두 번 소비되지 않는다. 실제 공격이 빗나가도 action은 정상 완료할 수 있다. `outcome`과 `effects_applied/actual_damage/hit_count`를 분리한다.

Released 이후 소유 제어 작업이 끝나도 전달은 승인된 수명을 따른다. controller의 취소 token과 released delivery의 token을 분리한다. 반대로 생성되지 않은 후속 타격까지 무조건 살려두지 않으며 `CancelScope` 계약을 명시한다.

## 4. 행동 상태의 권장 서비스

| 서비스 | 저장과 변경 원칙 |
| --- | --- |
| ModifierLedger | 기본값 + source handle별 modifier에서 유효값 파생. 제거는 handle 삭제; 반대 숫자 덧셈·역수 복구 금지 |
| ActionLease | owner와 막는 행동 집합·종료 조건 보존. 자기 lease만 해제 |
| AuraMembership | 현재 수혜자 집합에 add/remove delta. 이동·생성·철수·상태 변화의 정해진 관찰 지점에서 갱신 |
| ResourceStore | 값·cap·owner/lifetime·revision. grant/consume/decay를 typed transaction으로 적용 |
| ShieldPool | ResourceStore의 원천 pool + 수혜자 membership. 초기량 modifier와 잔량 소비 분리 |
| RelationStore | node/edge identity와 source scope. 진화/삭제에 따른 invalidation |
| OutcomeReceiptStore | 후속 조건이 요구하는 요약 결과만 수명 안에서 유지. 전체 로그를 truth로 사용하지 않음 |

## 5. 사건과 변경 단계

변경 요청과 이미 확정된 사실을 같은 이벤트로 만들지 않는다. 읽기 전용 관찰·계산 전 modifier·확정 후 반응의 권한을 나눈다.

피해 처리의 **부분 순서 제약**:

```text
대상/적중 자격 판정
 → (타격 인정 시) Strike occurrence와 이번 타격 인덱스 확정
 → 이번 타격에 적용할 조건부 수치 계산
 → 피해 산술·감쇄
 → 흡수·HP 손실·생존 중재
 → HP·pool·해당 스택 등의 원자적 commit
 → DamageReceipt / 필요 시 DeathReceipt
 → 확정 사실을 읽는 후속 반응
```

이것은 전체 battle queue의 전역 우선순위를 확정하지 않는다. lethal-hit/death 전파, 타이머/재입력, 여러 actor phase transition의 순서는 PG03에서 기존 계약과 비교해 표로 고정한다. 깊은 쐐기의 현재 타격 보너스를 이미 적용한 피해를 나중에 고쳐 구현하지 않는다.

`DamageReceipt`는 attempted/raw/mitigated/absorbed/actual_hp_loss/overkill/kill과 source·action·strike를 구분한다. `HealReceipt`는 요청량과 actual restoration을 구분한다. HP 비용은 DamageReceipt를 위조하지 않고 CostReceipt를 만든다.

DeathReceipt는 cleanup 전에 필요한 status/source/position 사실을 확보한다. 같은 unit의 death epoch는 한 번만 확정한다. 정상 사망 bookkeeping과 선택적인 OnDeath/OnKill 반응은 분리한다. 기본 공격 OnAttack/OnHit는 기존 의미를 유지하고 새로운 피해 사건을 별도로 제공한다.

## 6. 큐·원자성·오류

사건 subscriber가 피해 처리 stack에서 재귀적으로 world를 변경하지 않도록 한다. 일반 경로는 request→검증/계산→commit→facts→후속 request queue다. 계산 전 hook은 현재 transaction에 참여하는 검증된 제한 연산이며 임의 effect 실행 callback이 아니다.

권장 기본 단위는 **현재 공개 command 또는 simulation advance가 노출하는 원자적 경계**다. 구현 AI는 현재 경계를 조사하여 journal/copy-on-write/staged plan 중 방법을 선택한다. 임의로 “현재 effect 하나만 복원”해서 이미 공개한 성공 결과가 남는 구조로 축소하지 않는다.

같은 boundary의 예산 초과는 그 경계에서 아직 게시되지 않은 state·queue·RNG·ID·log를 복원하고 명시적 runtime fault를 반환한다. 이미 이전 timestamp에 확정된 과거 전체 연쇄를 되감는다는 뜻은 아니다. fault 이후 정상 전투 완료로 계속 밀어붙이지 않고 세션을 fault 상태로 보존하여 진단 또는 재시작을 제공한다. fault 표현과 pause/resume 포트는 DG에서 정한다.

동일 시각 병행은 CPU 병렬 mutation을 뜻하지 않는다. 고정된 입력 관찰 시점과 stable 실행 순서·합류 규칙을 계약으로 제공한다. 다음 wave spawn과 terminal은 필요한 actor death·boss/summon 정리가 반영된 승인된 boundary에서 평가한다.

## 7. 연쇄 인과와 유한성

`ChainContext`는 root occurrence와 persistent path frame을 가진다. 유물 반응 직전에 `(copy,effect)` 재방문을 검사하고 통과 시 child frame을 만든다. sibling branch는 각자 ancestor frame을 공유·확장하지만 전역 visited set을 공유하지 않는다. delayed child는 parent ancestry를 유지한다.

진짜 여러 부모의 결과를 하나로 합치는 join은 부모 context를 명시적으로 받는다. 권장안은 그 join에 실제 참여한 경로의 ancestor 제약을 합성하는 것이다. 이는 서로 관계없는 sibling 전체를 전역 합치는 것과 다르다. exact join 정책은 PG02의 검토 대상이다.

Timer마다 새 root를 만들지 않는다. 도메인이 생성하는 독립 regular attack·외부 입력과, proc가 만든 extra attack·지연 피해를 구분한다. 합법적인 주기 event도 고유 occurrence ID는 새로 받지만 inherited causal constraints의 제거 여부는 producer contract로 정한다. 임의 authoring flag로 순환을 세탁할 수 없게 한다.

Runtime 보장에는 recursion guard 외에도 fan-out·instance·event budget이 필요하다. 숫자는 CA22에서 최초로 도입하는 것이 아니라 기능 추가부터 계측·방어하고 CA22에서 실제 부하로 고정한다.

## 8. 저장·후퇴·재배치

Run 데이터와 attempt의 overlay를 분리한다. 전투 중 Run 귀속 자원 변경도 우선 attempt overlay에 반영하고 결과에서 사본 ID로 정확히 한 번 commit하는 구조를 권한다. Retreat는 entry snapshot과 RNG/ID continuation을 복원한다. Retry는 결과 확정 후 새 attempt인지 rollback인지 구분한다.

SavePoint에는 current schema의 사본·형태·Run 자원·resolved content fingerprint를 저장한다. compiled node index·Arc 주소·진행 중 전투 전체는 저장하지 않는다. 동일 프로세스 재접속은 살아 있는 세션의 조회/재연결이지 새 activation 시작이나 SavePoint load가 아니다.
