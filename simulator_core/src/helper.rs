pub fn is_trait_implemented<T: ?Sized + 'static>() -> bool {
    let type_id = std::any::TypeId::of::<T>();
    let debug_type_id = std::any::TypeId::of::<dyn std::fmt::Debug>();
    type_id == debug_type_id
}
