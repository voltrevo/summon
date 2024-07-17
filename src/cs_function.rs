use std::rc::Rc;

use crate::{
  bytecode::{Bytecode, DecoderMaker},
  bytecode_decoder::BytecodeDecoder,
  bytecode_stack_frame::BytecodeStackFrame,
  make_generator_frame::MakeGeneratorFrame,
};

use valuescript_vm::{
  internal_error_builtin::ToInternalError, vs_value::Val, StackFrame, ValTrait,
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
  fn typeof_(&self) -> valuescript_vm::vs_value::VsType {
    todo!()
  }

  fn to_number(&self) -> f64 {
    todo!()
  }

  fn to_index(&self) -> Option<usize> {
    todo!()
  }

  fn is_primitive(&self) -> bool {
    todo!()
  }

  fn is_truthy(&self) -> bool {
    todo!()
  }

  fn is_nullish(&self) -> bool {
    todo!()
  }

  fn bind(&self, params: Vec<Val>) -> Option<Val> {
    todo!()
  }

  fn as_bigint_data(&self) -> Option<num_bigint::BigInt> {
    todo!()
  }

  fn as_array_data(&self) -> Option<Rc<valuescript_vm::vs_array::VsArray>> {
    todo!()
  }

  fn as_class_data(&self) -> Option<Rc<valuescript_vm::vs_class::VsClass>> {
    todo!()
  }

  fn load_function(&self) -> valuescript_vm::LoadFunctionResult {
    todo!()
  }

  fn sub(&self, key: &Val) -> Result<Val, Val> {
    todo!()
  }

  fn has(&self, key: &Val) -> Option<bool> {
    todo!()
  }

  fn submov(&mut self, key: &Val, value: Val) -> Result<(), Val> {
    todo!()
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    todo!()
  }

  fn codify(&self) -> String {
    todo!()
  }
}

impl std::fmt::Display for CsFunction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[function]")
  }
}
