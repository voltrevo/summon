use std::collections::BTreeMap;

use valuescript_vm::{
  operations::{op_minus, op_mul, op_plus, op_triple_eq_impl},
  type_error_builtin::ToTypeError,
  unary_op::UnaryOp,
  vs_object::VsObject,
  vs_value::{ToDynamicVal, ToVal, Val, VsType},
  LoadFunctionResult, ValTrait,
};

use crate::{
  circuit_signal::{CircuitSignal, CircuitSignalData},
  val_dynamic_downcast::val_dynamic_downcast,
};

/**
 * Merges two values after branching.
 *
 * Consider this code:
 *
 * ```ts
 * let x = input;
 *
 * if (cond) {
 *   x += x;
 * }
 *
 * // merge here
 * ```
 *
 * Suppose `cond` and `input` are signals.
 *
 * The VM does not know whether to skip the `if` block, so it forks the
 * execution and can then merge it back together afterwards.
 *
 * After the merge, we want `x` to be `!cond * input + cond * (input + input)`.
 *
 * However, this is just the simplest case where `x` is just a signal. We
 * should also allow `x` to be any composite structure such as an array of N
 * signals, or any complex structure (nested arrays, objects, etc). We just
 * require that the structure is the same and we can merge element by element.
 * If the structure is different, we abort with a compilation error.
 */
pub fn arithmetic_merge(left_flag: &Val, left: &Val, right_flag: &Val, right: &Val) -> Val {
  if quick_val_eq(left, right) {
    return left.clone();
  }

  arithmetic_merge_impl(&gen_direct_merge(left_flag, right_flag), left, right)
}

fn gen_direct_merge<'a>(
  left_flag: &'a Val,
  right_flag: &'a Val,
) -> Box<dyn Fn(&'a Val, &'a Val) -> Val + 'a> {
  if let Some(direct_merge) = optimized_direct_merge(false, left_flag, right_flag) {
    return direct_merge;
  }

  if let Some(direct_merge) = optimized_direct_merge(true, left_flag, right_flag) {
    return direct_merge;
  }

  Box::new(|left, right| {
    op_plus(
      &op_mul(left_flag, left).unwrap(),
      &op_mul(right_flag, right).unwrap(),
    )
    .unwrap()
  })
}

fn optimized_direct_merge<'a>(
  swap: bool,
  left_flag: &'a Val,
  right_flag: &'a Val,
) -> Option<Box<dyn Fn(&'a Val, &'a Val) -> Val + 'a>> {
  let (left_flag, right_flag) = if swap {
    (right_flag, left_flag)
  } else {
    (left_flag, right_flag)
  };

  if let Some(left_flag) = val_dynamic_downcast::<CircuitSignal>(left_flag) {
    if let CircuitSignalData::UnaryOp(UnaryOp::Not, input) = &left_flag.data {
      if quick_val_eq(input, right_flag) {
        // left_flag = 1 - right_flag
        // out = left_flag * left + right_flag * right
        //     = (1 - right_flag) * left + right_flag * right
        //     = left + right_flag * (right - left)
        return Some(Box::new(move |left, right| {
          let (left, right) = if swap { (right, left) } else { (left, right) };

          op_plus(
            left,
            &op_mul(right_flag, &op_minus(right, left).unwrap()).unwrap(),
          )
          .unwrap()
        }));
      }
    }
  }

  None
}

fn arithmetic_merge_impl<'a>(
  direct_merge: &impl Fn(&'a Val, &'a Val) -> Val,
  left: &'a Val,
  right: &'a Val,
) -> Val {
  if quick_val_eq(left, right) {
    return left.clone();
  }

  if is_circuit_ish(left) && is_circuit_ish(right) {
    return direct_merge(left, right);
  }

  match (left, right) {
    (Val::Array(left_arr), Val::Array(right_arr)) => {
      if left_arr.elements.len() != right_arr.elements.len() {
        return CouldNotMerge(left.clone(), right.clone()).to_dynamic_val();
      }

      return (0..left_arr.elements.len())
        .map(|i| arithmetic_merge_impl(direct_merge, &left_arr.elements[i], &right_arr.elements[i]))
        .collect::<Vec<_>>()
        .to_val();
    }
    (Val::Object(left), Val::Object(right)) => {
      return VsObject {
        string_map: arithmetic_merge_map(direct_merge, &left.string_map, &right.string_map),
        symbol_map: arithmetic_merge_map(direct_merge, &left.symbol_map, &right.symbol_map),
        prototype: arithmetic_merge_impl(direct_merge, &left.prototype, &right.prototype),
      }
      .to_val()
    }
    _ => {}
  };

  if let Ok(true) = op_triple_eq_impl(left, right) {
    return left.clone();
  }

  CouldNotMerge(left.clone(), right.clone()).to_dynamic_val()
}

