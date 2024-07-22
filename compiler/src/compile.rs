use std::{cell::RefCell, collections::HashMap, rc::Rc};

use sim_circuit::arithmetic_circuit::{CircuitInfo, ConstantInfo};
use valuescript_compiler::{asm, assemble, Diagnostic, ResolvedPath};
use valuescript_vm::vs_value::{ToDynamicVal, Val, VsType};

use crate::{
  bytecode::{Bytecode, DecoderMaker},
  circuit::Circuit,
  circuit_builder::CircuitBuilder,
  circuit_signal::{CircuitSignal, CircuitSignalData},
  circuit_vm::CircuitVM,
  cs_function::CsFunction,
  id_generator::IdGenerator,
  resolve_entry_path::resolve_entry_path,
  val_dynamic_downcast::val_dynamic_downcast,
};

pub struct CompileOk {
  pub circuit: Circuit,
  pub diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

#[derive(Debug)]
pub struct CompileErr {
  pub diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

pub type CompileResult = Result<CompileOk, CompileErr>;

pub fn compile<ReadFile>(path: &str, read_file: ReadFile) -> CompileResult
where
  ReadFile: Fn(&str) -> Result<String, String>,
{
  let CompileArtifacts {
    name,
    main_asm,
    main,
    diagnostics,
  } = get_compile_artifacts(path, read_file)?;

  let (input_len, outputs) = run(main);

  let (bristol, output_ids, constants) = generate_circuit(input_len, outputs);
  let info = generate_circuit_info(name, main_asm, output_ids, constants);

  Ok(CompileOk {
    circuit: Circuit { info, bristol },
    diagnostics,
  })
}

struct CompileArtifacts {
  name: String,
  main_asm: asm::Function,
  main: Val,
  diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

fn get_compile_artifacts<ReadFile>(
  path: &str,
  read_file: ReadFile,
) -> Result<CompileArtifacts, CompileErr>
where
  ReadFile: Fn(&str) -> Result<String, String>,
{
  let entry_point = resolve_entry_path(path);

  let valuescript_compiler::CompileResult {
    module,
    diagnostics,
  } = valuescript_compiler::compile(entry_point, read_file);

  let module = match module {
    Some(module) => module,
    None => return Err(CompileErr { diagnostics }),
  };

  let (name, asm_fn) = get_asm_main(&module);

  let bytecode = Rc::new(Bytecode::new(assemble(&module)));

  let val = bytecode.decoder(0).decode_val(&mut vec![]);

  Ok(CompileArtifacts {
    name: name.clone(),
    main_asm: asm_fn.clone(),
    main: val,
    diagnostics,
  })
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

fn run(main: Val) -> (usize, Vec<Val>) {
  let param_count = match val_dynamic_downcast::<CsFunction>(&main) {
    Some(cs_fn) => cs_fn.parameter_count,
    None => panic!("Default export is not a regular function"),
  };

  let id_gen = Rc::new(RefCell::new(IdGenerator::new()));
  let mut input_args = Vec::<Val>::new();

  for _ in 0..param_count {
    input_args.push(
      CircuitSignal::new(&id_gen, Some(VsType::Number), CircuitSignalData::Input).to_dynamic_val(),
    );
  }

  let mut vm = CircuitVM::default();

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

fn generate_circuit(
  input_len: usize,
  outputs: Vec<Val>,
) -> (String, Vec<usize>, HashMap<usize, usize>) {
  let mut bristol = String::new();

  let mut builder = CircuitBuilder::default();
  builder.include_inputs(input_len);
  let output_ids = builder.include_outputs(&outputs);

  bristol.push_str(&format!("{} {}\n", builder.gates.len(), builder.wire_count));
  bristol.push_str(&input_len.to_string());

  for _ in 0..input_len {
    bristol.push_str(" 1");
  }

  bristol.push('\n');

  bristol.push_str(&outputs.len().to_string());

  for _ in 0..outputs.len() {
    bristol.push_str(" 1");
  }

  bristol.push('\n');
  bristol.push('\n');

  for gate in builder.gates {
    bristol.push_str(&gate);
    bristol.push('\n');
  }

  (bristol, output_ids, builder.constants)
}

fn generate_circuit_info(
  name: String,
  fn_: asm::Function,
  output_ids: Vec<usize>,
  constants: HashMap<usize, usize>,
) -> CircuitInfo {
  let mut circuit_info = CircuitInfo {
    input_name_to_wire_index: Default::default(),
    constants: Default::default(),
    output_name_to_wire_index: Default::default(),
  };

  for (i, reg) in fn_.parameters.iter().enumerate() {
    circuit_info
      .input_name_to_wire_index
      .insert(reg.name.clone(), i as u32);
  }

  for (value, wire_id) in constants {
    circuit_info.constants.insert(
      format!("constant_{}", value),
      ConstantInfo {
        value: value.to_string(),
        wire_index: wire_id as u32,
      },
    );
  }

  if output_ids.len() == 1 {
    circuit_info
      .output_name_to_wire_index
      .insert(name, output_ids[0] as u32);
  } else {
    for (i, output_id) in output_ids.iter().enumerate() {
      circuit_info
        .output_name_to_wire_index
        .insert(format!("{}[{}]", name, i), *output_id as u32);
    }
  }

  circuit_info
}
