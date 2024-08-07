use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

pub fn compile<ReadFile>(path: ResolvedPath, read_file: ReadFile) -> CompileResult
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

  let (output_ids, builder) = build(input_len, outputs);
  let circuit = generate_circuit(name, main_asm, output_ids, builder);

  Ok(CompileOk {
    circuit,
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
  path: ResolvedPath,
  read_file: ReadFile,
) -> Result<CompileArtifacts, CompileErr>
where
  ReadFile: Fn(&str) -> Result<String, String>,
{
  let valuescript_compiler::CompileResult {
    module,
    diagnostics,
  } = valuescript_compiler::compile(path, read_file);

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

fn build(input_len: usize, outputs: Vec<Val>) -> (Vec<usize>, CircuitBuilder) {
  let mut builder = CircuitBuilder::default();
  builder.include_inputs(input_len);
  let output_ids = builder.include_outputs(&outputs);

  (output_ids, builder)
}

fn generate_circuit(
  name: String,
  fn_: asm::Function,
  output_ids: Vec<usize>,
  builder: CircuitBuilder,
) -> Circuit {
  let mut inputs = HashMap::<String, usize>::new();
  for (i, reg) in fn_.parameters.iter().enumerate() {
    inputs.insert(reg.name.clone(), i);
  }

  let mut constants = HashMap::<usize, usize>::new();
  for (value, wire_id) in &builder.constants {
    constants.insert(*wire_id, *value);
  }

  let mut outputs = HashMap::<String, usize>::new();
  if output_ids.len() == 1 {
    outputs.insert(name, output_ids[0]);
  } else {
    for (i, output_id) in output_ids.iter().enumerate() {
      outputs.insert(format!("{}[{}]", name, i), *output_id);
    }
  }

  Circuit {
    size: builder.wire_count,
    inputs,
    constants,
    outputs,
    gates: builder.gates,
  }
}
