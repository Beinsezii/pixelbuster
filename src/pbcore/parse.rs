use super::Space;

use std::collections::HashMap;

// structs {{{
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    // Base
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Set,
    // Extended
    Abs,
    Acos,
    Acosh,
    Asin,
    Asinh,
    Atan,
    Atan2,
    Atanh,
    Cbrt,
    Ceil,
    Copysign,
    Cos,
    Cosh,
    Degrees,
    Diveuclid,
    Exp,
    Exp2,
    Expm1,
    Floor,
    Fract,
    Hypot,
    Ln,
    Ln1p,
    Log,
    Log2,
    Log10,
    Max,
    Min,
    Radians,
    Recip,
    Remeuclid,
    Round,
    Signum,
    Sin,
    Sinh,
    Sqrt,
    Tan,
    Tanh,
    Trunc,
    // Custom
    Invert,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmp {
    Gt,
    Lt,
    Eq,
    NEq,
    GtEq,
    LtEq,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Obj {
    Chan(usize),
    Var(usize),
    Num(f32),
    E,
    Pi,
    Rand,
    Row,
    Col,
    Width,
    Height,
    XNorm,
    YNorm,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Space(Space),
    Process {
        target: Obj,
        operation: Op,
        source: Obj,
    },
    If {
        left: Obj,
        cmp: Cmp,
        right: Obj,
        then: Box<Operation>,
    },
    Goto(usize),
    GotoTmp(String),
    Swap {
        t1: Obj,
        t2: Obj,
    },
}
// }}}

#[derive(Clone, Debug, PartialEq)]
pub enum OpError {
    Partial { line: usize, details: String },
    Unknown { line: usize },
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpError::Partial { line, details } => write!(f, "{} on line {}", details, line),

            OpError::Unknown { line } => write!(f, "Unknown operation on line {}", line),
        }
    }
}

fn tar(item: &str, space: Space) -> Result<Obj, ()> {
    match item.as_ref() {
        // don't hate I made these with a vim macro
        "c1" => Ok(Obj::Chan(0)),
        "c2" => Ok(Obj::Chan(1)),
        "c3" => Ok(Obj::Chan(2)),
        "c4" => Ok(Obj::Chan(3)),
        "v1" => Ok(Obj::Var(0)),
        "v2" => Ok(Obj::Var(1)),
        "v3" => Ok(Obj::Var(2)),
        "v4" => Ok(Obj::Var(3)),
        "v5" => Ok(Obj::Var(4)),
        "v6" => Ok(Obj::Var(5)),
        "v7" => Ok(Obj::Var(6)),
        "v8" => Ok(Obj::Var(7)),
        "v9" => Ok(Obj::Var(8)),
        "e1" => Ok(Obj::Var(9)),
        "e2" => Ok(Obj::Var(10)),
        "e3" => Ok(Obj::Var(11)),
        "e4" => Ok(Obj::Var(12)),
        "e5" => Ok(Obj::Var(13)),
        "e6" => Ok(Obj::Var(14)),
        "e7" => Ok(Obj::Var(15)),
        "e8" => Ok(Obj::Var(16)),
        "e9" => Ok(Obj::Var(17)),
        val => {
            if let Ok(c) = val.parse::<char>() {
                match space.channels().iter().enumerate().find(|(_, i)| **i == c) {
                    Some((n, _)) => Ok(Obj::Chan(n)),
                    None => Err(()),
                }
            } else {
                Err(())
            }
        }
    }
}

fn src(item: &str, space: Space) -> Result<Obj, ()> {
    match item.as_ref() {
        "e" => Ok(Obj::E),
        "pi" => Ok(Obj::Pi),
        "rand" => Ok(Obj::Rand),
        "row" => Ok(Obj::Row),
        "col" => Ok(Obj::Col),
        "width" => Ok(Obj::Width),
        "height" => Ok(Obj::Height),
        "xnorm" => Ok(Obj::XNorm),
        "ynorm" => Ok(Obj::YNorm),
        val => match val.parse::<f32>() {
            Ok(f) => Ok(Obj::Num(f)),
            Err(_) => tar(val, space),
        },
    }
}

