use std::{cell::RefCell, rc::Rc};

use num_bigint::BigInt;
use valuescript_vm::{
  binary_op::BinaryOp,
  type_error_builtin::ToTypeError,
  unary_op::UnaryOp,
  vs_value::{ToDynamicVal, Val},
  LoadFunctionResult, ValTrait,
};

use crate::id_generator::IdGenerator;
use valuescript_vm::vs_value::VsType;

#[derive(Clone)]
pub enum CircuitNumberData {
  Input,
  UnaryOp(UnaryOp, Rc<CircuitNumber>),
  BinaryOp(BinaryOp, Rc<CircuitNumber>, Val),
}

#[derive(Clone)]
pub struct CircuitNumber {
  pub data: CircuitNumberData,
  pub id: usize,
  pub id_generator: Rc<RefCell<IdGenerator>>,
}

impl CircuitNumber {
  pub fn new(id_generator: &Rc<RefCell<IdGenerator>>, data: CircuitNumberData) -> Self {
    CircuitNumber {
      data,
      id: id_generator.borrow_mut().gen(),
      id_generator: id_generator.clone(),
    }
  }
}

impl ValTrait for CircuitNumber {
  fn typeof_(&self) -> VsType {
    VsType::Number
  }

  fn to_number(&self) -> f64 {
    f64::NAN
  }

  fn to_index(&self) -> Option<usize> {
    panic!("Not implemented: using CircuitNumber as index")
  }

  fn is_primitive(&self) -> bool {
    false
  }

  fn is_truthy(&self) -> bool {
    panic!("Not implemented: truthiness of CircuitNumber")
  }

  fn is_nullish(&self) -> bool {
    false
  }

  fn bind(&self, _params: Vec<Val>) -> Option<Val> {
    None
  }

  fn as_bigint_data(&self) -> Option<BigInt> {
    None
  }

  fn as_array_data(&self) -> Option<Rc<valuescript_vm::vs_array::VsArray>> {
    None
  }

  fn as_class_data(&self) -> Option<Rc<valuescript_vm::vs_class::VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, _key: &Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn has(&self, _key: &Val) -> Option<bool> {
    None
  }

  fn submov(&mut self, _key: &Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of CircuitNumber".to_type_error())
  }

  fn override_binary_op(&self, op: BinaryOp, right: &Val) -> Option<Result<Val, Val>> {
    if right.typeof_() != VsType::Number {
      return None;
    }

    Some(Ok(
      CircuitNumber::new(
        &self.id_generator,
        CircuitNumberData::BinaryOp(op, Rc::new(self.clone()), right.clone()),
      )
      .to_dynamic_val(),
    ))
  }

  fn override_unary_op(&self, op: UnaryOp) -> Option<Result<Val, Val>> {
    Some(Ok(
      CircuitNumber::new(
        &self.id_generator,
        CircuitNumberData::UnaryOp(op, Rc::new(self.clone())),
      )
      .to_dynamic_val(),
    ))
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[CircuitNumber]")
  }

  fn codify(&self) -> String {
    "[CircuitNumber]".into()
  }
}

impl std::fmt::Display for CircuitNumber {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[CircuitNumber]")
  }
}
