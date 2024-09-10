use std::{
  fs::{self, File},
  io::BufWriter,
  path::Path,
};

use boolify::boolify;
use handle_diagnostics_cli::handle_diagnostics_cli;
use serde_json::to_string_pretty;
use summon_compiler::{bristol_depth, compile, resolve_entry_path, CompileOk};

mod handle_diagnostics_cli;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  if args.len() != 2 && args.len() != 4 {
    eprintln!("Usage: summonc main.ts [--boolify-width WIDTH]");
    std::process::exit(1);
  }

  let boolify_width = if args.len() == 4 {
    assert_eq!(args[2], "--boolify-width");
    Some(args[3].parse::<usize>().unwrap())
  } else {
    None
  };

  let entry_point = resolve_entry_path(&args[1]);

  let compile_result = compile(entry_point, |path| {
    fs::read_to_string(path).map_err(|e| e.to_string())
  });

  let diagnostics = match &compile_result {
    Ok(ok) => &ok.diagnostics,
    Err(err) => &err.diagnostics,
  };

  handle_diagnostics_cli(diagnostics);

  let CompileOk {
    circuit,
    diagnostics,
  } = compile_result.expect("Error should have caused earlier exit");

  handle_diagnostics_cli(&diagnostics);

  let output_dir = Path::new("output");

  if output_dir.exists() {
    fs::remove_dir_all(output_dir).unwrap();
  }

  fs::create_dir(output_dir).unwrap();

  let mut bristol_circuit = circuit.to_bristol();

  if let Some(boolify_width) = boolify_width {
    bristol_circuit = boolify(&bristol_circuit, boolify_width)
  }

  println!(
    "Wires: {}, Gates: {}, Depth: {}",
    bristol_circuit.wire_count,
    bristol_circuit.gates.len(),
    bristol_depth(&bristol_circuit),
  );

  bristol_circuit
    .write_bristol(&mut BufWriter::new(
      File::create("output/circuit.txt").unwrap(),
    ))
    .unwrap();
  println!("output/circuit.txt");

  fs::write(
    "output/circuit_info.json",
    to_string_pretty(&bristol_circuit.info).unwrap(),
  )
  .unwrap();
  println!("output/circuit_info.json");
}