fn op(item: &str) -> Result<Op, ()> {
    match item.as_ref() {
        // Base
        "+=" | "+" | "add" => Ok(Op::Add),
        "-=" | "-" | "sub" => Ok(Op::Sub),
        "*=" | "*" | "mul" => Ok(Op::Mul),
        "/=" | "/" | "div" => Ok(Op::Div),
        "%=" | "%" | "mod" => Ok(Op::Mod),
        "**" | "^" | "pow" => Ok(Op::Pow),
        "=" | "set" => Ok(Op::Set),
        // Extended
        "abs" => Ok(Op::Abs),
        "acos" => Ok(Op::Acos),
        "acosh" => Ok(Op::Acosh),
        "asin" => Ok(Op::Asin),
        "asinh" => Ok(Op::Asinh),
        "atan" => Ok(Op::Atan),
        "atan2" => Ok(Op::Atan2),
        "atanh" => Ok(Op::Atanh),
        "cbrt" => Ok(Op::Cbrt),
        "ceil" => Ok(Op::Ceil),
        "copysign" => Ok(Op::Copysign),
        "cos" => Ok(Op::Cos),
        "cosh" => Ok(Op::Cosh),
        "degrees" => Ok(Op::Degrees),
        "diveuclid" => Ok(Op::Diveuclid),
        "exp" => Ok(Op::Exp),
        "exp2" => Ok(Op::Exp2),
        "expm1" => Ok(Op::Expm1),
        "floor" => Ok(Op::Floor),
        "fract" => Ok(Op::Fract),
        "hypot" => Ok(Op::Hypot),
        "ln" => Ok(Op::Ln),
        "ln1p" => Ok(Op::Ln1p),
        "log" => Ok(Op::Log),
        "log2" => Ok(Op::Log2),
        "log10" => Ok(Op::Log10),
        "max" => Ok(Op::Max),
        "min" => Ok(Op::Min),
        "radians" => Ok(Op::Radians),
        "recip" => Ok(Op::Recip),
        "remeuclid" => Ok(Op::Remeuclid),
        "round" => Ok(Op::Round),
        "signum" => Ok(Op::Signum),
        "sin" => Ok(Op::Sin),
        "sinh" => Ok(Op::Sinh),
        "sqrt" => Ok(Op::Sqrt),
        "tan" => Ok(Op::Tan),
        "tanh" => Ok(Op::Tanh),
        "trunc" => Ok(Op::Trunc),
        // Custom
        "invert" => Ok(Op::Invert),
        _ => Err(()),
    }
}

fn cmp(item: &str) -> Result<Cmp, ()> {
    match item.as_ref() {
        "==" | "eq" => Ok(Cmp::Eq),
        "!=" | "!" | "neq" => Ok(Cmp::NEq),
        ">" | "gt" => Ok(Cmp::Gt),
        "<" | "lt" => Ok(Cmp::Lt),
        ">=" | "gteq" => Ok(Cmp::GtEq),
        "<=" | "lteq" => Ok(Cmp::LtEq),
        _ => Err(()),
    }
}

fn spc(item: &str) -> Result<Space, ()> {
    Space::try_from(item.as_ref())
}

fn oper_space(items: &[&str], space: &mut Space, line: usize) -> Result<Operation, OpError> {
    if items.as_ref().len() == 1 {
        match spc(items[0]) {
            Ok(s) => {
                *space = s;
                Ok(Operation::Space(s))
            }
            Err(()) => Err(OpError::Partial {
                line,
                details: "Invalid space change".to_string(),
            }),
        }
    } else {
        Err(OpError::Unknown { line })
    }
}

fn oper_process(items: &[&str], space: &mut Space, line: usize) -> Result<Operation, OpError> {
    if items.len() == 3 {
        let parsed = (tar(items[0], *space), op(items[1]), src(items[2], *space));
        if parsed.0.is_err() {
            Err(OpError::Partial {
                line,
                details: "Invalid target".to_string(),
            })
        } else if parsed.1.is_err() {
            Err(OpError::Partial {
                line,
                details: "Invalid operator".to_string(),
            })
        } else if parsed.2.is_err() {
            Err(OpError::Partial {
                line,
                details: "Invalid source".to_string(),
            })
        } else {
            Ok(Operation::Process {
                target: parsed.0.unwrap(),
                operation: parsed.1.unwrap(),
                source: parsed.2.unwrap(),
            })
        }
    } else {
        Err(OpError::Unknown { line })
    }
}

