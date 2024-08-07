use std::{cmp::max, collections::HashMap};

use valuescript_vm::{binary_op::BinaryOp, unary_op::UnaryOp};

use crate::{
  bristol_circuit::{CircuitInfo, ConstantInfo},
  bristol_op_strings::{to_bristol_binary_op, to_bristol_unary_op},
  BristolCircuit,
};

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
    let mut bristol = Vec::<String>::new();

    bristol.push(format!("{} {}", self.gates.len(), self.size));
    let mut input_line = self.inputs.len().to_string();

    for _ in 0..self.inputs.len() {
      input_line.push_str(" 1");
    }

    bristol.push(input_line);

    let mut output_line = self.outputs.len().to_string();

    for _ in 0..self.outputs.len() {
      output_line.push_str(" 1");
    }

    bristol.push(output_line);
    bristol.push("".into());

    for gate in &self.gates {
      bristol.push(match gate {
        Gate::Unary { op, input, output } => {
          format!("1 1 {} {} {}", input, output, to_bristol_unary_op(*op))
        }
        Gate::Binary {
          op,
          left,
          right,
          output,
        } => format!(
          "2 1 {} {} {} {}",
          left,
          right,
          output,
          to_bristol_binary_op(*op)
        ),
      });
    }

    let input_name_to_wire_index: HashMap<String, u32> = self
      .inputs
      .iter()
      .map(|(name, id)| (name.clone(), *id as u32))
      .collect();

    let constants: HashMap<String, ConstantInfo> = self
      .constants
      .iter()
      .map(|(id, value)| {
        (
          format!("constant_{}", value),
          ConstantInfo {
            value: value.to_string(),
            wire_index: *id as u32,
          },
        )
      })
      .collect();

    let output_name_to_wire_index: HashMap<String, u32> = self
      .outputs
      .iter()
      .map(|(name, id)| (name.clone(), *id as u32))
      .collect();

    BristolCircuit {
      info: CircuitInfo {
        input_name_to_wire_index,
        constants,
        output_name_to_wire_index,
      },
      bristol: bristol.join("\n"),
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
