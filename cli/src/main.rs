use std::{fs, path::Path};

use circuitscript_compiler::{compile, Circuit, CompileOk};
use handle_diagnostics_cli::handle_diagnostics_cli;
use serde_json::to_string_pretty;

mod handle_diagnostics_cli;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  if args.len() != 2 {
    eprintln!("Usage: csc main.ts");
    std::process::exit(1);
  }

  let compile_result = compile(&args[1], |path| {
    fs::read_to_string(path).map_err(|e| e.to_string())
  });

  let diagnostics = match &compile_result {
    Ok(ok) => &ok.diagnostics,
    Err(err) => &err.diagnostics,
  };

  handle_diagnostics_cli(diagnostics);

  let CompileOk {
    circuit: Circuit { info, bristol },
    diagnostics,
  } = compile_result.expect("Error should have caused earlier exit");

  handle_diagnostics_cli(&diagnostics);

  let output_dir = Path::new("output");

  if output_dir.exists() {
    fs::remove_dir_all(output_dir).unwrap();
  }

  fs::create_dir(output_dir).unwrap();

  fs::write("output/circuit.txt", bristol).unwrap();
  println!("output/circuit.txt");

  fs::write("output/circuit_info.json", to_string_pretty(&info).unwrap()).unwrap();
  println!("output/circuit_info.json");
}
