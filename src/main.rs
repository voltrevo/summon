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
use valuescript_compiler::{assemble, compile};
use valuescript_vm::{
  vs_value::{ToDynamicVal, Val},
  Bytecode, DecoderMaker, VirtualMachine,
};

mod circuit_number;
mod exit_command_failed;
mod handle_diagnostics_cli;
mod id_generator;
mod resolve_entry_path;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  let main = get_cli_default_export(&args);
  let (input_len, outputs) = run(&args, main);

  generate_circuit(input_len, outputs);
}

fn get_cli_default_export(args: &Vec<String>) -> Val {
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

  let bytecode = Rc::new(Bytecode::new(assemble(
    &compile_result
      .module
      .expect("Should have exited if module is None"),
  )));

  bytecode.decoder(0).decode_val(&mut vec![])
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
    Ok(_) => {
      eprintln!("Non-array result");
      std::process::exit(1);
    }
    Err(err) => {
      eprintln!("Uncaught exception: {}", err.pretty());
      std::process::exit(1);
    }
  }
}

fn generate_circuit(input_len: usize, outputs: Vec<Val>) {
  let output_dir = Path::new("output");

  if output_dir.exists() {
    fs::remove_dir_all(output_dir).unwrap();
  }

  fs::create_dir(output_dir).unwrap();

  let mut builder = CircuitBuilder::default();
  builder.include_inputs(input_len);
  builder.include_outputs(&outputs);

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

  pub fn include_outputs(&mut self, outputs: &Vec<Val>) {
    for output in outputs {
      for dep in get_dependencies(output) {
        self.include_val(&dep);
      }
    }

    for output in outputs {
      self.include_val(output);
    }
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

          self.gates.push(format!(
            "{} 1 {} {}",
            dependent_ids.len(),
            dependent_ids
              .iter()
              .map(|id| id.to_string())
              .collect::<Vec<String>>()
              .join(" "),
            wire_id
          ));
        }

        panic!("Can't include unrecognized type");
      }
      _ => panic!("Can't include unrecognized type"),
    }
  }
}

fn get_dependencies(val: &Val) -> Vec<Val> {
  if let Val::Dynamic(val) = val {
    if let Some(circuit_number) = val.as_any().downcast_ref::<CircuitNumber>() {
      return match circuit_number.data.clone() {
        CircuitNumberData::Input => vec![],
        CircuitNumberData::UnaryOp(_, input) => {
          vec![Val::Dynamic(Rc::new((*input).clone()))]
        }
        CircuitNumberData::BinaryOp(_, left, right) => {
          vec![Val::Dynamic(Rc::new((*left).clone())), right.clone()]
        }
      };
    }
  }

  vec![]
}
