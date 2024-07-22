use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub struct BristolCircuit {
  pub info: CircuitInfo,
  pub bristol: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitInfo {
  pub input_name_to_wire_index: HashMap<String, u32>,
  pub constants: HashMap<String, ConstantInfo>,
  pub output_name_to_wire_index: HashMap<String, u32>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantInfo {
  pub value: String,
  pub wire_index: u32,
}
