#![feature(test)]
#![feature(array_chunks)]

use std::os::raw::c_char;

pub mod pbcore;
pub use pbcore::{parse_ops, process, Operation, Space};

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
    pixels: *mut c_char,
    len: usize,
) {
    let len = len / 8;

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
        std::slice::from_raw_parts_mut(pixels.cast::<f32>(), len)
    };

    pixelbuster(
        &code,
        Space::try_from(channels.as_str()).unwrap_or(Space::SRGB),
        pixels,
        0,
        None,
    );
}
