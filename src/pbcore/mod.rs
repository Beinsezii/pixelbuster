use std::f32::consts::{E, PI};

use crossbeam_utils::thread::scope;
use rand::random;

pub mod parse;
pub use parse::{parse_ops, Obj, Op, Operation};

pub mod color;
pub use color::{convert_space, Space};

// TODO: make run-able without alpha.
// TODO: Result<>
pub fn process_segment<O: AsRef<[Operation]>>(
    ops: O,
    pixels: &mut [f32],
    vdefaults: Option<[f32; 9]>,
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
    let vdefaults = vdefaults.unwrap_or([0.0_f32; 9]);

    // TODO: std's new packed_simd
    for pixel in pixels.chunks_mut(4) {
        // reset space transforms for each pixel
        space = orig_space;
        // reset vars each iter
        let mut v = vdefaults;
        for op in ops.iter() {
            match op {
                Operation::Process {
                    target,
                    operation,
                    source,
                } => {
                    let src: f32 = match *source {
                        Obj::Chan(i) => pixel[i],
                        Obj::Var(i) => v[i],
                        Obj::Num(n) => n,
                        Obj::E => E,
                        Obj::Pi => PI,
                        Obj::Rand => random::<f32>(),
                    };

                    let tar: &mut f32 = match *target {
                        Obj::Chan(i) => &mut pixel[i],
                        Obj::Var(i) => &mut v[i],
                        _ => panic!("This shouldn't be reachable"),
                    };

                    match operation {
                        Op::Add => *tar += src,
                        Op::Sub => *tar -= src,
                        Op::Mul => *tar *= src,
                        Op::Div => *tar /= src,
                        Op::Mod => *tar %= src,
                        Op::Pow => *tar = tar.powf(src),
                        Op::Set => *tar = src,
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
                        Op::Cos => *tar = src.cos(),
                        Op::Cosh => *tar = src.cosh(),
                        Op::Floor => *tar = src.floor(),
                        Op::Log => *tar = tar.log(src),
                        Op::Max => *tar = tar.max(src),
                        Op::Min => *tar = tar.min(src),
                        Op::Round => *tar = src.round(),
                        Op::Sin => *tar = src.sin(),
                        Op::Sinh => *tar = src.sinh(),
                        Op::Sqrt => *tar = src.sqrt(),
                        Op::Tan => *tar = src.tan(),
                        Op::Tanh => *tar = src.tanh(),
                    };
                }
                Operation::Space(new_space) => {
                    if space == new_space {
                        continue;
                    }

                    convert_space(*space, *new_space, &mut pixel[0..3].try_into().unwrap());

                    space = new_space;
                }
            }
        }
        // restore to original if not already
        if orig_space != space {
            convert_space(*space, *orig_space, &mut pixel[0..3].try_into().unwrap());
        }
    }
} // }}}

pub fn process_multi<O: AsRef<[Operation]>>(
    ops: O,
    pixels: &mut [f32],
    vdefaults: Option<[f32; 9]>,
) {
    let ops: &[Operation] = ops.as_ref();

    if pixels.len() < 400 {
        // < 10x10 grid always single thread.
        // dumb way to make sure it splits well + overhead avoidance.
        process_segment(ops, pixels, vdefaults)
    } else {
        scope(|s| {
            let mut threads = Vec::new();

            let count: usize = num_cpus::get();
            let mut chunks: Vec<&mut [f32]> =
                pixels.chunks_mut((pixels.len() / 4) / count * 4).collect();

            for _ in 0..chunks.len() {
                let chunk: &mut [f32] = chunks.pop().unwrap();
                let op = ops.clone();
                threads.push(s.spawn(move |_| process_segment(&op, chunk, vdefaults)));
            }

            for t in threads {
                t.join().expect("Thread fail");
            }
        })
        .unwrap();
    }
}
