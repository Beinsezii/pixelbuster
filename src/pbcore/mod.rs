use std::f32::consts::{E, PI};
use std::thread::{self, ScopedJoinHandle};

use fastrand;

pub mod parse;
pub use parse::{parse_ops, Cmp, Obj, Op, OpError, Operation};

pub use colcon::{convert_space, Space};

// TODO: make run-able without alpha.
// TODO: Result<> instead of panic
fn process_segment<O: AsRef<[Operation]>>(
    ops: O,
    pixels: &mut [f32],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    externals: Option<[f32; 9]>,
) {
    // {{{
    assert!(pixels.len() % 4 == 0);

    let ops: &[Operation] = ops.as_ref();

    // needs an initial Space for reference
    let mut space = match ops.get(0) {
        Some(op) => match op {
            Operation::Space(space) => space,
            _ => return,
        },
        None => return,
    };

    let orig_space = space;
    let defaults: [f32; 18] = {
        let e = externals.unwrap_or([0.0_f32; 9]);
        [
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, e[0], e[1], e[2], e[3], e[4], e[5], e[6],
            e[7], e[8],
        ]
    };

    // TODO: std's new packed_simd
    for (n, pixel) in pixels.chunks_exact_mut(4).enumerate() {
        let pixel: &mut [f32; 4] = pixel.try_into().unwrap();
        // reset space transforms for each pixel
        space = orig_space;
        // reset vars each iter
        let mut v: [f32; 18] = defaults;
        let mut goto_breaker = 0;
        let mut iter = ops.iter();
        let mut op = match iter.next() {
            Some(o) => o,
            None => return,
        };

        macro_rules! tar {
            ($obj:expr) => {
                match $obj {
                    Obj::Chan(i) => &mut pixel[i],
                    Obj::Var(i) => &mut v[i],
                    _ => panic!("This shouldn't be reachable"),
                }
            };
        }

        macro_rules! src {
            ($obj:expr) => {
                match $obj {
                    Obj::Chan(i) => pixel[i],
                    Obj::Var(i) => v[i],
                    Obj::Num(n) => n,
                    Obj::E => E,
                    Obj::Pi => PI,
                    Obj::Rand => fastrand::f32(),
                    Obj::Col => ((n + x + y * width) % width) as f32,
                    Obj::Row => ((n + x + y * width) / width) as f32,
                    Obj::Width => width as f32,
                    Obj::Height => height as f32,
                    Obj::XNorm => (((n + x + y * width) % width) as f32) / width as f32,
                    Obj::YNorm => (((n + x + y * width) / width) as f32) / height as f32,
                }
            };
        }

        loop {
            match op {
                Operation::Process {
                    target,
                    operation,
                    source,
                } => {
                    let src: f32 = src!(*source);

                    let tar: &mut f32 = tar!(*target);

                    match operation {
                        // Base
                        Op::Add => *tar += src,
                        Op::Sub => *tar -= src,
                        Op::Mul => *tar *= src,
                        Op::Div => *tar /= src,
                        Op::Mod => *tar %= src,
                        Op::Pow => *tar = tar.powf(src),
                        Op::Set => *tar = src,
                        // Extended
                        Op::Abs => *tar = src.abs(),
                        Op::Acos => *tar = src.acos(),
                        Op::Acosh => *tar = src.acosh(),
                        Op::Asin => *tar = src.asin(),
                        Op::Asinh => *tar = src.asinh(),
                        Op::Atan => *tar = src.atan(),
                        Op::Atan2 => *tar = tar.atan2(src),
                        Op::Atanh => *tar = src.atanh(),
                        Op::Cbrt => *tar = src.cbrt(),
                        Op::Ceil => *tar = src.ceil(),
                        Op::Copysign => *tar = tar.copysign(src),
                        Op::Cos => *tar = src.cos(),
                        Op::Cosh => *tar = src.cosh(),
                        Op::Degrees => *tar = src.to_degrees(),
                        Op::Diveuclid => *tar = tar.div_euclid(src),
                        Op::Exp => *tar = src.exp(),
                        Op::Exp2 => *tar = src.exp2(),
                        Op::Expm1 => *tar = src.exp_m1(),
                        Op::Floor => *tar = src.floor(),
                        Op::Fract => *tar = src.fract(),
                        Op::Hypot => *tar = tar.hypot(src),
                        Op::Ln => *tar = src.ln(),
                        Op::Ln1p => *tar = src.ln_1p(),
                        Op::Log => *tar = tar.log(src),
                        Op::Log10 => *tar = src.log10(),
                        Op::Log2 => *tar = src.log2(),
                        Op::Max => *tar = tar.max(src),
                        Op::Min => *tar = tar.min(src),
                        Op::Radians => *tar = src.to_radians(),
                        Op::Recip => *tar = src.recip(),
                        Op::Remeuclid => *tar = tar.rem_euclid(src),
                        Op::Round => *tar = src.round(),
                        Op::Signum => *tar = src.signum(),
                        Op::Sin => *tar = src.sin(),
                        Op::Sinh => *tar = src.sinh(),
                        Op::Sqrt => *tar = src.sqrt(),
                        Op::Tan => *tar = src.tan(),
                        Op::Tanh => *tar = src.tanh(),
                        Op::Trunc => *tar = src.trunc(),
                        // Custom
                        Op::Invert => *tar = src - *tar,
                    };
                }
                Operation::Space(new_space) => {
                    convert_space(*space, *new_space, pixel);
                    space = new_space;
                }
                Operation::If {
                    left,
                    cmp,
                    right,
                    then,
                } => {
                    let left = src!(*left);

                    let right = src!(*right);

                    if match cmp {
                        Cmp::Eq => left == right,
                        Cmp::NEq => left != right,
                        Cmp::Gt => left > right,
                        Cmp::Lt => left < right,
                        Cmp::GtEq => left >= right,
                        Cmp::LtEq => left <= right,
                    } {
                        op = then.as_ref();
                        continue;
                    }
                }
                Operation::Goto(i) => {
                    if goto_breaker < 100 {
                        iter = ops[*i..].iter();
                        goto_breaker += 1;
                    } else {
                        break;
                    }
                }
                Operation::GotoTmp(_) => panic!("GotoTmp shouldn't be sent to process!"),
                // hypothetically should be safe, as the pointers can't be uninitialized?
                Operation::Swap { t1, t2 } => unsafe {
                    std::ptr::swap(tar!(*t1), tar!(*t2));
                },
            }
            match iter.next() {
                Some(o) => op = o,
                None => break,
            }
        }
        // restore to original if not already
        if space != orig_space {
            convert_space(*space, *orig_space, pixel)
        }
    }
} // }}}

