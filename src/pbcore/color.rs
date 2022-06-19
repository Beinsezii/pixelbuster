use core::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl TryFrom<&str> for Space {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, ()> {
        match value.to_ascii_lowercase().trim() {
            "srgb" | "rgb" | "srgba" | "rgba" => Ok(Space::SRGB),
            "lrgb" | "lrgba" => Ok(Space::LRGB),
            "xyz" | "xyza" => Ok(Space::XYZ),
            // TODO use alpha with LAB without using "c4"???
            "lab" | "laba" => Ok(Space::LAB),
            "lch" | "lcha" => Ok(Space::LCH),
            _ => Err(()),
        }
    }
}

#[rustfmt::skip]
pub fn convert_space(from: Space, to: Space, pixel: &mut [f32; 3]) {
    match (from, to) {
        // No-op
        (Space::SRGB, Space::SRGB)
        | (Space::LRGB, Space::LRGB)
        | (Space::XYZ, Space::XYZ)
        | (Space::LAB, Space::LAB)
        | (Space::LCH, Space::LCH) => (),
        // Up
        (Space::SRGB, Space::LRGB) => srgb_to_lrgb(pixel),
        (Space::SRGB, Space::XYZ) => {srgb_to_lrgb(pixel); lrgb_to_xyz(pixel)},
        (Space::SRGB, Space::LAB) => {srgb_to_lrgb(pixel); lrgb_to_xyz(pixel); xyz_to_lab(pixel)},
        (Space::SRGB, Space::LCH) => {srgb_to_lrgb(pixel); lrgb_to_xyz(pixel); xyz_to_lab(pixel); lab_to_lch(pixel)},
        (Space::LRGB, Space::XYZ) => lrgb_to_xyz(pixel),
        (Space::LRGB, Space::LAB) => {lrgb_to_xyz(pixel); xyz_to_lab(pixel)},
        (Space::LRGB, Space::LCH) => {lrgb_to_xyz(pixel); xyz_to_lab(pixel); lab_to_lch(pixel)},
        (Space::XYZ, Space::LAB) => xyz_to_lab(pixel),
        (Space::XYZ, Space::LCH) => {xyz_to_lab(pixel); lab_to_lch(pixel)},
        (Space::LAB, Space::LCH) => lab_to_lch(pixel),
        // Down
        (Space::LCH, Space::LAB) => lch_to_lab(pixel),
        (Space::LCH, Space::XYZ) => {lch_to_lab(pixel); lab_to_xyz(pixel)},
        (Space::LCH, Space::LRGB) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_lrgb(pixel)},
        (Space::LCH, Space::SRGB) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::LAB, Space::XYZ) => lab_to_xyz(pixel),
        (Space::LAB, Space::LRGB) => {lab_to_xyz(pixel); xyz_to_lrgb(pixel)},
        (Space::LAB, Space::SRGB) => {lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::XYZ, Space::LRGB) => xyz_to_lrgb(pixel),
        (Space::XYZ, Space::SRGB) => {xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::LRGB, Space::SRGB) => lrgb_to_srgb(pixel),
    }
}

pub fn convert_space_alpha(from: Space, to: Space, pixel: &mut [f32; 4]) {
    unsafe {
        convert_space(
            from,
            to,
            pixel.get_unchecked_mut(0..3).try_into().unwrap_unchecked(),
        )
    }
}

// UP {{{

pub fn srgb_to_irgb(pixel: [f32; 3]) -> [u8; 3] {
    [
        ((pixel[0] * 255.0) as u8).min(0).max(255),
        ((pixel[1] * 255.0) as u8).min(0).max(255),
        ((pixel[2] * 255.0) as u8).min(0).max(255),
    ]
}

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

pub fn srgb_to_lrgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| {
        if *c <= 0.04045 {
            *c /= 12.92
        } else {
            *c = ((*c + 0.055) / 1.055_f32).powf(2.4)
        }
    });
}

pub fn lrgb_to_xyz(pixel: &mut [f32; 3]) {
    *pixel = [
        (0.4124 * pixel[0] + 0.3576 * pixel[1] + 0.1805 * pixel[2]) * 100.0, // X
        (0.2126 * pixel[0] + 0.7152 * pixel[1] + 0.0722 * pixel[2]) * 100.0, // Y
        (0.0193 * pixel[0] + 0.1192 * pixel[1] + 0.9505 * pixel[2]) * 100.0, // Z
    ]
}

pub fn xyz_to_lab(pixel: &mut [f32; 3]) {
    // convert to D65 2 degrees
    pixel[0] /= 95.057;
    pixel[1] /= 100.0;
    pixel[2] /= 108.883;

    pixel.iter_mut().for_each(|c| {
        if *c > 0.008856 {
            *c = c.powf(1.0 / 3.0)
        } else {
            *c = (7.787 * *c) + (16.0 / 116.0)
        }
    });

    *pixel = [
        (116.0 * pixel[1]) - 16.0,
        500.0 * (pixel[0] - pixel[1]),
        200.0 * (pixel[1] - pixel[2]),
    ]
}

/// Return CIE LCH
pub fn lab_to_lch(pixel: &mut [f32; 3]) {
    let mut h = pixel[2].atan2(pixel[1]);
    if h > 0.0 {
        h = (h / PI) * 180.0
    } else {
        h = 360.0 - ((h.abs() / PI) * 180.0)
    }

    *pixel = [
        pixel[0],
        ((pixel[1].powi(2)) + (pixel[2].powi(2))).sqrt(),
        h,
    ]
}

// UP }}}

// DOWN {{{

pub fn irgb_to_srgb(pixel: [u8; 3]) -> [f32; 3] {
    [
        pixel[0] as f32 / 255.0,
        pixel[1] as f32 / 255.0,
        pixel[2] as f32 / 255.0,
    ]
}

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

pub fn lrgb_to_srgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| {
        if *c <= 0.0031308 {
            *c *= 12.92
        } else {
            *c = 1.055 * (c.powf(1.0 / 2.4)) - 0.055
        }
    });
}

/// Set from XYZ
pub fn xyz_to_lrgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| *c /= 100.0);
    *pixel = [
        3.2406 * pixel[0] - 1.5372 * pixel[1] - 0.4986 * pixel[2],
        -0.9689 * pixel[0] + 1.8758 * pixel[1] + 0.0415 * pixel[2],
        0.0557 * pixel[0] - 0.2040 * pixel[1] + 1.0570 * pixel[2],
    ];
}

/// Set from CIE LAB
pub fn lab_to_xyz(pixel: &mut [f32; 3]) {
    let mut xyz = [0.0_f32; 3];
    xyz[1] = (pixel[0] + 16.0) / 116.0;
    xyz[0] = (pixel[1] / 500.0) + xyz[1];
    xyz[2] = xyz[1] - (pixel[2] / 200.0);

    xyz.iter_mut().for_each(|c| {
        if c.powi(3) > 0.008856 {
            *c = c.powi(3)
        } else {
            *c = (*c - (16.0 / 116.0)) / 7.787
        }
    });

    // convert back from D65 2 degrees
    *pixel = [xyz[0] * 95.057, xyz[1] * 100.0, xyz[2] * 108.883]
}

/// Set from CIE LCH
pub fn lch_to_lab(pixel: &mut [f32; 3]) {
    let c = pixel[1];
    pixel[1] = pixel[2].to_radians().cos() * c;
    pixel[2] = pixel[2].to_radians().sin() * c;
}

// DOWN }}}
