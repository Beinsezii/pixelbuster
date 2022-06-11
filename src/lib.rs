#![feature(test)]

use std::os::raw::c_char;

pub mod pbcore;
pub use pbcore::{parse_ops, process_multi, process_segment, Operation, Space};

pub fn pixelbuster<S: AsRef<str>>(
    code: S,
    space: Space,
    pixels: &mut [f32],
    vdefaults: Option<[f32; 9]>,
) {
    process_multi(parse_ops(code, space).0, pixels, vdefaults);
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
        None,
    );
}
