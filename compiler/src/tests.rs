#[cfg(test)]
mod tests_ {
  use std::{collections::HashMap, fs, path::PathBuf};

  use crate::{compile, CompileOk};

  #[test]
  fn test_annotations() {
    let test_cases = find_test_cases("../examples");

    for TestCase {
      path,
      input,
      expected_output,
    } in test_cases
    {
      println!("Test {}: {:?} => {:?}", path, input, expected_output);

      let CompileOk {
        circuit,
        diagnostics: _,
      } = compile(&path, |p| fs::read_to_string(p).map_err(|e| e.to_string()))
        .expect("Compile failed");

      let inputs = circuit
        .inputs
        .iter()
        .map(|(name, i)| (name.clone(), input[*i]))
        .collect::<HashMap<_, _>>();

      let outputs = circuit.eval(&inputs);

      let mut output_names = circuit.outputs.iter().collect::<Vec<_>>();
      output_names.sort_by(|(_, id_a), (_, id_b)| id_a.cmp(id_b));

      let output_name_to_index = output_names
        .iter()
        .enumerate()
        .map(|(i, (name, _))| ((*name).clone(), i))
        .collect::<HashMap<_, _>>();

      for (name, value) in &outputs {
        let wire_id = output_name_to_index[name];
        assert_eq!(*value, expected_output[wire_id]);
      }
    }
  }

  #[derive(Debug)]
  struct TestCase {
    path: String,
    input: Vec<usize>,
    expected_output: Vec<usize>,
  }

  fn parse_test_case(path: &str, line: &str) -> Option<TestCase> {
    let line = line.trim();
    if !line.starts_with("//! test ") {
      return None;
    }

    let parts: Vec<&str> = line["//! test ".len()..].split(" => ").collect();
    if parts.len() != 2 {
      return None;
    }

    let input: Vec<usize> = parts[0]
      .trim()
      .trim_start_matches('[')
      .trim_end_matches(']')
      .split(',')
      .filter_map(|s| s.trim().parse().ok())
      .collect();

    let expected_output: Vec<usize> = parts[1]
      .trim()
      .trim_start_matches('[')
      .trim_end_matches(']')
      .split(',')
      .filter_map(|s| s.trim().parse().ok())
      .collect();

    Some(TestCase {
      path: path.to_string(),
      input,
      expected_output,
    })
  }

  fn find_test_cases(dir: &str) -> Vec<TestCase> {
    let mut test_cases = Vec::new();

    for path in read_dir_recursive(dir) {
      let content = fs::read_to_string(&path).expect("Unable to read file");

      for line in content.lines() {
        if let Some(test_case) = parse_test_case(path.to_str().unwrap(), line) {
          test_cases.push(test_case);
        }
      }
    }

    test_cases
  }

  fn read_dir_recursive(dir: &str) -> Vec<PathBuf> {
    let mut res = Vec::<PathBuf>::new();

    for entry in fs::read_dir(dir).expect("Directory not found") {
      let path = entry.expect("Unable to read entry").path();

      if path.is_file() {
        res.push(path);
      } else if path.is_dir() {
        res.append(&mut read_dir_recursive(path.to_str().unwrap()));
      }
    }

    res
  }
}
