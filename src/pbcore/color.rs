use core::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Space {
    SRGB,
    HSV,
    XYZ,
    LAB,
    LCH,
}

impl ToString for Space {
    fn to_string(&self) -> String {
        match self {
            Space::SRGB => String::from("rgba"),
            Space::HSV => String::from("hsva"),
            Space::XYZ => String::from("xyza"),
            Space::LAB => String::from("laba"),
            Space::LCH => String::from("lcha"),
        }
    }
}

impl TryFrom<&str> for Space {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, ()> {
        match value.to_ascii_lowercase().trim() {
            "srgb" | "rgb" | "srgba" | "rgba" => Ok(Space::SRGB),
            "hsv" | "hsva" => Ok(Space::HSV),
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
        | (Space::HSV, Space::HSV)
        | (Space::XYZ, Space::XYZ)
        | (Space::LAB, Space::LAB)
        | (Space::LCH, Space::LCH) => (),
        // Up
        (Space::SRGB, Space::HSV) => srgb_to_hsv(pixel),
        (Space::SRGB, Space::XYZ) => srgb_to_xyz(pixel),
        (Space::SRGB, Space::LAB) => {srgb_to_xyz(pixel); xyz_to_lab(pixel)},
        (Space::SRGB, Space::LCH) => {srgb_to_xyz(pixel); xyz_to_lab(pixel); lab_to_lch(pixel)},
        (Space::XYZ, Space::LAB) => xyz_to_lab(pixel),
        (Space::XYZ, Space::LCH) => {xyz_to_lab(pixel); lab_to_lch(pixel)},
        (Space::LAB, Space::LCH) => lab_to_lch(pixel),
        (Space::HSV, Space::XYZ) => {hsv_to_srgb(pixel); srgb_to_xyz(pixel)},
        (Space::HSV, Space::LAB) => {hsv_to_srgb(pixel); srgb_to_xyz(pixel); xyz_to_lab(pixel)},
        (Space::HSV, Space::LCH) => {hsv_to_srgb(pixel); srgb_to_xyz(pixel); xyz_to_lab(pixel); lab_to_lch(pixel)},
        // Down
        (Space::LCH, Space::LAB) => lch_to_lab(pixel),
        (Space::LCH, Space::XYZ) => {lch_to_lab(pixel); lab_to_xyz(pixel)},
        (Space::LCH, Space::SRGB) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_srgb(pixel)},
        (Space::LCH, Space::HSV) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::LAB, Space::XYZ) => lab_to_xyz(pixel),
        (Space::LAB, Space::SRGB) => {lab_to_xyz(pixel); xyz_to_srgb(pixel)},
        (Space::LAB, Space::HSV) => {lab_to_xyz(pixel); xyz_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::XYZ, Space::SRGB) => xyz_to_srgb(pixel),
        (Space::XYZ, Space::HSV) => {xyz_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::HSV, Space::SRGB) => hsv_to_srgb(pixel),
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

// source: https://www.easyrgb.com/en/math.php

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
//      value      hex += str(chr((n - 10) + 65) if n >= 10 else n)
//     }

//     hex
// }

pub fn srgb_to_hsv(pixel: &mut [f32; 3]) {
    let vmin = pixel[0].min(pixel[1]).min(pixel[2]);
    let vmax = pixel[0].max(pixel[1]).max(pixel[2]);
    let dmax = vmax - vmin;

    let v = vmax;

    let (h, s) = if dmax == 0.0 {
        (0.0, 0.0)
    } else {
        let s = dmax / vmax;

        let dr = (((vmax - pixel[0]) / 6.0) + (dmax / 2.0)) / dmax;
        let dg = (((vmax - pixel[1]) / 6.0) + (dmax / 2.0)) / dmax;
        let db = (((vmax - pixel[2]) / 6.0) + (dmax / 2.0)) / dmax;

        let mut h = if pixel[0] == vmax {
            db - dg
        } else if pixel[1] == vmax {
            (1.0 / 3.0) + dr - db
        } else {
            (2.0 / 3.0) + dg - dr
        };

        if h < 0.0 {
            h += 1.0
        } else if h > 1.0 {
            h -= 1.0
        };
        (h, s)
    };
    *pixel = [h, s, v];
}

pub fn srgb_to_xyz(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| {
        if *c <= 0.04045 {
            *c /= 12.92
        } else {
            *c = ((*c + 0.055) / 1.055_f32).powf(2.4)
        }
    });
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

pub fn hsv_to_srgb(pixel: &mut [f32; 3]) {
    if pixel[1] == 0.0 {
        *pixel = [pixel[2]; 3];
    } else {
        let mut var_h = pixel[0] * 6.0;
        if var_h == 6.0 {
            var_h = 0.0
        }
        let var_i = var_h.trunc();
        let var_1 = pixel[2] * (1.0 - pixel[1]);
        let var_2 = pixel[2] * (1.0 - pixel[1] * (var_h - var_i));
        let var_3 = pixel[2] * (1.0 - pixel[1] * (1.0 - (var_h - var_i)));

        *pixel = if var_i == 0.0 {
            [pixel[2], var_3, var_1]
        } else if var_i == 1.0 {
            [var_2, pixel[2], var_1]
        } else if var_i == 2.0 {
            [var_1, pixel[2], var_3]
        } else if var_i == 3.0 {
            [var_1, var_2, pixel[2]]
        } else if var_i == 4.0 {
            [var_3, var_1, pixel[2]]
        } else {
            [pixel[2], var_1, var_2]
        }
    }
}

/// Set from XYZ
pub fn xyz_to_srgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| *c /= 100.0);
    *pixel = [
        3.2406 * pixel[0] - 1.5372 * pixel[1] - 0.4986 * pixel[2],
        -0.9689 * pixel[0] + 1.8758 * pixel[1] + 0.0415 * pixel[2],
        0.0557 * pixel[0] - 0.2040 * pixel[1] + 1.0570 * pixel[2],
    ];
    pixel.iter_mut().for_each(|c| {
        if *c <= 0.0031308 {
            *c *= 12.92
        } else {
            *c = 1.055 * (c.powf(1.0 / 2.4)) - 0.055
        }
    });
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

