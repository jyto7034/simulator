use uuid::Uuid;

use crate::game::data::equipment_data::EquipmentType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotLayoutKind {
    /// Weapon/Suit/Accessory 1개씩
    ByType,
    /// 타입 무시, 총 3개
    Any3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquippedRef {
    /// 소유 인스턴스 UUID (복사본마다 다름)
    pub instance_uuid: Uuid,
    /// 아이템 베이스 UUID (같은 아이템이면 동일)
    pub base_uuid: Uuid,
    pub equipment_type: EquipmentType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemSlotError {
    SlotOccupied,
    SlotFull,
    DuplicateBaseDisallowed,
    CannotRepresentInLayout,
}

#[derive(Debug, Clone)]
pub struct ItemSlot {
    layout: SlotLayout,
}

#[derive(Debug, Clone)]
enum SlotLayout {
    ByType {
        weapon: Option<EquippedRef>,
        suit: Option<EquippedRef>,
        accessory: Option<EquippedRef>,
    },
    Any3 {
        items: Vec<EquippedRef>,
    },
}

impl Default for ItemSlot {
    fn default() -> Self {
        Self::new(SlotLayoutKind::ByType)
    }
}

impl ItemSlot {
    pub fn new(kind: SlotLayoutKind) -> Self {
        let layout = match kind {
            SlotLayoutKind::ByType => SlotLayout::ByType {
                weapon: None,
                suit: None,
                accessory: None,
            },
            SlotLayoutKind::Any3 => SlotLayout::Any3 { items: Vec::new() },
        };

        Self { layout }
    }

    pub fn layout_kind(&self) -> SlotLayoutKind {
        match self.layout {
            SlotLayout::ByType { .. } => SlotLayoutKind::ByType,
            SlotLayout::Any3 { .. } => SlotLayoutKind::Any3,
        }
    }

    pub fn len(&self) -> usize {
        match &self.layout {
            SlotLayout::ByType {
                weapon,
                suit,
                accessory,
            } => weapon.is_some() as usize + suit.is_some() as usize + accessory.is_some() as usize,
            SlotLayout::Any3 { items } => items.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains_base(&self, base_uuid: Uuid) -> bool {
        self.iter().any(|r| r.base_uuid == base_uuid)
    }

    /// 장착.
    ///
    /// - 귀속 룰: 이미 슬롯이 차 있으면 교체 없이 무조건 실패합니다.
    /// - 중복(base_uuid) 룰: `allow_duplicate_base == false`이면 같은 base_uuid를 추가할 수 없습니다.
    pub fn equip(
        &mut self,
        item: EquippedRef,
        allow_duplicate_base: bool,
    ) -> Result<(), ItemSlotError> {
        if !allow_duplicate_base && self.contains_base(item.base_uuid) {
            return Err(ItemSlotError::DuplicateBaseDisallowed);
        }

        match &mut self.layout {
            SlotLayout::ByType {
                weapon,
                suit,
                accessory,
            } => {
                let target = match item.equipment_type {
                    EquipmentType::Weapon => weapon,
                    EquipmentType::Suit => suit,
                    EquipmentType::Accessory => accessory,
                };

                if target.is_some() {
                    return Err(ItemSlotError::SlotOccupied);
                }

                *target = Some(item);
                Ok(())
            }
            SlotLayout::Any3 { items } => {
                if items.len() >= 3 {
                    return Err(ItemSlotError::SlotFull);
                }
                items.push(item);
                Ok(())
            }
        }
    }

    /// 해제 아이템 전용: 모든 장착 아이템을 해제하고 반환합니다.
    pub fn unequip_all(&mut self) -> Vec<EquippedRef> {
        match &mut self.layout {
            SlotLayout::ByType {
                weapon,
                suit,
                accessory,
            } => {
                let mut removed = Vec::new();
                if let Some(item) = weapon.take() {
                    removed.push(item);
                }
                if let Some(item) = suit.take() {
                    removed.push(item);
                }
                if let Some(item) = accessory.take() {
                    removed.push(item);
                }
                removed
            }
            SlotLayout::Any3 { items } => std::mem::take(items),
        }
    }

    /// 결정적 순서로 iterate 합니다.
    pub fn iter(&self) -> Box<dyn Iterator<Item = &EquippedRef> + '_> {
        match &self.layout {
            SlotLayout::ByType {
                weapon,
                suit,
                accessory,
            } => Box::new(weapon.iter().chain(suit.iter()).chain(accessory.iter())),
            SlotLayout::Any3 { items } => Box::new(items.iter()),
        }
    }

    pub fn switch_layout(&mut self, kind: SlotLayoutKind) -> Result<(), ItemSlotError> {
        if self.layout_kind() == kind {
            return Ok(());
        }

        let equipped: Vec<EquippedRef> = self.iter().copied().collect();
        let equipped_len = equipped.len();

        let mut next = ItemSlot::new(kind);
        for item in equipped.into_iter() {
            match kind {
                SlotLayoutKind::ByType => {
                    // ByType에서는 base 중복 여부는 의미 없고(슬롯 단일),
                    // 표현 가능성만 체크하면 된다.
                    next.equip(item, true)?;
                }
                SlotLayoutKind::Any3 => {
                    // Any3로 갈 때는 그대로 담으면 된다.
                    next.equip(item, true)?;
                }
            }
        }

        // Any3 -> ByType에서 표현 불가능한 경우를 명확한 에러로 매핑
        if matches!(kind, SlotLayoutKind::ByType) && next.len() != equipped_len {
            return Err(ItemSlotError::CannotRepresentInLayout);
        }

        self.layout = next.layout;
        Ok(())
    }
}
