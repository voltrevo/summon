use valuescript_vm::{binary_op::BinaryOp, unary_op::UnaryOp};

pub fn to_bristol_unary_op(unary_op: UnaryOp) -> String {
  match unary_op {
    UnaryOp::Plus => "AUnaryAdd",
    UnaryOp::Minus => "AUnarySub",
    UnaryOp::Not => "ANot",
    UnaryOp::BitNot => "ABitNot",
  }
  .to_string()
}

pub fn to_bristol_binary_op(binary_op: BinaryOp) -> String {
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