fn quick_val_eq(left: &Val, right: &Val) -> bool {
  match (left, right) {
    (Val::Void, Val::Void) => true,
    (Val::Undefined, Val::Undefined) => true,
    (Val::Null, Val::Null) => true,
    (Val::Bool(left), Val::Bool(right)) => left == right,
    (Val::Number(left), Val::Number(right)) => left == right,
    (Val::BigInt(left), Val::BigInt(right)) => left == right,
    (Val::Symbol(left), Val::Symbol(right)) => left == right,
    (Val::String(left), Val::String(right)) => std::ptr::eq(&**left, &**right),
    (Val::Array(left), Val::Array(right)) => std::ptr::eq(&**left, &**right),
    (Val::Object(left), Val::Object(right)) => std::ptr::eq(&**left, &**right),
    (Val::Function(left), Val::Function(right)) => std::ptr::eq(&**left, &**right),
    (Val::Class(left), Val::Class(right)) => std::ptr::eq(&**left, &**right),
    (Val::Static(left), Val::Static(right)) => std::ptr::eq(&**left, &**right),
    (Val::Dynamic(left), Val::Dynamic(right)) => std::ptr::eq(&**left, &**right),
    (Val::CopyCounter(left), Val::CopyCounter(right)) => std::ptr::eq(&**left, &**right),
    (Val::StoragePtr(left), Val::StoragePtr(right)) => std::ptr::eq(&**left, &**right),
    _ => false,
  }
}

fn is_circuit_ish(val: &Val) -> bool {
  match val {
    Val::Bool(_) => true,
    Val::Number(_) => true,
    Val::Dynamic(_) => val_dynamic_downcast::<CircuitSignal>(val).is_some(),
    _ => false,
  }
}

fn arithmetic_merge_map<'a, K: std::cmp::Ord + Clone>(
  direct_merge: &impl Fn(&'a Val, &'a Val) -> Val,
  left: &'a BTreeMap<K, Val>,
  right: &'a BTreeMap<K, Val>,
) -> BTreeMap<K, Val> {
  if left.len() != right.len() {
    panic!("Could not merge");
  }

  let mut res = BTreeMap::<K, Val>::new();

  for (k, left_value) in left {
    match right.get(k) {
      Some(right_value) => res.insert(
        k.clone(),
        arithmetic_merge_impl(direct_merge, left_value, right_value),
      ),
      None => panic!("Could not merge"),
    };
  }

  res
}

#[derive(Clone)]
pub struct CouldNotMerge(pub Val, pub Val);

impl ValTrait for CouldNotMerge {
  fn typeof_(&self) -> valuescript_vm::vs_value::VsType {
    VsType::Object
  }

  fn to_number(&self) -> f64 {
    f64::NAN
  }

  fn to_index(&self) -> Option<usize> {
    None
  }

  fn is_primitive(&self) -> bool {
    false
  }

  fn is_truthy(&self) -> bool {
    true
  }

  fn is_nullish(&self) -> bool {
    false
  }

  fn bind(&self, _params: Vec<Val>) -> Option<Val> {
    None
  }

  fn as_bigint_data(&self) -> Option<num_bigint::BigInt> {
    None
  }

  fn as_array_data(&self) -> Option<std::rc::Rc<valuescript_vm::vs_array::VsArray>> {
    None
  }

  fn as_class_data(&self) -> Option<std::rc::Rc<valuescript_vm::vs_class::VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, _key: &Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn has(&self, _key: &Val) -> Option<bool> {
    Some(false)
  }

  fn submov(&mut self, _key: &Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign subscript to CouldNotMerge".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CouldNotMerge({}, {})", self.0.pretty(), self.1.pretty())
  }

  fn codify(&self) -> String {
    format!("CouldNotMerge({}, {})", self.0, self.1)
  }
}

impl std::fmt::Display for CouldNotMerge {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CouldNotMerge({}, {})", self.0, self.1)
  }
}
