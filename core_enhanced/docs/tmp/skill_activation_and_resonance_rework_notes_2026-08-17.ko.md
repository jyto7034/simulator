# 스킬 발동 방식과 공명 회복 재개편 메모

> 2026-10-03 차기 정책: [공명과 스킬 사용](../game_rules/combat.md#공명과-스킬-사용)이 최신 목표다.
> 배치/재배치 최대치 20% 내림·초당 1 자동 충전, 기본 공격/가한 피해/피격 획득과 Rest 공명 보너스 제거,
> 유효 입력 접수 즉시 전량 소비·시전부터 유형별 사용 종료까지 모든 획득 잠금·종료 후 별도 1초 잠금 없음,
> Stun/Freeze 중 모든 양수 획득 금지를 적용한다. 방어·작업·침묵은 자동 충전을 막지 않는다.
> 아래는 당시 구현·설계 비교 기록이며 상충하는 옛 규칙을 차기 정책으로 적용하지 않는다. 코드 이행 완료를 뜻하지 않는다.

> 상태: 임시 정책 메모 / 구현 지시나 canonical 계약이 아님
>
> 작성일: 2026-08-17

## 목적

현재 스킬은 공명이 최대가 되면 자동 시전되거나, 스킬 파편을 장착한 플레이어 unit이 수동으로 시전하는 두 흐름을 함께 사용한다. 이 구조를 명시적인 정책으로 정리하고, TFT에서 유래한 현재 공명 획득 규칙을 유지할지 또는 게임의 실제 전투 의도에 맞게 바꿀지 결정한다.

보스나 특수 unit의 `n`칸 특수 게이지는 공명의 표시만 빌린 별칭으로 만들지 않는다. 예를 들어 4칸 게이지는 `0..=4`의 이산 스택을 authority로 두고 UI가 0/25/50/75/100%로 표현한다. 공명과 같은 충전·최대치 시전·소비 규칙을 의도적으로 공유하는 경우에만 공명을 재사용한다.

## 현재 구현 사실

### 스킬 발동 mode

- `SkillActivationMode`에는 `Manual`, `Auto`, `Triggered`가 있다.
- 현재 battle profile을 만들 때 mode가 정해지며, 전투 중 mode를 바꾸는 Core command, server mapping, Unity-facing DTO command는 없다.
- 스킬 파편을 장착한 플레이어 unit은 현재 무조건 `Manual`이다.
- Corroded employee와 Boss summon은 현재 무조건 `Auto`다. 일반 hostile/Boss도 별도의 RON mode authoring이 없어 기본 `Auto` 경로를 사용한다.
- `Manual`은 플레이어 소유 active combatant만 `ActivateSkill`로 실행할 수 있다.
- `Triggered` enum은 존재하지만, 현행 profile authoring이나 공개 command에서 선택하여 동작하는 실사용 경로는 확인되지 않았다. 새 정책에서 요구하지 않는다면 아직 기능으로 광고하거나 노출하지 않는다.

### Auto/Manual의 공명 결합

- `Auto` unit은 공명이 최대치에 *도달하는 전이*에서 시전을 예약한다.
- `Manual` unit은 공명이 최대치여야 하지만, 자동 예약은 하지 않는다.
- Auto/Manual 시전이 끝나면 현재 공명을 0으로 만들고 `gain_lock_ms` 동안 새 공명 획득을 막는다.
- 음수 `ModifyResonance` 소비는 gain lock과 별개로 적용된다.

### 현재 공명 값과 획득 규칙

각 combat profile의 RON은 `resonance: (start, max, gain_lock_ms)`를 가진다. 현재 live 데이터는 대부분 `0 / 100 / 1000ms`를 사용한다. 시작값은 최대값을 넘지 않도록 clamp되고, runtime은 최대값 0을 1로 보정한다.

현재 공명 증감 원인은 다음과 같다.

| 원인 | 현재 규칙 | data authoring 여부 |
| --- | --- | --- |
| 기본 공격 발사 | `+10` | 없음 — Core 고정 |
| 기본 공격으로 피해를 줌 | 실제 피해량의 `floor(damage / 10)` | 없음 — Core 고정 |
| 기본 공격 피해를 받음 | 실제 피해량의 `floor(damage / 10)` | 없음 — Core 고정 |
| Platform Rest | 무기 profile의 tick당 공명 회복값. 현재 기본 profile은 `+5` | 무기 profile 값은 가능 |
| 스킬 효과 | `ModifyResonance(amount: i32)` | 가능. 양수/음수 모두 가능 |
| 소모 아이템 | 초기 공명을 최대치의 비율만큼 추가 | 현재 플레이어 준비 단계 전용 |

양수 획득은 active unit, hard CC 미적용, gain lock 해제 상태여야 한다. 모든 증감은 `0..=max`로 clamp되고 `ResonanceChanged` event로 투영된다.

Elite/Boss와 플레이어 unit의 HUD에는 HP + 공명이 투영된다. 즉 현재 HUD 모양은 참고할 수 있으나, 이 사실만으로 보스 특수 게이지의 authority를 공명으로 정하지는 않는다.

## 정책 결정 1 — 스킬 파편의 Auto/Manual

결정할 질문: 스킬 파편마다 발동 mode를 authored 계약으로 둘 것인가, 아니면 전투 중 플레이어가 Auto/Manual을 toggle할 수 있게 할 것인가?

### 선택지 A: 파편별 authored mode

- 각 active skill fragment 또는 그것이 부여하는 skill profile에 `activation_mode: Manual | Auto`를 명시한다.
- 전투 중에는 바꾸지 않는다. player는 Manual 파편만 직접 시전하고 Auto 파편은 공명이 가득 차면 Core가 시전한다.
- 파편이 의도한 조작감, target 방식, 밸런스와 UI를 content 단위로 고정할 수 있다.
- 가장 작은 장기 구조다. 현재의 profile-time mode 결정 방식과도 자연스럽게 이어진다.

### 선택지 B: 플레이어가 전투 중 toggle

- 파편이 `toggleable`임을 author하고, player unit에만 Auto↔Manual command를 제공한다.
- command의 수용 시점, casting/focus 중 변경 가능 여부, full resonance에서 Auto를 끌 때 예약된 cast를 취소할지, Manual에서 Auto로 켤 때 즉시 예약할지, reconnect projection 및 UI optimistic state를 모두 계약으로 정해야 한다.
- toggle은 단순 UI 옵션이 아니라 전투 outcome과 공명 소비 시점을 바꾸므로 server-authoritative command와 focused state-transition test가 필요하다.

### 초기 권장안

먼저 **선택지 A**를 기본 정책으로 채택한다. `Manual | Auto`를 파편/skill profile 단위의 명시 data로 만들고, 특별히 전술적 선택 자체를 콘텐츠로 삼을 파편에만 후속 Goal에서 toggle을 추가한다.

이렇게 하면 “언제든 편하게 켜고 끄는 옵션”이 실제 시전 예약·target·재접속 상태와 충돌하는 일을 피하면서도, Auto 파편과 Manual 파편의 플레이 감각을 명확히 분리할 수 있다.

`Triggered`는 별도의 event/condition authority, 중복 방지 key, 소비 시점과 producer가 설계되기 전까지 selectable mode에 넣지 않는다.

## 정책 결정 2 — TFT 계승 공명 회복 규칙

결정할 질문: 기본 공격 발사/피해량 기반의 고정 공명 획득을 현행대로 유지할 것인가, 게임의 스킬 주기와 보스 설계에 맞게 개편할 것인가?

### 유지할 경우

- 기존 combat cadence와 플레이어의 직관을 보존한다.
- 단, `+10`, `damage / 10`, Platform Rest `+5`가 공명 최대치 100과 함께 하나의 고정 패키지임을 canonical 문서에 명시한다.
- 보스나 특수 unit이 이 규칙을 의도적으로 사용할지, 각 source를 선택적으로 끌 수 있는 authoring이 필요한지는 별도 계약으로 결정한다.

### 개편할 경우

- 자동 획득 원인을 `ResonanceGainProfile` 같은 명시적 policy/data로 분리한다. 최소 후보는 `on_basic_attack_release`, `on_basic_damage_dealt`, `on_basic_damage_received`, `on_platform_rest_tick`이다.
- 각 source는 disabled 또는 고정값/비율을 author할 수 있어야 하며, damage 기반이라면 반올림·최소치·상한을 정확히 정의한다.
- 최대치, 시전 소비, gain lock은 획득 profile과 별도의 규칙으로 남긴다. 이를 섞으면 “특수 게이지를 채우지만 스킬 시전은 하지 않는” 요구를 표현하기 어렵다.
- 새 profile을 도입한다면 기존 TFT식 행동을 legacy fallback으로 조용히 유지하지 않는다. 기존 live content를 명시 값으로 migration하고, 이전 schema를 삭제할 시점과 범위를 정한다.

### 초기 권장안

플레이어 스킬 파편의 기본 공명 루프는 당장은 유지하되, **Boss/특수 unit까지 같은 고정 자동 획득을 공유해야 하는지는 재검토**한다. 특수 unit에 “정확히 한 칸씩 채우는” 설계가 필요하다면 공명 profile을 억지로 변형하는 대신 독립 스택/게이지 계약을 사용한다.

공명 자체를 개편하기로 한다면 source별 authoring을 추가하는 방향이 적합하다. 단순히 보스마다 `max: 4`를 쓰는 방식은 기본 공격 `+10`과 피해 기반 획득 때문에 4스택 의미를 보존하지 못한다.

## 유지해야 할 불변식

- Core만 공명/스킬 readiness/current state의 authority다. Unity가 공명 변화를 재추론하거나 Auto cast를 독자 예약하지 않는다.
- 실패하거나 거부된 command는 공명, 시전 예약, cooldown, event, battle revision을 바꾸지 않는다.
- Auto 최대치 도달은 동일한 전이에 정확히 한 번만 예약된다. 중복 packet, reconnect projection 또는 repeated observation이 재시전을 만들지 않는다.
- Manual/Auto 변경 기능을 도입한다면 같은 battle-time command는 idempotency, event ordering과 reconnect projection을 가진다.
- 공명과 특수 스택은 서로 별도 authority다. 하나를 다른 하나의 화면 표기용 alias로 두지 않는다.
- Boss/summon 정책은 Wave/Boss policy의 neutral fact/terminal authority 경계를 침범하지 않는다.

## 후속 조사 및 구현 전 질문

1. 파편 mode는 `Manual | Auto`만으로 충분한가? 아니면 phase/event가 직접 시전하는 `Triggered` skill이 실제 콘텐츠 요구인가?
2. Auto 파편은 player가 target을 지정하지 않는 `AutomaticUnit` targeting만 허용할 것인가? Auto + player-targeted skill을 허용한다면 target authority는 무엇인가?
3. 선택지 B를 채택한다면 toggle은 battle entry에만 허용할지, 전투 중에도 허용할지? 전투 중이라면 full resonance의 기존 예약을 어떤 순간에 처리할지?
4. 공명 획득 profile은 player/hostile/Boss/summon 모두에 적용할 하나의 schema인가, player 공명과 Boss 특수 스택을 별도 계약으로 유지할 것인가?
5. 공명 최대치의 시전은 항상 전량 소비(0으로 reset)인가, 부분 비용·유지·다중 charge가 필요한 콘텐츠가 있는가?

## 구현 시 필수 검증 범주

- RON loading: mode 및 공명 profile의 누락/unknown/invalid 값을 startup에서 거부한다.
- Auto: 최대치 전이 1회 예약, gain lock, 시전 취소/사망/Hard CC/Silence, reconnect 후 중복 예약 없음.
- Manual: full gate, targeting, 실패 요청의 무변화, fragment별 mode projection.
- Toggle을 도입하는 경우: 가능한 전환 시점, pending cast 처리, concurrent request/idempotency, server message order와 reconnect snapshot.
- 공명 profile 개편: 각 획득 source의 값, rounding, clamp, gain lock, HUD/event DTO, 이전 live RON 명시 migration.
- 특수 스택: 0..=N 증감, N칸 UI 투영, phase/skill condition, 공명과의 독립성.

## 비범위

- 이 문서는 Auto/Manual 정책이나 공명 수치를 확정하지 않는다.
- production code, RON, DTO, Unity UI, save 형식은 이 메모만으로 변경하지 않는다.
- conditional Wave/Boss summon Final Freeze의 현재 의미를 변경하지 않는다.
