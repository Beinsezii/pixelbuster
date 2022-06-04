#![feature(test)]

use std::os::raw;
use std::sync::Arc;

mod pb_core;
use pb_core::{parse_ops, process_multi};

#[no_mangle]
pub extern "C" fn pixelbuster(
    // {{{
    code: *const raw::c_char,
    channels: *const raw::c_char,
    pixels: *mut raw::c_char,
    len: usize) {

    let len = len/8;

    let code = unsafe {
        assert!(!code.is_null());
        std::ffi::CStr::from_ptr(code).to_str().expect("Invalid code string").to_string()
    };
    let channels = unsafe {
        assert!(!channels.is_null());
        std::ffi::CStr::from_ptr(channels).to_str().expect("Invalid channels string").to_string()
    };
    let pixels = unsafe {
        assert!(!pixels.is_null());
        std::slice::from_raw_parts_mut(pixels.cast::<f32>(), len)
    };

    let ops = Arc::new(parse_ops(code, channels));

    process_multi(&*ops, pixels, None);
} // }}}

#[cfg(test)]
mod tests {
    use crate::pixelbuster;
    use std::ffi::CString;
    use std::os::raw;

    extern crate test;
    use test::Bencher;

    #[bench]
    fn bench_simple(b: &mut test::bench::Bencher) {
        let count = 4000 * 3000 * 4;
        let mut pixels = Vec::<f32>::new();
        pixels.reserve(count);
        for _ in 0..count {pixels.push(rand::random::<f32>())}
        b.iter(|| {
            pixelbuster(
                CString::new("r + 1\n\
                r + 1\n\
                r + 1\n\
                r + 1\n\
                r + 1\n\
                r - 5").unwrap().into_raw(),
                CString::new("rgba").unwrap().into_raw(),
                pixels.as_mut_ptr().cast::<raw::c_char>(),
                count,
            );
        });
    }

    #[bench]
    fn bench_complex(b: &mut test::bench::Bencher) {
        let count = 4000 * 3000 * 4;
        let mut pixels = Vec::<f32>::new();
        pixels.reserve(count);
        for _ in 0..count {pixels.push(rand::random::<f32>())}
        b.iter(|| {
            pixelbuster(
                CString::new("v = r\n\
                r + pi\n\
                r - pi\n\
                r * 2\n\
                r / 2\n\
                r sqrt r\n\
                r pow 2\n\
                r min 100\n\
                r max 0\n\
                r abs r\n\
                r log e\n\
                r round r\n\
                r = v").unwrap().into_raw(),
                CString::new("rgba").unwrap().into_raw(),
                pixels.as_mut_ptr().cast::<raw::c_char>(),
                count,
            );
        });
    }

    #[bench]
    fn bench_real(b: &mut Bencher) {
        let count = 4000 * 3000 * 4;
        let mut pixels = Vec::<f32>::new();
        pixels.reserve(count);
        for _ in 0..count {pixels.push(rand::random::<f32>())}
        b.iter(|| {
            pixelbuster(
                CString::new("v = 100\n\
                v2 = l\n\
                v2 / 1.25\n\
                v - v2\n\
                c * v").unwrap().into_raw(),
                CString::new("rgba").unwrap().into_raw(),
                pixels.as_mut_ptr().cast::<raw::c_char>(),
                count,
            );
        });
    }
}