fn oper_if(items: &[&str], space: &mut Space, line: usize) -> Result<Operation, OpError> {
    if items.len() > 4 {
        if items[0] != "if" {
            return Err(OpError::Unknown { line });
        }
        let parsed = (src(items[1], *space), cmp(items[2]), src(items[3], *space));
        if parsed.0.is_err() {
            Err(OpError::Partial {
                line,
                details: "Invalid target".to_string(),
            })
        } else if parsed.1.is_err() {
            Err(OpError::Partial {
                line,
                details: "Invalid operator".to_string(),
            })
        } else if parsed.2.is_err() {
            Err(OpError::Partial {
                line,
                details: "Invalid source".to_string(),
            })
        } else {
            Ok(Operation::If {
                left: parsed.0.unwrap(),
                cmp: parsed.1.unwrap(),
                right: parsed.2.unwrap(),
                then: Box::new(parse_op(&items[4..], space, line)?),
            })
        }
    } else {
        Err(OpError::Unknown { line })
    }
}

fn oper_jmp(items: &[&str], _space: &mut Space, line: usize) -> Result<Operation, OpError> {
    if items.len() == 2 {
        if items[0] == "goto" || items[0] == "jmp" {
            Ok(Operation::GotoTmp(items[1].to_string()))
        } else {
            Err(OpError::Unknown { line })
        }
    } else {
        Err(OpError::Unknown { line })
    }
}

fn oper_swap(items: &[&str], space: &mut Space, line: usize) -> Result<Operation, OpError> {
    if items.len() == 3 {
        if items[0] == "swap" {
            let parsed = (tar(items[1], *space), tar(items[2], *space));
            if parsed.0.is_err() {
                Err(OpError::Partial {
                    line,
                    details: "Invalid left target".to_string(),
                })
            } else if parsed.1.is_err() {
                Err(OpError::Partial {
                    line,
                    details: "Invalid right target".to_string(),
                })
            } else {
                Ok(Operation::Swap {
                    t1: parsed.0.unwrap(),
                    t2: parsed.1.unwrap(),
                })
            }
        } else {
            Err(OpError::Unknown { line })
        }
    } else {
        Err(OpError::Unknown { line })
    }
}

fn parse_op(items: &[&str], space: &mut Space, line: usize) -> Result<Operation, OpError> {
    let mut results = [oper_process, oper_space, oper_if, oper_jmp, oper_swap]
        .iter()
        .map(|f| f(&items, space, line));

    let mut non_unknown = None;
    loop {
        match results.next() {
            Some(i) => match i {
                Ok(o) => break Ok(o),
                Err(e) => {
                    if non_unknown == None
                        && match e {
                            OpError::Partial { .. } => true,
                            OpError::Unknown { .. } => false,
                        }
                    {
                        non_unknown = Some(e)
                    }
                }
            },
            None => {
                break Err(match non_unknown {
                    Some(e) => e,
                    None => OpError::Unknown { line },
                })
            }
        }
    }
}

fn goto_conv(op: Option<Operation>, labels: &HashMap<String, usize>) -> Option<Operation> {
    match op {
        Some(Operation::GotoTmp(s)) => labels.get(&s).map(|i| Operation::Goto(*i)),
        Some(Operation::If {
            left,
            cmp,
            right,
            then,
        }) => goto_conv(Some(*then), labels).map(|t| Operation::If {
            left,
            cmp,
            right,
            then: Box::new(t),
        }),
        other => other,
    }
}

pub fn parse_ops<S: AsRef<str>>(code: S, mut space: Space) -> (Vec<Operation>, Vec<OpError>) {
    // {{{
    let mut line = 0;
    let mut operations = Vec::<Operation>::new();
    let mut errs = Vec::<OpError>::new();
    let mut labels = HashMap::<String, usize>::new();
    // initial Space
    operations.push(Operation::Space(space));
    let mut items = Vec::<&str>::new();
    for fullrow in code.as_ref().to_ascii_lowercase().trim().split('\n') {
        line += 1;
        for row in fullrow.split(';') {
            if row.starts_with('#') {
                continue;
            } else if row.ends_with('\\') {
                items.extend_from_slice(
                    &row[0..row.len() - 1]
                        .split_ascii_whitespace()
                        .collect::<Vec<&str>>(),
                );
                continue;
            } else if row.starts_with(":") {
                labels.insert(row[1..].to_string(), operations.len());
            } else {
                items.extend_from_slice(
                    &row[0..row.len()]
                        .split_ascii_whitespace()
                        .collect::<Vec<&str>>(),
                );
            }
            if items.is_empty() {
                items = Vec::new();
                continue;
            }

            match parse_op(&items, &mut space, line) {
                Ok(o) => operations.push(o),
                Err(e) => errs.push(e),
            }

            items = Vec::new();
        }
    }

    operations = operations
        .into_iter()
        .filter_map(|o| goto_conv(Some(o), &labels))
        .collect();

    (operations, errs)
} // }}}
