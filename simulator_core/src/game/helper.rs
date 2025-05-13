#[macro_export]
macro_rules! downcast_effect {
    ($effect:expr, $target_type:ty) => {
        if $effect.get_effect_type() == <$target_type>::static_effect_type() {
            if let Some(specific) = $effect.as_any().downcast_ref::<$target_type>() {
                Some(specific)
            } else {
                None
            }
        } else {
            None
        }
    };
}

pub struct Resoruce {}
impl Resoruce {
    pub fn new(a: usize, b: usize) -> Self {
        Self {}
    }
}
