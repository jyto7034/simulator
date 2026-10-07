# A08 — 작성→컴파일→실행을 연결하는 설계 예제

아래 모든 문법은 **RON 형식의 제안 의사자료**다. 현재 parser가 받는 문법이나 실행 검증된 새 콘텐츠가 아니다. 숫자는 별도 시험용이며 공식 콘텐츠의 밸런스·미정 정책을 승인하는 의미가 없다. 구현 AI는 선택한 구조를 실제 schema로 옮긴 뒤 정식 fixture와 receipt를 만들어야 한다.

## E01 — 범위 피해 뒤 한 번 자기 회복

풍운승려의 구조를 시험하는 최소 입력. 이 시험에서는 적 둘에게 각각 HP 손실이 발생하고 시전자도 회복 가능한 상태라고 가정한다. 보호막만 맞힌 경우의 유효 적중 의미는 이 예제로 확정하지 않는다.

```ron
// 제안 의사자료: 타입/함수는 미구현
(
    id: "example:burst_with_recovery",
    effects: [
        (id: "bite", role: "primary_damage", tags: ["area", "physical"],
         scope: Area(anchor: AcceptedTargetTile, shape: Circle(radius_tiles: 1)),
         operation: Damage(kind: Physical,
             amount: Mul(ReadStat(Caster, Attack), Ratio(200, 100)),
             evaluate_at: Release)),
        (id: "recover", role: "followup_self_heal", tags: ["heal"],
         scope: SelfUnit,
         operation: Heal(amount: Mul(ReadStat(Caster, MaxHp), Ratio(10, 100)),
             evaluate_at: Execute)),
    ],
    flow: Sequence([
        Invoke(effect: "bite", wait: EffectsSettled, result: "bite_receipt"),
        Branch(
            when: Greater(ReadResult("bite_receipt", ActualHpLossTotal), 0),
            then: Invoke(effect: "recover", wait: EffectsSettled, result: "heal_receipt"),
            otherwise: NoOp,
        ),
    ]),
)
```

Compiler는 `bite_receipt`가 먼저 생산되고 분기에서 참조 가능한지, Heal의 scope가 SelfUnit인지, Attack·MaxHp 식의 타입·평가 시점을 검사한다. op별 결과는 result slot에 배치하고 작성 ID와 source map을 유지한다.

시험 입력은 Attack 100, 시전자 최대 HP 1000/현재 HP 500, 적 두 명 HP 500/방어 0, 추가 modifier 없음이다. 문서상 기대값은 각 적 HP 300, 시전자 HP 600이다. 실제 수치 결과는 damage kernel의 승인 산술에 맞춰 테스트한다. 피해 대상 수만큼 자기 회복을 반복하지 않는다.

범위 유물은 `primary_damage`의 footprint만 변경한다. 적이 셋이 되어도 회복 100은 한 번이며 `recover`의 scope/source/lifetime은 전후 동일해야 한다. 이 예제의 ActualHpLossTotal 조건은 시험 프로필의 선택이며 원문의 “유효 적중” 전체 의미는 PG03에서 확정한다.

## E02 — 보스의 사격 완료 지점과 취소

캐릭터 고유 기믹을 발명하지 않고 기존 스킬·이동 기능을 조합하는 **보스 인수용 fixture**다.

```ron
// 제안 의사자료: Pattern 부분만 발췌
(
    id: "example:shot_then_rejoin",
    parameters: [(id: "attack", kind: SkillRef), (id: "rejoin_timeout", kind: Duration)],
    entry: "shoot",
    states: [
        (id: "shoot",
         tasks: [CastSkill(id: "shot", skill: Parameter("attack"),
             target: LockedTarget, aim: FromTargetSnapshot,
             wait: Released, on_state_exit: DetachReleased)],
         transitions: [
             OnResult(task: "shot", result: Completed, next: State("rejoin")),
             OnResult(task: "shot", result: TargetUnavailable, next: Outcome("no_target")),
             OnResult(task: "shot", result: ActionBlocked, next: Outcome("blocked")),
             OnResult(task: "shot", result: Canceled, next: Outcome("canceled")),
         ]),
        (id: "rejoin",
         tasks: [RejoinRoute(id: "return", timeout: Parameter("rejoin_timeout"))],
         transitions: [MapAllNormalResults(task: "return", next: Outcome("done"))]),
    ],
)
```

실제 닫힌 result variant와 catch-all 지원은 CA16-D에서 확정해야 한다. 이 예제의 arms가 현재 enum의 완전한 목록이라고 주장하지 않는다.

시간 fixture: t0 접수, t100 release, t300 impact라고 주입한다. Released를 기다리는 보스는 t100에 복귀 작업을 시작하고 투사체는 t300에 독립 해결된다. EffectsSettled를 기다리는 변형은 t300 이후 결과를 보고 전이한다. 두 문법은 동일하다고 처리하면 안 된다.

