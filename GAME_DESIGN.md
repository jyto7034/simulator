# 게임 설계 문서 (Lobotomy Corporation 기반)

> **작성일**: 2025-10-29
> **버전**: 1.4
> **목적**: 로보토미 코퍼레이션 IP 기반 1vs1 오토배틀 로그라이크 게임 설계

---

## 📋 목차

1. [개요](#개요)
2. [시간 시스템](#시간-시스템)
3. [시련 시스템](#시련-시스템)
4. [자원 및 화폐](#자원-및-화폐)
5. [장비 시스템](#장비-시스템)
6. [아티팩트 시스템](#아티팩트-시스템)
7. [시너지 시스템](#시너지-시스템)
8. [Zone 구조](#zone-구조)
9. [보상 시스템](#보상-시스템)
10. [게임 진행 흐름](#게임-진행-흐름)
11. [ECS 구현 설계](#ecs-구현-설계)

---

## 개요

### 프로젝트 개요

**로보토미 코퍼레이션 IP 기반 온라인 1vs1 오토배틀 게임**
- **게임 서버**: Rust (Actix Actor 모델)
- **게임 코어**: Rust (bevy_ecs)
- **클라이언트**: Unity
- **게임 연산**: 보안을 위해 Game Server에서 전부 처리
- **클라이언트 역할**: 연산 결과 시각화만 담당

### 게임 장르

시련 기반 턴제 로그라이크 오토배틀 게임
- 이벤트 선택 → PvE 진압 → PvP 시련 → 다음 시련 단계
- E.G.O 추출, 엔케팔린, 환상체, E.G.O 선물 등 이벤트
- 경험치 획득 → 레벨업 시스템

---

## 시간 시스템

### 기존 시스템 (The Bazaar)
```
Day 1, Day 2, Day 3...
Hour 0-5
```

### 로보토미 시스템

#### 1. OrdealLevel (시련 단계) - Day 대체

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrdealLevel {
    Dawn,      // 여명 
    Noon,      // 정오 
    Dusk,      // 어스름 
    Midnight,  // 자정 
    White,     // 백색 - 엔드게임
}
```

**위험 등급:**
- 여명: ZAYIN
- 정오: TETH
- 어스름: HE
- 자정: WAW
- 백색: ALEPH

#### 2. ManagementPhase (관리 단계) - Hour 대체

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManagementPhase {
    First,        // Phase 1 (이벤트/상점)
    Second,       // Phase 2 (이벤트/상점)
    Suppression,  // Phase 3 (PvE 전투)
    Fourth,       // Phase 4 (이벤트/상점)
    Fifth,        // Phase 5 (이벤트/상점 or Ordeal for Dusk/Midnight)
    Sixth,        // Phase 6 (Ordeal for Dawn/Noon/White)
}
```

#### 3. CurrentTime Resource

```rust
#[derive(Resource)]
pub struct CurrentTime {
    pub ordeal_level: OrdealLevel,  // 여명/정오/어스름/자정/백색
    pub phase: ManagementPhase,     // First~Sixth
    pub absolute_day: u32,          // 실제 Day 카운트 (1, 2, 3...)
}

impl CurrentTime {
    /// 현재 시련 단계의 최대 Phase 수
    pub fn max_phase(&self) -> u32 {
        match self.ordeal_level {
            OrdealLevel::Dawn | OrdealLevel::Noon | OrdealLevel::White => 6,
            OrdealLevel::Dusk | OrdealLevel::Midnight => 5,
        }
    }
}
```

**표시 예시:**
- "여명 - Phase 1"
- "정오 - Phase 3: 진압 작업"
- "어스름 - Phase 5: 시련 조우"
- "백색 - Phase 6: 백색의 시련"

---

## 시련 시스템

### 시련 색상 (OrdealColor)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrdealColor {
    Green,    // 녹빛 - 로봇, 창조와 인생
    Violet,   // 자색 - 외계, 구원과 이해
    Crimson,  // 핏빛 - 광대, 즐거움과 욕망
    Amber,    // 호박색 - 벌레, 생존과 경쟁
    Indigo,   // 쪽빛 - 산업/전쟁 (정오 전용, 26일 이후)
    White,    // 백색 - 인간, 정의와 질서 (46일 이후)
}
```

### 시련 단계별 색상 구성

| 시련 단계 | 등장 색상 | 색상 수 | 비고 |
|---------|---------|--------|------|
| **여명** | 녹빛, 자색, 핏빛, 호박색 | 4개 | 초보자 학습 단계 |
| **정오** | 녹빛, 자색, 핏빛, 쪽빛 | 4개 | 쪽빛은 26일부터 |
| **어스름** | 녹빛, 핏빛, 호박색 | **3개** | 높은 난이도, 높은 보상 |
| **자정** | 녹빛, 자색, 호박색 | **3개** | 최고 난이도, 최고 보상 |
| **백색** | 백색 | **1개** | 엔드게임 |

### 색상별 특성

```rust
impl OrdealColor {
    pub fn theme(&self) -> &str {
        match self {
            Self::Green => "창조와 인생",
            Self::Violet => "구원과 이해",
            Self::Crimson => "즐거움과 욕망",
            Self::Amber => "생존과 경쟁",
            Self::Indigo => "산업과 전쟁",
            Self::White => "정의와 질서",
        }
    }

    pub fn weak_to_damage(&self) -> DamageType {
        match self {
            Self::Green => DamageType::Black,   // B 피해
            Self::Violet => DamageType::White,  // W 피해
            Self::Crimson => DamageType::Red,   // R 피해
            Self::Amber => DamageType::Red,     // R 피해
            Self::Indigo => DamageType::Black,  // B 피해
            Self::White => DamageType::Pale,    // P 피해 (특수)
        }
    }
}
```

### 시련 정보

```rust
#[derive(Debug, Clone)]
pub struct OrdealInfo {
    pub level: OrdealLevel,      // 여명/정오/어스름/자정/백색
    pub color: OrdealColor,      // 녹빛/자색/핏빛/호박색/쪽빛/백색
    pub difficulty: u32,         // 1~10
}

impl OrdealInfo {
    pub fn name(&self) -> String {
        match self.color {
            OrdealColor::White => {
                format!("백색 {}의 시련", self.level.korean_name())
            }
            _ => {
                format!("{} {}의 시련",
                    self.color.korean_name(),
                    self.level.korean_name())
            }
        }
    }
}
```

**예시:**
- "녹빛 여명의 시련"
- "핏빛 정오의 시련"
- "호박색 어스름의 시련"
- "백색 자정의 시련"

---

## 자원 및 화폐

### 1. 엔케팔린 (Enkephalin)

**로보토미의 주요 자원** - 환상체로부터 추출한 에너지

```rust
#[derive(Component)]
pub struct Enkephalin {
    pub amount: u32,
}
```

**용도:**
- E.G.O 장비 구매
- 상점 새로고침
- 환상체 획득
- E.G.O 선물 구매

**획득 방법:**
- 진압 작업 (PvE) 완료
- 시련 (PvP) 승리
- 이벤트 보상


---

## 장비 시스템

### 1. E.G.O Equipment (추출 장비)

**환상체로부터 추출한 무기와 방어구**

```rust
#[derive(Debug, Clone, Copy)]
pub enum EgoType {
    Weapon,   // E.G.O 무기
    Suit,     // E.G.O 방어구
    Gift,     // 환상체 선물 (악세서리)
}

#[derive(Component)]
pub struct EgoEquipment {
    pub equipment_id: String,
    pub equipment_name: String,
    pub ego_type: EgoType,
    pub cost: u32,  // 엔케팔린 비용
    pub risk_level: RiskLevel,
    pub source_abnormality: Option<String>,  // 출처 환상체
}
```

#### 환상체 위험 등급

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    ZAYIN = 0,   // 위험도 최하
    TETH = 1,
    HE = 2,
    WAW = 3,
    ALEPH = 4,   // 위험도 최상
}
```

### 2. 장비 슬롯

**환상체당 3개 슬롯:**
```
환상체 (Der Freischütz)
├─ 무기 슬롯      [마탄의 사수 총]
├─ 방어구 슬롯    [Der Freischütz 슈트]
└─ 선물 슬롯      [은탄환]
```

### 3. 장비 등급

| 등급 | 스탯 보너스 | 특수 효과 | 드랍률 |
|------|------------|----------|--------|
| Common (회색) | +10% | 없음 | 50% |
| Rare (파랑) | +20% | 없음 | 30% |
| Epic (보라) | +35% | 약한 효과 | 15% |
| Legendary (주황) | +50% | 강력한 효과 | 5% |

### 4. 환상체 선물 (Gift)

**환상체가 직원에게 주는 특수 아이템** - 악세서리 슬롯

**특징:**
- 각 환상체마다 고유한 선물
- 특수 능력 부여
- 시너지 효과

**획득 방법:** TBD (미정)

**예시:**
- "은탄환" (Der Freischütz) - 첫 공격 크리티컬
- "나비의 시간" (장례식의 나비들) - 회복 능력
- "미소 짓는 가면" (미소 짓는 시체들의 산) - HP 재생

---

## 아티팩트 시스템

### E.G.O Gift (E.G.O 선물)

**E.G.O에서 파생된 특수 능력** - 덱 전체에 영향을 주는 강력한 효과

**배경 설정:**
- E.G.O를 연구하면서 발견한 부가 효과
- 환상체의 본질을 추출한 것
- 환상체 선물(Gift)과 다르게 덱 전체에 영향

**획득 방법:** TBD (미정)

```rust
#[derive(Component)]
pub struct EgoGift {
    pub gift_id: String,
    pub gift_name: String,
    pub description: String,
    pub effect: EgoGiftEffect,
    pub rarity: EgoGiftRarity,
    pub source_abnormality: Option<String>,  // 출처 환상체
}

#[derive(Debug, Clone, Copy)]
pub enum EgoGiftRarity {
    Common,     // 일반
    Rare,       // 희귀
    Epic,       // 영웅
    Legendary,  // 전설
}
```

### E.G.O 선물 효과 종류

#### 1. 타입 기반 (환상체 등급)

```
"여명의 깨달음" (Rare)
- 효과: ZAYIN 등급 환상체 모든 스탯 +15%
- 출처: 다수의 ZAYIN 환상체 관리 경험

"WAW의 본질" (Epic)
- 효과: WAW 등급 환상체 공격력 +30%
- 출처: WAW 환상체 E.G.O 연구

"ALEPH의 권능" (Legendary)
- 효과: ALEPH 등급 환상체 모든 스탯 +20%
- 출처: ALEPH 환상체 격리 성공
```

#### 2. 시련 저항

```
"녹빛 저항" (Rare)
- 효과: 녹빛 시련 피해 -30%
- 출처: 녹빛 시련 생존 경험

"어스름의 생존 본능" (Epic)
- 효과: 어스름 시련에서 첫 사망 무효 (1회)
- 출처: 어스름 시련 극복
```

#### 3. 전투 기반

```
"선제 공격의 의지" (Rare)
- 효과: 전투 시작 후 첫 5초간 모든 환상체 공격력 +50%
- 출처: 공격형 E.G.O 연구

"역경의 힘" (Epic)
- 효과: 아군 환상체 HP 50% 이하일 때 공격 속도 -30%
- 출처: 생존형 E.G.O 연구

"복수의 의지" (Legendary)
- 효과: 아군 환상체 사망 시 남은 환상체 공격력 +40% (누적)
- 출처: Nothing There E.G.O
```

#### 4. 시너지 기반

```
"전술적 배치" (Rare)
- 효과: Back 레인에 환상체 있고 Mid 레인에 환상체 있을 때 Back 공격력 +35%
- 출처: 부서 배치 최적화 연구

"속도 공명" (Epic)
- 효과: 공격 속도 500ms 이하 환상체 2개 이상 시 모두 공격 속도 -100ms
- 출처: 1.76 MHz E.G.O
```

### 구분: 환상체 선물 vs E.G.O 선물

| 항목 | 환상체 선물 (Gift) | E.G.O 선물 (E.G.O Gift) |
|------|-------------------|----------------------|
| **슬롯** | 악세서리 슬롯 | 덱 전체 (별도) |
| **효과 범위** | 장착한 환상체 1개 | 덱 전체 |
| **획득** | TBD (미정) | TBD (미정) |
| **예시** | "은탄환", "나비의 시간" | "여명의 깨달음", "ALEPH의 권능" |

### 소지 제한

- 덱당 **5-7개** 장착 가능
- 조건 중복 가능 (중첩 효과)

---

## 시너지 시스템

### 환상체 조합 시너지

**롤토체스 시너지 시스템과 동일** - 특정 테마/그룹의 환상체를 여러 개 모으면 추가 효과 발동

```rust
#[derive(Debug, Clone)]
pub struct SynergyTrait {
    pub trait_id: String,
    pub trait_name: String,
    pub description: String,
    pub thresholds: Vec<SynergyThreshold>,
}

#[derive(Debug, Clone)]
pub struct SynergyThreshold {
    pub required_count: u32,      // 필요한 환상체 개수
    pub effect: SynergyEffect,    // 발동 효과
}
```

### 시너지 그룹 예시

#### 1. 새 (Birds) 시너지
```
관련 환상체:
- 심판 새 (Judgement Bird)
- 큰 새 (Big Bird)
- 징벌 새 (Punishing Bird)
- 작은 새들 (Small Birds)

(2): 모든 새 +10% 공격력
(3): 모든 새 +20% 공격력, +10% 방어력
(4): 모든 새 +35% 공격력, +20% 방어력
     특수: "묵시록의 새" 효과 - 첫 공격 시 광역 피해
```

#### 2. 종교 (Religion) 시너지
```
관련 환상체:
- 하얀 밤 (WhiteNight)
- 고백 (Confession)
- 산의 왕 (Mountain of Smiling Bodies)
- 하나뿐인 죄와 수백의 선 (One Sin and Hundreds of Good Deeds)

(2): 모든 종교 환상체 +15% HP
(3): 모든 종교 환상체 +25% HP, +10% 회복
(4): 모든 종교 환상체 +40% HP, +20% 회복
     특수: "신성" 효과 - 사망 시 인접 아군 부활 (1회)
```

#### 3. 기계 (Machine) 시너지
```
관련 환상체:
- 1.76 MHz (Don't Touch Me)
- 벌집 (Opened Can of WellCheers)
- 돌격 드릴 (Charging Chuck)

(2): 모든 기계 -15% 공격 속도 (빨라짐)
(3): 모든 기계 -25% 공격 속도, +20% 공격력
     특수: "오버클럭" 효과 - 주기적으로 3초간 공격 속도 2배
```

#### 4. 동화 (Fairy Tale) 시너지
```
관련 환상체:
- 빨간 구두 (Red Shoes)
- 마법 소녀 (Magical Girl)
- 백설공주의 사과 (Snow White's Apple)
- 헨젤과 그레텔의 오븐 (Hansel and Gretel's Oven)

(2): 모든 동화 +10% 이동 속도
(3): 모든 동화 +20% 이동 속도, +15% 회피
(4): 모든 동화 +30% 이동 속도, +25% 회피
     특수: "동화는 끝나지 않는다" - 스킬 쿨다운 -20%
```

#### 5. 공포 (Horror) 시너지
```
관련 환상체:
- Nothing There
- 분노한 여신 (Funeral of the Dead Butterflies)
- 피의 목욕 (Blood Bath)

(2): 모든 공포 +20% 공격력
(3): 모든 공포 +35% 공격력, +15% 생명력 흡수
     특수: "공포" 효과 - 공격 시 적에게 공포 디버프 (공격력 -20%)
```

#### 6. ALEPH 시너지
```
관련 환상체:
- ALEPH 등급 환상체 전부

(1): ALEPH 환상체 +15% 모든 스탯
(2): ALEPH 환상체 +30% 모든 스탯
(3): ALEPH 환상체 +50% 모든 스탯
     특수: "ALEPH 공명" - 다른 아군 전체 +10% 모든 스탯
```

### 시너지 중첩

- **여러 시너지 동시 활성화 가능**
  - 예: "심판 새"는 새(2) + ALEPH(1) 동시 효과
- **동일 시너지 중복 계산 안 됨**
  - 예: "심판 새" 2마리 = 새(2), 새(4) 아님

### 시너지 표시

**전투 전 화면:**
```
====================================
현재 활성 시너지
====================================
🐦 새 (3/4)
   ✓ +20% 공격력
   ✓ +10% 방어력
   ⏸ 묵시록의 새 (4개 필요)

📿 종교 (2/4)
   ✓ +15% HP
   ⏸ +25% HP, +10% 회복 (3개 필요)

⚙️ ALEPH (1/3)
   ✓ +15% 모든 스탯
   ⏸ +30% 모든 스탯 (2개 필요)
====================================
```

---

## Zone 구조

### ZoneType (영역 타입)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    // === 관리 단계 (이벤트/상점) ===
    CentralCommand,        // 중앙본부 (선택)
    EgoExtraction,         // E.G.O 추출 (상점)
    AbnormalityArchive,    // 환상체 보관소 (환상체 획득)
    AgentTraining,         // 직원 훈련 (레벨업)
    SephirahCore,          // 세피라 코어 (특수 이벤트)

    // === 전투 ===
    SuppressionWork,       // 진압 작업 (PvE)
    OrdealEncounter,       // 시련 조우 (PvP)
}
```

### Zone별 설명

#### 1. 중앙본부 (CentralCommand)
- 이벤트/상점 선택
- 3가지 선택지 중 1개 선택

#### 2. E.G.O 추출 (EgoExtraction)
- 상점 (엔케팔린으로 구매)
- E.G.O 무기/방어구 진열
- 새로고침 가능
- 나가기 버튼

#### 3. 환상체 보관소 (AbnormalityArchive)
- 환상체 획득
- 등급별 환상체 진열
- 새로고침 불가

#### 4. 직원 훈련 (AgentTraining)
- 레벨업
- 스킬 선택지 3개
- 나가기

#### 5. 세피라 코어 (SephirahCore)
- 특수 이벤트
- 세피라와 대화
- 특별 보상

#### 6. 진압 작업 (SuppressionWork)
- PvE 전투
- 3개 환상체 중 1개 선택
- 등급별 분류 (예: ZAYIN 1개, TETH 1개, HE+ 1개 등 시련 단계에 따라 조정)

#### 7. 시련 조우 (OrdealEncounter)
- PvP 전투
- Ghost 빌드와 자동 전투
- 시련 색상 선택 (3~4가지)

---

## 보상 시스템

### 리스크-리워드 구조

#### 여명/정오 (4가지 색상)
```
선택지: 많음 (4개)
난이도: 낮음~보통
보상: 일반적

- 엔케팔린: 500~2000
- E.G.O: Common~Rare
- E.G.O 선물: Common~Rare
```

#### 어스름 (3가지 색상 - 높은 보상!)
```
선택지: 적음 (3개)
난이도: 높음
보상: 희귀 ⭐

- 엔케팔린: 2500~4000
- E.G.O: Epic 확정 ⭐
- E.G.O 선물: Epic 확정 ⭐
- WAW 환상체 선물 50% 확률
- 패배 시 페널티: -500 엔케팔린
```

#### 자정 (3가지 색상 - 최고 보상!)
```
선택지: 적음 (3개)
난이도: 극한
보상: 전설 ⭐⭐

- 엔케팔린: 4000~6000
- E.G.O: Legendary 50~100% 확률 ⭐⭐
- E.G.O 선물: Legendary 확정 ⭐⭐
- ALEPH 환상체 선물 30~50% 확률 ⭐⭐⭐
- 특수: 고유 E.G.O 선물 10% 확률 ⭐⭐⭐⭐
- 패배 시 페널티: -1000 엔케팔린, 환상체 1마리 손실 가능
```

#### 백색 (1가지 - 엔드게임)
```
선택지: 없음 (1개 강제)
난이도: 불가능
보상: 유일 ⭐⭐⭐⭐

- 엔케팔린: 8000~12000
- E.G.O: Unique 확정
- E.G.O 선물: Unique 확정
- 백색 환상체 선물
```

### 보상 티어

```rust
impl OrdealLevel {
    pub fn reward_tier(&self) -> RewardTier {
        match self {
            Self::Dawn => RewardTier::Common,
            Self::Noon => RewardTier::Rare,
            Self::Dusk => RewardTier::Epic,      // 3개 색상
            Self::Midnight => RewardTier::Legendary,  // 3개 색상
            Self::White => RewardTier::Unique,
        }
    }

    pub fn enkephalin_reward_range(&self) -> (u32, u32) {
        match self {
            Self::Dawn => (500, 1000),
            Self::Noon => (1000, 2000),
            Self::Dusk => (2500, 4000),
            Self::Midnight => (4000, 6000),
            Self::White => (8000, 12000),
        }
    }
}
```

---

## 게임 진행 흐름

### 전체 구조

```
여명의 시련 ZAYIN
├─ Phase 1: 녹빛 여명 (이벤트/상점)
├─ Phase 2: 자색 여명 (이벤트/상점)
├─ Phase 3: 진압 작업 (PvE 전투)
├─ Phase 4: 핏빛 여명 (이벤트/상점)
├─ Phase 5: 호박색 여명 (이벤트/상점)
└─ Phase 6: 여명의 시련 (PvP 전투 - 4가지 색상 선택)
     ↓
정오의 시련 TETH 
├─ Phase 1: 녹빛 정오 (이벤트/상점)
├─ Phase 2: 자색 정오 (이벤트/상점)
├─ Phase 3: 진압 작업 (PvE 전투)
├─ Phase 4: 핏빛 정오 (이벤트/상점)
├─ Phase 5: 쪽빛 정오 (이벤트/상점) 
└─ Phase 6: 정오의 시련 (PvP 전투 - 4가지 색상 선택)
     ↓
어스름의 시련 HE 
├─ Phase 1: 녹빛 어스름 (이벤트/상점)
├─ Phase 2: 핏빛 어스름 (이벤트/상점)
├─ Phase 3: 진압 작업 (PvE 전투)
├─ Phase 4: 호박색 어스름 (이벤트/상점)
└─ Phase 5: 어스름의 시련 (PvP 전투 - 3가지 색상 선택) ⚠️
     ↓
자정의 시련 WAW
├─ Phase 1: 녹빛 자정 (이벤트/상점)
├─ Phase 2: 자색 자정 (이벤트/상점)
├─ Phase 3: 진압 작업 (PvE 전투)
├─ Phase 4: 호박색 자정 (이벤트/상점)
└─ Phase 5: 자정의 시련 (PvP 전투 - 3가지 색상 선택) ⚠️⚠️
     ↓
백색의 시련 ALEPH
├─ Phase 1: 백색 여명 (이벤트/상점)
├─ Phase 2: 백색 정오 (이벤트/상점)
├─ Phase 3: 진압 작업 (PvE 전투)
├─ Phase 4: 백색 어스름 (이벤트/상점)
├─ Phase 5: 백색 자정 (이벤트/상점)
└─ Phase 6: 백색의 시련 (PvP 전투 - 1가지 강제) ⚠️⚠️⚠️
```

### Phase별 Zone 매핑

| Phase | Zone | 내용 |
|-------|------|------|
| First, Second, Fourth, Fifth | CentralCommand → EgoExtraction / AbnormalityArchive / AgentTraining / SephirahCore | 이벤트/상점 선택 |
| Suppression (Phase 3) | SuppressionWork | PvE 전투 (3개 중 1개 선택) |
| Fifth (Dusk/Midnight only) | OrdealEncounter | PvP 전투 (색상 선택) |
| Sixth (Dawn/Noon/White only) | OrdealEncounter | PvP 전투 (색상 선택) |

**참고:**
- **여명/정오/백색**: Phase 6에서 시련 발생 (총 6 Phase)
- **어스름/자정**: Phase 5에서 시련 발생 (총 5 Phase) - 준비 시간 1단계 부족!

### 시련 선택 화면 예시

#### 여명 단계 (4가지)
```
====================================
시련 선택 - 여명의 시련
====================================

[1] 녹빛 여명의 시련
    난이도: ★☆☆☆☆
    보상: 엔케팔린 500~800
          Common E.G.O 확정

[2] 자색 여명의 시련
    난이도: ★☆☆☆☆
    보상: 엔케팔린 600~900
          Common E.G.O 확정

[3] 핏빛 여명의 시련
    난이도: ★★☆☆☆
    보상: 엔케팔린 700~1000
          Rare E.G.O 20% 확률

[4] 호박색 여명의 시련
    난이도: ★★☆☆☆
    보상: 엔케팔린 800~1100
          Rare E.G.O 선물 30% 확률
====================================
```

#### 어스름 단계 (3가지 - 높은 보상!)
```
====================================
시련 선택 - 어스름의 시련
====================================

[1] 녹빛 어스름의 시련
    난이도: ★★★★☆
    보상: 엔케팔린 2500~3500
          Epic E.G.O 확정 ⭐
          WAW 환상체 선물 50% 확률

[2] 핏빛 어스름의 시련
    난이도: ★★★★★
    보상: 엔케팔린 3000~4000
          Epic E.G.O 확정 ⭐
          Epic E.G.O 선물 확정 ⭐

[3] 호박색 어스름의 시련
    난이도: ★★★★☆
    보상: 엔케팔린 2800~3800
          Legendary E.G.O 30% 확률 ⭐⭐
          WAW 환상체 선물 확정
====================================

⚠️ 어스름 시련은 극도로 위험합니다.
   패배 시 엔케팔린 -500 페널티
====================================
```

---

## 핵심 설계 철학

### 1. 선택의 의미

**4가지 선택 (여명/정오):**
- 안전한 학습
- 피할 색상 선택 가능
- "녹빛은 약하니까 녹빛만 하자"

**3가지 선택 (어스름/자정):**
- 피할 수 없음
- 모든 색상이 강력함
- 어떤 걸 선택해도 위험
- 리스크를 감수해야 함

### 2. 리스크-리워드 균형

```
여명 4개 평균 보상: 750 엔케팔린
어스름 3개 평균 보상: 3000 엔케팔린 (4배!)

하지만:
- 어스름은 패배 확률 높음
- 패배 시 페널티
- 실질적 기대값은 비슷하거나 약간 높음
```

### 3. 진행 곡선

```
Day 1~10 (여명 ZAYIN):
- 4가지 시련 색상 선택 → 학습
- 6 Phase 구조 → 충분한 준비 시간
- 낮은 난이도
- 안정적 성장

Day 11~20 (정오 TETH):
- 4가지 시련 색상 선택 → 여전히 안정
- 6 Phase 구조 → 충분한 준비 시간
- 중간 난이도

Day 21~35 (어스름 HE):
- 3가지 시련 색상 선택 → 긴장감 상승! ⚠️
- 5 Phase 구조 → 준비 시간 1단계 부족! 🔥
- 높은 난이도
- 큰 보상 or 큰 손실

Day 36~45 (자정 WAW):
- 3가지 시련 색상 선택 → 최대 긴장! ⚠️⚠️
- 5 Phase 구조 → 준비 시간 1단계 부족! 🔥
- 극한 난이도
- 게임 체인저급 보상

Day 46+ (백색 ALEPH):
- 1가지 시련만 → 피할 수 없음 ⚠️⚠️⚠️
- 6 Phase 구조 → 마지막 기회
- 엔드게임
```

---

## ECS 구현 설계

### 개요

**게임 코어 (core 프로젝트)**는 bevy_ecs를 사용하여 순수 게임 로직을 구현합니다.

**설계 원칙:**
- 인프라와 게임 로직 분리
- 순수 함수 중심 (테스트 용이성)
- 확장성 고려 (시스템 변경 대비)

---

### 업그레이드 시스템

#### The Bazaar 방식 (구매 시 Tier 증가)

**설계 결정:**
- 중복 소유 불가 (같은 환상체 1개만)
- 상점에서 같은 환상체 재구매 → Tier 증가
- 최대 Tier: 3

**구현 예시:**
```rust
fn shop_purchase_system(
    mut commands: Commands,
    shop_item: Abnormality,
    mut owned: Query<(Entity, &mut Abnormality), With<InBag>>,
    inventory: Res<PlayerInventory>,
) -> Result<(), String> {
    // 1. 중복 체크
    if let Some((entity, mut owned_abnormality)) = owned.iter_mut()
        .find(|(_, a)| a.id == shop_item.id)
    {
        // 이미 소유 중 → Tier 증가
        if owned_abnormality.tier >= 3 {
            return Err("이미 최대 등급입니다!".to_string());
        }

        owned_abnormality.tier += 1;

        // Tier 증가 시 스탯도 증가
        if let Ok((entity, mut health, mut attack)) =
            owned.get_component::<Health>(entity)
        {
            health.max = calculate_health_by_tier(
                &owned_abnormality,
                owned_abnormality.tier
            );
            attack.damage = calculate_attack_by_tier(
                &owned_abnormality,
                owned_abnormality.tier
            );
        }

        Ok(())
    } else {
        // 신규 구매 → 가방 추가
        check_bag_capacity()?;

        commands.spawn((
            shop_item,
            Health::default(),
            Attack::default(),
            InBag,
        ));

        Ok(())
    }
}

// Tier별 스탯 계산
fn calculate_health_by_tier(abnormality: &Abnormality, tier: u8) -> u32 {
    let base = match abnormality.risk_level {
        RiskLevel::ZAYIN => 500,
        RiskLevel::TETH => 600,
        RiskLevel::HE => 800,
        RiskLevel::WAW => 1000,
        RiskLevel::ALEPH => 1500,
    };

    // Tier 1: 100%, Tier 2: 140%, Tier 3: 200%
    match tier {
        1 => base,
        2 => (base as f32 * 1.4) as u32,
        3 => base * 2,
        _ => base,
    }
}
```

**확장성 고려:**
- Tier → Star 전환 가능 (롤토체스 방식)
- 수동 합성 시스템 추가 가능
- 업그레이드 방식 변경 용이

```rust
// 확장 예시: 수동 합성 (미래)
#[derive(Component)]
pub struct Star {
    pub level: u8,  // 1성, 2성, 3성
}

fn manual_merge_system(
    mut commands: Commands,
    selected: Vec<Entity>,  // 플레이어가 선택한 3개
    query: Query<&Abnormality, With<InBag>>,
) -> Result<(), String> {
    // 같은 환상체 3개인지 확인
    if selected.len() != 3 {
        return Err("3개를 선택해주세요!".to_string());
    }

    let first = query.get(selected[0])?;
    if !selected.iter().all(|&e| {
        query.get(e).map(|a| a.id == first.id).unwrap_or(false)
    }) {
        return Err("같은 환상체를 선택해주세요!".to_string());
    }

    // 3개 제거
    for &entity in &selected {
        commands.entity(entity).despawn();
    }

    // 2성 1개 생성
    commands.spawn((
        first.clone(),
        Star { level: 2 },
        InBag,
    ));

    Ok(())
}
```

---

### 중복 체크 로직

**환상체 중복 방지:**
```rust
fn check_duplicate_abnormality(
    id: &str,
    query: Query<&Abnormality, With<InBag>>,
) -> bool {
    query.iter().any(|a| a.id == id)
}
```

**E.G.O 선물 중복 체크 (장착만):**
```rust
fn check_equipped_gift_limit(
    query: Query<&Equipped>,
    inventory: Res<PlayerInventory>,
) -> Result<(), String> {
    let count = query.iter().count();
    if count >= inventory.max_gifts {
        return Err("선물 장착 공간이 부족합니다!".to_string());
    }
    Ok(())
}
```

---

### 시스템 구성 예시

```rust
// core/src/game/systems.rs

// 상점 시스템
pub fn shop_purchase_system(/* ... */) { }

// 가방 관리 시스템
pub fn bag_management_system(/* ... */) { }

// 전투 배치 시스템
pub fn deploy_system(/* ... */) { }

// E.G.O 선물 효과 적용 시스템
pub fn apply_gift_effects_system(/* ... */) { }

// 전투 시뮬레이션 시스템
pub fn battle_simulation_system(/* ... */) { }

// 레벨업 시스템
pub fn levelup_system(/* ... */) { }
```

---

### 확장성 전략

#### 1. Component 기반 능력 시스템

```rust
// 기본 Component
#[derive(Component)]
pub struct Health { pub current: u32, pub max: u32 }

#[derive(Component)]
pub struct Attack { pub damage: u32, pub interval_ms: u64 }

// 확장 Component
#[derive(Component)]
pub struct Regeneration { pub amount: u32, pub interval_ms: u64 }

#[derive(Component)]
pub struct Shield { pub amount: u32, pub duration_ms: u64 }

#[derive(Component)]
pub struct Poison { pub damage_per_tick: u32, pub remaining_ticks: u32 }
```

→ 새로운 능력 추가 시 Component만 추가하면 됨

#### 2. 업그레이드 방식 전환 대비

```rust
// 현재: Tier만 사용
pub struct Abnormality {
    pub tier: u8,  // 1, 2, 3
}

// 미래: Tier + Star 동시 사용
pub struct Abnormality {
    pub tier: u8,   // 구매 횟수
    pub star: u8,   // 합성 횟수
}

// 계산식도 유연하게
fn calculate_final_stats(tier: u8, star: u8) -> Stats {
    let tier_multiplier = 1.0 + (tier as f32 - 1.0) * 0.4;  // Tier당 +40%
    let star_multiplier = star as f32;                       // Star당 3배

    base_stats * tier_multiplier * star_multiplier
}
```

#### 3. 이벤트 시스템

```rust
#[derive(Event)]
pub enum GameEvent {
    AbnormalityAcquired { id: String },
    AbnormalityUpgraded { id: String, new_tier: u8 },
    GiftEquipped { gift_id: String, slot: usize },
    BattleStarted,
    BattleEnded { winner: PlayerId },
}

// System에서 Event 발행
fn shop_purchase_system(
    mut events: EventWriter<GameEvent>,
) {
    events.send(GameEvent::AbnormalityAcquired {
        id: "der_freischutz".to_string()
    });
}

// Event 구독
fn achievement_system(
    mut events: EventReader<GameEvent>,
) {
    for event in events.iter() {
        match event {
            GameEvent::AbnormalityUpgraded { id, new_tier } => {
                // 업적 체크
            }
            _ => {}
        }
    }
}
```

---

### game_server에서 core 사용

```rust
// game_server/src/game/player_game_actor/mod.rs
use game_core::game::world::GameWorld;
use game_core::game::components::*;

pub struct PlayerGameActor {
    world: GameWorld,  // bevy_ecs World
    player_id: Uuid,
}

impl PlayerGameActor {
    pub fn handle_shop_purchase(&mut self, item_id: &str) -> Result<(), String> {
        // core의 System 호출
        self.world.run_system::<shop_purchase_system>(item_id)?;

        // 상태 변경 후 클라이언트에 통지
        Ok(())
    }

    pub fn handle_deploy_to_battle(&mut self, entity_id: Uuid, lane: Lane) {
        self.world.run_system::<deploy_system>(entity_id, lane);
    }
}
```

---

## 참고 자료

### 원작 (Lobotomy Corporation)
- [나무위키 - 시련](https://namu.wiki/w/Lobotomy%20Corporation/%EC%8B%9C%EB%A0%A8)
- [Lobotomy Corporation Wiki - Ordeals](https://lobotomycorporation.wiki.gg/wiki/Ordeals)

### 관련 문서
- `ARCHITECTURE_STATUS.md` - 전체 아키텍처 현황
- `BATTLE_SYSTEM_DESIGN.md` - 전투 시스템 상세
- `EQUIPMENT_ARTIFACT_DESIGN.md` - 장비/아티팩트 시스템 (통합됨)

---

## 변경 이력

| 날짜 | 버전 | 변경 사항 |
|------|------|-----------|
| 2025-10-29 | 1.0 | 초안 작성 (로보토미 코퍼레이션 설정 기반 게임 설계) |
| 2025-10-29 | 1.1 | 아티팩트 시스템 수정: Cogito(코골로) → E.G.O Gift(E.G.O 선물) |
| 2025-10-29 | 1.2 | 위험 등급 재구성 (ZAYIN 추가), Phase 구조 변경 (어스름/자정 5 Phase, 백색 Phase 네이밍) |
| 2025-10-29 | 1.3 | 시너지 시스템 추가 (롤토체스 스타일), 기프트 획득 방법 TBD 처리 |
| 2025-11-01 | 1.4 | ECS 구현 설계 추가 (bevy_ecs 기반, The Bazaar 방식 업그레이드, Marker Component, 확장성 전략) |

---

**작성자**: Development Team
**최종 수정**: 2025-11-01
