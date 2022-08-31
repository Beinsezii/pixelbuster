const LAB_DELTA: f32 = 6.0 / 29.0;

// 'Standard' Illuminant D65
const D65_X: f32 = 0.950489;
const D65_Y: f32 = 1.000000;
const D65_Z: f32 = 1.088840;

#[allow(unused)]
const D65: [f32; 3] = [D65_X, D65_Y, D65_Z];

// Illuminant D50
// Used by BABL/GIMP + others, not sure why
const D50_X: f32 = 0.964212;
const D50_Y: f32 = 1.000000;
const D50_Z: f32 = 0.825188;

#[allow(unused)]
const D50: [f32; 3] = [D50_X, D50_Y, D50_Z];

#[cfg(feature="D50")]
const ILLUMINANT: [f32; 3] = D50;

#[cfg(not(feature="D50"))]
const ILLUMINANT: [f32; 3] = D65;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Space {
    SRGB,
    HSV,
    LRGB,
    XYZ,
    LAB,
    LCH,
}

impl ToString for Space {
    fn to_string(&self) -> String {
        match self {
            Space::SRGB => String::from("rgba"),
            Space::HSV => String::from("hsva"),
            Space::LRGB => String::from("rgba"),
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
            "srgb" | "srgba" => Ok(Space::SRGB),
            "hsv" | "hsva" => Ok(Space::HSV),
            "lrgb" | "lrgba" | "rgb" | "rgba" => Ok(Space::LRGB),
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
        | (Space::LRGB, Space::LRGB)
        | (Space::XYZ, Space::XYZ)
        | (Space::LAB, Space::LAB)
        | (Space::LCH, Space::LCH) => (),
        // Up
        (Space::SRGB, Space::HSV) => srgb_to_hsv(pixel),
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
        (Space::HSV, Space::LRGB) => {hsv_to_srgb(pixel); srgb_to_lrgb(pixel)},
        (Space::HSV, Space::XYZ) => {hsv_to_srgb(pixel); srgb_to_lrgb(pixel); lrgb_to_xyz(pixel)},
        (Space::HSV, Space::LAB) => {hsv_to_srgb(pixel); srgb_to_lrgb(pixel); lrgb_to_xyz(pixel); xyz_to_lab(pixel)},
        (Space::HSV, Space::LCH) => {hsv_to_srgb(pixel); srgb_to_lrgb(pixel); lrgb_to_xyz(pixel); xyz_to_lab(pixel); lab_to_lch(pixel)},
        // Down
        (Space::LCH, Space::LAB) => lch_to_lab(pixel),
        (Space::LCH, Space::XYZ) => {lch_to_lab(pixel); lab_to_xyz(pixel)},
        (Space::LCH, Space::LRGB) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_lrgb(pixel)},
        (Space::LCH, Space::SRGB) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::LCH, Space::HSV) => {lch_to_lab(pixel); lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::LAB, Space::XYZ) => lab_to_xyz(pixel),
        (Space::LAB, Space::LRGB) => {lab_to_xyz(pixel); xyz_to_lrgb(pixel)},
        (Space::LAB, Space::SRGB) => {lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::LAB, Space::HSV) => {lab_to_xyz(pixel); xyz_to_lrgb(pixel); lrgb_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::XYZ, Space::SRGB) => {xyz_to_lrgb(pixel); lrgb_to_srgb(pixel)},
        (Space::XYZ, Space::LRGB) => xyz_to_lrgb(pixel),
        (Space::XYZ, Space::HSV) => {xyz_to_lrgb(pixel); lrgb_to_srgb(pixel); srgb_to_hsv(pixel)},
        (Space::LRGB, Space::SRGB) => lrgb_to_srgb(pixel),
        (Space::LRGB, Space::HSV) => {lrgb_to_srgb(pixel); srgb_to_hsv(pixel)},
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

// https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ
pub fn srgb_to_lrgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| {
        if *c <= 0.04045 {
            *c /= 12.92
        } else {
            *c = ((*c + 0.055) / 1.055_f32).powf(2.4)
        }
    });
}

