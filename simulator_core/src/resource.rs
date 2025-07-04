///
/// 수정 가능한 자원에 대해서 증감 및 관련 편의 메소드를 제공하는 trait
///

pub trait Resource {
    /// 현재 값을 가져옴
    fn get_value(&self) -> i32;

    /// 기본 값을 가져옴
    fn get_base_value(&self) -> i32;

    /// 현재 값을 설정
    fn set_value(&mut self, value: i32);

    /// 값을 증가
    fn increase(&mut self, amount: i32) {
        self.set_value(self.get_value() + amount);
    }

    /// 값을 감소
    fn decrease(&mut self, amount: i32) {
        self.set_value(self.get_value() - amount);
    }

    /// 최소값 제한
    fn get_min_value(&self) -> Option<i32> {
        None // 기본적으로는 제한 없음
    }

    /// 최대값 제한
    fn get_max_value(&self) -> Option<i32> {
        None // 기본적으로는 제한 없음
    }

    /// 제한을 고려한 값 설정
    fn set_value_with_limits(&mut self, value: i32) {
        let value = if let Some(min) = self.get_min_value() {
            value.max(min)
        } else {
            value
        };

        let value = if let Some(max) = self.get_max_value() {
            value.min(max)
        } else {
            value
        };

        self.set_value(value);
    }

    /// 퍼센트 기반 증가
    fn increase_percent(&mut self, percent: f32) {
        let increase = (self.get_base_value() as f32 * percent).round() as i32;
        self.increase(increase);
    }
}

/// `Resource` 트레이트를 확장하여 자원에 대한 추가적인 편의 메소드를 제공합니다.
/// 주로 현재 값과 기본 값 간의 비교, 초기화 등에 사용됩니다.
// TODO: 더 많은 확장 기능 (예: 버프/디버프 적용/해제) 추가 고려
// TODO: 특정 조건에 따라 값을 변경하는 로직 추가 고려
pub trait ResourceExtension: Resource {
    /// 기본값으로 초기화
    fn reset_to_base(&mut self) {
        self.set_value(self.get_base_value());
    }

    /// 현재 값이 기본값보다 높은지 확인
    fn is_buffed(&self) -> bool {
        self.get_value() > self.get_base_value()
    }

    /// 현재 값이 기본값보다 낮은지 확인
    fn is_debuffed(&self) -> bool {
        self.get_value() < self.get_base_value()
    }

    /// 현재 값이 기본값과 다른지 확인
    fn is_modified(&self) -> bool {
        self.get_value() != self.get_base_value()
    }

    /// 버프/디버프의 차이값 반환
    fn get_modification_amount(&self) -> i32 {
        self.get_value() - self.get_base_value()
    }
}

#[derive(Clone)]
/// 카드 스펙에 사용되는 자원을 나타내는 구조체입니다.
/// `value`는 현재 값, `base`는 기본 값을 저장합니다.
// TODO: 자원 타입에 대한 제네릭 파라미터 추가 고려 (현재는 i32로 고정)
// TODO: 디버깅 편의를 위한 `Debug` 트레이트 구현 고려
pub struct CardSpecsResource {
    value: i32,
    base: i32,
}

/// `CardSpecsResource` 구조체에 `Resource` 트레이트를 구현합니다.
impl Resource for CardSpecsResource {
    fn get_value(&self) -> i32 {
        self.value
    }

    fn get_base_value(&self) -> i32 {
        self.base
    }

    fn set_value(&mut self, value: i32) {
        self.value = value;
    }
}

/// `CardSpecsResource` 구조체에 `ResourceExtension` 트레이트를 구현합니다.
impl ResourceExtension for CardSpecsResource {}

impl CardSpecsResource {
    /// 새로운 `CardSpecsResource` 인스턴스를 생성합니다.
    ///
    /// # Arguments
    ///
    /// * `value` - 자원의 초기 값 및 기본 값으로 설정될 값입니다.
    pub fn new(value: i32) -> Self {
        Self { value, base: value }
    }
}
