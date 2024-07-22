use std::any::Any;

use valuescript_vm::vs_value::Val;

pub fn val_dynamic_downcast<T: Any>(val: &Val) -> Option<&T> {
  match val {
    Val::Dynamic(dynamic) => dynamic.as_any().downcast_ref::<T>(),
    _ => None,
  }
}