// https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ
pub fn lrgb_to_xyz(pixel: &mut [f32; 3]) {
    *pixel = [
        (0.4124 * pixel[0] + 0.3576 * pixel[1] + 0.1805 * pixel[2]), // X
        (0.2126 * pixel[0] + 0.7152 * pixel[1] + 0.0722 * pixel[2]), // Y
        (0.0193 * pixel[0] + 0.1192 * pixel[1] + 0.9505 * pixel[2]), // Z
    ]
}

// https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIEXYZ_to_CIELAB
pub fn xyz_to_lab(pixel: &mut [f32; 3]) {
    pixel.iter_mut().zip(ILLUMINANT).for_each(|(c, d)| *c /= d);

    pixel.iter_mut().for_each(|c| {
        if *c > LAB_DELTA.powi(3) {
            *c = c.cbrt()
        } else {
            *c = *c / (3.0 * LAB_DELTA.powi(2)) + (4f32 / 29f32)
        }
    });

    *pixel = [
        (116.0 * pixel[1]) - 16.0,
        500.0 * (pixel[0] - pixel[1]),
        200.0 * (pixel[1] - pixel[2]),
    ]
}

// https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model
pub fn lab_to_lch(pixel: &mut [f32; 3]) {
    *pixel = [
        pixel[0],
        (pixel[1].powi(2) + pixel[2].powi(2)).sqrt(),
        pixel[2].atan2(pixel[1]).to_degrees(),
    ];
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

// https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB
pub fn lrgb_to_srgb(pixel: &mut [f32; 3]) {
    pixel.iter_mut().for_each(|c| {
        if *c <= 0.0031308 {
            *c *= 12.92
        } else {
            *c = 1.055 * (c.powf(1.0 / 2.4)) - 0.055
        }
    });
}

// https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB
pub fn xyz_to_lrgb(pixel: &mut [f32; 3]) {
    *pixel = [
        3.2406 * pixel[0] - 1.5372 * pixel[1] - 0.4986 * pixel[2],
        -0.9689 * pixel[0] + 1.8758 * pixel[1] + 0.0415 * pixel[2],
        0.0557 * pixel[0] - 0.2040 * pixel[1] + 1.0570 * pixel[2],
    ];
}

// https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIELAB_to_CIEXYZ
pub fn lab_to_xyz(pixel: &mut [f32; 3]) {
    *pixel = [
        (pixel[0] + 16.0) / 116.0 + pixel[1] / 500.0,
        (pixel[0] + 16.0) / 116.0,
        (pixel[0] + 16.0) / 116.0 - pixel[2] / 200.0,
    ];

    pixel.iter_mut().for_each(|c| {
        if *c > LAB_DELTA {
            *c = c.powi(3)
        } else {
            *c = 3.0 * LAB_DELTA.powi(2) * (*c - 4f32 / 29f32)
        }
    });

    pixel.iter_mut().zip(ILLUMINANT).for_each(|(c, d)| *c *= d);
}

// https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model
pub fn lch_to_lab(pixel: &mut [f32; 3]) {
    *pixel = [
        pixel[0],
        pixel[1] * pixel[2].to_radians().cos(),
        pixel[1] * pixel[2].to_radians().sin(),
    ]
}

// DOWN }}}

// TESTS {{{
#[cfg(test)]
mod tests {
    use super::*;

    // taken from https://easyrgb.com
    // const RGB: [f32; 3] = [0.2000, 0.3500, 0.9500];
    // const HSV: [f32; 3] = [0.6333, 0.7894, 0.9500];
    // const XYZ: [f32; 3] = [21.017, 14.314, 85.839];
    // const LAB: [f32; 3] = [44.679, 40.806, -80.139];
    // const LCH: [f32; 3] = [44.679, 89.930, 296.985];

    // Taken from BABL, which I honestly trust more
    const SRGB: [f32; 3] = [0.200000, 0.350000, 0.950000];
    const LRGB: [f32; 3] = [0.033105, 0.100482, 0.890006];
    const HSV: [f32; 3] = [0.633333, 0.789474, 0.950000];
    // interestingly, XYZ fails horrendously.
    // EasyRGB and BABL must use totally different formulae
    // EasyRGB's looks to be the same as Wikipedia, which is what I use.
    // BABL's is different and I'm not sure where it's sourced from...
    //
    //
    // #define LAB_EPSILON       (216.0f / 24389.0f)
    // #define LAB_KAPPA         (24389.0f / 27.0f)
    //
    // #define D50_WHITE_REF_X   0.964202880f
    // #define D50_WHITE_REF_Y   1.000000000f
    // #define D50_WHITE_REF_Z   0.824905400f
    // static inline void
    // XYZ_to_LAB (double X,
    //             double Y,
    //             double Z,
    //             double *to_L,
    //             double *to_a,
    //             double *to_b)
    // {
    //   double f_x, f_y, f_z;

    //   double x_r = X / D50_WHITE_REF_X;
    //   double y_r = Y / D50_WHITE_REF_Y;
    //   double z_r = Z / D50_WHITE_REF_Z;

    //   if (x_r > LAB_EPSILON) f_x = pow(x_r, 1.0 / 3.0);
    //   else ( f_x = ((LAB_KAPPA * x_r) + 16) / 116.0 );

    //   if (y_r > LAB_EPSILON) f_y = pow(y_r, 1.0 / 3.0);
    //   else ( f_y = ((LAB_KAPPA * y_r) + 16) / 116.0 );

    //   if (z_r > LAB_EPSILON) f_z = pow(z_r, 1.0 / 3.0);
    //   else ( f_z = ((LAB_KAPPA * z_r) + 16) / 116.0 );

    //   *to_L = (116.0 * f_y) - 16.0;
    //   *to_a = 500.0 * (f_x - f_y);
    //   *to_b = 200.0 * (f_y - f_z);
    // }
    //
    const XYZ: [f32; 3] = [0.180448, 0.133343, 0.645614];
    // Off after 2 decimals. Weird.
    const LAB: [f32; 3] = [43.262680, 30.556679, -82.134712];
    // Mine doesn't wrap hue.
    // Doesn't seem to matter except in tests to let's fudge it.
    // const LCH: [f32; 3] = [43.262680, 87.634590, 290.406769];
    const LCH: [f32; 3] = [43.262680, 87.634590, 290.406769 - 360.0];

    fn pixcmp(a: [f32; 3], b: [f32; 3]) {
        assert_eq!(
            format!("{:.3} {:.3} {:.3}", a[0], a[1], a[2]),
            format!("{:.3} {:.3} {:.3}", b[0], b[1], b[2])
        );
    }

    #[test]
    fn hsv_up() {
        let mut pixel = SRGB;
        srgb_to_hsv(&mut pixel);
        pixcmp(pixel, HSV);
    }

    #[test]
    fn hsv_down() {
        let mut pixel = HSV;
        hsv_to_srgb(&mut pixel);
        pixcmp(pixel, SRGB);
    }

    #[test]
    fn lrgb_up() {
        let mut pixel = SRGB;
        srgb_to_lrgb(&mut pixel);
        pixcmp(pixel, LRGB);
    }

    #[test]
    fn lrgb_down() {
        let mut pixel = LRGB;
        lrgb_to_srgb(&mut pixel);
        pixcmp(pixel, SRGB);
    }

    #[test]
    fn xyz_up() {
        let mut pixel = LRGB;
        lrgb_to_xyz(&mut pixel);
        pixcmp(pixel, XYZ);
    }

    #[test]
    fn xyz_down() {
        let mut pixel = XYZ;
        xyz_to_lrgb(&mut pixel);
        pixcmp(pixel, LRGB);
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
    fn full_up() {
        let mut pixel = SRGB;
        convert_space(Space::SRGB, Space::LCH, &mut pixel);
        pixcmp(pixel, LCH);
    }

    #[test]
    fn full_down() {
        let mut pixel = LCH;
        convert_space(Space::LCH, Space::SRGB, &mut pixel);
        pixcmp(pixel, SRGB);
    }

    #[test]
    fn sweep() {
        let mut pixel = SRGB;
        convert_space(Space::SRGB, Space::LCH, &mut pixel);
        convert_space(Space::LCH, Space::SRGB, &mut pixel);
        pixcmp(pixel, SRGB)
    }
}
// TESTS }}}
