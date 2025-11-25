# Core 프로젝트 UUID 일관성 리팩토링 설계서

**작성일**: 2025-01-XX
**목적**: 내부는 UUID, 외부 관측은 name으로 일관성 있는 설계 적용

---

## 1. 현재 문제점

### 1.1 식별자 혼용 문제
```rust
// ❌ 문제 1: id, name, uuid를 혼용
AbnormalityMetadata {
    id: String,      // "f-01-02" (RON 로딩용? 런타임용?)
    name: String,    // "Scorched Girl" (표시용)
    uuid: Uuid,      // f0102000-... (내부 로직용)
}

// ❌ 문제 2: ShopProduct가 런타임에 String id 보유
ShopProduct::Abnormality(String)  // "scorched_girl"
→ 메타데이터 접근할 때마다 game_data.get_by_id() 호출 필요

// ❌ 문제 3: 클라이언트와 name으로 통신
client.send(item.name)  // String 전송
→ name → uuid 변환 오버헤드
→ name 중복/변경 가능성
```

### 1.2 불필요한 메모리 사용
```rust
// ❌ id_map이 프로그램 종료까지 메모리에 상주
AbnormalityDatabase {
    items: Vec<AbnormalityMetadata>,
    uuid_map: HashMap<Uuid, ...>,     // 필요함
    id_map: HashMap<String, ...>,     // 로딩 후 불필요
}
```

### 1.3 get_by_id() 메서드 남용
```rust
// ❌ 런타임에 id로 조회
game_data.get_abnormality_from_product("scorched_girl")
game_data.get_equipment_from_product("justitia")
```

---

## 2. 설계 원칙

### 2.1 레이어별 역할 분리
```
┌─────────────────┐
│   RON File      │  id (String) ← 디자이너 가독성
└────────┬────────┘
         │ Hydrate (1회 변환)
         ↓
┌─────────────────┐
│   Runtime       │  uuid (Uuid) ← 내부 일관성
│   (GameCore)    │
└────────┬────────┘
         │ API Response
         ↓
┌─────────────────┐
│ Client/Logging  │  { uuid, name } ← 표시 + 식별
└─────────────────┘
```

### 2.2 핵심 규칙
1. **RON 파일**: `id` 사용 (디자이너 편의성)
2. **Hydrate 단계**: `id → uuid` 변환 (1회만)
3. **Runtime**: `uuid`만 사용
4. **API 응답**: `{ uuid, name }` 함께 전송
5. **로깅**: `uuid`로 조회 → `name` 표시

---

## 3. 변경 사항

### 3.1 RON 파일 (✅ 변경 없음)
**결정**: id 유지 (디자이너 가독성 중요)

```ron
// ✅ 그대로 유지
ShopMetadata(
    visible_items: [
        Abnormality("scorched_girl"),
        Equipment("justitia"),
    ],
)
```

**이유**:
- 디자이너가 UUID 외우기 어려움
- RON 파일 수정 시 가독성 중요
- Hydrate 단계에서 자동 변환 가능

---

### 3.2 Database 타입 변경

#### Before (❌)
```rust
pub struct AbnormalityDatabase {
    pub items: Vec<AbnormalityMetadata>,

    #[serde(skip)]
    uuid_map: HashMap<Uuid, AbnormalityMetadata>,

    #[serde(skip)]
    id_map: HashMap<String, AbnormalityMetadata>,  // ❌ 불필요
}

impl AbnormalityDatabase {
    pub fn get_by_id(&self, id: &str) -> Option<&AbnormalityMetadata> {
        self.id_map.get(id)  // ❌ 런타임에 사용
    }
}
```

#### After (✅)
```rust
pub struct AbnormalityDatabase {
    pub items: Vec<AbnormalityMetadata>,

    #[serde(skip)]
    uuid_map: HashMap<Uuid, AbnormalityMetadata>,
    // id_map 제거
}

impl AbnormalityDatabase {
    /// RON 역직렬화 후 uuid_map만 초기화
    pub fn init_map(&mut self) {
        self.uuid_map = self.items.iter()
            .map(|item| (item.uuid, item.clone()))
            .collect();
    }

    /// UUID로만 조회
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&AbnormalityMetadata> {
        self.uuid_map.get(uuid)
    }

    // get_by_id() 제거
}

// 로딩 시에만 사용하는 임시 헬퍼
fn build_id_to_uuid_map(items: &[AbnormalityMetadata]) -> HashMap<String, Uuid> {
    items.iter()
        .map(|item| (item.id.clone(), item.uuid))
        .collect()
}
```

