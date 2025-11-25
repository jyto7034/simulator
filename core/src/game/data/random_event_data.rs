use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 랜덤 이벤트 위험도
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventRiskLevel {
    Low,
    Medium,
    High,
}

use crate::game::events::event_selection::random::RandomEventType;

/// 랜덤 이벤트 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEventMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub event_type: RandomEventType,
    pub name: String,
    pub description: String,
    pub image: String,
    pub risk_level: EventRiskLevel,
}

/// RON 파일 최상위 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEventDatabase {
    pub events: Vec<RandomEventMetadata>,

    #[serde(skip)]
    event_map: HashMap<Uuid, RandomEventMetadata>,
}

impl RandomEventDatabase {
    /// Database 생성 (HashMap 초기화)
    pub fn new(events: Vec<RandomEventMetadata>) -> Self {
        let event_map = events.iter().map(|e| (e.uuid, e.clone())).collect();

        Self { events, event_map }
    }

    /// RON 역직렬화 후 HashMap 초기화
    pub fn init_map(&mut self) {
        self.event_map = self.events.iter().map(|e| (e.uuid, e.clone())).collect();
    }

    /// ID로 메타데이터 조회 (여전히 O(n), 자주 사용 안함)
    pub fn get_by_id(&self, id: &str) -> Option<&RandomEventMetadata> {
        self.events.iter().find(|e| e.id == id)
    }

    /// UUID로 메타데이터 조회 (O(1))
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&RandomEventMetadata> {
        self.event_map.get(uuid)
    }
}
