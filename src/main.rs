use std::{
  cell::RefCell,
  collections::HashMap,
  fs::{self, File},
  path::Path,
  rc::Rc,
};

use std::io::Write;

use circuit_number::{CircuitNumber, CircuitNumberData};
use exit_command_failed::exit_command_failed;
use handle_diagnostics_cli::handle_diagnostics_cli;
use id_generator::IdGenerator;
use num_traits::ToPrimitive;
use resolve_entry_path::resolve_entry_path;
use serde::Serialize;
use serde_json::to_string_pretty;
use valuescript_compiler::{asm, assemble, compile};
use valuescript_vm::{
  binary_op::BinaryOp,
  unary_op::UnaryOp,
  vs_value::{ToDynamicVal, Val},
  Bytecode, DecoderMaker, ValTrait, VirtualMachine,
};

mod bytecode;
mod bytecode_decoder;
mod bytecode_stack_frame;
mod circuit_number;
mod circuit_vm;
mod cs_function;
mod exit_command_failed;
mod generator;
mod handle_diagnostics_cli;
mod id_generator;
mod make_generator_frame;
mod resolve_entry_path;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  let (name, fn_, main) = get_cli_default_export(&args);
  let (input_len, outputs) = run(&args, main);

  let (output_ids, constants) = generate_circuit(input_len, outputs);
  generate_circuit_info(name, fn_, output_ids, constants);
}

fn get_cli_default_export(args: &Vec<String>) -> (String, asm::Function, Val) {
  if args.len() != 2 {
    exit_command_failed(args, None, &format!("Usage: {} <main file>", args[0]));
  }

  let entry_point = resolve_entry_path(&args[1]);

  let compile_result = compile(entry_point, |path| {
    std::fs::read_to_string(path).map_err(|err| err.to_string())
  });

  for (path, diagnostics) in compile_result.diagnostics.iter() {
    handle_diagnostics_cli(&path.path, diagnostics);
  }

  let module = compile_result
    .module
    .expect("Should have exited if module is None");

  let (name, asm_fn) = get_asm_main(&module);

  let bytecode = Rc::new(Bytecode::new(assemble(&module)));

  let val = bytecode.decoder(0).decode_val(&mut vec![]);

  (name.clone(), asm_fn.clone(), val)
}

fn run(args: &[String], main: Val) -> (usize, Vec<Val>) {
  let param_count = match &main {
    Val::Function(main) => main.parameter_count,
    _ => exit_command_failed(args, None, "Default export is not a regular function"),
  };

  let id_gen = Rc::new(RefCell::new(IdGenerator::new()));
  let mut input_args = Vec::<Val>::new();

  for _ in 0..param_count {
    input_args.push(CircuitNumber::new(&id_gen, CircuitNumberData::Input).to_dynamic_val());
  }

  let mut vm = VirtualMachine::default();

  let res = vm.run(None, &mut Val::Undefined, main, input_args);

  match res {
    Ok(Val::Array(vs_array)) => (param_count, vs_array.elements.clone()),
    Ok(val) => (param_count, vec![val]),
    Err(err) => {
      eprintln!("Uncaught exception: {}", err.pretty());
      std::process::exit(1);
    }
  }
}

fn generate_circuit(input_len: usize, outputs: Vec<Val>) -> (Vec<usize>, HashMap<usize, usize>) {
  let output_dir = Path::new("output");

  if output_dir.exists() {
    fs::remove_dir_all(output_dir).unwrap();
  }

  fs::create_dir(output_dir).unwrap();

  let mut builder = CircuitBuilder::default();
  builder.include_inputs(input_len);
  let output_ids = builder.include_outputs(&outputs);

  let mut circuit_txt = File::create("output/circuit.txt").unwrap();

  writeln!(
    circuit_txt,
    "{} {}",
    builder.gates.len(),
    builder.wire_count
  )
  .unwrap();

  write!(circuit_txt, "{}", input_len).unwrap();

  for _ in 0..input_len {
    write!(circuit_txt, " 1").unwrap();
  }

  writeln!(circuit_txt).unwrap();

  write!(circuit_txt, "{}", outputs.len()).unwrap();

  for _ in 0..outputs.len() {
    write!(circuit_txt, " 1").unwrap();
  }

  writeln!(circuit_txt).unwrap();
  writeln!(circuit_txt).unwrap();

  for gate in builder.gates {
    writeln!(circuit_txt, "{}", gate).unwrap();
  }

  (output_ids, builder.constants)
}

