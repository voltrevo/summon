use std::cmp::Ordering;
use std::mem::take;
use std::rc::Rc;

use valuescript_vm::internal_error_builtin::ToInternalError;
use valuescript_vm::operations::op_mul;
use valuescript_vm::vs_value::{ToVal, Val};
use valuescript_vm::{FirstStackFrame, FrameStepOk, StackFrame};

use crate::bytecode_decoder::BytecodeType;
use crate::bytecode_stack_frame::BytecodeStackFrame;

#[derive(Clone)]
pub struct CircuitVMBranch {
  pub flag: Val,
  pub frame: Rc<StackFrame>,
  pub stack: Vec<Rc<StackFrame>>,
  pub alt_branch: Option<Box<CircuitVMBranch>>,
}

impl Default for CircuitVMBranch {
  fn default() -> Self {
    CircuitVMBranch {
      flag: 1f64.to_val(),
      frame: Rc::new(Box::new(FirstStackFrame::new())),
      stack: Default::default(),
      alt_branch: None,
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
      FrameStepOk::Continue => {
        if let Some(frame) = self
          .frame_mut()
          .as_any_mut()
          .downcast_mut::<BytecodeStackFrame>()
        {
          if let Some(fork_info) = take(&mut frame.fork_info) {
            let mut alt_branch = self.clone();

            self.flag = op_mul(&self.flag, &fork_info.flag).unwrap();
            alt_branch.flag = op_mul(&alt_branch.flag, &fork_info.alt_flag).unwrap();
            alt_branch.frame = Rc::new(Box::new(fork_info.alt_frame));

            self.alt_branch = Some(Box::new(alt_branch));
          }
        }
      }
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
      self.frame.as_any().downcast_ref::<BytecodeStackFrame>(),
      other.frame.as_any().downcast_ref::<BytecodeStackFrame>(),
    ) {
      (Some(self_frame), Some(other_frame)) => {
        match (
          self_frame.decoder.peek_type(),
          other_frame.decoder.peek_type(),
        ) {
          (BytecodeType::End, BytecodeType::End) => Ordering::Equal,
          (BytecodeType::End, _) => Ordering::Less,
          (_, BytecodeType::End) => Ordering::Greater,

          // We prefer working on an earlier bytecode position, but we have a max-heap so we need to
          // reverse the order so that the earlier position will be higher priority.
          _ => other_frame.decoder.pos.cmp(&self_frame.decoder.pos),
        }
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