**변경 대상**:
- `AbnormalityDatabase`
- `EquipmentDatabase`
- `ArtifactDatabase`

---

### 3.3 ShopProduct 역할 명확화

#### Before (❌)
```rust
// ❌ 역직렬화용 타입을 런타임에도 사용
pub enum ShopProduct {
    Abnormality(String),  // id를 런타임에 보유
    Equipment(String),
    Artifact(String),
}

impl ShopProduct {
    pub fn uuid(&self, game_data: &GameData) -> Uuid {
        // ❌ 매번 get_by_id() 호출
        match self {
            Self::Abnormality(id) => game_data.get_abnormality_from_product(id).unwrap().uuid,
            ...
        }
    }
}
```

#### After (✅)
```rust
// ✅ RON 역직렬화 전용 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopProductRaw {
    Abnormality(String),  // id
    Equipment(String),
    Artifact(String),
}

impl ShopProductRaw {
    /// id → uuid 변환 (Hydrate 시에만 사용)
    fn resolve_uuid(&self, items: &[AbnormalityMetadata | EquipmentMetadata | ...]) -> Option<Uuid> {
        let id_to_uuid_map = build_id_to_uuid_map(items);
        match self {
            Self::Abnormality(id) => id_to_uuid_map.get(id).copied(),
            Self::Equipment(id) => id_to_uuid_map.get(id).copied(),
            Self::Artifact(id) => id_to_uuid_map.get(id).copied(),
        }
    }
}

// ✅ 런타임용 타입 (ItemReference 재사용)
pub type ShopItem = ItemReference;  // Arc<Metadata> 포함, uuid 있음
```

---

### 3.4 ShopMetadata 변경

#### Before (❌)
```rust
pub struct ShopMetadata {
    pub name: String,
    pub uuid: Uuid,

    #[serde(rename = "items")]
    pub items_raw: Vec<ShopProduct>,  // ❌ String id 보유

    #[serde(skip)]
    pub visible_items: Vec<ShopItem>,  // ItemReference

    #[serde(skip)]
    pub hidden_items: Vec<ShopItem>,
}

impl ShopMetadata {
    pub fn hydrate(&mut self, game_data: &GameData, rng: &mut R) {
        // ❌ 매번 get_by_id() 호출
        let resolved = self.items_raw.iter()
            .map(|p| resolve_item_reference(p, game_data))
            .collect();
        ...
    }
}
```

#### After (✅)
```rust
pub struct ShopMetadata {
    pub name: String,
    pub uuid: Uuid,

    // ✅ RON 역직렬화 전용 필드 (rename으로 호환성 유지)
    #[serde(rename = "visible_items")]
    items_raw: Vec<ShopProductRaw>,

    // ✅ Hydrate 후 사용하는 필드
    #[serde(skip)]
    pub visible_items: Vec<ShopItem>,  // ItemReference

    #[serde(skip)]
    pub hidden_items: Vec<ShopItem>,
}

impl ShopMetadata {
    /// Hydrate: id → uuid 변환 후 ItemReference 로딩
    pub fn hydrate(&mut self, game_data: &GameData, rng: &mut R) -> Result<(), GameError> {
        // 1. id → uuid 변환 맵 생성 (1회만)
        let id_to_uuid_map = build_id_to_uuid_map(&game_data.abnormalities.items);

        // 2. items_raw → uuid 변환
        let uuids: Vec<Uuid> = self.items_raw.iter()
            .filter_map(|raw| raw.resolve_uuid(&id_to_uuid_map))
            .collect();

        // 3. uuid → ItemReference 조회 (O(1))
        let mut items: Vec<ShopItem> = uuids.iter()
            .filter_map(|uuid| game_data.item_uuid_map.get(uuid).cloned())
            .collect();

        // 4. Shuffle 및 visible/hidden 분리
        if self.can_reroll {
            items.shuffle(rng);
            let half = (items.len() + 1) / 2;
            self.visible_items = items.iter().take(half).cloned().collect();
            self.hidden_items = items.into_iter().skip(half).collect();
        } else {
            self.visible_items = items;
            self.hidden_items.clear();
        }

        Ok(())
    }
}
```

---

### 3.5 GameData 변경