#[derive(Default)]
struct CircuitBuilder {
  gates: Vec<String>,
  wire_count: usize,
  wires_included: HashMap<usize, usize>, // CircuitNumber.id -> wire_id
  constants: HashMap<usize, usize>,      // value -> wire_id
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
        if let Some(circuit_number) = dyn_val.as_any().downcast_ref::<CircuitNumber>() {
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
            CircuitNumberData::Input => panic!("Input should have been included earlier"),
            CircuitNumberData::UnaryOp(unary_op, _) => to_bristol_unary_op(*unary_op),
            CircuitNumberData::BinaryOp(binary_op, _, _) => to_bristol_binary_op(*binary_op),
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
    if let Some(circuit_number) = val.as_any().downcast_ref::<CircuitNumber>() {
      return match &circuit_number.data {
        CircuitNumberData::Input => vec![],
        CircuitNumberData::UnaryOp(_, input) => {
          vec![input.clone()]
        }
        CircuitNumberData::BinaryOp(_, left, right) => {
          vec![left.clone(), right.clone()]
        }
      };
    }
  }

  vec![]
}

fn get_asm_main(module: &asm::Module) -> (&String, &asm::Function) {
  let main_ptr = match &module.export_default {
    asm::Value::Pointer(ptr) => ptr,
    _ => panic!("Expected pointer"),
  };

  let fn_ = match resolve_ptr(module, main_ptr).unwrap() {
    asm::DefinitionContent::Function(fn_) => fn_,
    _ => panic!("Expected function"),
  };

  let meta = match resolve_ptr(module, fn_.meta.as_ref().unwrap()).unwrap() {
    asm::DefinitionContent::Meta(meta) => meta,
    _ => panic!("Expected meta"),
  };

  (&meta.name, fn_)
}

fn resolve_ptr<'a>(
  module: &'a asm::Module,
  ptr: &asm::Pointer,
) -> Option<&'a asm::DefinitionContent> {
  for defn in &module.definitions {
    if &defn.pointer == ptr {
      return Some(&defn.content);
    }
  }

  None
}

fn generate_circuit_info(
  name: String,
  fn_: asm::Function,
  output_ids: Vec<usize>,
  constants: HashMap<usize, usize>,
) {
  let mut circuit_info = CircuitInfo::default();

  for (i, reg) in fn_.parameters.iter().enumerate() {
    circuit_info
      .input_name_to_wire_index
      .insert(reg.name.clone(), i);
  }

  for (value, wire_id) in constants {
    circuit_info.constants.insert(
      format!("constant_{}", value),
      ConstantInfo {
        value: value.to_string(),
        wire_index: wire_id,
      },
    );
  }

  if output_ids.len() == 1 {
    circuit_info
      .output_name_to_wire_index
      .insert(name, output_ids[0]);
  } else {
    for (i, output_id) in output_ids.iter().enumerate() {
      circuit_info
        .output_name_to_wire_index
        .insert(format!("{}[{}]", name, i), *output_id);
    }
  }

  fs::write(
    "output/circuit_info.json",
    to_string_pretty(&circuit_info).unwrap(),
  )
  .unwrap();
}

#[derive(Default, Serialize)]
struct CircuitInfo {
  input_name_to_wire_index: HashMap<String, usize>,
  constants: HashMap<String, ConstantInfo>,
  output_name_to_wire_index: HashMap<String, usize>,
}

#[derive(Serialize)]
struct ConstantInfo {
  value: String,
  wire_index: usize,
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
