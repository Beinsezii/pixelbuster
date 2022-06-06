//TODO: process() = parse_ops() + process_multi()
use std::f32::consts::PI;

use crossbeam::scope;
use rand::random;

#[derive(Clone, Copy, PartialEq, Eq)]
// TODO: HSV???
pub enum Space {
    SRGB,
    LRGB,
    XYZ,
    LAB,
    LCH,
}

impl ToString for Space {
    fn to_string(&self) -> String {
        match self {
            Space::SRGB => String::from("rgba"),
            Space::LRGB => String::from("rgba"),
            Space::XYZ => String::from("xyza"),
            Space::LAB => String::from("lab"),
            Space::LCH => String::from("lcha"),
        }
    }
}

// Color {{{
// TODO: is this wasteful? bench some other options, like 1000 pure FNs
// Long term I should 100% implement some hand-made conversion system that is smart enough to
// perform ops like going from LAB -> LCH directly instead of down to SRGB then back up.
#[derive(Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

impl Color {
    // as {{{

    /// Return sRGB
    pub fn as_srgb(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    /// Return Integer sRGB
    // pub fn as_irgb(self) -> [u8; 3] {
    //     [
    //         ((self.r * 255.0) as u8).min(0).max(255),
    //         ((self.g * 255.0) as u8).min(0).max(255),
    //         ((self.b * 255.0) as u8).min(0).max(255),
    //     ]
    // }

    /// Return hex string
    // pub fn as_hex(self) -> String {
    //     let mut hex = String::from("#");

    //     for x in self.as_irgb() {
    //         n1 = int(x / 16)
    //         n2 = x % 16
    //         for n in (n1, n2):
    //             hex += str(chr((n - 10) + 65) if n >= 10 else n)
    //     }

    //     hex
    // }

    /// Return linear RGB
    pub fn as_lrgb(self) -> [f32; 3] {
        let mut rgb = self.as_srgb();
        rgb.iter_mut().for_each(|c| {
            if *c <= 0.04045 {
                *c /= 12.92
            } else {
                *c = ((*c + 0.055) / 1.055_f32).powf(2.4)
            }
        });
        rgb
    }

    /// Return XYZ
    pub fn as_xyz(self) -> [f32; 3] {
        let rgb = self.as_lrgb();
        [
            (0.4124 * rgb[0] + 0.3576 * rgb[1] + 0.1805 * rgb[2]) * 100.0, // X
            (0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]) * 100.0, // Y
            (0.0193 * rgb[0] + 0.1192 * rgb[1] + 0.9505 * rgb[2]) * 100.0, // Z
        ]
    }

    /// Return CIE LAB
    pub fn as_lab(self) -> [f32; 3] {
        // convert to D65 2 degrees
        let mut xyz = self.as_xyz();
        xyz[0] /= 95.057;
        xyz[1] /= 100.0;
        xyz[2] /= 108.883;

        xyz.iter_mut().for_each(|c| {
            if *c > 0.008856 {
                *c = c.powf(1.0 / 3.0)
            } else {
                *c = (7.787 * *c) + (16.0 / 116.0)
            }
        });

        [
            (116.0 * xyz[1]) - 16.0,
            500.0 * (xyz[0] - xyz[1]),
            200.0 * (xyz[1] - xyz[2]),
        ]
    }

    /// Return CIE LCH
    pub fn as_lch(self) -> [f32; 3] {
        let lab = self.as_lab();

        let mut h = lab[2].atan2(lab[1]);
        if h > 0.0 {
            h = (h / PI) * 180.0
        } else {
            h = 360.0 - ((h.abs() / PI) * 180.0)
        }

        [lab[0], ((lab[1].powi(2)) + (lab[2].powi(2))).sqrt(), h]
    }

    // as }}}

    // set {{{

    pub fn set_srgb(&mut self, rgb: [f32; 3]) {
        self.r = rgb[0];
        self.g = rgb[1];
        self.b = rgb[2];
    }

