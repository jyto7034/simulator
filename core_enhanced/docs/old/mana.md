## 공명(=마나) 시스템 수정 계획

### 목표/규칙(확정)

- 유닛별 자원: 공명\_current, 공명\_max, 공명\_start.
- 공격 판정이 발생하면(명중 여부 무관) 즉시 +10 공명.
- 피해로 HP가 실제 감소할 때 floor(실제 감소 HP \* 0.1) 공명.
  - 오버킬은 “실제 감소한 HP”만 인정.
- 공명이 가득 차면 해당 공격/피격 처리(데미지/사망 처리 포함)가 끝난 직후 자동 시전 시작.
- 스킬 사용(시전) 중에는 어떤 경로로도 공명 획득 불가(공격/피격/아이템/아티팩트/능력/버프 포함).
- 스킬 시전이 끝나면 공명을 0으로 만들고, 이후 1000ms 동안 공명 획득 락.
- 공명이 가득 찬 “그 순간” 즉사(HP 0)면 스킬은 무시.

———

### 1) 데이터 모델 변경(정적/런타임 분리)

정적 데이터(AbnormalityMetadata)

- 파일: core/src/game/data/abnormality_data.rs
- 필드 추가(기존 데이터 호환 위해 #[serde(default)] 사용 권장)
  - resonance_start: u32
  - resonance_max: u32
  - (선택) resonance_lock_ms: u64 (기본 1000)

런타임 상태(RuntimeUnit)

- 파일: core/src/game/battle/core/mod.rs
- RuntimeUnit에 추가
  - resonance_current: u32
  - resonance_max: u32
  - resonance_gain_locked_until_ms: u64 (락)
  - casting_until_ms: u64 (시전/채널링 동안 공명 획득 차단 + 기본공격 차단)
  - pending_cast: bool 또는 pending_cast_at_ms: Option<u64> (중복 발동 방지)

———

### 2) 공명 획득 훅(단일 진입점으로 통일)

공명은 어디서든 “함수 2개”만 통해 변경되게 한다(추적/결정성/버그 방지).

- fn can_gain_resonance(unit, now_ms) -> bool
  - 조건: now_ms >= resonance_gain_locked_until_ms AND now_ms >= casting_until_ms
- fn add_resonance(unit, amount, now_ms)
  - can_gain_resonance 불가면 no-op
  - clamp: min(current + amount, max)
  - 만땅이면 “즉시 시전 예약 플래그”만 세팅(실제 시전은 이벤트 처리 끝난 뒤)

———

### 3) 공명 획득 위치(공격/피격)

공격(+10)

- 위치: BattleEvent::Attack 처리에서 “공격 판정 발생” 시점
- 명중 여부와 무관하게 공격자에게 +10 (락/캐스팅 중이면 0)

피격(+10% of 실제 감소 HP)

- 위치: HP를 실제로 감소시키는 코드 바로 옆
  - 기본공격 피해 적용 직후
  - ApplyHeal(flat<0) 등 “음수 힐 = 피해” 적용 직후
- 계산: gained = floor(actual_hp_decrease \* 0.1)
- 즉사 무시 규칙:
  - 해당 감소로 HP가 0이 된 경우 “만땅 트리거/예약”은 스킵

———

### 4) 자동 시전(이벤트 큐 기반, 결정성 유지)

즉시 실행하지 않고, 이벤트 큐에 시전 이벤트를 예약해 처리 순서를 고정한다.

- BattleEvent 확장(예시)
  - AutoCastStart { time_ms, caster_instance_id }
  - AutoCastEnd { time_ms, caster_instance_id } (채널링 종료/락 적용)

예약 규칙

- 공명이 만땅이 되면, 해당 Attack/피격 이벤트 처리 루틴의 “마지막”에서:
  - caster가 살아있고(HP>0) 만땅 상태이며 pending이 아니면 AutoCastStart(time_ms=now) push

시전 처리

- AutoCastStart:
  - 생존/락/캐스팅 중 여부 재검증 (죽었으면 no-op)
  - “어떤 스킬을 쓸지”는 일단 임시 규칙으로 한 개만 선택(나중에 확장)
  - 스킬 타입이 채널링이면 casting_until_ms = now + duration, AutoCastEnd 예약
  - 즉시형이면 AutoCastEnd(time_ms=now)로 통일 처리(종료 훅에서 공명 리셋/락 적용)
- AutoCastEnd:
  - 공명 0
  - resonance_gain_locked_until_ms = now + 1000
  - pending_cast 해제

기본 공격과의 상호작용

- Attack 이벤트 처리 시 now < casting_until_ms면 공격을 폐기하지 말고 time_ms=casting_until_ms로 재스
  케줄(공격 사이클 보존)

———

### 5) 스킬 선택/확장성(구현은 추후)

- 현재는 “단일 스킬 자동 시전”만 가정하고, 확장 포인트만 마련한다.
- 확장 방식 후보(추후 구현용)
  - AbilityTrigger::OnResonanceFull을 추가하고, 해당 트리거 스킬들 중 선택
  - SkillDef에 priority/conditions를 두고 결정적 선택
  - AbnormalityMetadata에 auto_ability_id를 두고 명시적 지정

———

### 6) 테스트(계획만)

- 공격 판정 시 +10(명중 무관), 캐스팅/락 중이면 0
- 피격으로 HP 감소 시 floor(dmg\*0.1), 캐스팅/락 중이면 0
- 만땅 시 AutoCastStart가 “이벤트 처리 종료 직후” 예약/실행
- 캐스팅 중 공명 획득 완전 차단(공격/피격/기타 경로 모두)
- 시전 종료 후 공명 0 + 1000ms 락
- 만땅이 된 순간 즉사면 시전 없음
- 채널링 동안 기본공격이 지연되고 종료 후 재개

———
