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
pub struct CardSpecsResource {
    value: i32,
    base: i32,
}

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

impl ResourceExtension for CardSpecsResource {}

impl CardSpecsResource {
    pub fn new(value: i32) -> Self {
        Self { value, base: value }
    }
}
