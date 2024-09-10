use std::cmp::max;

use bristol_circuit::BristolCircuit;

pub fn bristol_depth(circuit: &BristolCircuit) -> usize {
  let mut wire_depths = vec![0usize; circuit.wire_count];

  for gate in &circuit.gates {
    let depth = 1 + gate.inputs.iter().map(|i| wire_depths[*i]).fold(0, max);

    for o in &gate.outputs {
      wire_depths[*o] = depth;
    }
  }

  let max_depth = wire_depths.iter().fold(0, |a, b| max(a, *b));

  max_depth
}