    ///  Set from integer RGB
    // pub fn set_irgb(&mut self, irgb: [u8; 3]) {
    //     return self.set_srgb([
    //         irgb[0] as f32 / 255.0,
    //         irgb[1] as f32 / 255.0,
    //         irgb[2] as f32 / 255.0,
    //     ]);
    // }

    // /// Set from hex string
    // pub fn set_hex(self, hex: str) {
    //     hex = hex.lstrip('#').upper()

    //     hexR = hex[0:2]
    //     hexG = hex[2:4]
    //     hexB = hex[4:6]

    //     rgb = [0, 0, 0]
    //     for n, x in enumerate((hexR, hexG, hexB)):
    //         # 16s place
    //         if x[0].isalpha():
    //             rgb[n] += (ord(x[0]) - 65 + 10) * 16
    //         elif x[0].isdigit():
    //             rgb[n] += int(x[0]) * 16
    //         else:
    //             print("This should be impossible.")
    //             raise ValueError
    //         # 1s place
    //         if x[1].isalpha():
    //             rgb[n] += (ord(x[1]) - 65 + 10)
    //         elif x[1].isdigit():
    //             rgb[n] += int(x[1])
    //         else:
    //             print("This should be impossible.")
    //             raise ValueError

    //     return self.set_irgb(*rgb)
    // }

    /// Set from linear RGB
    pub fn set_lrgb(&mut self, mut lrgb: [f32; 3]) {
        lrgb.iter_mut().for_each(|c| {
            if *c <= 0.0031308 {
                *c *= 12.92
            } else {
                *c = 1.055 * (c.powf(1.0 / 2.4)) - 0.055
            }
        });
        self.set_srgb(lrgb)
    }

    /// Set from XYZ
    pub fn set_xyz(&mut self, mut xyz: [f32; 3]) {
        xyz.iter_mut().for_each(|c| *c /= 100.0);
        self.set_lrgb([
            3.2406 * xyz[0] - 1.5372 * xyz[1] - 0.4986 * xyz[2],
            -0.9689 * xyz[0] + 1.8758 * xyz[1] + 0.0415 * xyz[2],
            0.0557 * xyz[0] - 0.2040 * xyz[1] + 1.0570 * xyz[2],
        ]);
    }

    /// Set from CIE LAB
    pub fn set_lab(&mut self, lab: [f32; 3]) {
        let mut xyz = [0.0_f32; 3];
        xyz[1] = (lab[0] + 16.0) / 116.0;
        xyz[0] = (lab[1] / 500.0) + xyz[1];
        xyz[2] = xyz[1] - (lab[2] / 200.0);

        xyz.iter_mut().for_each(|c| {
            if c.powi(3) > 0.008856 {
                *c = c.powi(3)
            } else {
                *c = (*c - (16.0 / 116.0)) / 7.787
            }
        });

        // convert back from D65 2 degrees
        xyz[0] *= 95.057;
        xyz[1] *= 100.0;
        xyz[2] *= 108.883;

        self.set_xyz(xyz)
    }

    /// Set from CIE LCH
    pub fn set_lch(&mut self, mut lch: [f32; 3]) {
        let c = lch[1];
        lch[1] = lch[2].to_radians().cos() * c;
        lch[2] = lch[2].to_radians().sin() * c;
        self.set_lab(lch)
    }

    // set }}}
}
// Color }}}

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
}

#[derive(Clone, Copy, PartialEq)]
pub enum Operation {
    Space(Space),
    Process {
        target: Obj,
        operation: Op,
        value: Obj,
    },
}
// }}}

