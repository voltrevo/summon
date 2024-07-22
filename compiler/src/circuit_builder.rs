use std::collections::HashMap;

use num_traits::ToPrimitive;
use valuescript_vm::{binary_op::BinaryOp, unary_op::UnaryOp, vs_value::Val, ValTrait};

use crate::circuit_signal::{CircuitSignal, CircuitSignalData};

#[derive(Default)]
pub struct CircuitBuilder {
  pub gates: Vec<String>,
  pub wire_count: usize,
  pub wires_included: HashMap<usize, usize>, // CircuitSignal.id -> wire_id
  pub constants: HashMap<usize, usize>,      // value -> wire_id
}

impl CircuitBuilder {
  pub fn include_inputs(&mut self, input_len: usize) {
    for i in 0..input_len {
      self.wires_included.insert(i, i);
    }

    self.wire_count = input_len;
  }

  pub fn include_outputs(&mut self, outputs: &Vec<Val>) -> Vec<usize> {
    for output in outputs {
      for dep in get_dependencies(output) {
        self.include_val(&dep);
      }
    }

    let mut output_ids = vec![];

    for output in outputs {
      output_ids.push(self.include_val(output));
    }

    output_ids
  }

  pub fn include_val(&mut self, val: &Val) -> usize {
    match val {
      Val::Number(number) => {
        if *number != number.trunc() {
          panic!("Cannot use non-integer constant");
        }

        let value = number.to_usize().unwrap();

        if let Some(wire_id) = self.constants.get(&value) {
          return *wire_id;
        }

        let wire_id = self.wire_count;
        self.wire_count += 1;
        self.constants.insert(value, wire_id);

        wire_id
      }
      Val::Dynamic(dyn_val) => {
        if let Some(circuit_number) = dyn_val.as_any().downcast_ref::<CircuitSignal>() {
          if let Some(wire_id) = self.wires_included.get(&circuit_number.id) {
            return *wire_id;
          }

          let dependent_ids = get_dependencies(val)
            .iter()
            .map(|dep| self.include_val(dep))
            .collect::<Vec<usize>>();

          let wire_id = self.wire_count;
          self.wire_count += 1;

          let bristol_op_string = match &circuit_number.data {
            CircuitSignalData::Input => panic!("Input should have been included earlier"),
            CircuitSignalData::UnaryOp(unary_op, _) => to_bristol_unary_op(*unary_op),
            CircuitSignalData::BinaryOp(binary_op, _, _) => to_bristol_binary_op(*binary_op),
          };

          self.gates.push(format!(
            "{} 1 {} {} {}",
            dependent_ids.len(),
            dependent_ids
              .iter()
              .map(|id| id.to_string())
              .collect::<Vec<String>>()
              .join(" "),
            wire_id,
            bristol_op_string,
          ));

          self.wires_included.insert(circuit_number.id, wire_id);

          return wire_id;
        }

        panic!("Can't include unrecognized type ({}) 1", val.codify());
      }
      _ => panic!("Can't include unrecognized type ({}) 2", val.codify()),
    }
  }
}

fn get_dependencies(val: &Val) -> Vec<Val> {
  if let Val::Dynamic(val) = val {
    if let Some(circuit_number) = val.as_any().downcast_ref::<CircuitSignal>() {
      return match &circuit_number.data {
        CircuitSignalData::Input => vec![],
        CircuitSignalData::UnaryOp(_, input) => {
          vec![input.clone()]
        }
        CircuitSignalData::BinaryOp(_, left, right) => {
          vec![left.clone(), right.clone()]
        }
      };
    }
  }

  vec![]
}

fn to_bristol_unary_op(unary_op: UnaryOp) -> String {
  match unary_op {
    UnaryOp::Plus => "AUnaryAdd",
    UnaryOp::Minus => "AUnarySub",
    UnaryOp::Not => "ANot",
    UnaryOp::BitNot => "ABitNot",
  }
  .to_string()
}

fn to_bristol_binary_op(binary_op: BinaryOp) -> String {
  match binary_op {
    BinaryOp::Plus => "AAdd",
    BinaryOp::Minus => "ASub",
    BinaryOp::Mul => "AMul",
    BinaryOp::Div => "ADiv",
    BinaryOp::Mod => "AMod",
    BinaryOp::Exp => "AExp",
    BinaryOp::LooseEq => "AEq",
    BinaryOp::LooseNe => "ANeq",
    BinaryOp::Eq => "AEq",
    BinaryOp::Ne => "ANeq",
    BinaryOp::And => "ABoolAnd",
    BinaryOp::Or => "ABoolOr",
    BinaryOp::Less => "ALt",
    BinaryOp::LessEq => "ALEq",
    BinaryOp::Greater => "AGt",
    BinaryOp::GreaterEq => "AGEq",
    BinaryOp::BitAnd => "ABitAnd",
    BinaryOp::BitOr => "ABitOr",
    BinaryOp::BitXor => "AXor",
    BinaryOp::LeftShift => "AShiftL",
    BinaryOp::RightShift => "AShiftR",
    BinaryOp::RightShiftUnsigned => "AShiftR",
  }
  .to_string()
}
