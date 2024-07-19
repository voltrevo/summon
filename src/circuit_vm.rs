use std::{cmp::Ordering, collections::BinaryHeap, mem::take, rc::Rc};

use valuescript_vm::{
  internal_error_builtin::ToInternalError,
  vs_value::{ToVal, Val},
  CallResult, FirstStackFrame, LoadFunctionResult, StackFrameTrait, ValTrait,
};

use crate::{
  arithmetic_merge::arithmetic_merge,
  circuit_vm_branch::{
    as_bytecode_stack_frame, as_bytecode_stack_frame_mut, as_first_stack_frame, CircuitVMBranch,
  },
};

#[derive(Default)]
pub struct CircuitVM {
  pub branch: CircuitVMBranch,
  pub alt_branches: BinaryHeap<CircuitVMBranch>,
}

impl CircuitVM {
  pub fn run(
    &mut self,
    step_limit: Option<usize>,
    this: &mut Val,
    fn_: Val,
    args: Vec<Val>,
  ) -> Result<Val, Val> {
    let mut frame = match fn_.load_function() {
      LoadFunctionResult::StackFrame(f) => f,
      _ => return Err("fn_ is not a function".to_internal_error()),
    };

    frame.write_this(false, take(this))?;

    for a in args {
      frame.write_param(a);
    }

    self.branch = CircuitVMBranch {
      flag: 1f64.to_val(),
      frame: Rc::new(frame),
      stack: vec![Rc::new(Box::new(FirstStackFrame::new()))],
      alt_branch: None,
    };

    let res = match step_limit {
      Some(step_limit) => 'b: {
        let mut step_count = 0;

        while step_count < step_limit {
          self.step()?;
          step_count += 1;

          if self.branch.stack.is_empty() {
            assert!(self.alt_branches.is_empty());
            break 'b self.branch.frame_mut().get_call_result();
          }
        }

        return Err("step limit reached".to_internal_error());
      }
      None => {
        while !self.branch.stack.is_empty() {
          self.step()?;
        }

        self.branch.frame_mut().get_call_result()
      }
    };

    let CallResult {
      return_,
      this: updated_this,
    } = res;

    *this = updated_this;

    Ok(return_)
  }

  pub fn step(&mut self) -> Result<(), Val> {
    self.assert_current_branch_best();
    assert!(self.branch.alt_branch.is_none());

    self.branch.step()?;

    if let Some(alt_branch) = take(&mut self.branch.alt_branch) {
      self.alt_branches.push(*alt_branch);
    }

    loop {
      if let Some(alt_branch) = self.alt_branches.peek() {
        match self.branch.cmp(alt_branch) {
          Ordering::Less => {
            // Since the current branch is a lower priority than the best alt branch, adopt the best
            // alt branch.
            let alt_branch = self.alt_branches.pop().unwrap();
            self.set_branch(alt_branch);

            continue;
          }
          Ordering::Equal => {
            if let Some(current_frame) = as_first_stack_frame(&self.branch.frame) {
              let alt_frame = as_first_stack_frame(&alt_branch.frame)
                .expect("Should be first stack frame since the branches are equal");

              let mut new_frame = FirstStackFrame::new();

              new_frame.apply_call_result(CallResult {
                return_: arithmetic_merge(
                  &self.branch.flag,
                  &current_frame.call_result.return_,
                  &alt_branch.flag,
                  &alt_frame.call_result.return_,
                ),
                this: arithmetic_merge(
                  &self.branch.flag,
                  &current_frame.call_result.this,
                  &alt_branch.flag,
                  &alt_frame.call_result.this,
                ),
              });

              let mut new_frame = Rc::new(Box::new(new_frame) as Box<dyn StackFrameTrait>);

              std::mem::swap(&mut self.branch.frame, &mut new_frame);
              self.branch.flag = 1f64.to_val();

              self.alt_branches.pop();

              continue;
            }

            if let (Some(current_frame), Some(alt_frame)) = (
              as_bytecode_stack_frame_mut(&mut self.branch.frame),
              as_bytecode_stack_frame(&alt_branch.frame),
            ) {
              assert!(current_frame.can_merge(alt_frame));

              assert!(self.branch.stack.len() == alt_branch.stack.len());

              // Because of the way we prefer deeper stacks, step once at a time, and a step cannot
              // simultaneously pop and push frame(s), it should not be possible to have unequal stack
              // traces here.
              for i in 0..self.branch.stack.len() {
                assert!(std::ptr::eq(
                  self.branch.stack[i].as_ref(),
                  alt_branch.stack[i].as_ref()
                ));
              }

              let mut new_registers = Vec::<Val>::new();

              assert!(current_frame.registers.len() == alt_frame.registers.len());

              for i in 0..current_frame.registers.len() {
                new_registers.push(arithmetic_merge(
                  &self.branch.flag,
                  &current_frame.registers[i],
                  &alt_branch.flag,
                  &alt_frame.registers[i],
                ));
              }

              current_frame.registers = new_registers;
              self.branch.flag = 1f64.to_val();

              self.alt_branches.pop();

              continue;
            }

            break;
          }
          Ordering::Greater => {
            break;
          }
        }
      }

      break;
    }

    Ok(())
  }

  fn set_branch(&mut self, mut new_branch: CircuitVMBranch) {
    std::mem::swap(&mut self.branch, &mut new_branch);
    self.alt_branches.push(new_branch);
  }

  fn assert_current_branch_best(&self) {
    if let Some(alt_branch) = self.alt_branches.peek() {
      assert!(&self.branch > alt_branch);
    }
  }
}