pub fn parse_ops<S: AsRef<str>>(ops: S, mut space: Space) -> Vec<Operation> {
    // {{{
    let mut line = 0;
    let mut result = Vec::<Operation>::new();
    let mut chs = space.to_string();
    // initial Space
    result.push(Operation::Space(space));
    for op in ops.as_ref().to_ascii_lowercase().trim().split('\n') {
        line += 1;
        let items = op.split_ascii_whitespace().collect::<Vec<&str>>();

        if items.len() == 0 {
            continue;
        } else if items.len() == 1 {
            result.push(Operation::Space({
                space = match items[0] {
                    "srgb" | "rgb" | "srgba" | "rgba" => Space::SRGB,
                    "lrgb" | "lrgba" => Space::LRGB,
                    "xyz" | "xyza" => Space::XYZ,
                    // TODO use alpha with LAB without using "c4"???
                    "lab" | "laba" => Space::LAB,
                    "lch" | "lcha" => Space::LCH,
                    _ => {
                        println!("Invalid space operation on line {}", line);
                        continue;
                    }
                };
                chs = space.to_string();
                space
            }));
            continue;
        } else if items.len() != 3 {
            println!("Invalid number of args on operation line {}", line);
            continue;
        }

        result.push(Operation::Process {
            target: match items[0] {
                // don't hate I made these with a vim macro
                "c1" => Obj::Chan(0),
                "c2" => Obj::Chan(1),
                "c3" => Obj::Chan(2),
                "c4" => Obj::Chan(3),
                "v1" | "v" => Obj::Var(0),
                "v2" => Obj::Var(1),
                "v3" => Obj::Var(2),
                "v4" => Obj::Var(3),
                "v5" => Obj::Var(4),
                "v6" => Obj::Var(5),
                "v7" => Obj::Var(6),
                "v8" => Obj::Var(7),
                "v9" => Obj::Var(8),
                val => match chs.find(val) {
                    Some(n) => Obj::Chan(n),
                    None => {
                        println!("Invalid target on operation line {}", line);
                        continue;
                    }
                },
            },

            operation: match items[1] {
                "+=" | "+" | "add" => Op::Add,
                "-=" | "-" | "sub" => Op::Sub,
                "*=" | "*" | "mul" => Op::Mul,
                "/=" | "/" | "div" => Op::Div,
                "%=" | "%" | "mod" => Op::Mod,
                "**" | "^" | "pow" => Op::Pow,
                "=" | "set" => Op::Set,
                "abs" => Op::Abs,
                "acos" => Op::Acos,
                "acosh" => Op::Acosh,
                "asin" => Op::Asin,
                "asinh" => Op::Asinh,
                "atan" => Op::Atan,
                "atan2" => Op::Atan2,
                "atanh" => Op::Atanh,
                "cbrt" => Op::Cbrt,
                "ceil" => Op::Ceil,
                "cos" => Op::Cos,
                "cosh" => Op::Cosh,
                "floor" => Op::Floor,
                "log" => Op::Log,
                "max" => Op::Max,
                "min" => Op::Min,
                "round" => Op::Round,
                "sin" => Op::Sin,
                "sinh" => Op::Sinh,
                "sqrt" => Op::Sqrt,
                "tan" => Op::Tan,
                "tanh" => Op::Tanh,
                _ => {
                    println!("Invalid math operator on operation line {}", line);
                    continue;
                }
            },

            value: match items[2] {
                "c1" => Obj::Chan(0),
                "c2" => Obj::Chan(1),
                "c3" => Obj::Chan(2),
                "c4" => Obj::Chan(3),
                "v1" | "v" => Obj::Var(0),
                "v2" => Obj::Var(1),
                "v3" => Obj::Var(2),
                "v4" => Obj::Var(3),
                "v5" => Obj::Var(4),
                "v6" => Obj::Var(5),
                "v7" => Obj::Var(6),
                "v8" => Obj::Var(7),
                "v9" => Obj::Var(8),
                "e" => Obj::E,
                "pi" => Obj::Pi,
                "rand" => Obj::Rand,
                val => {
                    match chs.find(val) {
                        Some(n) => Obj::Chan(n),
                        None => {
                            match val.parse::<f32>() {
                                Ok(n) => Obj::Num(n),
                                Err(_) => {
                                    // yeah so it's a pyramid
                                    println!("Invalid value on operation line {}", line);
                                    continue;
                                }
                            }
                        }
                    }
                }
            },
        });
    }

    result
} // }}}

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

    let mut v = vdefaults.unwrap_or([0.0_f32; 9]);

    // TODO: std's new packed_simd
    for pixel in pixels.chunks_mut(4) {
        // reset space transforms for each pixel
        space = orig_space;
        for op in ops.iter() {
            match op {
                Operation::Process {
                    value,
                    target,
                    operation,
                } => {
                    let val: f32 = match *value {
                        Obj::Chan(i) => pixel[i],
                        Obj::Var(i) => v[i],
                        Obj::Num(n) => n,
                        Obj::E => std::f32::consts::E,
                        Obj::Pi => std::f32::consts::PI,
                        Obj::Rand => random::<f32>(),
                    };

                    let tar: &mut f32 = match *target {
                        Obj::Chan(i) => &mut pixel[i],
                        Obj::Var(i) => &mut v[i],
                        _ => panic!("This shouldn't be reachable"),
                    };

                    match operation {
                        Op::Add => *tar += val,
                        Op::Sub => *tar -= val,
                        Op::Mul => *tar *= val,
                        Op::Div => *tar /= val,
                        Op::Mod => *tar %= val,
                        Op::Pow => *tar = tar.powf(val),
                        Op::Set => *tar = val,
                        Op::Abs => *tar = val.abs(),
                        Op::Acos => *tar = val.acos(),
                        Op::Acosh => *tar = val.acosh(),
                        Op::Asin => *tar = val.asin(),
                        Op::Asinh => *tar = val.asinh(),
                        Op::Atan => *tar = val.atan(),
                        Op::Atan2 => *tar = tar.atan2(val),
                        Op::Atanh => *tar = val.atanh(),
                        Op::Cbrt => *tar = val.cbrt(),
                        Op::Ceil => *tar = val.ceil(),
                        Op::Cos => *tar = val.cos(),
                        Op::Cosh => *tar = val.cosh(),
                        Op::Floor => *tar = val.floor(),
                        Op::Log => *tar = tar.log(val),
                        Op::Max => *tar = tar.max(val),
                        Op::Min => *tar = tar.min(val),
                        Op::Round => *tar = val.round(),
                        Op::Sin => *tar = val.sin(),
                        Op::Sinh => *tar = val.sinh(),
                        Op::Sqrt => *tar = val.sqrt(),
                        Op::Tan => *tar = val.tan(),
                        Op::Tanh => *tar = val.tanh(),
                    };
                }
                Operation::Space(new_space) => {
                    if space == new_space {
                        continue;
                    }
                    let mut col = Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    };
                    let mut subslice = [pixel[0], pixel[1], pixel[2]];
                    match space {
                        Space::SRGB => col.set_srgb(subslice),
                        Space::LRGB => col.set_lrgb(subslice),
                        Space::XYZ => col.set_xyz(subslice),
                        Space::LAB => col.set_xyz(subslice),
                        Space::LCH => col.set_lch(subslice),
                    }
                    subslice = match new_space {
                        Space::SRGB => col.as_srgb(),
                        Space::LRGB => col.as_lrgb(),
                        Space::XYZ => col.as_xyz(),
                        Space::LAB => col.as_xyz(),
                        Space::LCH => col.as_lch(),
                    };

                    pixel[0] = subslice[0];
                    pixel[1] = subslice[1];
                    pixel[2] = subslice[2];

                    space = new_space;
                }
            }
        }
        // restore to original if not already
        if orig_space != space {
            let mut col = Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            };
            let mut subslice = [pixel[0], pixel[1], pixel[2]];
            match space {
                Space::SRGB => col.set_srgb(subslice),
                Space::LRGB => col.set_lrgb(subslice),
                Space::XYZ => col.set_xyz(subslice),
                Space::LAB => col.set_xyz(subslice),
                Space::LCH => col.set_lch(subslice),
            }
            subslice = match orig_space {
                Space::SRGB => col.as_srgb(),
                Space::LRGB => col.as_lrgb(),
                Space::XYZ => col.as_xyz(),
                Space::LAB => col.as_xyz(),
                Space::LCH => col.as_lch(),
            };

            pixel[0] = subslice[0];
            pixel[1] = subslice[1];
            pixel[2] = subslice[2];
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
