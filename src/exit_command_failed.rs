use std::process::exit;

pub fn exit_command_failed(args: &[String], context: Option<&str>, msg: &str) -> ! {
  println!("Command failed: {:?}", args);

  if let Some(context) = context {
    println!("  {}", context);
  }

  println!("  {}", msg);

  exit(1);
}
