use std::any::Any;
use std::cmp::Ordering;
use std::rc::Rc;

use valuescript_vm::internal_error_builtin::ToInternalError;
use valuescript_vm::vs_value::{ToVal, Val};
use valuescript_vm::{FirstStackFrame, FrameStepOk, StackFrame};

use crate::bytecode_stack_frame::BytecodeStackFrame;

pub struct CircuitVMBranch {
  pub flag: Val,
  pub frame: Rc<StackFrame>,
  pub stack: Vec<Rc<StackFrame>>,
  pub sub_branch: Option<Box<CircuitVMBranch>>,
}

impl Default for CircuitVMBranch {
  fn default() -> Self {
    CircuitVMBranch {
      flag: 1f64.to_val(),
      frame: Rc::new(Box::new(FirstStackFrame::new())),
      stack: Default::default(),
      sub_branch: None,
    }
  }
}

impl CircuitVMBranch {
  pub fn frame_mut(&mut self) -> &mut StackFrame {
    Rc::make_mut(&mut self.frame)
  }

  pub fn step(&mut self) -> Result<(), Val> {
    let step_ok = match self.frame_mut().step() {
      Ok(step_ok) => step_ok,
      Err(e) => return self.handle_exception(e),
    };

    match step_ok {
      FrameStepOk::Continue => {}
      FrameStepOk::Pop(call_result) => {
        self.pop();
        self.frame_mut().apply_call_result(call_result);
      }
      FrameStepOk::Push(new_frame) => {
        self.push(Rc::new(new_frame));
      }
      // TODO: Internal errors
      FrameStepOk::Yield(_) => {
        return self.handle_exception("Unexpected yield".to_internal_error())
      }
      FrameStepOk::YieldStar(_) => {
        return self.handle_exception("Unexpected yield*".to_internal_error())
      }
    }

    Ok(())
  }

  pub fn push(&mut self, mut frame: Rc<StackFrame>) {
    std::mem::swap(&mut self.frame, &mut frame);
    self.stack.push(frame);
  }

  pub fn pop(&mut self) {
    // This name is accurate after the swap
    let mut old_frame = self.stack.pop().unwrap();
    std::mem::swap(&mut self.frame, &mut old_frame);
  }

  pub fn handle_exception(&mut self, mut exception: Val) -> Result<(), Val> {
    while !self.stack.is_empty() {
      if self.frame.can_catch_exception(&exception) {
        self.frame_mut().catch_exception(&mut exception);
        return Ok(());
      }

      if self.stack.is_empty() {
        return Err(exception);
      }

      self.pop();
    }

    Err(exception)
  }
}

impl Ord for CircuitVMBranch {
  fn cmp(&self, other: &Self) -> Ordering {
    let depth_cmp = self.stack.len().cmp(&other.stack.len());

    if depth_cmp != Ordering::Equal {
      // Prefer deeper stacks
      return depth_cmp;
    }

    match (
      as_bytecode_stack_frame(&self.frame),
      as_bytecode_stack_frame(&other.frame),
    ) {
      (Some(self_frame), Some(other_frame)) => {
        // We prefer working on an earlier bytecode position, but we have a max-heap so we need to
        // reverse the order so that the earlier position will be higher priority.
        other_frame.decoder.pos.cmp(&self_frame.decoder.pos)
      }
      // No preference for non-bytecode frames
      _ => Ordering::Equal,
    }
  }
}

impl PartialOrd for CircuitVMBranch {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Eq for CircuitVMBranch {}

impl PartialEq for CircuitVMBranch {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}

pub fn as_bytecode_stack_frame(frame: &Rc<StackFrame>) -> Option<&BytecodeStackFrame> {
  (frame.as_ref() as &dyn Any).downcast_ref::<BytecodeStackFrame>()
}

pub fn as_bytecode_stack_frame_mut(frame: &mut Rc<StackFrame>) -> Option<&mut BytecodeStackFrame> {
  (Rc::make_mut(frame) as &mut dyn Any).downcast_mut::<BytecodeStackFrame>()
}

pub fn as_first_stack_frame(frame: &Rc<StackFrame>) -> Option<&FirstStackFrame> {
  (frame.as_ref() as &dyn Any).downcast_ref::<FirstStackFrame>()
}