t50 상태 전환은 아직 release하지 않은 스킬 취소를 검증한다. t150 전환은 이미 날아가는 투사체를 강제로 지우지 않는 경우를 검증한다. 이전 state generation의 완료가 나중에 와도 새 state를 전이시키지 않는다. boss 이름을 바꾸고 같은 parameterized pattern을 재사용한다.

## E03 — 공유 보호막을 단일 수혜자로 제한

```text
ShieldEffect site = spore_pool
Resource owner = caster가 만든 effect instance
Membership = 원래 자격을 갖춘 수혜자 집합
Initial capacity = S (원문 공식 식은 미정)

generic transform:
  require SharedShield + MutableRecipientScope + InitialCapacity
  restrict membership to SelectedEligibleUnit
  set initial capacity expression to 2 * S
  preserve resource owner / duration / resource-exhausted end / cast source
```

Compiler는 초기량 변경을 pool 생성에 한 번만 결합한다. 수혜자별 loop 안에서 2배를 다시 적용하지 않는다. 원천 owner는 selected recipient가 아니라 기존 source다. SingleRecipient 변환과 accessory 증가의 정확한 합성은 PG01/PG04 결정이 필요하다.

공유량 600·A 피해 400·B 피해 350의 산술 fixture에서는 감쇄된 양을 순서대로 소비하여 B HP만 150 감소한다. 단일 전환 시험은 원래 대상 자격·개인/공용 우선순위를 따로 주입한다. 원문의 침식 4S 비교안이나 S 계산식을 여기서 확정하지 않는다.

## E04 — Hello / Goodbye

```text
AbilityState Idle
  normal cast accepted → 공명 비용 commit → Hello activation 시작
Hello
  basic attack에 PiercingDelivery override lease
  일반 기본 공격·저지 병행
  timeout → lease 제거 → Idle (Goodbye 없음)
  accepted reinput → remaining duration snapshot
                   → Goodbye skill damage action
                   → lease 제거 → Idle
```

compiler는 추가 입력이 어느 activation을 대상으로 하는지, 비용이 새 일반 시전과 다른지, 모든 종료 경로가 lease를 정리하는지 검사한다. 런타임은 expiration과 reinput 중 승인된 하나의 전이를 commit한다.

Hello basic attack은 무기 직접 공격 provenance, Goodbye는 직접 skill provenance다. 원본 무기를 교체한 게 아니라 temporary action profile overlay를 씌운다. 처음 값을 저장했다가 다른 modifier 변경까지 덮어 복원하지 않는다. 관련 없는 공격·보호 효과는 자신의 수명을 따른다.

## E05 — 태아 집중과 단일 전환

채널 activation이 ActionLease와 AuraMembership을 소유한다. regular basic attack 피해·속도만 변경하는 modifier를 수혜자에게 바인딩한다. 공급자의 Attack stat 전체를 올리거나 소환물·파편 피해를 자동 증폭하지 않는다.

집중 시작→범위 내 다른 eligible 직원 수혜→입장/이탈 delta→집중 종료와 lease/membership 제거를 검증한다. 이미 release된 공격은 고정된 snapshot, 그 뒤 새 공격은 새 modifier 집합을 사용한다. windup 속도 반영 경계도 원문과 대조한다.

단일 전환은 recipient 선택과 basic damage/speed 두 property를 각각 바꾼다. 승인된 상태별 +20/+40/+80%와 +15/+20/+30은 교체값이며 모두 배율 곱으로 해석하지 않는다. 집중 최대시간·저지·침식 penalty는 미정 gate다.

## E06 — 래티씨아 진화·연결과 원천별 전파

빈 eligible tile에서 `TrapDefinition(small)`을 생성한다. 내 source가 만든 small이 있는 tile에서 다시 시전하면 비용 commit과 `ReplaceObjectForm(big)`을 같은 transaction으로 처리한다. 큰 친구가 있으면 잘못된 추가 중첩을 거부한다. 진화의 node identity 유지/교체 세부는 DG에서 선택하되 old edge·trigger·event가 새 node에 잘못 적용되지 않아야 한다.

전용 유물은 지정된 small/big 생성 효과만 선택해 `ProximityRelation`에 가입시킨다. 접근 또는 segment-crossing 사건이 network activation을 생성하고, 정한 순서의 attack→consume을 진행한다. consumed node와 stale 링크는 다시 발동하지 않는다. 전체 소비/유효 node만 소비는 원문 미정이다.

별도 감염 fixture는 같은 relation 서비스를 강제로 쓰지 않아도 된다. StatusMembership + ResourceAggregator로 source별 살아 있는 숙주를 집계하고 detonation transaction에서 선택 집합과 위력을 snapshot→소비→피해 요청한다. 그 detonation context에는 재감염·사망 전파 suppression을 명시한다. 감염 membership과 설치물 graph를 공통 이름 때문에 하나로 합치지 않는다.

## 7. 예제가 구현에서 증거가 되는 조건

각 E 항목을 RON 파일·기대 trace/state·정상/실패/종료 case로 변환하고, 실제 current parser/compiler/consumer로 실행한 명령·revision·결과를 남긴다. 이 문서의 손계산·도식만으로 테스트 통과를 기록하지 않는다.