// TESTS {{{
#[cfg(test)]
mod tests {
    use super::*;

    const RGB: [f32; 3] = [0.2000, 0.3500, 0.9500];
    const HSV: [f32; 3] = [0.6333, 0.7894, 0.9500];
    const XYZ: [f32; 3] = [21.017, 14.314, 85.839];
    const LAB: [f32; 3] = [44.679, 40.806, -80.139];
    const LCH: [f32; 3] = [44.679, 89.930, 296.985];

    fn pixcmp(a: [f32; 3], b: [f32; 3]) {
        (0..3).for_each(|n| assert_eq!(format!("{:.3}", a[n]), format!("{:.3}", b[n])));
    }

    #[test]
    fn hsv_up() {
        let mut pixel = RGB;
        srgb_to_hsv(&mut pixel);
        pixcmp(pixel, HSV);
    }

    #[test]
    fn hsv_down() {
        let mut pixel = HSV;
        hsv_to_srgb(&mut pixel);
        pixcmp(pixel, RGB);
    }

    #[test]
    fn xyz_up() {
        let mut pixel = RGB;
        srgb_to_xyz(&mut pixel);
        pixcmp(pixel, XYZ);
    }

    #[test]
    fn xyz_down() {
        let mut pixel = XYZ;
        xyz_to_srgb(&mut pixel);
        pixcmp(pixel, RGB);
    }

    #[test]
    fn lab_up() {
        let mut pixel = XYZ;
        xyz_to_lab(&mut pixel);
        pixcmp(pixel, LAB);
    }

    #[test]
    fn lab_down() {
        let mut pixel = LAB;
        lab_to_xyz(&mut pixel);
        pixcmp(pixel, XYZ);
    }

    #[test]
    fn lch_up() {
        let mut pixel = LAB;
        lab_to_lch(&mut pixel);
        pixcmp(pixel, LCH);
    }

    #[test]
    fn lch_down() {
        let mut pixel = LCH;
        lch_to_lab(&mut pixel);
        pixcmp(pixel, LAB);
    }

    #[test]
    fn sweep() {
        let mut pixel = RGB;
        convert_space(Space::SRGB, Space::LCH, &mut pixel);
        convert_space(Space::LCH, Space::SRGB, &mut pixel);
        pixcmp(pixel, RGB)
    }
}
// TESTS }}}
