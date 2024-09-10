use std::{cmp::max, collections::HashMap};

use bristol_circuit::{BristolCircuit, CircuitInfo, ConstantInfo, Gate as BristolGate};
use valuescript_vm::{binary_op::BinaryOp, unary_op::UnaryOp};

use crate::bristol_op_strings::{to_bristol_binary_op, to_bristol_unary_op};

#[derive(Default)]
pub struct Circuit {
  pub size: usize,
  pub inputs: HashMap<String, usize>,
  pub constants: HashMap<usize, usize>, // wire_id -> value
  pub outputs: HashMap<String, usize>,
  pub gates: Vec<Gate>,
}

pub enum Gate {
  Unary {
    op: UnaryOp,
    input: usize,
    output: usize,
  },
  Binary {
    op: BinaryOp,
    left: usize,
    right: usize,
    output: usize,
  },
}

impl Circuit {
  pub fn eval<N: CircuitNumber>(&self, inputs: &HashMap<String, N>) -> HashMap<String, N> {
    let mut wire_values = vec![N::zero(); self.size];

    for (name, wire_id) in &self.inputs {
      let value = inputs.get(name).expect("Missing input");
      wire_values[*wire_id] = value.clone();
    }

    for (wire_id, value) in &self.constants {
      wire_values[*wire_id] = N::from_usize(*value);
    }

    for gate in &self.gates {
      match gate {
        Gate::Unary { op, input, output } => {
          wire_values[*output] = N::unary_op(*op, &wire_values[*input])
        }
        Gate::Binary {
          op,
          left,
          right,
          output,
        } => wire_values[*output] = N::binary_op(*op, &wire_values[*left], &wire_values[*right]),
      }
    }

    let mut res = HashMap::<String, N>::new();

    for (name, wire_id) in &self.outputs {
      res.insert(name.clone(), wire_values[*wire_id].clone());
    }

    res
  }

  pub fn depth(&self) -> usize {
    let mut wire_depths = vec![0usize; self.size];

    for gate in &self.gates {
      match gate {
        Gate::Unary {
          op: _,
          input,
          output,
        } => wire_depths[*output] = 1 + wire_depths[*input],
        Gate::Binary {
          op: _,
          left,
          right,
          output,
        } => wire_depths[*output] = 1 + max(wire_depths[*left], wire_depths[*right]),
      }
    }

    let max_depth = wire_depths.iter().fold(0, |a, b| max(a, *b));

    max_depth
  }

  pub fn to_bristol(&self) -> BristolCircuit {
    let mut bristol_gates = Vec::<BristolGate>::new();

    for gate in &self.gates {
      bristol_gates.push(match gate {
        Gate::Unary { op, input, output } => BristolGate {
          inputs: vec![*input],
          outputs: vec![*output],
          op: to_bristol_unary_op(*op),
        },
        Gate::Binary {
          op,
          left,
          right,
          output,
        } => BristolGate {
          inputs: vec![*left, *right],
          outputs: vec![*output],
          op: to_bristol_binary_op(*op),
        },
      });
    }

    let input_name_to_wire_index: HashMap<String, usize> = self
      .inputs
      .iter()
      .map(|(name, id)| (name.clone(), *id))
      .collect();

    let constants: HashMap<String, ConstantInfo> = self
      .constants
      .iter()
      .map(|(id, value)| {
        (
          format!("constant_{}", value),
          ConstantInfo {
            value: value.to_string(),
            wire_index: *id,
          },
        )
      })
      .collect();

    let output_name_to_wire_index: HashMap<String, usize> = self
      .outputs
      .iter()
      .map(|(name, id)| (name.clone(), *id))
      .collect();

    BristolCircuit {
      wire_count: self.size,
      info: CircuitInfo {
        input_name_to_wire_index,
        constants,
        output_name_to_wire_index,
      },
      io_widths: None,
      gates: bristol_gates,
    }
  }
}

pub trait CircuitNumber: Clone {
  fn zero() -> Self;
  fn from_usize(x: usize) -> Self;
  fn unary_op(op: UnaryOp, input: &Self) -> Self;
  fn binary_op(op: BinaryOp, left: &Self, right: &Self) -> Self;
}

impl CircuitNumber for usize {
  fn zero() -> Self {
    0
  }

  fn from_usize(x: usize) -> Self {
    x
  }

  fn unary_op(op: UnaryOp, input: &Self) -> Self {
    let input = *input;

    match op {
      UnaryOp::Plus => input,
      UnaryOp::Minus => 0usize.wrapping_sub(input),
      UnaryOp::Not => (input == 0) as usize,
      UnaryOp::BitNot => !input,
    }
  }

  fn binary_op(op: BinaryOp, left: &Self, right: &Self) -> Self {
    let left = *left;
    let right = *right;

    match op {
      BinaryOp::Plus => left.wrapping_add(right),
      BinaryOp::Minus => left.wrapping_sub(right),
      BinaryOp::Mul => left.wrapping_mul(right),
      BinaryOp::Div => left / right,
      BinaryOp::Mod => left % right,
      BinaryOp::Exp => left.wrapping_pow(right as u32),
      BinaryOp::LooseEq => (left == right) as usize,
      BinaryOp::LooseNe => (left != right) as usize,
      BinaryOp::Eq => (left == right) as usize,
      BinaryOp::Ne => (left != right) as usize,
      BinaryOp::And => (left != 0 && right != 0) as usize,
      BinaryOp::Or => (left != 0 || right != 0) as usize,
      BinaryOp::Less => (left < right) as usize,
      BinaryOp::LessEq => (left <= right) as usize,
      BinaryOp::Greater => (left > right) as usize,
      BinaryOp::GreaterEq => (left >= right) as usize,
      BinaryOp::BitAnd => left & right,
      BinaryOp::BitOr => left | right,
      BinaryOp::BitXor => left ^ right,
      BinaryOp::LeftShift => left.wrapping_shl(right as u32),
      BinaryOp::RightShift => left.wrapping_shr(right as u32),
      BinaryOp::RightShiftUnsigned => left.wrapping_shr(right as u32),
    }
  }
}