#### Before (❌)
```rust
impl GameData {
    // ❌ id로 조회하는 메서드들
    pub fn get_abnormality_from_product(&self, id: &str) -> Option<&AbnormalityMetadata> {
        self.abnormalities.get_by_id(id)
    }

    pub fn get_equipment_from_product(&self, id: &str) -> Option<&EquipmentMetadata> {
        self.equipments.get_by_id(id)
    }

    pub fn get_artifact_from_product(&self, id: &str) -> Option<&ArtifactMetadata> {
        self.artifacts.get_by_id(id)
    }
}
```

#### After (✅)
```rust
impl GameData {
    // ✅ uuid로만 조회
    pub fn get_item_by_uuid(&self, uuid: &Uuid) -> Option<&ItemReference> {
        self.item_uuid_map.get(uuid)
    }

    pub fn get_item_price(&self, uuid: &Uuid) -> Option<u32> {
        self.get_item_by_uuid(uuid).map(|item| item.price())
    }

    pub fn get_item_name(&self, uuid: &Uuid) -> Option<&str> {
        self.get_item_by_uuid(uuid).map(|item| item.name())
    }

    // get_xxx_from_product() 메서드 모두 제거
}
```

---

### 3.6 클라이언트 통신 프로토콜

#### Before (❌)
```rust
// ❌ name만 전송
BehaviorResult::RequestPhaseData(PhaseEvent {
    shop: ShopMetadata {
        name: "아티팩트 상인 라헬",
        visible_items: [
            { name: "One Sin", ... }  // name만
        ]
    }
})
```

#### After (✅)
```rust
// ✅ uuid + name 함께 전송
BehaviorResult::RequestPhaseData(PhaseEvent {
    shop: ShopMetadata {
        uuid: "550e8400-...",
        name: "아티팩트 상인 라헬",
        visible_items: [
            {
                uuid: "650e8400-...",
                name: "One Sin",
                price: 100,
            }
        ]
    }
})

// 클라이언트는 uuid를 저장하고 전송
PlayerBehavior::PurchaseItem {
    item_uuid: Uuid::parse_str("650e8400-...").unwrap(),
    item_category: Category::Artifact,
}
```

---

### 3.7 로깅 개선

#### Before (❌)
```rust
tracing::info!("Player purchased item: {}", uuid);
// 출력: Player purchased item: 650e8400-e29b-41d4-a716-446655440011
```

#### After (✅)
```rust
// ✅ UUID로 name 조회 후 로깅
let name = game_data.get_item_name(&uuid).unwrap_or("Unknown");
tracing::info!("Player purchased item: {} ({})", name, uuid);
// 출력: Player purchased item: One Sin (650e8400-...)

// 또는 Display trait 구현
impl fmt::Display for ItemReference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.uuid())
    }
}

tracing::info!("Player purchased item: {}", item_ref);
```

---

## 4. 마이그레이션 전략

### 4.1 변경 순서 (의존성 고려)

```
1. Database 타입 (AbnormalityDatabase, EquipmentDatabase, ArtifactDatabase)
   - id_map 제거
   - get_by_id() 제거
   - init_map() 수정

2. ShopProductRaw 타입 추가 (기존 ShopProduct 대체)
   - resolve_uuid() 메서드 추가

3. ShopMetadata
   - items_raw 타입 변경
   - hydrate() 로직 개선

4. GameData
   - get_xxx_from_product() 제거
   - Hydrate 시 임시 id_map 생성 로직 추가

5. ShopExecutor 및 기타 사용처
   - uuid 기반 조회로 변경

6. 테스트 코드 수정
```

### 4.2 점진적 vs 일괄 변경

**선택**: **일괄 변경** (Breaking Change)

**이유**:
- id/uuid 혼용 상태는 버그 유발 가능성 높음
- 중간 상태 유지 시 코드 복잡도 증가
- 현재 프로젝트 초기 단계, 호환성 부담 적음

---

## 5. 영향도 분석

### 5.1 변경 파일 목록

#### Core 변경 (필수)
```
core/src/game/data/
├── abnormality_data.rs    (id_map 제거, get_by_id 제거)
├── equipment_data.rs      (id_map 제거, get_by_id 제거)
├── artifact_data.rs       (id_map 제거, get_by_id 제거)
├── shop_data.rs           (ShopProductRaw 추가, hydrate 개선)
├── mod.rs                 (get_xxx_from_product 제거)
└── random_event_data.rs   (필요시)

core/src/game/events/event_selection/
└── shop.rs                (uuid 기반 조회로 변경)

core/src/game/
├── world.rs               (영향 없음 - 이미 uuid 사용)
└── behavior.rs            (영향 없음 - API는 uuid)
```

