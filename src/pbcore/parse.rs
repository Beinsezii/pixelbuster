use super::Space;

// structs {{{
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Set,
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
    Cos,
    Cosh,
    Floor,
    Log,
    Max,
    Min,
    Round,
    Sin,
    Sinh,
    Sqrt,
    Tan,
    Tanh,
}

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub enum Operation {
    Space(Space),
    Process {
        target: Obj,
        operation: Op,
        source: Obj,
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
        val => match space.to_string().to_ascii_lowercase().find(val) {
            Some(n) => Ok(Obj::Chan(n)),
            None => Err(()),
        },
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
        "+=" | "+" | "add" => Ok(Op::Add),
        "-=" | "-" | "sub" => Ok(Op::Sub),
        "*=" | "*" | "mul" => Ok(Op::Mul),
        "/=" | "/" | "div" => Ok(Op::Div),
        "%=" | "%" | "mod" => Ok(Op::Mod),
        "**" | "^" | "pow" => Ok(Op::Pow),
        "=" | "set" => Ok(Op::Set),
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
        "cos" => Ok(Op::Cos),
        "cosh" => Ok(Op::Cosh),
        "floor" => Ok(Op::Floor),
        "log" => Ok(Op::Log),
        "max" => Ok(Op::Max),
        "min" => Ok(Op::Min),
        "round" => Ok(Op::Round),
        "sin" => Ok(Op::Sin),
        "sinh" => Ok(Op::Sinh),
        "sqrt" => Ok(Op::Sqrt),
        "tan" => Ok(Op::Tan),
        "tanh" => Ok(Op::Tanh),
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

pub fn parse_ops<S: AsRef<str>>(code: S, mut space: Space) -> (Vec<Operation>, Vec<OpError>) {
    // {{{
    let mut line = 0;
    let mut operations = Vec::<Operation>::new();
    let mut errs = Vec::<OpError>::new();
    let fns = &[oper_space, oper_process];
    // initial Space
    operations.push(Operation::Space(space));
    let mut items = Vec::<&str>::new();
    for row in code.as_ref().to_ascii_lowercase().trim().split('\n') {
        line += 1;
        if row.starts_with('#') {
            continue;
        } else if row.ends_with('\\') {
            items.extend_from_slice(
                &row[0..row.len() - 1]
                    .split_ascii_whitespace()
                    .collect::<Vec<&str>>(),
            );
            continue;
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
        let mut results = fns
            .iter()
            .map(|f| f(&items, &mut space, line))
            .collect::<Vec<Result<Operation, OpError>>>()
            .into_iter();
        items = Vec::new();
        let mut non_unknown = None;
        if loop {
            match results.next() {
                Some(i) => match i {
                    Ok(o) => {
                        operations.push(o);
                        break false;
                    }
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
                None => break true,
            }
        } {
            errs.push(non_unknown.unwrap_or(OpError::Unknown { line }))
        }
    }

    (operations, errs)
} // }}}
