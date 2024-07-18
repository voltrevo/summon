use std::rc::Rc;

use crate::{
  bytecode::{Bytecode, DecoderMaker},
  bytecode_decoder::BytecodeDecoder,
  bytecode_stack_frame::BytecodeStackFrame,
  make_generator_frame::MakeGeneratorFrame,
};

use valuescript_vm::{
  internal_error_builtin::ToInternalError,
  type_error_builtin::ToTypeError,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_value::{ToDynamicVal, Val, VsType},
  LoadFunctionResult, StackFrame, ValTrait,
};

#[derive(Debug, Clone)]
pub struct CsFunction {
  pub bytecode: Rc<Bytecode>,
  pub meta_pos: Option<usize>,
  pub is_generator: bool,
  pub register_count: usize,
  pub parameter_count: usize,
  pub start: usize,
  pub binds: Vec<Val>,
}

impl CsFunction {
  pub fn bind(&self, params: Vec<Val>) -> Self {
    let mut new_binds = self.binds.clone();

    for p in params {
      new_binds.push(p);
    }

    Self {
      bytecode: self.bytecode.clone(),
      meta_pos: self.meta_pos,
      is_generator: self.is_generator,
      register_count: self.register_count,
      parameter_count: self.parameter_count,
      start: self.start,
      binds: new_binds,
    }
  }

  #[allow(dead_code)] // TODO: CsFunction comparison
  pub fn content_hash(&self) -> Result<[u8; 32], Val> {
    match self.meta_pos {
      Some(p) => match self.bytecode.decoder(p).decode_meta().content_hash {
        Some(content_hash) => Ok(content_hash),
        None => Err("content_hash missing".to_internal_error()),
      },
      None => Err("Can't get content_hash without meta_pos".to_internal_error()),
    }
  }

  pub fn make_bytecode_frame(&self) -> BytecodeStackFrame {
    let mut registers: Vec<Val> = Vec::with_capacity(self.register_count - 1);

    registers.push(Val::Undefined);
    registers.push(Val::Undefined);

    for bind_val in &self.binds {
      registers.push(bind_val.clone());
    }

    while registers.len() < registers.capacity() {
      registers.push(Val::Void);
    }

    BytecodeStackFrame {
      decoder: BytecodeDecoder {
        bytecode: self.bytecode.clone(),
        pos: self.start,
      },
      registers,
      const_this: true,
      param_start: self.binds.len() + 2,
      param_end: self.parameter_count + 2,
      this_target: None,
      return_target: None,
      catch_setting: None,
    }
  }

  pub fn make_frame(&self) -> StackFrame {
    let frame = self.make_bytecode_frame();

    match self.is_generator {
      false => Box::new(frame),
      true => Box::new(MakeGeneratorFrame::new(frame)),
    }
  }
}

impl ValTrait for CsFunction {
  fn typeof_(&self) -> VsType {
    VsType::Function
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

  fn bind(&self, params: Vec<Val>) -> Option<Val> {
    Some(self.bind(params).to_dynamic_val())
  }

  fn as_bigint_data(&self) -> Option<num_bigint::BigInt> {
    None
  }

  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    None
  }

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::StackFrame(self.make_frame())
  }

  fn sub(&self, _key: &Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn has(&self, _key: &Val) -> Option<bool> {
    Some(false)
  }

  fn submov(&mut self, _key: &Val, _value: Val) -> Result<(), Val> {
    Err("TODO: function subscript assignment".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Function]\x1b[39m")
  }

  fn codify(&self) -> String {
    "() => { [unavailable] }".to_string()
  }
}

impl std::fmt::Display for CsFunction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[function]")
  }
}