#### 테스트 변경
```
core/tests/
├── common/mod.rs          (테스트 헬퍼 수정)
└── dawn_phase1_selection.rs  (uuid 기반 테스트로 변경)
```

#### RON 파일 (변경 없음)
```
game_resources/data/
├── abnormalities.ron      (✅ 그대로)
├── equipments.ron         (✅ 그대로)
├── artifacts.ron          (✅ 그대로)
└── events/shops.ron       (✅ 그대로)
```

### 5.2 Game Server 영향도

**영향**: **최소**

**이유**:
- `GameCore::execute()` API는 이미 uuid 기반
- `BehaviorResult`에 uuid 포함되도록 수정만 필요
- game_server는 GameCore를 블랙박스로 사용

**필요한 변경**:
```rust
// game_server/src/game/player_game_actor.rs
// BehaviorResult 응답에 uuid 포함 확인
match result {
    BehaviorResult::RequestPhaseData(event) => {
        // ✅ event.shop.uuid 사용
        // ✅ event.shop.visible_items[].uuid() 사용
    }
}
```

---

## 6. 리스크 및 대응

### 6.1 리스크

| 리스크 | 심각도 | 대응 |
|--------|--------|------|
| Hydrate 시 id → uuid 변환 실패 | 중 | 로딩 시 검증, 에러 로그 명확히 |
| RON 파일 오타로 id 매칭 실패 | 중 | init_map() 에서 누락 체크, 패닉 |
| 테스트 코드 대량 수정 필요 | 하 | 테스트 헬퍼 함수로 일괄 처리 |
| 클라이언트와 통신 프로토콜 변경 | 상 | 현재 클라이언트 없음, 미래 고려 |

### 6.2 롤백 전략

- Git 브랜치 사용 (`feat/uuid-refactoring`)
- 커밋 단위: 파일별 변경 (원자적)
- 문제 발생 시 revert 가능

---

## 7. 기대 효과

### 7.1 성능
- **id_map 제거**: 메모리 사용량 감소
- **HashMap 키 크기**: String → Uuid (16 bytes vs 힙 할당)
- **get_by_id() 호출 제거**: O(1) → O(1) (하지만 문자열 해싱 제거)

### 7.2 코드 품질
- **타입 안전성**: Uuid는 컴파일 타임 체크
- **일관성**: 내부 로직에서 단일 식별자 사용
- **가독성**: 레이어별 역할 명확 (RON=id, Runtime=uuid, Display=name)

### 7.3 유지보수성
- **버그 감소**: 잘못된 id 문자열 전달 불가
- **리팩토링 안전성**: id 변경해도 uuid 불변
- **디버깅 개선**: uuid로 name 조회 → 로그 가독성

---

## 8. 승인 후 작업 계획

### Phase 1: Database 타입 (1-2시간)
1. `abnormality_data.rs` 수정
2. `equipment_data.rs` 수정
3. `artifact_data.rs` 수정
4. `random_event_data.rs` 수정 (필요시)

### Phase 2: ShopProduct → ShopProductRaw (1시간)
1. `shop_data.rs`에 `ShopProductRaw` 추가
2. `ShopMetadata.items_raw` 타입 변경
3. `hydrate()` 로직 개선

### Phase 3: GameData 정리 (30분)
1. `get_xxx_from_product()` 제거
2. 로딩 로직에서 임시 id_map 사용

### Phase 4: 사용처 수정 (1시간)
1. `shop.rs` 수정
2. 기타 id 기반 조회 제거

### Phase 5: 테스트 수정 (1시간)
1. `common/mod.rs` 헬퍼 수정
2. 테스트 케이스 uuid 기반으로 변경

### Phase 6: 로깅 개선 (30분)
1. Display trait 구현
2. 로그 메시지 개선

**총 예상 시간**: 약 6시간

---

## 9. 체크리스트

- [ ] 설계서 승인
- [ ] Database 타입 리팩토링
- [ ] ShopProductRaw 구현
- [ ] ShopMetadata hydrate 개선
- [ ] GameData 정리
- [ ] 사용처 수정
- [ ] 테스트 수정
- [ ] 빌드 성공 확인
- [ ] 테스트 통과 확인
- [ ] 커밋 및 푸시

---

**검토 요청**: 위 설계로 진행해도 될까요? 수정 사항이 있으면 알려주세요.
