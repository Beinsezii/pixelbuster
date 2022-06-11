use std::f32::consts::PI;

use crossbeam_utils::thread::scope;
use rand::random;

pub mod parse;
pub use parse::{parse_ops, Operation, Op, Obj};

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
