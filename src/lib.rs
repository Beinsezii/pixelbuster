#![feature(test)]
#![feature(array_chunks)]

use std::os::raw::c_char;

pub mod pbcore;
pub use pbcore::{parse_ops, process, Operation, Space};

pub const HELP: &str = "\
Valid lines:
    * {target} {operation} {source}
    * {space}
    * if {source} {comparison} {source} {line}
    * :{label}
    * jmp {label} or goto {label}
    * swap {target} {target}

Quick Example:
    r ** 2
    g / b
    lch
    h sqrt h

Target:
    Where to write the operation result to.

    * Channel (channel letters like 'r' 'g' 'b' 'a', or c1 ... c4),
    * Variable (v1, v2...v9) or (e1, e2...9)

Source:
    Where to source operation data from

    * any valid Target
    * any constant numeric value, eg '3.14'
    * 'pi'
    * 'e'
    * 'rand' - random val between 0.0 -> 1.0
    * 'col' - pixel X
    * 'row' - pixel Y
    * 'width' - width of image
    * 'height' - height of image
    * 'xnorm' - pixel X on scale of 0.0 -> 1.0
    * 'ynorm' - pixel Y on scale of 0.0 -> 1.0

Operation:
    Operations that take 2 values will source from target and source in order
    Eg: 'r log 2' translates to 'r = r.log(2)'

    * '+=' or '+' or 'add' => Add
    * '-=' or '-' or 'sub' => Sub
    * '*=' or '*' or 'mul' => Mul
    * '/=' or '/' or 'div' => Div
    * '%=' or '%' or 'mod' => Mod
    * '**' or '^' or 'pow' => Pow
    * '=' or 'set' => Set
    * abs acos acosh asin asinh atan atan2 atanh cbrt ceil copysign cos cosh
    * degrees diveuclid exp exp2 expm1 floor fract hypot ln ln1p log log10 log2
    * max min radians recip remeuclid round signum sin sinh sqrt tan tanh trunc
    * invert - a invert b == a = b - a

Comparison:
    * '==' or 'eq'
    * '!=' or '!' or 'neq'
    * '>' or 'gt'
    * '<' or 'lt'
    * '>=' or 'gteq'
    * '<=' or 'lteq'

Notes:
    Lines beginning with '#' are ignored
    Lines ending with '\\' are continued to next
    ';' counts as a linebreak anywhere in code

    There's a hard limit of 99 jumps per pixel as it can't detect loops

    v1 through v9 start at 0.0 every pixel

    e1 through e9 are 'external variables' that can be assigned starting values
    Useful for creating things like UI control sliders";

pub fn pixelbuster<S: AsRef<str>>(
    code: S,
    space: Space,
    pixels: &mut [f32],
    width: usize,
    externals: Option<[f32; 9]>,
) {
    process(parse_ops(code, space).0, pixels, width, externals);
}

#[no_mangle]
pub extern "C" fn pixelbuster_ffi(
    code: *const c_char,
    channels: *const c_char,
    pixels: *mut u8,
    pixels_size: usize,
    width: usize,
) {
    pixelbuster_ffi_ext(
        code,
        channels,
        pixels,
        pixels_size,
        width,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    )
}

#[no_mangle]
pub extern "C" fn pixelbuster_ffi_ext(
    code: *const c_char,
    channels: *const c_char,
    pixels: *mut u8,
    pixels_size: usize,
    width: usize,
    e1: f32,
    e2: f32,
    e3: f32,
    e4: f32,
    e5: f32,
    e6: f32,
    e7: f32,
    e8: f32,
    e9: f32,
) {
    let code = unsafe {
        assert!(!code.is_null());
        std::ffi::CStr::from_ptr(code)
            .to_str()
            .expect("Invalid code string")
            .to_string()
    };
    let channels = unsafe {
        assert!(!channels.is_null());
        std::ffi::CStr::from_ptr(channels)
            .to_str()
            .expect("Invalid channels string")
            .to_string()
    };
    let pixels = unsafe {
        assert!(!pixels.is_null());
        std::slice::from_raw_parts_mut(pixels.cast::<f32>(), pixels_size / 4)
    };

    pixelbuster(
        &code,
        Space::try_from(channels.as_str()).unwrap_or(Space::SRGB),
        pixels,
        width,
        Some([e1, e2, e3, e4, e5, e6, e7, e8, e9]),
    );
}

#[no_mangle]
pub extern "C" fn pb_help_ffi() -> *mut c_char {
    std::ffi::CString::new(HELP).unwrap().into_raw()
}