pub fn process<O: AsRef<[Operation]>>(
    ops: O,
    pixels: &mut [f32],
    mut width: usize,
    externals: Option<[f32; 9]>,
) {
    let ops: &[Operation] = ops.as_ref();
    let height = if width == 0 {
        width = usize::MAX;
        usize::MAX
    } else {
        pixels.len() / 4 / width
    };

    if pixels.len() < 400 {
        // < 10x10 grid always single thread.
        // dumb way to make sure it splits well + overhead avoidance.
        process_segment(ops, pixels, 0, 0, width, height, externals)
    } else {
        let chunk_size: usize = pixels.len() / 4 / thread::available_parallelism().unwrap() * 4;
        let chunks: Vec<(usize, &mut [f32])> = pixels.chunks_mut(chunk_size).enumerate().collect();
        thread::scope(|scoped| {
            chunks
                .into_iter()
                .map(|(n, chunk)| {
                    scoped.spawn(move || {
                        process_segment(
                            ops,
                            chunk,
                            (chunk_size / 4 * n) % width,
                            (chunk_size / 4 * n) / width,
                            width,
                            height,
                            externals,
                        )
                    })
                })
                .collect::<Vec<ScopedJoinHandle<()>>>()
                .into_iter()
                .for_each(|jh| jh.join().unwrap());
        });
    }
}
